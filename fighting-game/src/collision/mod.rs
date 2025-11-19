use crate::datatypes::{BoundingBox, BoundingShape, Vector2};
use crate::world::EntityID;

mod hit_data;
pub use hit_data::*;

#[derive(Debug, Clone)]
pub struct CollisionShape(BoundingBox);

impl CollisionShape {
    pub const fn get_bounding_box(&self) -> &BoundingBox {
        &self.0
    }
    pub const fn new(bounding_box: BoundingBox) -> Self {
        Self(bounding_box)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttackID(u64);
impl AttackID {
    pub fn new<T: std::hash::Hash>(value: T) -> AttackID {
        use std::hash::Hasher;

        let mut s = std::hash::DefaultHasher::new();
        value.hash(&mut s);
        Self(s.finish())
    }
}
impl Into<AttackID> for &str {
    fn into(self) -> AttackID {
        AttackID::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct AttackData {
    pub attack: HitData,
    pub priority: usize,
    pub hitbox_id: usize,
    pub hit_level: HitLevel,
    pub attack_id: AttackID,
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

#[derive(Debug, Clone, Copy)]
pub enum HitConnectionStatus {
    Hit,
    Blocked,
    Invuln,
}

#[derive(Debug, Clone)]
pub(crate) struct Hurtbox {
    pub shape: CollisionShape,
    pub owner: EntityID,
}
#[derive(Debug, Clone)]
pub(crate) struct Hitbox {
    pub shape: CollisionShape,
    pub attack_data: AttackData,
    pub owner: EntityID,
}

pub(crate) struct ThrowBox {
    pub shape: CollisionShape,
    pub owner: EntityID,
    pub throw_success: Box<dyn crate::characters::Player>,
}
impl Clone for ThrowBox {
    fn clone(&self) -> Self {
        Self {
            shape: self.shape.clone(),
            owner: self.owner.clone(),
            throw_success: self.throw_success.clone(),
        }
    }
}
pub struct ThrowSuccess {}

impl CollisionShape {
    pub fn overlaps(&self, other: &CollisionShape) -> bool {
        self.0.intersects(&other.0)
    }
    pub fn at_position(self, position: Vector2) -> Self {
        CollisionShape(self.0.transformed(position))
    }
}

impl Hitbox {
    #[allow(dead_code)]
    pub const fn new(owner: EntityID, shape: CollisionShape, attack_data: AttackData) -> Self {
        Self {
            owner,
            shape,
            attack_data,
        }
    }
    pub fn at_position(mut self, position: Vector2) -> Self {
        self.shape = self.shape.at_position(position);
        self
    }
}
impl Hurtbox {
    #[allow(dead_code)]
    pub const fn new(owner: EntityID, shape: CollisionShape) -> Self {
        Self { owner, shape }
    }
    pub fn at_position(mut self, position: Vector2) -> Self {
        self.shape = self.shape.at_position(position);
        self
    }
}
impl ThrowBox {
    pub fn at_position(mut self, position: Vector2) -> Self {
        self.shape = self.shape.at_position(position);
        self
    }
}
