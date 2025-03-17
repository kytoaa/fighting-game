use super::*;
use characters::Entity;
use datatypes::Vector2;
use input::InputHandler;
use std::sync::Arc;
use std::sync::Mutex;

pub enum Character {
    Sol,
}

pub fn create_world(
    player_1: (Character, Arc<Mutex<InputHandler>>),
    player_2: (Character, Arc<Mutex<InputHandler>>),
) -> world::World {
    let mut char_1 = get_character(player_1.0, 0);
    char_1.set_position(Vector2::new(-50.0, 0.0));

    let mut char_2 = get_character(player_2.0, 1);
    char_2.set_position(Vector2::new(50.0, 0.0));

    world::World::new([Some(char_1), Some(char_2)], [player_1.1, player_2.1])
}

fn get_character(character: Character, player: usize) -> Box<dyn Entity> {
    match character {
        Character::Sol => characters::sol::initial_state(player),
    }
}
