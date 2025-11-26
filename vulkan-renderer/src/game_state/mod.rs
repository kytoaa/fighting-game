use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};
use fighting_game::datatypes::Vector2;
use winit::{event::ElementState, keyboard::KeyCode};

mod versus_game;
use versus_game::Game;

pub struct GameState {
    asset_manager: asset_manager::AssetManager,
    state: Option<GameStateInner>,
    connection_type: crate::netcode::ConnectionType,
}

impl GameState {
    pub fn new(
        asset_manager: asset_manager::AssetManager,
        connection_type: crate::netcode::ConnectionType,
    ) -> Self {
        GameState {
            asset_manager,
            state: Some(GameStateInner::CharacterSelect(match connection_type {
                crate::netcode::ConnectionType::Offline => CharacterSelect {
                    input_manager: crate::input::CharacterSelectInputManager::offline(),
                    connection: None,
                },
                crate::netcode::ConnectionType::Host(addr) => CharacterSelect {
                    input_manager: crate::input::CharacterSelectInputManager::host(),
                    connection: Some(
                        crate::netcode::CharacterSelectConnection::host(addr).unwrap(),
                    ),
                },
                crate::netcode::ConnectionType::Client(addr) => CharacterSelect {
                    input_manager: crate::input::CharacterSelectInputManager::client(),
                    connection: Some(
                        crate::netcode::CharacterSelectConnection::join(addr).unwrap(),
                    ),
                },
            })),
            connection_type,
        }
    }
    pub fn asset_manager(&self) -> &asset_manager::AssetManager {
        &self.asset_manager
    }
    pub fn keyboard_input(&mut self, key: KeyCode, state: ElementState) {
        match self.state.as_mut().unwrap() {
            GameStateInner::Game(game, _) => game.set_keyboard_key_state(key, state),
            GameStateInner::CharacterSelect(character_select) => {
                character_select
                    .input_manager
                    .set_keyboard_key_state(key, state);
            }
        }
    }
    pub fn update(
        &mut self,
    ) -> (
        Vec<crate::renderer::Sprite>,
        Vec<crate::renderer::Primative>,
    ) {
        match self.state.take().unwrap() {
            GameStateInner::CharacterSelect(mut char_select) => {
                char_select
                    .input_manager
                    .set_keyboard_key_state(KeyCode::KeyJ, ElementState::Pressed);
                char_select
                    .input_manager
                    .set_keyboard_key_state(KeyCode::KeyJ, ElementState::Released);

                char_select.input_manager.remote_request_start();

                let connected_text = char_select
                    .input_manager
                    .connected_input_devices()
                    .into_iter()
                    .enumerate()
                    .filter(|(_, c)| *c)
                    .map(|(i, _)| {
                        let dir = (i as isize * 2 - 1).signum() as f32;
                        crate::renderer::Sprite {
                            sprite: self
                                .asset_manager
                                .get_sprite_handle("ui/connected_text.png")
                                .unwrap(),
                            position: Vector2::new(
                                WINDOW_WIDTH as f32 / 11.0 * dir,
                                WINDOW_HEIGHT as f32 / 12.0,
                            ),
                            facing_left: false,
                            depth: 0.5,
                            scale: (2.0, 2.0),
                        }
                    })
                    .chain(
                        [crate::renderer::Sprite {
                            sprite: self
                                .asset_manager
                                .get_sprite_handle("ui/background.png")
                                .unwrap(),
                            position: Vector2::ZERO,
                            facing_left: false,
                            depth: 0.7,
                            scale: (WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
                        }]
                        .into_iter(),
                    )
                    .collect();

                /*if (char_select.input_manager.should_start()
                && (matches!(
                    &self.connection_type,
                    crate::netcode::ConnectionType::Offline,
                ) || (matches!(
                    &self.connection_type,
                    crate::netcode::ConnectionType::Host(_)
                ) && char_select.input_manager.remote_has_requested_start())))
                || (matches!(
                    &self.connection_type,
                    crate::netcode::ConnectionType::Client(_),
                ) && char_select.input_manager.remote_has_requested_start())*/
                if true {
                    println!("updated");
                    if let crate::netcode::ConnectionType::Host(_) = &self.connection_type {
                        char_select.connection.as_mut().unwrap().send_start();
                    }
                    self.state = Some(GameStateInner::Game(
                        Game::init(
                            (
                                fighting_game::initialization::Character::Sol,
                                fighting_game::initialization::Character::Sol,
                            ),
                            char_select.input_manager.start_game().unwrap(),
                            &self.asset_manager,
                        ),
                        char_select
                            .connection
                            .map(|connection| connection.into_game_connection().unwrap()),
                    ))
                } else {
                    if let Some(true) = char_select.connection.as_mut().map(|c| c.requested_start())
                    {
                        char_select.input_manager.remote_request_start();
                    }
                    /*if matches!(
                        &self.connection_type,
                        crate::netcode::ConnectionType::Client(_),
                    ) {
                        char_select.connection.as_mut().unwrap().request_start();
                    }*/
                    char_select.connection.as_mut().map(|c| c.request_start());
                    char_select.input_manager.update();
                    _ = self
                        .state
                        .insert(GameStateInner::CharacterSelect(char_select));
                }
                (connected_text, vec![])
            }
            GameStateInner::Game(mut game, mut connection) => {
                let should_skip_frame = !connection
                    .as_mut()
                    .map(|c| c.can_continue())
                    .unwrap_or(true);

                if let Some(rollback) = connection
                    .as_ref()
                    .map(|c| c.get_packet())
                    .flatten()
                    .map(|packet| game.set_remote_key_state(packet))
                    .flatten()
                {
                    println!("rollback! {} frames", rollback.frames());

                    let inputs: Vec<_> = rollback
                        .player_inputs()
                        .zip(rollback.remote_inputs())
                        .map(|(l, r)| {
                            if matches!(
                                self.connection_type,
                                crate::netcode::ConnectionType::Client(_)
                            ) {
                                [r.to_input_state().unwrap(), l.to_input_state().unwrap()]
                            } else {
                                [l.to_input_state().unwrap(), r.to_input_state().unwrap()]
                            }
                        })
                        .collect();

                    let frames = rollback.frames();

                    game.rollback_and_resimulate(frames, inputs.into_iter());
                }
                if let Some(connection) = connection.as_mut() {
                    if connection.current_desync() > 1 && connection.current_frame() % 6 == 0 {
                        connection.set_frames_to_wait(1);
                    }
                }

                if !should_skip_frame {
                    game.input_manager()
                        .update()
                        .map(|packet| connection.as_ref().unwrap().send_packet(packet));
                }

                let (mut r, reset) = if !should_skip_frame {
                    game.update(Some(&self.asset_manager))
                } else {
                    println!("skipped a frame");
                    (game.get_render_info(&self.asset_manager), false)
                };

                if reset {
                    let mut input = game.take_input_manager();
                    input.update();
                    _ = self
                        .state
                        .insert(GameStateInner::CharacterSelect(CharacterSelect {
                            input_manager: crate::input::CharacterSelectInputManager::from(input),
                            connection: None,
                        }));
                    r = self.update();
                } else {
                    _ = self.state.insert(GameStateInner::Game(game, connection));
                }
                r
            }
        }
    }
}

enum GameStateInner {
    Game(Game, Option<crate::netcode::GameConnection>),
    CharacterSelect(CharacterSelect),
}

struct CharacterSelect {
    input_manager: crate::input::CharacterSelectInputManager,
    connection: Option<crate::netcode::CharacterSelectConnection>,
}
