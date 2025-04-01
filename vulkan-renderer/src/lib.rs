use fighting_game::datatypes::Vector2;
use winit::window::Window;

pub mod renderer;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

static STATIC_ASSETS: asset_manager::static_data::StaticAssets =
    asset_manager_macros::generate_static_asset_manager_from_dir!("./assets/");

pub struct App {
    state: GameState,
    renderer: Option<renderer::Renderer>,
    window: Option<Window>,

    asset_manager: asset_manager::AssetManager,

    key_states: std::collections::HashMap<winit::keyboard::PhysicalKey, winit::event::ElementState>,

    previous_time: std::time::SystemTime,
    show_hitboxes: bool,
    show_fps: bool,
    debug_paused: bool,
}

pub struct RenderData {
    sprites_to_render: [Vec<renderer::Material>; 2],
    index: usize,
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default()
                        .with_inner_size(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                        .with_resizable(false),
                )
                .unwrap(),
        );
        self.renderer = Some(
            renderer::Renderer::init(
                &self.window.as_ref().unwrap(),
                &self.window.as_ref().unwrap(),
                (WINDOW_WIDTH, WINDOW_HEIGHT),
            )
            .unwrap(),
        );
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                self.key_states.insert(event.physical_key, event.state);
                if let (
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyQ),
                    winit::event::ElementState::Pressed,
                ) = (event.physical_key, event.state)
                {
                    self.show_hitboxes = !self.show_hitboxes;
                }
                if let (
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyW),
                    winit::event::ElementState::Pressed,
                ) = (event.physical_key, event.state)
                {
                    self.debug_paused = !self.debug_paused;
                }
                if let (
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyE),
                    winit::event::ElementState::Pressed,
                ) = (event.physical_key, event.state)
                {
                    let input_states = self.get_input_states();
                    _ = self.state.update(&self.asset_manager, false, input_states);
                }
                if let (
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit1),
                    winit::event::ElementState::Pressed,
                ) = (event.physical_key, event.state)
                {
                    self.show_fps = !self.show_fps;
                }
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                let time = std::time::SystemTime::now()
                    .duration_since(self.previous_time)
                    .unwrap();
                if self.show_fps {
                    println!("{}", 1.0 / time.as_secs_f64());
                }
                std::thread::sleep(
                    std::time::Duration::from_secs_f64(1.0 / 60.0).saturating_sub(time),
                );
                self.previous_time = std::time::SystemTime::now();

                let input_states = self.get_input_states();
                let mut sprite_data =
                    self.state
                        .update(&self.asset_manager, self.debug_paused, input_states);

                if self.show_hitboxes {
                    sprite_data = match &self.state {
                        GameState::Game(game) => sprite_data
                            .into_iter()
                            .chain(game.world().get_hurtboxes().filter_map(|hitbox| {
                                match &hitbox.shape {
                                    fighting_game::collision::CollisionShape::Box(b) => {
                                        Some(renderer::Material::Rect {
                                            pos: b.position(),
                                            size: b.size(),
                                            color: renderer::RectColor::Green,
                                        })
                                    }
                                    fighting_game::collision::CollisionShape::Circle(_) => None,
                                }
                            }))
                            .chain(game.world().get_hitboxes().filter_map(|hitbox| {
                                match &hitbox.shape {
                                    fighting_game::collision::CollisionShape::Box(b) => {
                                        Some(renderer::Material::Rect {
                                            pos: b.position(),
                                            size: b.size(),
                                            color: renderer::RectColor::Red,
                                        })
                                    }
                                    fighting_game::collision::CollisionShape::Circle(_) => None,
                                }
                            }))
                            .chain(game.world().get_players().iter().map(|player| {
                                let collider = player.get_collider_world_space();
                                renderer::Material::Rect {
                                    pos: collider.position(),
                                    size: collider.size(),
                                    color: renderer::RectColor::Blue,
                                }
                            }))
                            .collect(),
                    }
                }

                self.renderer
                    .as_mut()
                    .unwrap()
                    .draw_frame(&self.asset_manager, &sprite_data);

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
}

impl App {
    pub fn init() {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let asset_manager = STATIC_ASSETS.into_asset_manager();

        /*let render_data = RenderData {
            sprites_to_render: [
                vec![(
                    asset_manager.get_sprite_handle("idle.png").unwrap(),
                    Vector2::ZERO,
                    false,
                    0.5,
                )],
                vec![(
                    asset_manager.get_sprite_handle("idle.png").unwrap(),
                    Vector2::ZERO,
                    false,
                    0.5,
                )],
            ],
            index: 0,
        };*/

        event_loop
            .run_app(&mut App {
                state: GameState::create_game(),
                renderer: None,
                window: None,

                key_states: {
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyD),
                        winit::event::ElementState::Released,
                    );
                    map.insert(
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyF),
                        winit::event::ElementState::Released,
                    );
                    map.insert(
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyG),
                        winit::event::ElementState::Released,
                    );
                    map.insert(
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Space),
                        winit::event::ElementState::Released,
                    );
                    map.insert(
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyJ),
                        winit::event::ElementState::Released,
                    );
                    map.insert(
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyU),
                        winit::event::ElementState::Released,
                    );
                    map.insert(
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyI),
                        winit::event::ElementState::Released,
                    );
                    map.insert(
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyK),
                        winit::event::ElementState::Released,
                    );
                    map
                },

                asset_manager,

                previous_time: std::time::SystemTime::now(),
                show_hitboxes: false,
                show_fps: false,
                debug_paused: false,
            })
            .unwrap();
    }

    fn get_input_states(&mut self) -> [fighting_game::input::InputState; 2] {
        use fighting_game::input::{Button, ButtonState, InputState};
        use winit::event::ElementState;
        use winit::keyboard::{KeyCode, PhysicalKey};

        let light = match self
            .key_states
            .get(&PhysicalKey::Code(KeyCode::KeyJ))
            .unwrap()
        {
            ElementState::Released => ButtonState::Up,
            ElementState::Pressed => ButtonState::Down,
        };
        let mid = match self
            .key_states
            .get(&PhysicalKey::Code(KeyCode::KeyU))
            .unwrap()
        {
            ElementState::Released => ButtonState::Up,
            ElementState::Pressed => ButtonState::Down,
        };
        let heavy = match self
            .key_states
            .get(&PhysicalKey::Code(KeyCode::KeyI))
            .unwrap()
        {
            ElementState::Released => ButtonState::Up,
            ElementState::Pressed => ButtonState::Down,
        };
        let utility = match self
            .key_states
            .get(&PhysicalKey::Code(KeyCode::KeyK))
            .unwrap()
        {
            ElementState::Released => ButtonState::Up,
            ElementState::Pressed => ButtonState::Down,
        };

        fn button_state(
            button: KeyCode,
            states: &std::collections::HashMap<PhysicalKey, ElementState>,
        ) -> f32 {
            match states.get(&PhysicalKey::Code(button)).unwrap() {
                ElementState::Pressed => 1.0,
                ElementState::Released => 0.0,
            }
        }

        let dir = Vector2::new(
            0.0 - button_state(KeyCode::KeyD, &self.key_states)
                + button_state(KeyCode::KeyG, &self.key_states),
            0.0 - button_state(KeyCode::KeyF, &self.key_states)
                + button_state(KeyCode::Space, &self.key_states),
        )
        .into();

        let mut other = InputState::default();

        [
            InputState {
                dir,
                button_states: fighting_game::input::ButtonStates {
                    light,
                    mid,
                    heavy,
                    utility,
                },
            },
            other,
        ]
    }
}

impl RenderData {
    pub fn get_sprite_names(&self) -> &[renderer::Material] {
        &self.sprites_to_render[self.index]
    }
}

enum GameState {
    Game(fighting_game::Game),
}

impl GameState {
    // TODO: add character selection
    pub fn create_game() -> Self {
        Self::Game(fighting_game::Game::init(
            fighting_game::initialization::Character::Sol,
            fighting_game::initialization::Character::Sol,
        ))
    }
}

impl GameState {
    pub fn update(
        &mut self,
        assets: &asset_manager::AssetManager,
        paused: bool,
        input_states: [fighting_game::input::InputState; 2],
    ) -> Vec<renderer::Material> {
        match self {
            GameState::Game(game) => {
                if !paused {
                    game.update(input_states);
                }
                game.world()
                    .get_players()
                    .iter()
                    .map(|player| {
                        let (frame, offset) = player.frame_name().unwrap();
                        let mut sprite_name = frame.into_string();
                        sprite_name.push_str(".png");
                        //println!("{}", &sprite_name);
                        renderer::Material::Sprite(
                            assets.get_sprite_handle(&sprite_name).unwrap(),
                            player.position() + offset,
                            !player.get_direction(),
                            0.5,
                        )
                    })
                    .collect()
            }
        }
    }
}
