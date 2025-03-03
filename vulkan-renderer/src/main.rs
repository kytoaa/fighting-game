use fighting_game::world::World;

fn main() {
    App::init().run().unwrap();
}

struct App {
    state: GameState,
    renderer: vulkan_renderer::renderer::Renderer,
}

impl App {
    fn init() -> Self {
        todo!();
    }
    fn run(self) -> Result<(), ()> {
        todo!();
    }
}

enum GameState {
    Game(World),
}
