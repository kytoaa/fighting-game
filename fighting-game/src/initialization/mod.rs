use super::*;
use characters::Player;
use datatypes::Vector2;
use input::InputHandler;
use std::sync::Arc;
use std::sync::Mutex;

pub enum Character {
    Sol,
}

pub(crate) fn create_world(player_1: Character, player_2: Character) -> world::World {
    world::World::new((
        |id| get_character(player_1, Vector2::new(-50.0, 0.0), id),
        |id| get_character(player_2, Vector2::new(50.0, 0.0), id),
    ))
}

fn get_character(
    character: Character,
    position: Vector2,
    id: world::EntityID,
) -> (Box<dyn Player>, characters::CharacterInitInfo) {
    match character {
        Character::Sol => (
            Box::new(characters::sol::initial_state(id, position)),
            characters::sol::init_info(),
        ),
    }
}
