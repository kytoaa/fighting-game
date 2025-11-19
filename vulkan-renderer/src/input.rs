#![allow(dead_code)]

use fighting_game::datatypes::Vector2;
use gilrs::{self, Gilrs};

#[derive(Debug)]
enum InputDevice {
    Keyboard(KeyboardState),
    Gamepad(gilrs::GamepadId),
    Remote(fighting_game::input::InputState),
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

#[derive(Debug)]
pub struct KeyboardState {
    keys: std::collections::HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
}
impl KeyboardState {
    pub fn new() -> Self {
        Self {
            keys: Default::default(),
        }
    }
}

pub struct CharacterSelectInputManager {
    gilrs: Gilrs,
    active_input_sources: [InputDevice; 2],
}

pub struct GameInputManager {
    gilrs: Gilrs,
    active_input_sources: [InputDevice; 2],
}

impl From<GameInputManager> for CharacterSelectInputManager {
    fn from(value: GameInputManager) -> Self {
        Self {
            gilrs: value.gilrs,
            active_input_sources: value.active_input_sources.map(|source| match source {
                InputDevice::None => InputDevice::None,
                g @ InputDevice::Gamepad(_) => g,
                InputDevice::Keyboard(_) => InputDevice::Keyboard(KeyboardState::new()),
                InputDevice::Remote(_) => InputDevice::Remote(Default::default()),
            }),
        }
    }
}

impl CharacterSelectInputManager {
    pub fn new() -> Self {
        let gilrs = gilrs::GilrsBuilder::new().build().unwrap();

        Self {
            gilrs,
            active_input_sources: [const { InputDevice::None }; 2],
        }
    }
    pub fn update(&mut self) {
        while let Some(gilrs::Event { id, event, .. }) = self.gilrs.next_event() {
            match event {
                gilrs::EventType::ButtonPressed(button, _) => match button {
                    gilrs::Button::South => self.try_add_input_device(InputDevice::Gamepad(id)),
                    gilrs::Button::East => self.try_remove_input_device(InputDevice::Gamepad(id)),
                    _ => {}
                },
                gilrs::EventType::Connected => {
                    let gamepad = self.gilrs.gamepad(id);

                    // BUG: windows interacting differently to linux with switch pro controller,
                    // might be an issue with my device specifically
                    #[cfg(windows)]
                    if let Some(0x057e) = gamepad.vendor_id() {
                        println!("nintendo controller, rebinding buttons");

                        let mut mapping = gilrs::Mapping::new();
                        mapping.insert_btn(
                            gamepad.button_code(gilrs::Button::South).unwrap(),
                            gilrs::Button::East,
                        );
                        mapping.insert_btn(
                            gamepad.button_code(gilrs::Button::East).unwrap(),
                            gilrs::Button::South,
                        );
                        mapping.insert_btn(
                            gamepad.button_code(gilrs::Button::North).unwrap(),
                            gilrs::Button::West,
                        );
                        mapping.insert_btn(
                            gamepad.button_code(gilrs::Button::West).unwrap(),
                            gilrs::Button::North,
                        );

                        gamepad
                            .axis_code(gilrs::Axis::LeftStickX)
                            .map(|c| Some((c, gamepad.axis_code(gilrs::Axis::LeftStickY)?)))
                            .flatten()
                            .map(|(x, y)| {
                                mapping.insert_axis(x, gilrs::Axis::LeftStickX);
                                mapping.insert_axis(y, gilrs::Axis::LeftStickY)
                            });

                        mapping.insert_btn(
                            gamepad.button_code(gilrs::Button::DPadUp).unwrap(),
                            gilrs::Button::DPadUp,
                        );
                        mapping.insert_btn(
                            gamepad.button_code(gilrs::Button::DPadDown).unwrap(),
                            gilrs::Button::DPadDown,
                        );
                        mapping.insert_btn(
                            gamepad.button_code(gilrs::Button::DPadLeft).unwrap(),
                            gilrs::Button::DPadLeft,
                        );
                        mapping.insert_btn(
                            gamepad.button_code(gilrs::Button::DPadRight).unwrap(),
                            gilrs::Button::DPadRight,
                        );
                        self.gilrs.set_mapping(id.into(), &mapping, None).unwrap();
                    }
                }
                _ => continue,
            }
        }
    }
    pub fn set_keyboard_key_state(
        &mut self,
        key: winit::keyboard::KeyCode,
        state: winit::event::ElementState,
    ) {
        if let winit::event::ElementState::Pressed = state {
            match key {
                winit::keyboard::KeyCode::KeyJ => {
                    self.try_add_input_device(InputDevice::Keyboard(KeyboardState::new()))
                }
                winit::keyboard::KeyCode::KeyK => {
                    self.try_remove_input_device(InputDevice::Keyboard(KeyboardState::new()))
                }
                _ => {}
            }
        }

        match &mut self.active_input_sources {
            [InputDevice::Keyboard(kb_state), _] | [_, InputDevice::Keyboard(kb_state)] => {
                _ = kb_state.keys.insert(key, state)
            }
            _ => return,
        }
    }
    fn try_add_input_device(&mut self, device: InputDevice) {
        println!("requested to add {:?}", device);
        match &mut self.active_input_sources {
            [a @ InputDevice::None, b] if *b != device => *a = device,
            [a, b @ InputDevice::None] if *a != device => *b = device,
            _ => (),
        }
        println!("devices: {:?}", self.active_input_sources);
    }
    fn try_remove_input_device(&mut self, device: InputDevice) {
        println!("requested to remove {:?}", device);
        _ = self
            .active_input_sources
            .iter_mut()
            .find(|d| **d == device)
            .map(|s| *s = InputDevice::None)
    }
    pub fn should_start(&self) -> bool {
        self.active_input_sources
            .iter()
            .any(|d| d.pressed_start(&self.gilrs))
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
    pub fn connected_input_devices(&self) -> [bool; 2] {
        [
            !(self.active_input_sources[0] == InputDevice::None),
            !(self.active_input_sources[1] == InputDevice::None),
        ]
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
    pub fn update(&mut self) {
        while let Some(_) = self.gilrs.next_event() {}
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
            match states.get(&button).unwrap_or(&ElementState::Released) {
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
        const fn bool_as_button_state(b: bool) -> fighting_game::input::ButtonState {
            match b {
                true => fighting_game::input::ButtonState::Down,
                false => fighting_game::input::ButtonState::Up,
            }
        }

        match self {
            Self::None => Ok(fighting_game::input::InputState::default()),
            Self::Remote(state) => Ok(state.clone()),
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

                let dir = {
                    let dpad_dir = Vector2::new(
                        gamepad
                            .button_code(gilrs::Button::DPadRight)
                            .map(|code| if state.is_pressed(code) { 1.0 } else { 0.0 })
                            .unwrap_or_default()
                            - gamepad
                                .button_code(gilrs::Button::DPadLeft)
                                .map(|code| if state.is_pressed(code) { 1.0 } else { 0.0 })
                                .unwrap_or_default(),
                        gamepad
                            .button_code(gilrs::Button::DPadUp)
                            .map(|code| if state.is_pressed(code) { 1.0 } else { 0.0 })
                            .unwrap_or_default()
                            - gamepad
                                .button_code(gilrs::Button::DPadDown)
                                .map(|code| if state.is_pressed(code) { 1.0 } else { 0.0 })
                                .unwrap_or_default(),
                    );

                    let stick_dir = Vector2::new(
                        gamepad
                            .axis_data(gilrs::Axis::LeftStickX)
                            .map(|a| a.value())
                            .unwrap_or_default(),
                        gamepad
                            .axis_data(gilrs::Axis::LeftStickY)
                            .map(|a| a.value())
                            .unwrap_or_default(),
                    );

                    let stick_dir = if stick_dir.magnitude() >= 0.3 {
                        stick_dir.normalized().rounded()
                    } else {
                        Vector2::ZERO
                    };

                    ((dpad_dir + stick_dir) / 2.0).rounded()
                }
                .into();

                Ok(fighting_game::input::InputState {
                    dir,
                    button_states: fighting_game::input::ButtonStates {
                        light: bool_as_button_state(gamepad.is_pressed(gilrs::Button::West)),
                        mid: bool_as_button_state(gamepad.is_pressed(gilrs::Button::North)),
                        heavy: bool_as_button_state(gamepad.is_pressed(gilrs::Button::East)),
                        utility: bool_as_button_state(gamepad.is_pressed(gilrs::Button::South)),
                    },
                })
            }
        }
    }
    fn pressed_start(&self, gilrs: &Gilrs) -> bool {
        match self {
            Self::Keyboard(kb_state) => kb_state
                .keys
                .get(&winit::keyboard::KeyCode::KeyU)
                .map(|state| !state.is_pressed())
                .unwrap_or_default(),
            Self::Gamepad(id) => gilrs
                .connected_gamepad(*id)
                .map(|gamepad| {
                    gamepad
                        .button_data(gilrs::Button::West)
                        .map(|state| state.value() > 0.5)
                })
                .flatten()
                .unwrap_or_default(),
            Self::None => false,
            Self::Remote(state) => {
                state.button_states.mid == fighting_game::input::ButtonState::Down
            }
        }
    }
}

#[derive(Debug)]
pub enum InputManagerGameStartError {
    NotEnoughPlayers,
}
#[derive(Debug)]
pub enum InputManagerGetStateError {
    ControllerDisconnected(gilrs::GamepadId),
}
