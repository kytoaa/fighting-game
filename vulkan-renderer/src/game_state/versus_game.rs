use fighting_game::datatypes::Vector2;
use fighting_game::initialization::Character as Char;

const ROUND_START_FRAMES: usize = 3 * 60 + 60;
const ROUND_END_FRAMES: usize = 2 * 60;
const WIN_FRAMES: usize = 2 * 60;

pub struct Game {
    game: fighting_game::Game,
    ui: crate::ui::PlayerUi,
    input_manager: crate::input::GameInputManager,
    state: VersusGameState,
}
enum VersusGameState {
    RoundStart { frames: usize },
    Game,
    RoundWon { frames: usize },
    GameWon { frames: usize, p1_won: bool },
}

impl Game {
    pub fn init(
        characters: (Char, Char),
        input_manager: crate::input::GameInputManager,
        asset_manager: &asset_manager::AssetManager,
    ) -> Self {
        Self {
            game: fighting_game::Game::init(characters.0, characters.1),
            ui: crate::ui::PlayerUi::new(asset_manager),
            input_manager,
            state: VersusGameState::RoundStart {
                frames: ROUND_START_FRAMES,
            },
        }
    }

    pub fn set_keyboard_key_state(
        &mut self,
        key: winit::keyboard::KeyCode,
        state: winit::event::ElementState,
    ) {
        self.input_manager.set_keyboard_key_state(key, state);
    }

    pub fn update(
        &mut self,
        asset_manager: &asset_manager::AssetManager,
    ) -> (
        (
            Vec<crate::renderer::Sprite>,
            Vec<crate::renderer::Primative>,
        ),
        bool,
    ) {
        self.input_manager.update();

        let (sprites, primatives) = match &mut self.state {
            VersusGameState::RoundStart { frames } => {
                if *frames == ROUND_START_FRAMES {
                    self.game.update(Default::default());
                }

                *frames -= 1;

                let frames = *frames;

                if frames == 0 {
                    self.state = VersusGameState::Game;
                    println!("game started");
                }

                let fighting_game::GameState { player_1, player_2 } = self.game.get_gamestate();
                self.ui.update(&player_1, &player_2, Some(99));

                let timer_index = (frames + 45) / 60;

                (
                    match timer_index {
                        0..=3 => {
                            let path = match timer_index {
                                0 => "ui/countdown/go.png",
                                1 => "ui/countdown/1.png",
                                2 => "ui/countdown/2.png",
                                3 => "ui/countdown/3.png",
                                _ => unreachable!(),
                            };
                            vec![crate::renderer::Sprite {
                                sprite: asset_manager.get_sprite_handle(path).unwrap(),
                                position: Vector2::ZERO,
                                facing_left: false,
                                depth: 0.5,
                                scale: (6.0, 6.0),
                            }]
                            .into_iter()
                        }
                        _ => vec![].into_iter(),
                    },
                    vec![].into_iter(),
                )
            }
            VersusGameState::Game => {
                let input_states = self.input_manager.get_input_state().map(Result::unwrap);

                let game_status = self.game.update(input_states);

                let mut seconds_left = None;

                match game_status {
                    fighting_game::GameStatus::Draw => {
                        println!("round ended");
                        self.state = VersusGameState::RoundWon {
                            frames: ROUND_END_FRAMES,
                        };
                    }
                    fighting_game::GameStatus::RoundWon(p) => {
                        self.ui.incr_wins(p == 0);
                        println!("round ended");
                        self.state = VersusGameState::RoundWon {
                            frames: ROUND_END_FRAMES,
                        };
                    }
                    fighting_game::GameStatus::Running { frames_left } => {
                        seconds_left = Some(frames_left / 60);
                    }
                    fighting_game::GameStatus::GameWon(winner) => {
                        self.ui.incr_wins(winner == 0);
                        println!("round ended");
                        self.state = VersusGameState::GameWon {
                            frames: WIN_FRAMES,
                            p1_won: match winner {
                                0 => true,
                                1 => false,
                                _ => unreachable!(),
                            },
                        }
                    }
                }

                let fighting_game::GameState { player_1, player_2 } = self.game.get_gamestate();
                self.ui.update(&player_1, &player_2, seconds_left);

                (vec![].into_iter(), vec![].into_iter())
            }
            VersusGameState::RoundWon { frames } => {
                *frames -= 1;

                let frames = *frames;

                if frames == 0 {
                    println!("round starting");
                    self.state = VersusGameState::RoundStart {
                        frames: ROUND_START_FRAMES,
                    };
                }

                (
                    match frames {
                        10..50 => vec![crate::renderer::Sprite {
                            sprite: asset_manager
                                .get_sprite_handle("ui/countdown/end.png")
                                .unwrap(),
                            position: Vector2::ZERO,
                            facing_left: false,
                            depth: 0.2,
                            scale: (2.0, 2.0),
                        }]
                        .into_iter(),
                        _ => vec![].into_iter(),
                    },
                    vec![].into_iter(),
                )
            }
            VersusGameState::GameWon { frames, p1_won: _ } => {
                *frames -= 1;

                let frames = *frames;

                if frames == 0 {
                    return ((vec![], vec![]), true);
                }

                (
                    match frames {
                        10..50 => vec![crate::renderer::Sprite {
                            sprite: asset_manager
                                .get_sprite_handle("ui/countdown/end.png")
                                .unwrap(),
                            position: Vector2::ZERO,
                            facing_left: false,
                            depth: 0.2,
                            scale: (2.0, 2.0),
                        }]
                        .into_iter(),
                        _ => vec![].into_iter(),
                    },
                    vec![].into_iter(),
                )
            }
        };

        let (ui_sprites, ui_primatives) = self.ui.get_render_info();

        let sprites = self
            .game
            .render_state()
            .map(|e| {
                let mut s = e.sprite_name.into_string();
                s.push_str(".png");
                crate::renderer::Sprite {
                    sprite: asset_manager
                        .get_sprite_handle(&s)
                        .expect("failed to find sprite"),
                    position: e.position + Vector2::DOWN * 30.0,
                    depth: e.depth,
                    facing_left: e.flipped,
                    scale: (6.0, 6.0),
                }
            })
            .chain(ui_sprites)
            .chain(sprites)
            .collect();

        let primatives = ui_primatives.chain(primatives).collect();

        ((sprites, primatives), false)
    }
}
