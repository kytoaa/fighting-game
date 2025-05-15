use crate::datatypes::{BoundingBox, BoundingShape, Vector2};
use crate::world::EntityID;

mod hit_data;
pub use hit_data::*;

pub struct CollisionShape(BoundingBox);

#[derive(Debug, Clone)]
pub struct AttackData {
    attack: HitData,
    hit_level: HitLevel,
}

#[derive(Debug, Clone, Copy)]
pub enum HitLevel {
    Light,
    Medium,
    Heavy,
    SuperHeavy,
    Custom(usize),
}

impl HitLevel {
    pub fn get_hitstop_frames(self) -> usize {
        match self {
            Self::Light => 8,
            Self::Medium => 10,
            Self::Heavy => 12,
            Self::SuperHeavy => 20,
            Self::Custom(frames) => frames,
        }
    }
    pub const BLOCKED_HITSTOP_FRAMES: usize = 8;
}

#[derive(Clone, Copy)]
pub enum HitConnection {
    Hit,
    Blocked,
    Invuln,
}

pub struct Hurtbox {
    pub shape: CollisionShape,
    pub owner: EntityID,
}
pub struct Hitbox {
    shape: CollisionShape,
    info: AttackData,
    owner: EntityID,
}

impl CollisionShape {
    pub fn overlaps(&self, other: &CollisionShape) -> bool {
        self.0.intersects(&other.0)
    }
    pub fn at_position(self, position: Vector2) -> Self {
        CollisionShape(self.0.transformed(position))
    }
}
impl Hitbox {
    pub fn at_position(mut self, position: Vector2) -> Self {
        self.shape = self.shape.at_position(position);
        self
    }
}
impl Hurtbox {
    pub fn at_position(mut self, position: Vector2) -> Self {
        self.shape = self.shape.at_position(position);
        self
    }
}
