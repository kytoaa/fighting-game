use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};
use fighting_game::datatypes::Vector2;
use winit::{event::ElementState, keyboard::KeyCode};

mod versus_game;
use versus_game::Game;

pub struct GameState {
    asset_manager: asset_manager::AssetManager,
    state: Option<GameStateInner>,
}

impl GameState {
    pub fn new(asset_manager: asset_manager::AssetManager) -> Self {
        GameState {
            asset_manager,
            state: Some(GameStateInner::CharacterSelect(CharacterSelect {
                input_manager: crate::input::CharacterSelectInputManager::new(),
            })),
        }
    }
    pub fn asset_manager(&self) -> &asset_manager::AssetManager {
        &self.asset_manager
    }
    pub fn keyboard_input(&mut self, key: KeyCode, state: ElementState) {
        match self.state.as_mut().unwrap() {
            GameStateInner::Game(game) => game.set_keyboard_key_state(key, state),
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

                if char_select.input_manager.should_start() {
                    println!("updated");
                    self.state = Some(GameStateInner::Game(Game::init(
                        (
                            fighting_game::initialization::Character::Sol,
                            fighting_game::initialization::Character::Sol,
                        ),
                        char_select.input_manager.start_game().unwrap(),
                        &self.asset_manager,
                    )))
                } else {
                    char_select.input_manager.update();
                    _ = self
                        .state
                        .insert(GameStateInner::CharacterSelect(char_select));
                }
                (connected_text, vec![])
            }
            GameStateInner::Game(mut game) => {
                let (mut r, reset) = game.update(&self.asset_manager);
                if reset {
                    let mut input = game.take_input_manager();
                    input.update();
                    _ = self
                        .state
                        .insert(GameStateInner::CharacterSelect(CharacterSelect {
                            input_manager: crate::input::CharacterSelectInputManager::from(input),
                        }));
                    r = self.update();
                } else {
                    _ = self.state.insert(GameStateInner::Game(game));
                }
                r
            }
        }
    }
}

enum GameStateInner {
    Game(Game),
    CharacterSelect(CharacterSelect),
}

struct CharacterSelect {
    input_manager: crate::input::CharacterSelectInputManager,
}
