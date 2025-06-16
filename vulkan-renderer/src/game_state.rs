use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};
use fighting_game::datatypes::Vector2;
use winit::{event::ElementState, keyboard::KeyCode};

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
            GameStateInner::Game(game) => game.input_manager.set_keyboard_key_state(key, state),
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
                            scale: (4.0, 4.0),
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
                    self.state = Some(GameStateInner::Game(Game {
                        game: fighting_game::Game::init(
                            fighting_game::initialization::Character::Sol,
                            fighting_game::initialization::Character::Sol,
                        ),
                        ui: crate::ui::PlayerUi::new(&self.asset_manager),
                        input_manager: char_select
                            .input_manager
                            .start_game()
                            .expect("error creating input manager"),
                    }))
                } else {
                    char_select.input_manager.update();
                    _ = self
                        .state
                        .insert(GameStateInner::CharacterSelect(char_select));
                }
                (connected_text, vec![])
            }
            GameStateInner::Game(Game {
                mut game,
                mut ui,
                mut input_manager,
            }) => {
                input_manager.update();
                let input_states = input_manager.get_input_state().map(Result::unwrap);

                let _game_status = game.update(input_states);

                let fighting_game::GameState { player_1, player_2 } = game.get_gamestate();
                ui.update(&player_1, &player_2);

                let (ui_sprites, ui_primatives) = ui.get_render_info();

                let sprites = game
                    .render_state()
                    .map(|e| {
                        let mut s = e.sprite_name.into_string();
                        s.push_str(".png");
                        crate::renderer::Sprite {
                            sprite: self
                                .asset_manager
                                .get_sprite_handle(&s)
                                .expect("failed to find sprite"),
                            position: e.position + Vector2::DOWN * 30.0,
                            depth: e.depth,
                            facing_left: e.flipped,
                            scale: (6.0, 6.0),
                        }
                    })
                    .chain(ui_sprites)
                    .collect();

                _ = self.state.insert(GameStateInner::Game(Game {
                    game,
                    ui,
                    input_manager,
                }));

                (sprites, ui_primatives.collect())
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

struct Game {
    game: fighting_game::Game,
    ui: crate::ui::PlayerUi,
    input_manager: crate::input::GameInputManager,
}
