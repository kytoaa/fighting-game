use fighting_game::datatypes::Vector2;
use winit::window::Window;

mod game_state;
mod input;
mod renderer;
mod ui;

use game_state::GameState;

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 360;

static STATIC_ASSETS: asset_manager::static_data::StaticAssets =
    asset_manager_macros::generate_static_asset_manager_from_dir!("./assets/");

pub struct App {
    renderer: Option<renderer::Renderer>,
    window: Option<Window>,

    game_state: GameState,

    previous_time: std::time::SystemTime,
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
                let keycode = match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(k) => k,
                    _ => return,
                };
                self.game_state.keyboard_input(keycode, event.state);
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                let time = std::time::SystemTime::now()
                    .duration_since(self.previous_time)
                    .unwrap();
                std::thread::sleep(
                    std::time::Duration::from_secs_f64(1.0 / 60.0).saturating_sub(time),
                );
                self.previous_time = std::time::SystemTime::now();

                let (sprite_data, primatives) = self.game_state.update();

                self.renderer.as_mut().unwrap().draw_frame(
                    &self.game_state.asset_manager(),
                    &sprite_data,
                    &primatives,
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

        let asset_manager = STATIC_ASSETS.into_asset_manager();

        event_loop
            .run_app(&mut App {
                renderer: None,
                window: None,

                game_state: GameState::new(asset_manager),

                previous_time: std::time::SystemTime::now(),
            })
            .unwrap();
    }
}
