use fighting_game::{datatypes::Vector2, input::Button};
use gilrs::{self, Gamepad, Gilrs};

enum InputDevice {
    Keyboard(KeyboardState),
    Gamepad(gilrs::GamepadId),
    None,
}
impl InputDevice {
    const fn is_keyboard(&self) -> Option<&KeyboardState> {
        match self {
            Self::Keyboard(state) => Some(state),
            _ => None,
        }
    }
    const fn is_gamepad(&self) -> Option<gilrs::GamepadId> {
        match self {
            Self::Gamepad(id) => Some(*id),
            _ => None,
        }
    }
}
impl PartialEq for InputDevice {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (InputDevice::Gamepad(a), InputDevice::Gamepad(b)) => a == b,
            (InputDevice::Keyboard(_), InputDevice::Keyboard(_)) => true,
            (InputDevice::None, InputDevice::None) => true,
            _ => false,
        }
    }
}

pub struct KeyboardState {
    keys: std::collections::HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
}

pub struct CharacterSelectInputManager {
    gilrs: Gilrs,
    active_input_sources: [InputDevice; 2],
}

pub struct GameInputManager {
    gilrs: Gilrs,
    active_input_sources: [InputDevice; 2],
}

impl CharacterSelectInputManager {
    pub fn new() -> Self {
        let gilrs = Gilrs::new().unwrap();

        Self {
            gilrs,
            active_input_sources: [const { InputDevice::None }; 2],
        }
    }
    pub fn update(&mut self) {
        while let Some(gilrs::Event { id, event, .. }) = self.gilrs.next_event() {
            match event {
                gilrs::EventType::ButtonPressed(button, _) => match button {
                    gilrs::Button::Start => self.try_add_input_device(InputDevice::Gamepad(id)),
                    gilrs::Button::East => self.try_remove_input_device(InputDevice::Gamepad(id)),
                    _ => (),
                },
                _ => continue,
            }
        }
    }
    pub fn try_add_input_device(&mut self, device: InputDevice) {
        match &mut self.active_input_sources {
            [a @ InputDevice::None, _] if *a != device => *a = device,
            [_, b @ InputDevice::None] if *b != device => *b = device,
            _ => (),
        }
    }
    pub fn try_remove_input_device(&mut self, device: InputDevice) {
        _ = self
            .active_input_sources
            .iter_mut()
            .find(|d| **d == device)
            .map(|s| *s = InputDevice::None)
    }
    pub fn start_game(self) -> Result<GameInputManager, InputManagerGameStartError> {
        use InputDevice as Dev;

        match self.active_input_sources {
            [Dev::None, Dev::None] => Err(InputManagerGameStartError::NotEnoughPlayers),
            [Dev::Keyboard(_), Dev::Keyboard(_)] => unreachable!(),
            [Dev::Gamepad(a), Dev::Gamepad(b)] if a == b => unreachable!(),
            _ => Ok(GameInputManager {
                gilrs: self.gilrs,
                active_input_sources: self.active_input_sources,
            }),
        }
    }
}

impl GameInputManager {
    pub fn get_input_state(
        &self,
    ) -> [Result<fighting_game::input::InputState, InputManagerGetStateError>; 2] {
        [
            self.active_input_sources[0].get_state(&self.gilrs),
            self.active_input_sources[1].get_state(&self.gilrs),
        ]
    }
    pub fn set_keyboard_key_state(
        &mut self,
        key: winit::keyboard::KeyCode,
        state: winit::event::ElementState,
    ) {
        self.active_input_sources
            .iter_mut()
            .for_each(|device| match device {
                InputDevice::Keyboard(kb_state) => _ = kb_state.keys.insert(key, state),
                _ => (),
            })
    }
}

impl InputDevice {
    fn get_state(
        &self,
        gilrs: &Gilrs,
    ) -> Result<fighting_game::input::InputState, InputManagerGetStateError> {
        use winit::{event::ElementState, keyboard::KeyCode};

        fn button_state(
            button: KeyCode,
            states: &std::collections::HashMap<KeyCode, ElementState>,
        ) -> f32 {
            match states.get(&button).unwrap() {
                ElementState::Pressed => 1.0,
                ElementState::Released => 0.0,
            }
        }

        const fn as_button_state(
            element_state: &ElementState,
        ) -> fighting_game::input::ButtonState {
            match element_state {
                ElementState::Pressed => fighting_game::input::ButtonState::Down,
                ElementState::Released => fighting_game::input::ButtonState::Up,
            }
        }
        const fn f32_as_button_state(x: f32) -> fighting_game::input::ButtonState {
            match x > 0.5 {
                true => fighting_game::input::ButtonState::Down,
                false => fighting_game::input::ButtonState::Up,
            }
        }

        match self {
            Self::None => Ok(fighting_game::input::InputState::default()),
            Self::Keyboard(keyboard_state) => Ok(fighting_game::input::InputState {
                dir: Vector2::new(
                    0.0 - button_state(KeyCode::KeyD, &keyboard_state.keys)
                        + button_state(KeyCode::KeyG, &keyboard_state.keys),
                    0.0 - button_state(KeyCode::KeyF, &keyboard_state.keys)
                        + button_state(KeyCode::Space, &keyboard_state.keys),
                )
                .into(),
                button_states: fighting_game::input::ButtonStates {
                    light: keyboard_state
                        .keys
                        .get(&KeyCode::KeyJ)
                        .map(as_button_state)
                        .unwrap_or(fighting_game::input::ButtonState::Up),
                    mid: keyboard_state
                        .keys
                        .get(&KeyCode::KeyU)
                        .map(as_button_state)
                        .unwrap_or(fighting_game::input::ButtonState::Up),
                    heavy: keyboard_state
                        .keys
                        .get(&KeyCode::KeyI)
                        .map(as_button_state)
                        .unwrap_or(fighting_game::input::ButtonState::Up),
                    utility: keyboard_state
                        .keys
                        .get(&KeyCode::KeyK)
                        .map(as_button_state)
                        .unwrap_or(fighting_game::input::ButtonState::Up),
                },
            }),
            Self::Gamepad(id) => {
                let gamepad = gilrs
                    .connected_gamepad(*id)
                    .ok_or(InputManagerGetStateError::ControllerDisconnected(*id))?;

                let state = gamepad.state();

                let dir = Vector2::new(
                    gamepad
                        .axis_code(gilrs::Axis::DPadX)
                        .map(|code| state.axis_data(code))
                        .flatten()
                        .map(|d| d.value().round())
                        .unwrap_or_default(),
                    gamepad
                        .axis_code(gilrs::Axis::DPadY)
                        .map(|code| state.axis_data(code))
                        .flatten()
                        .map(|d| d.value().round())
                        .unwrap_or_default(),
                )
                .into();

                Ok(fighting_game::input::InputState {
                    dir,
                    button_states: fighting_game::input::ButtonStates {
                        light: gamepad
                            .button_code(gilrs::Button::South)
                            .map(|code| state.button_data(code))
                            .flatten()
                            .map(|d| f32_as_button_state(d.value()))
                            .unwrap_or(fighting_game::input::ButtonState::Up),
                        mid: gamepad
                            .button_code(gilrs::Button::West)
                            .map(|code| state.button_data(code))
                            .flatten()
                            .map(|d| f32_as_button_state(d.value()))
                            .unwrap_or(fighting_game::input::ButtonState::Up),
                        heavy: gamepad
                            .button_code(gilrs::Button::North)
                            .map(|code| state.button_data(code))
                            .flatten()
                            .map(|d| f32_as_button_state(d.value()))
                            .unwrap_or(fighting_game::input::ButtonState::Up),
                        utility: gamepad
                            .button_code(gilrs::Button::East)
                            .map(|code| state.button_data(code))
                            .flatten()
                            .map(|d| f32_as_button_state(d.value()))
                            .unwrap_or(fighting_game::input::ButtonState::Up),
                    },
                })
            }
        }
    }
}

pub enum InputManagerGameStartError {
    NotEnoughPlayers,
}
pub enum InputManagerGetStateError {
    ControllerDisconnected(gilrs::GamepadId),
}
