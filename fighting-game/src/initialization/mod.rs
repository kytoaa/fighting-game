use super::*;
use characters::Player;
use datatypes::Vector2;

#[derive(Debug, Clone, Copy)]
pub enum Character {
    Sol,
}

pub(crate) fn create_world(world_builder: WorldBuilder) -> world::World {
    let WorldBuilder {
        characters: [player_1, player_2],
        bursts: [player_1_burst, player_2_burst],
    } = world_builder;

    world::World::new((
        |id| get_character(player_1, Vector2::new(-15.0, 0.0), id, player_1_burst),
        |id| get_character(player_2, Vector2::new(15.0, 0.0), id, player_2_burst),
    ))
}

fn get_character(
    character: Character,
    position: Vector2,
    id: world::EntityID,
    burst: u32,
) -> (Box<dyn Player>, characters::CharacterInitInfo) {
    match character {
        Character::Sol => (
            Box::new(characters::sol::initial_state(id, position)),
            characters::sol::init_info().into_init_info(burst),
        ),
    }
}

pub(crate) struct WorldBuilder {
    characters: [Character; 2],
    bursts: [u32; 2],
}
impl WorldBuilder {
    pub fn with_characters(player_1: Character, player_2: Character) -> Self {
        Self {
            characters: [player_1, player_2],
            bursts: [10000; 2],
        }
    }
    pub fn with_bursts(self, player_1: u32, player_2: u32) -> Self {
        Self {
            bursts: [player_1, player_2],
            ..self
        }
    }
}
