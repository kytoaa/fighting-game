mod characters;
pub mod collision;
pub mod datatypes;
pub mod initialization;
pub mod input;
pub mod world;

pub struct Game {
    input_providers: [input::InputHandler; 2],
    world: world::World,
}

impl Game {
    pub fn init(player_1: initialization::Character, player_2: initialization::Character) -> Self {
        let world = initialization::create_world(player_1, player_2);

        let input_providers = [input::InputHandler::new(), input::InputHandler::new()];

        Self {
            world,
            input_providers,
        }
    }
}
impl Game {
    pub fn update(&mut self, input_states: [input::InputState; 2]) {
        self.input_providers
            .iter_mut()
            .zip(input_states.iter())
            .for_each(|(provider, state)| provider.update(state));

        if self.world.in_hitstop() {
            self.input_providers
                .iter_mut()
                .for_each(|ip| ip.decrement_action_buffers = false);
        } else {
            self.input_providers
                .iter_mut()
                .for_each(|ip| ip.decrement_action_buffers = true);
        }

        self.world.update(&self.input_providers);
    }
    pub fn world(&self) -> &world::World {
        &self.world
    }
    pub fn get_gamestate(&self) -> GameState {
        GameState {
            player_1: self.world.get_player_state(world::EntityID::PLAYER_1_ID),
            player_2: self.world.get_player_state(world::EntityID::PLAYER_2_ID),
        }
    }
    pub fn get_combo_hits(&self) -> Option<usize> {
        self.world.get_combo_hits()
    }
}

pub struct PlayerState {
    pub health_percent: f32,
    pub burst_percent: f32,
    pub scaling_percent: f32,
}

pub struct GameState {
    pub player_1: PlayerState,
    pub player_2: PlayerState,
}
