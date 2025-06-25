mod characters;
pub mod collision;
pub mod datatypes;
pub mod initialization;
pub mod input;
pub mod world;

pub struct Game {
    input_providers: [input::InputHandler; 2],
    world: world::World,
    has_won: [bool; 2],
    characters: [initialization::Character; 2],
    reset: bool,
}

impl Game {
    pub fn init(player_1: initialization::Character, player_2: initialization::Character) -> Self {
        let world = initialization::create_world(initialization::WorldBuilder::with_characters(
            player_1, player_2,
        ));

        let input_providers = [input::InputHandler::new(), input::InputHandler::new()];

        Self {
            world,
            input_providers,
            has_won: [false; 2],
            characters: [player_1, player_2],
            reset: false,
        }
    }
}
impl Game {
    pub fn update(&mut self, input_states: [input::InputState; 2]) -> GameStatus {
        if self.reset {
            let game_state = self.get_gamestate();

            self.world = initialization::create_world(
                initialization::WorldBuilder::with_characters(
                    self.characters[0],
                    self.characters[1],
                )
                .with_bursts(
                    (game_state.player_1.burst_percent * 10000.0) as u32 + 3000,
                    (game_state.player_2.burst_percent * 10000.0) as u32 + 3000,
                ),
            );
            self.reset = false;
        }
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

        let status = self.world.update(&self.input_providers);

        if let GameStatus::RoundWon(p) = status {
            return match self.has_won[p] {
                true => GameStatus::GameWon(p),
                false => {
                    self.reset = true;
                    self.has_won[p] = true;
                    self.input_providers
                        .iter_mut()
                        .for_each(|i| i.clear_history());
                    GameStatus::RoundWon(p)
                }
            };
        }

        status
    }
    pub fn get_gamestate(&self) -> GameState {
        GameState {
            player_1: self.world.get_player_state(world::EntityID::PLAYER_1_ID),
            player_2: self.world.get_player_state(world::EntityID::PLAYER_2_ID),
        }
    }
    pub fn render_state<'a>(&'a self) -> impl Iterator<Item = EntityRenderInfo> + use<'a> {
        self.world
            .get_players()
            .into_iter()
            .filter_map(|player| {
                let frame_name = player.frame_name();
                match frame_name {
                    Some((sprite_name, offset)) => Some(EntityRenderInfo {
                        position: player.position() + offset,
                        flipped: !player.get_direction(),
                        depth: if self.world.in_combo(player.id()) {
                            0.61
                        } else {
                            0.6
                        },
                        sprite_name,
                    }),
                    None => None,
                }
            })
            .chain(self.world.get_non_player_entities().filter_map(|entity| {
                match entity.frame_name() {
                    Some((sprite_name, offset)) => Some(EntityRenderInfo {
                        position: entity.position() + offset,
                        flipped: !entity.dir(),
                        depth: if entity.draw_behind_players() {
                            0.7
                        } else {
                            0.5
                        },
                        sprite_name,
                    }),
                    None => None,
                }
            }))
    }
    pub fn get_hitboxes<'a>(&'a self) -> impl Iterator<Item = SpawnedCollider> + use<'a> {
        self.world
            .get_hitboxes()
            .map(|h| SpawnedCollider::Hitbox(h.shape.get_bounding_box().clone()))
            .chain(
                self.world
                    .get_hurtboxes()
                    .map(|h| SpawnedCollider::Hurtbox(h.shape.get_bounding_box().clone())),
            )
            .chain(
                self.world
                    .get_throwboxes()
                    .map(|h| SpawnedCollider::Throwbox(h.shape.get_bounding_box().clone())),
            )
            .chain(
                self.world
                    .get_players()
                    .map(|p| SpawnedCollider::Collider(p.get_collider_world_space())),
            )
    }
    pub fn get_combo_hits(&self) -> Option<usize> {
        self.world.get_combo_hits()
    }
}
pub enum SpawnedCollider {
    Hitbox(crate::datatypes::BoundingBox),
    Hurtbox(crate::datatypes::BoundingBox),
    Throwbox(crate::datatypes::BoundingBox),
    Collider(crate::datatypes::BoundingBox),
}

pub struct EntityRenderInfo {
    pub position: datatypes::Vector2,
    pub flipped: bool,
    pub depth: f32,
    pub sprite_name: Box<str>,
}

pub struct PlayerState {
    pub health_percent: f32,
    pub burst_percent: f32,
    pub scaling_percent: f32,
    pub meter_percent: f32,
    pub combo_damage_health_percent: Option<f32>,
}

pub struct GameState {
    pub player_1: PlayerState,
    pub player_2: PlayerState,
}

pub enum GameStatus {
    Running { frames_left: usize },
    RoundWon(usize),
    Draw,
    GameWon(usize),
}
