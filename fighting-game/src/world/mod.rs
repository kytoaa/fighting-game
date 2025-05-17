use super::collision::{CollisionShape, HitLevel, Hitbox, Hurtbox};
use super::datatypes::{BoundingShape, Vector2};
use super::input::InputHandler;

mod collision;
mod damaging;
mod physics;
mod players;

use damaging::ComboInfo;
use players::TrackedPlayerData;

const DELTA: f32 = 1.0 / 60.0;
const BORDER_X: f32 = 100.0;
const MIN_WALL_BOUNCE_HEIGHT: f32 = 4.0;

pub struct World {
    players: [Option<Box<dyn crate::characters::Entity>>; 2],
    player_data: [TrackedPlayerData; 2],

    combo: Option<ComboInfo>,

    hurtboxes: Vec<Spawn<Hurtbox>>,
    hitboxes: Vec<Spawn<Hitbox>>,

    id_counter: usize,

    hitstop_frames_left: usize,

    frame: usize,
}

struct Spawn<T>(T, usize);

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct EntityID(usize, EntityType);
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum EntityType {
    Unique,
    Owned(usize),
}
impl EntityID {
    pub const fn id(&self) -> usize {
        self.0
    }
    pub const fn is_player(&self) -> bool {
        self.id() < 2
    }
}

impl World {
    pub fn new(
        players: (
            impl FnOnce(
                EntityID,
            ) -> (
                Box<dyn crate::characters::Entity>,
                crate::characters::CharacterInitInfo,
            ),
            impl FnOnce(
                EntityID,
            ) -> (
                Box<dyn crate::characters::Entity>,
                crate::characters::CharacterInitInfo,
            ),
        ),
    ) -> Self {
        let ((a, a_info), (b, b_info)) = (
            (players.0)(EntityID(0, EntityType::Unique)),
            (players.1)(EntityID(1, EntityType::Unique)),
        );
        Self {
            players: [Some(a), Some(b)],
            player_data: [
                TrackedPlayerData::new(a_info.max_health),
                TrackedPlayerData::new(b_info.max_health),
            ],
            combo: None,

            hurtboxes: vec![],
            hitboxes: vec![],

            id_counter: 2,

            hitstop_frames_left: 0,

            frame: 0,
        }
    }
}

impl World {
    pub fn update(&mut self, input_providers: &[InputHandler]) {
        self.update_hitbox_hurtboxes();

        self.frame += 1;

        if self.hitstop_frames_left > 0 {
            self.hitstop_frames_left -= 1;
            return;
        }

        for i in 0..2 {
            let player = self.players[i].take().unwrap();
            let input_provider = &input_providers[i];
            let player = player.update(self, &input_provider);

            if let Some(combo) = &self.combo {
                if combo.target().id() == i && player.actionable() {
                    self.combo = None;
                }
            }
            _ = self.players[i].insert(player);
        }

        self.move_players();
    }

    pub fn spawn_hurtbox(&mut self, hurtbox: Hurtbox, position: Vector2) {
        self.hurtboxes.push(Spawn(hurtbox.at_position(position), 1));
    }
    pub fn spawn_hitbox(&mut self, hitbox: Hitbox, position: Vector2) {
        self.hitboxes.push(Spawn(hitbox.at_position(position), 1));
    }
    const fn is_grounded(&self, shape: &CollisionShape) -> bool {
        shape.get_bounding_box().min.y <= 0.01
    }
    const fn create_id(&mut self) -> EntityID {
        let n = self.id_counter;
        self.id_counter += 1;
        EntityID(n, EntityType::Unique)
    }

    pub fn get_players(&self) -> Box<[&dyn crate::characters::Entity; 2]> {
        Box::new([
            self.players[0].as_ref().unwrap().as_ref(),
            self.players[1].as_ref().unwrap().as_ref(),
        ])
    }
    pub fn get_hurtboxes(&self) -> impl Iterator<Item = &Hurtbox> {
        self.hurtboxes.iter().map(|s| &s.0)
    }
    pub fn get_hitboxes(&self) -> impl Iterator<Item = &Hitbox> {
        self.hitboxes.iter().map(|s| &s.0)
    }

    pub fn trigger_hitstop(&mut self, frames: usize) {
        self.hitstop_frames_left = frames.max(self.hitstop_frames_left);
    }

    pub const fn in_hitstop(&self) -> bool {
        self.hitstop_frames_left > 0
    }
}
