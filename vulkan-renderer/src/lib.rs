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

    sol_position: Vector2,
    previous_time: std::time::SystemTime,
    sol_sprite: bool,
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
                if let winit::keyboard::PhysicalKey::Code(key_code) = event.physical_key {
                    match key_code {
                        winit::keyboard::KeyCode::KeyA => self.sol_position += Vector2::LEFT,
                        winit::keyboard::KeyCode::KeyD => self.sol_position += Vector2::RIGHT,
                        winit::keyboard::KeyCode::KeyS => self.sol_position += Vector2::DOWN,
                        winit::keyboard::KeyCode::KeyW => self.sol_position += Vector2::UP,
                        winit::keyboard::KeyCode::Space => self.sol_sprite = true,
                        winit::keyboard::KeyCode::ShiftLeft => self.sol_sprite = false,
                        _ => (),
                    }
                }
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                let time = std::time::SystemTime::now()
                    .duration_since(self.previous_time)
                    .unwrap();
                self.previous_time = std::time::SystemTime::now();

                println!("{}", 1.0 / time.as_secs_f64());

                self.renderer.as_mut().unwrap().draw_frame(
                    &self.asset_manager,
                    vec![(
                        self.asset_manager
                            .get_sprite_handle(match self.sol_sprite {
                                true => "gunflame.png",
                                false => "j.d.png",
                            })
                            .unwrap(),
                        self.sol_position.y(-self.sol_position.y),
                        false,
                        0.5,
                    )],
                );

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

        event_loop
            .run_app(&mut App {
                state: GameState::Game(()),
                renderer: None,
                window: None,

                asset_manager: STATIC_ASSETS.into_asset_manager(),

                sol_position: Vector2::ZERO,
                previous_time: std::time::SystemTime::now(),
                sol_sprite: true,
            })
            .unwrap();
    }
    pub fn run(self) -> Result<(), ()> {
        todo!();
    }
}

enum GameState {
    Game(()),
}
