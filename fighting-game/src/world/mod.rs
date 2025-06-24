use super::collision::{CollisionShape, Hitbox, Hurtbox, ThrowBox};
use super::datatypes::Vector2;
use super::input::{Action, Button, InputHandler};

mod collision;
mod damaging;
mod physics;
mod players;

use damaging::ComboInfo;
use players::TrackedPlayerData;

const MAX_GAME_FRAMES: usize = 60 * 100;

const DELTA: f32 = 1.0 / 60.0;
const BORDER_X: f32 = 100.0;
const MIN_WALL_BOUNCE_HEIGHT: f32 = 0.0;

impl EntityID {
    pub const PLAYER_1_ID: EntityID = EntityID(0, EntityType::Unique);
    pub const PLAYER_2_ID: EntityID = EntityID(1, EntityType::Unique);
}
const CANCEL_ACTION: Action = Action::MultiplePress(Button::Mid, Button::Heavy);

pub(crate) struct World {
    players: [Option<Box<dyn crate::characters::Player>>; 2],
    player_data: [TrackedPlayerData; 2],

    combo: Option<ComboInfo>,

    non_player_entities:
        Option<std::collections::HashMap<usize, Box<dyn crate::characters::NonPlayerEntity>>>,

    hurtboxes: Vec<Spawn<Hurtbox>>,
    hitboxes: Vec<Spawn<Hitbox>>,
    throwboxes: Vec<ThrowBox>,

    id_counter: usize,

    hitstop_frames_left: usize,
    superfreeze_frames_left: usize,

    frame: usize,
}

#[derive(Debug)]
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
    pub const fn other_player(&self) -> Self {
        match self {
            EntityID(0, EntityType::Unique) => EntityID(1, EntityType::Unique),
            EntityID(1, EntityType::Unique) => EntityID(0, EntityType::Unique),
            EntityID(_, EntityType::Owned(0)) => EntityID(1, EntityType::Unique),
            EntityID(_, EntityType::Owned(1)) => EntityID(0, EntityType::Unique),
            _ => unreachable!(),
        }
    }
}

impl World {
    pub(crate) fn new(
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
        let ((a, a_info), (b, b_info)) = (
            (players.0)(EntityID::PLAYER_1_ID),
            (players.1)(EntityID::PLAYER_2_ID),
        );
        Self {
            players: [Some(a), Some(b)],
            player_data: [
                TrackedPlayerData::new(a_info.max_health, a_info.burst),
                TrackedPlayerData::new(b_info.max_health, a_info.burst),
            ],
            combo: None,

            non_player_entities: Some(Default::default()),

            hurtboxes: vec![],
            hitboxes: vec![],
            throwboxes: vec![],

            id_counter: 2,

            hitstop_frames_left: 0,
            superfreeze_frames_left: 0,

            frame: 0,
        }
    }
}

impl World {
    pub(crate) fn update(&mut self, input_providers: &[InputHandler]) -> crate::GameStatus {
        let throw = self.update_throw_boxes();

        if !throw {
            self.update_hitbox_hurtboxes();
        } else {
            self.decrement_hitbox_hurtbox_frame_timers();
        }

        if self.hitstop_frames_left > 0 {
            self.hitstop_frames_left -= 1;
            return crate::GameStatus::Running {
                frames_left: MAX_GAME_FRAMES.saturating_sub(self.frame),
            };
        }
        if self.superfreeze_frames_left > 0 {
            self.superfreeze_frames_left -= 1;
            return crate::GameStatus::Running {
                frames_left: MAX_GAME_FRAMES.saturating_sub(self.frame),
            };
        }

        self.frame += 1;

        let mut non_player_entities: std::collections::HashMap<
            usize,
            Box<dyn crate::characters::NonPlayerEntity>,
        > = self
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

        if let Some(e) = self.non_player_entities.take() {
            e.into_iter().for_each(|(id, entity)| {
                non_player_entities.insert(id, entity);
            });
        }
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
            } else if check_for_burst(input_provider) && self.try_spend_burst(player.id()) {
                println!("BURST");
                self.trigger_superfreeze(15);

                self.burst(player.as_ref());

                player.cancel_state()
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

        if self.player_data.iter().all(|d| d.health == 0) {
            return crate::GameStatus::Draw;
        }

        for (i, player_data) in self.player_data.iter().enumerate() {
            if player_data.health == 0 {
                return crate::GameStatus::RoundWon(i);
            }
        }

        if self.frame == MAX_GAME_FRAMES {
            return crate::GameStatus::RoundWon(
                self.player_data
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, d)| d.health)
                    .map(|(i, _)| i)
                    .unwrap(),
            );
        }

        return crate::GameStatus::Running {
            frames_left: MAX_GAME_FRAMES.saturating_sub(self.frame),
        };
    }

    pub(crate) fn spawn_hurtbox(&mut self, hurtbox: Hurtbox, position: Vector2) {
        self.hurtboxes.push(Spawn(hurtbox.at_position(position), 1));
    }
    pub(crate) fn spawn_hitbox(&mut self, hitbox: Hitbox, position: Vector2) {
        self.hitboxes.push(Spawn(hitbox.at_position(position), 1));
    }
    pub(crate) fn spawn_throwbox(&mut self, throwbox: ThrowBox, position: Vector2) {
        self.throwboxes.push(throwbox.at_position(position));
    }
    pub(crate) fn spawn_non_player_entity(
        &mut self,
        entity: Box<dyn crate::characters::NonPlayerEntity>,
    ) {
        match self.non_player_entities.as_mut() {
            Some(e) => _ = e.insert(entity.id().id(), entity),
            None => {
                self.non_player_entities = Some(std::collections::HashMap::from([(
                    entity.id().id(),
                    entity,
                )]))
            }
        }
    }
    const fn is_grounded(&self, shape: &CollisionShape) -> bool {
        shape.get_bounding_box().min.y <= 0.01
    }

    pub(crate) fn get_players(&self) -> Box<[&dyn crate::characters::Player; 2]> {
        Box::new([
            self.players[0].as_ref().unwrap().as_ref(),
            self.players[1].as_ref().unwrap().as_ref(),
        ])
    }
    pub(crate) fn get_non_player_entities(
        &self,
    ) -> impl Iterator<Item = &dyn crate::characters::NonPlayerEntity> {
        self.non_player_entities
            .as_ref()
            .unwrap()
            .values()
            .map(|e| e.as_ref())
    }
    pub(crate) fn get_hurtboxes(&self) -> impl Iterator<Item = &Hurtbox> {
        self.hurtboxes.iter().map(|s| &s.0)
    }
    pub(crate) fn get_hitboxes(&self) -> impl Iterator<Item = &Hitbox> {
        self.hitboxes.iter().map(|s| &s.0)
    }
    pub(crate) fn get_throwboxes(&self) -> impl Iterator<Item = &ThrowBox> {
        self.throwboxes.iter().map(|s| s)
    }

    pub(crate) fn trigger_hitstop(&mut self, frames: usize) {
        self.hitstop_frames_left = frames.max(self.hitstop_frames_left);
    }
    pub(crate) fn trigger_superfreeze(&mut self, frames: usize) {
        self.superfreeze_frames_left = frames.max(self.superfreeze_frames_left);
    }

    pub(crate) const fn in_hitstop(&self) -> bool {
        self.hitstop_frames_left > 0
    }
    pub(crate) const fn in_superfreeze(&self) -> bool {
        self.superfreeze_frames_left > 0
    }

    pub(crate) fn create_new_entity_id(&mut self, entity_type: EntityType) -> EntityID {
        let id = self.id_counter;
        self.id_counter += 1;
        EntityID(id, entity_type)
    }
    pub(crate) fn try_spend_meter(&mut self, player: EntityID, meter_cost: u32) -> bool {
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
    pub(crate) fn try_spend_burst(&mut self, player: EntityID) -> bool {
        if !player.is_player() {
            panic!();
        }
        if self.player_data[player.id()].burst_meter == TrackedPlayerData::BURST_MAX {
            self.player_data[player.id()].burst_meter = 0;
            true
        } else {
            false
        }
    }
    pub(crate) fn burst(&mut self, player: &dyn crate::characters::Player) {
        let id = player.id();
        let other_player = self.players[id.other_player().id()].take().unwrap();
        let dir = other_player.position() - player.position();

        let p = if dir.magnitude() <= 30.0 {
            let (p, _) = other_player.hit(
                crate::collision::HitData::grounded(
                    0,
                    crate::collision::HitEffect::launcher(
                        Vector2::new(dir.x.signum() * 200.0, 150.0),
                        crate::collision::KnockdownType::Soft,
                    )
                    .build(),
                    0,
                    crate::collision::Proration::percent(100),
                    0,
                )
                .block_pushback(30.0 * dir.x.signum())
                .attack_type(crate::collision::AttackType::Mid)
                .air_from_grounded(|g| g)
                .counterhit_ground_from_grounded(|g| g)
                .counterhit_air_from_air(|g| g)
                .build()
                .as_on_hit_hitdata(false, false, |_| 0),
            );

            p
        } else {
            other_player
        };

        _ = self.players[id.other_player().id()].insert(p);
    }
    pub(crate) fn set_entity_position(&mut self, entity: EntityID, position: Vector2) {
        if entity.is_player() {
            self.players[entity.id()]
                .as_mut()
                .unwrap()
                .set_position(position)
        } else {
            self.non_player_entities
                .as_mut()
                .unwrap()
                .get_mut(&entity.id())
                .unwrap()
                .set_position(position)
        }
    }
    pub(crate) fn get_entity_position(&self, entity: EntityID) -> Vector2 {
        if entity.is_player() {
            self.players[entity.id()].as_ref().unwrap().position()
        } else {
            self.non_player_entities
                .as_ref()
                .unwrap()
                .get(&entity.id())
                .unwrap()
                .position()
        }
    }
    pub(crate) fn set_entity_velocity(&mut self, entity: EntityID, velocity: Vector2) {
        if entity.is_player() {
            self.players[entity.id()]
                .as_mut()
                .unwrap()
                .set_velocity(velocity)
        }
    }
    pub(crate) fn get_entity_velocity(&self, entity: EntityID) -> Vector2 {
        if entity.is_player() {
            self.players[entity.id()].as_ref().unwrap().velocity()
        } else {
            Vector2::ZERO
        }
    }
    pub(crate) fn get_player_state(&self, player: EntityID) -> crate::PlayerState {
        if !player.is_player() {
            panic!()
        }
        let player_data = &self.player_data[player.id()];
        crate::PlayerState {
            health_percent: player_data.health as f32 / player_data.max_health as f32,
            burst_percent: player_data.burst_meter as f32 / TrackedPlayerData::BURST_MAX as f32,
            scaling_percent: if player_data.scaling < 0 {
                player_data.scaling as f32 / 10000.0
            } else {
                player_data.scaling as f32 / 20000.0
            },
            meter_percent: player_data.meter as f32 / 10000.0,
            combo_damage_health_percent: self
                .combo
                .as_ref()
                .map(|combo| {
                    if *combo.target() == player && player_data.health != 0 {
                        Some(combo.total_damage() as f32 / player_data.max_health as f32)
                    } else {
                        None
                    }
                })
                .flatten(),
        }
    }
    pub(crate) fn get_combo_hits(&self) -> Option<usize> {
        self.combo.as_ref().map(|c| c.hits())
    }
}

fn check_for_burst(input: &InputHandler) -> bool {
    input.has_action(&Action::Pressed(Button::Utility, None))
        && input.has_action(&Action::Pressed(Button::Heavy, None))
}
