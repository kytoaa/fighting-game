use super::collision::{CollisionShape, Hitbox, Hurtbox};
use super::datatypes::Vector2;
use super::input::{Action, Button, InputHandler};

mod collision;
mod damaging;
mod physics;
mod players;

use damaging::ComboInfo;
use players::TrackedPlayerData;

const DELTA: f32 = 1.0 / 60.0;
const BORDER_X: f32 = 100.0;
const MIN_WALL_BOUNCE_HEIGHT: f32 = 0.0;

const PLAYER_1_ID: EntityID = EntityID(0, EntityType::Unique);
const PLAYER_2_ID: EntityID = EntityID(1, EntityType::Unique);
const CANCEL_ACTION: Action = Action::MultiplePress(Button::Mid, Button::Heavy);

pub struct World {
    players: [Option<Box<dyn crate::characters::Player>>; 2],
    player_data: [TrackedPlayerData; 2],

    combo: Option<ComboInfo>,

    non_player_entities:
        Option<std::collections::HashMap<usize, Box<dyn crate::characters::NonPlayerEntity>>>,

    hurtboxes: Vec<Spawn<Hurtbox>>,
    hitboxes: Vec<Spawn<Hitbox>>,

    id_counter: usize,

    hitstop_frames_left: usize,
    superfreeze_frames_left: usize,

    frame: usize,
}

#[derive(Debug)]
struct Spawn<T>(T, usize);

pub(super) struct UpdateInfo {
    pub(super) reset_input: bool,
}

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
    pub const fn owned_by(&self, other: &EntityID) -> bool {
        match (self.1, other.1) {
            (EntityType::Owned(_), EntityType::Owned(_)) => false,
            (EntityType::Owned(p), EntityType::Unique) => p == other.0,
            (EntityType::Unique, EntityType::Owned(p)) => p == other.0,
            (EntityType::Unique, EntityType::Unique) => false,
        }
    }
    pub const fn get_owner(&self) -> usize {
        match self.1 {
            EntityType::Unique => self.0,
            EntityType::Owned(p) => p,
        }
    }
}

impl World {
    pub fn new(
        players: (
            impl FnOnce(
                EntityID,
            ) -> (
                Box<dyn crate::characters::Player>,
                crate::characters::CharacterInitInfo,
            ),
            impl FnOnce(
                EntityID,
            ) -> (
                Box<dyn crate::characters::Player>,
                crate::characters::CharacterInitInfo,
            ),
        ),
    ) -> Self {
        let ((a, a_info), (b, b_info)) = ((players.0)(PLAYER_1_ID), (players.1)(PLAYER_2_ID));
        Self {
            players: [Some(a), Some(b)],
            player_data: [
                TrackedPlayerData::new(a_info.max_health),
                TrackedPlayerData::new(b_info.max_health),
            ],
            combo: None,

            non_player_entities: Some(Default::default()),

            hurtboxes: vec![],
            hitboxes: vec![],

            id_counter: 2,

            hitstop_frames_left: 0,
            superfreeze_frames_left: 0,

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
        if self.superfreeze_frames_left > 0 {
            self.superfreeze_frames_left -= 1;
            return;
        }

        let non_player_entities = self
            .non_player_entities
            .take()
            .unwrap()
            .into_iter()
            .filter_map(|(hashmap_id, mut entity)| {
                let id = entity.id();
                let input_handler = match id.1 {
                    EntityType::Owned(player) if player < 2 => Some(&input_providers[player]),
                    EntityType::Owned(p) => panic!("entity {} is owned by {}", id.0, p),
                    EntityType::Unique => None,
                };
                let result = entity.update(self, input_handler);

                match result {
                    crate::characters::EntityUpdateResult::Continue => Some((hashmap_id, entity)),
                    crate::characters::EntityUpdateResult::Remove => None,
                    crate::characters::EntityUpdateResult::ReplaceWith(e) => Some((e.id().id(), e)),
                }
            })
            .collect();

        _ = self.non_player_entities.insert(non_player_entities);

        for i in 0..2 {
            let player = self.players[i].take().unwrap();
            let input_provider = &input_providers[i];

            let player = if input_provider.has_action(&CANCEL_ACTION) && player.can_cancel() {
                if self.try_spend_meter(player.id(), TrackedPlayerData::CANCEL_COST) {
                    println!("CANCEL");
                    println!("{} meter remaining", self.player_data[i].meter);
                    self.trigger_superfreeze(20);

                    player.cancel_state()
                } else {
                    println!("NOT ENOUGH METER");
                    println!("{} meter remaining", self.player_data[i].meter);
                    player.update(self, &input_provider)
                }
            } else {
                player.update(self, &input_provider)
            };

            if let Some(combo) = &self.combo {
                if combo.target().id() == i && !player.in_hitstun() {
                    self.combo = None;
                    println!("combo reset");
                }
            }
            _ = self.players[i].insert(player);
        }

        self.update_player_meters();

        self.move_players();
    }

    pub fn spawn_hurtbox(&mut self, hurtbox: Hurtbox, position: Vector2) {
        self.hurtboxes.push(Spawn(hurtbox.at_position(position), 1));
    }
    pub fn spawn_hitbox(&mut self, hitbox: Hitbox, position: Vector2) {
        self.hitboxes.push(Spawn(hitbox.at_position(position), 1));
    }
    pub fn spawn_non_player_entity(&mut self, entity: Box<dyn crate::characters::NonPlayerEntity>) {
        self.non_player_entities
            .as_mut()
            .unwrap()
            .insert(entity.id().id(), entity);
    }
    const fn is_grounded(&self, shape: &CollisionShape) -> bool {
        shape.get_bounding_box().min.y <= 0.01
    }

    pub fn get_players(&self) -> Box<[&dyn crate::characters::Player; 2]> {
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
    pub fn trigger_superfreeze(&mut self, frames: usize) {
        self.superfreeze_frames_left = frames.max(self.superfreeze_frames_left);
    }

    pub const fn in_hitstop(&self) -> bool {
        self.hitstop_frames_left > 0
    }
    pub const fn in_superfreeze(&self) -> bool {
        self.superfreeze_frames_left > 0
    }

    pub fn create_new_entity_id(&mut self, entity_type: EntityType) -> EntityID {
        let id = self.id_counter;
        self.id_counter += 1;
        EntityID(id, entity_type)
    }
    pub fn try_spend_meter(&mut self, player: EntityID, meter_cost: u32) -> bool {
        if !player.is_player() {
            panic!();
        }
        if self.player_data[player.id()].meter < meter_cost {
            false
        } else {
            self.player_data[player.id()].meter -= meter_cost;
            true
        }
    }
}
