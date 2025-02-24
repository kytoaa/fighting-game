use crate::datatypes::Vector2;
use std::collections::{HashMap, VecDeque};

pub mod directions;

use directions::Motion;

pub use directions::InputDir;

const BUFFER_LENGTH: usize = 3;
const INPUT_HISTORY_LENGTH: usize = 60;
const DOUBLE_PRESS_FRAMES: usize = 10;

pub struct InputHandler {
    direction_queue: VecDeque<InputDir>,
    button_states: HashMap<Button, ButtonState>,
    buffered_actions: Option<Vec<BufferedAction>>,
}
impl InputHandler {
    pub fn new() -> InputHandler {
        InputHandler {
            direction_queue: std::iter::repeat(InputDir::Dir5)
                .take(INPUT_HISTORY_LENGTH)
                .collect(),
            button_states: {
                let mut map = HashMap::new();
                map.insert(Button::Light, ButtonState::Up);
                map.insert(Button::Mid, ButtonState::Up);
                map.insert(Button::Heavy, ButtonState::Up);
                map.insert(Button::Utility, ButtonState::Up);
                map
            },
            buffered_actions: Some(vec![]),
        }
    }
}

impl InputHandler {
    pub fn has_motion(&self, motion: &Motion) -> bool {
        if self.direction_queue.len() < motion.frames {
            return false;
        }

        let mut motion_index = 0;
        for dir in self
            .direction_queue
            .iter()
            .skip(self.direction_queue.len() - motion.frames - 1)
        {
            if let Some(d) = motion.directions.get(motion_index) {
                if d == dir {
                    motion_index += 1;
                } else if motion.fail_directions.contains(dir) {
                    motion_index = 0;
                }
            }
        }
        if motion_index == motion.directions.len() {
            true
        } else {
            false
        }
    }
    pub fn has_action(&self, action: &Action) -> bool {
        self.buffered_actions
            .as_ref()
            .unwrap()
            .iter()
            .find(|a| &a.action == action)
            .is_some()
    }
    pub fn has_motion_input(&self, motion: &Motion, action: &Action) -> bool {
        self.has_motion(motion) && self.has_action(action)
    }
    pub fn get_state(&self, button: Button) -> ButtonState {
        *self.button_states.get(&button).unwrap()
    }

    pub fn update(&mut self, button_states: HashMap<Button, ButtonState>, dir: InputDir) {
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

        _ = self.direction_queue.pop_front();
        self.direction_queue.push_back(dir);

        if self
            .direction_queue
            .iter()
            .skip(INPUT_HISTORY_LENGTH - DOUBLE_PRESS_FRAMES - 1)
            .skip_while(|d| **d != InputDir::Dir5)
            .skip_while(|d| **d != dir)
            .find(|d| **d == InputDir::Dir5)
            .is_some()
        {
            self.buffered_actions
                .as_mut()
                .unwrap()
                .push(BufferedAction::new(Action::DoublePress(dir)));
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
                .push(BufferedAction::new(Action::JumpPress(dir)));
        }

        for (button, state) in button_states.iter() {
            let prev_state = self.button_states[button];
            match (state, prev_state) {
                (ButtonState::Down, ButtonState::Up) => self
                    .buffered_actions
                    .as_mut()
                    .unwrap()
                    .push(BufferedAction::new(Action::Pressed(*button))),
                (ButtonState::Up, ButtonState::Down) => self
                    .buffered_actions
                    .as_mut()
                    .unwrap()
                    .push(BufferedAction::new(Action::Released(*button))),
                _ => (),
            }
        }
        self.button_states = button_states;
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

#[derive(PartialEq, Debug)]
struct ButtonPress {
    button: Button,
    state: ButtonState,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonState {
    Up,
    Down,
}

#[derive(PartialEq, Debug)]
pub enum Action {
    Pressed(Button),
    Released(Button),
    DoublePress(InputDir),
    JumpPress(InputDir),
}

#[derive(PartialEq, Debug)]
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
