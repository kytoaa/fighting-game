use crate::{
    input::CharacterSelectInputManager,
    netcode::{CharacterSelectConnection, ConnectionError, ConnectionType, GameConnection},
    WINDOW_HEIGHT, WINDOW_WIDTH,
};
use fighting_game::datatypes::Vector2;
use winit::{event::ElementState, keyboard::KeyCode};

mod versus_game;
use versus_game::Game;

pub struct GameState {
    asset_manager: asset_manager::AssetManager,
    state: Option<GameStateInner>,
    connection_type: ConnectionType,
}

impl GameState {
    pub fn new(
        asset_manager: asset_manager::AssetManager,
        connection_type: ConnectionType,
    ) -> Result<Self, ConnectionError> {
        Ok(GameState {
            asset_manager,
            state: Some(GameStateInner::CharacterSelect(match connection_type {
                ConnectionType::Offline => CharacterSelect {
                    input_manager: CharacterSelectInputManager::offline(),
                    connection: None,
                },
                ConnectionType::Host(addr) => CharacterSelect {
                    input_manager: CharacterSelectInputManager::host(),
                    connection: Some(CharacterSelectConnection::host(addr)?),
                },
                ConnectionType::Client(addr) => CharacterSelect {
                    input_manager: CharacterSelectInputManager::client(),
                    connection: Some(CharacterSelectConnection::join(addr)?),
                },
            })),
            connection_type,
        })
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
    ) -> Result<
        (
            Vec<crate::renderer::Sprite>,
            Vec<crate::renderer::Primative>,
        ),
        ConnectionError,
    > {
        match self.state.take().unwrap() {
            GameStateInner::CharacterSelect(mut char_select) => {
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

                if char_select.input_manager.should_start()
                    && char_select
                        .connection
                        .as_ref()
                        .map(|c| c.should_start())
                        .unwrap_or(matches!(self.connection_type, ConnectionType::Offline))
                {
                    println!("updated");

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
                    if let Some(true) = char_select
                        .connection
                        .as_mut()
                        .map(CharacterSelectConnection::has_start_been_requested)
                    {
                        char_select.input_manager.remote_request_start();
                    }

                    if char_select.input_manager.should_start() {
                        char_select.connection.as_mut().unwrap().request_start();
                    }

                    char_select.input_manager.update();
                    _ = self
                        .state
                        .insert(GameStateInner::CharacterSelect(char_select));
                }
                Ok((connected_text, vec![]))
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
                            if matches!(self.connection_type, ConnectionType::Client(_)) {
                                [r.to_input_state().unwrap(), l.to_input_state().unwrap()]
                            } else {
                                [l.to_input_state().unwrap(), r.to_input_state().unwrap()]
                            }
                        })
                        .collect();

                    let frames = rollback.frames();

                    game.rollback_and_resimulate(frames, inputs.into_iter())
                        .map_err(ConnectionError::Timeout)?;
                }
                if let Some(connection) = connection.as_mut() {
                    if connection.current_desync() > 1 && connection.current_frame() % 6 == 0 {
                        connection.set_frames_to_wait(1);
                    }
                }

                if !should_skip_frame {
                    if let Some(packet) = game.input_manager().update() {
                        connection.as_ref().unwrap().send_packet(packet?);
                    }
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
                            input_manager: CharacterSelectInputManager::from(input),
                            connection: match &self.connection_type {
                                ConnectionType::Host(_) => connection.map(|c| {
                                    CharacterSelectConnection::host(c.addr().local()).unwrap()
                                }),
                                ConnectionType::Client(_) => connection.map(|c| {
                                    CharacterSelectConnection::join(c.addr().remote()).unwrap()
                                }),
                                ConnectionType::Offline => None,
                            },
                        }));
                    r = self.update()?;
                } else {
                    _ = self.state.insert(GameStateInner::Game(game, connection));
                }
                Ok(r)
            }
        }
    }
}

enum GameStateInner {
    Game(Game, Option<GameConnection>),
    CharacterSelect(CharacterSelect),
}

struct CharacterSelect {
    input_manager: CharacterSelectInputManager,
    connection: Option<CharacterSelectConnection>,
}
