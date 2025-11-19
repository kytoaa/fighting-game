use crate::datatypes::Vector2;
use std::collections::VecDeque;

pub mod directions;

use directions::Motion;

pub use directions::InputDir;

const BUFFER_LENGTH: usize = 3;
const INPUT_HISTORY_LENGTH: usize = 60;
const DOUBLE_PRESS_FRAMES: usize = 14;

#[derive(Clone)]
pub struct InputHandler {
    direction_queue: VecDeque<InputDir>,
    button_states: ButtonStates,
    buffered_actions: Option<Vec<BufferedAction>>,
    pub decrement_action_buffers: bool,
    directions_to_dequeue: usize,
}
impl InputHandler {
    pub fn new() -> InputHandler {
        InputHandler {
            direction_queue: std::iter::repeat(InputDir::Dir5)
                .take(INPUT_HISTORY_LENGTH)
                .collect(),
            button_states: ButtonStates::default(),
            buffered_actions: Some(vec![]),
            decrement_action_buffers: true,
            directions_to_dequeue: 0,
        }
    }
}
impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler {
    pub fn clear_history(&mut self) {
        self.direction_queue
            .iter_mut()
            .for_each(|d| *d = InputDir::Dir5);
        self.buffered_actions = Some(vec![]);
    }
    pub fn has_motion(&self, motion: &Motion) -> bool {
        if self.direction_queue.len() < motion.frames {
            return false;
        }

        let mut motion_index = 0;
        for dir in self
            .direction_queue
            .iter()
            .skip(self.direction_queue.len() - motion.frames - self.directions_to_dequeue)
        {
            if let Some(d) = motion.directions.get(motion_index) {
                if d == dir {
                    motion_index += 1;
                } else if motion.fail_directions.contains(dir) {
                    motion_index = 0;
                }
            }
        }
        motion_index == motion.directions.len()
    }
    pub fn has_action(&self, action: &Action) -> bool {
        if let Action::MultiplePress(a, b) = action {
            self.buffered_actions
                .as_ref()
                .unwrap()
                .iter()
                .find(|action| match action.action {
                    Action::Pressed(button, _) => button == *a,
                    _ => false,
                })
                .is_some()
                && self
                    .buffered_actions
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|action| match action.action {
                        Action::Pressed(button, _) => button == *b,
                        _ => false,
                    })
                    .is_some()
        } else {
            self.buffered_actions
                .as_ref()
                .unwrap()
                .iter()
                .any(|a| match action {
                    Action::Pressed(button, None) => match a.action {
                        Action::Pressed(b, _) if b == *button => true,
                        _ => false,
                    },
                    _ => a.action == *action,
                })
        }
    }
    pub fn has_motion_input(&self, motion: &Motion, action: &Action) -> bool {
        self.has_motion(motion) && self.has_action(action)
    }
    pub fn get_state(&self, button: Button) -> ButtonState {
        self.button_states.get_button_state(button)
    }

    pub fn update(&mut self, input_state: &InputState) {
        if self.decrement_action_buffers {
            self.buffered_actions = Some(
                self.buffered_actions
                    .take()
                    .unwrap()
                    .into_iter()
                    .filter_map(|mut action| {
                        action.frames_left -= 1;
                        if action.frames_left <= 0 {
                            None
                        } else {
                            Some(action)
                        }
                    })
                    .collect(),
            );
            for _ in 0..self.directions_to_dequeue {
                _ = self.direction_queue.pop_front();
            }
            self.directions_to_dequeue = 1;
        } else {
            self.directions_to_dequeue += 1;
        }

        self.direction_queue.push_back(input_state.dir);

        if self
            .direction_queue
            .iter()
            .skip(INPUT_HISTORY_LENGTH - DOUBLE_PRESS_FRAMES - 1)
            .skip_while(|d| **d != InputDir::Dir5)
            .skip_while(|d| **d != input_state.dir)
            .find(|d| **d == InputDir::Dir5)
            .is_some()
        {
            self.buffered_actions
                .as_mut()
                .unwrap()
                .push(BufferedAction::new(Action::DoublePress(input_state.dir)));
        }
        if self
            .direction_queue
            .iter()
            .skip(INPUT_HISTORY_LENGTH - 3)
            .find(|d| Into::<Vector2>::into(**d).y != 1.0)
            .is_some()
        {
            self.buffered_actions
                .as_mut()
                .unwrap()
                .push(BufferedAction::new(Action::JumpPress(input_state.dir)));
        }

        for (button, state) in input_state.button_states.get_states() {
            let prev_state = self.button_states.get_button_state(button);
            match (state, prev_state) {
                (ButtonState::Down, ButtonState::Up) => self
                    .buffered_actions
                    .as_mut()
                    .unwrap()
                    .push(BufferedAction::new(Action::Pressed(
                        button,
                        Some(input_state.dir),
                    ))),
                (ButtonState::Up, ButtonState::Down) => self
                    .buffered_actions
                    .as_mut()
                    .unwrap()
                    .push(BufferedAction::new(Action::Released(button))),
                _ => (),
            }
        }
        self.button_states = input_state.button_states.clone();
    }

    pub fn input_dir(&self) -> InputDir {
        *self.direction_queue.back().unwrap()
    }
    pub fn move_dir(&self) -> Vector2 {
        self.direction_queue.back().unwrap().into()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Button {
    Light,
    Mid,
    Heavy,
    Utility,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonState {
    Up,
    Down,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Action {
    Pressed(Button, Option<InputDir>),
    Released(Button),
    DoublePress(InputDir),
    JumpPress(InputDir),
    MultiplePress(Button, Button),
}

#[derive(PartialEq, Debug, Clone)]
struct BufferedAction {
    action: Action,
    frames_left: usize,
}
impl BufferedAction {
    const fn new(action: Action) -> Self {
        Self {
            action,
            frames_left: BUFFER_LENGTH,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ButtonStates {
    pub light: ButtonState,
    pub mid: ButtonState,
    pub heavy: ButtonState,
    pub utility: ButtonState,
}

#[derive(Debug, Default, Clone)]
pub struct InputState {
    pub dir: InputDir,
    pub button_states: ButtonStates,
}

impl ButtonStates {
    const fn get_button_state(&self, button: Button) -> ButtonState {
        match button {
            Button::Light => self.light,
            Button::Mid => self.mid,
            Button::Heavy => self.heavy,
            Button::Utility => self.utility,
        }
    }
    fn get_states(&self) -> impl Iterator<Item = (Button, ButtonState)> {
        [
            (Button::Light, self.get_button_state(Button::Light)),
            (Button::Mid, self.get_button_state(Button::Mid)),
            (Button::Heavy, self.get_button_state(Button::Heavy)),
            (Button::Utility, self.get_button_state(Button::Utility)),
        ]
        .into_iter()
    }
}
impl Default for ButtonStates {
    fn default() -> Self {
        Self {
            light: ButtonState::Up,
            mid: ButtonState::Up,
            heavy: ButtonState::Up,
            utility: ButtonState::Up,
        }
    }
}
