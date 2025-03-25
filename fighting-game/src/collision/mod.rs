use crate::datatypes::{BoundingBox, BoundingCircle, BoundingShape, Vector2};

pub enum CollisionShape {
    Box(BoundingBox),
    Circle(BoundingCircle),
}

#[derive(Clone, Copy)]
pub enum HitType {
    Light,
    Medium,
    Heavy,
    SuperHeavy,
    Custom(usize),
}
impl HitType {
    pub fn get_hitstop_frames(self) -> usize {
        match self {
            Self::Light => 4,
            Self::Medium => 8,
            Self::Heavy => 12,
            Self::SuperHeavy => 20,
            Self::Custom(frames) => frames,
        }
    }
    pub const BLOCKED_HITSTOP_FRAMES: usize = 4;
}

#[derive(Debug, Clone)]
pub struct HitInfo {
    pub damage: u16,
    pub hitstun: usize,
    pub blockstun: usize,
    pub hit_effect: HitEffect,
}

pub struct AttackData {
    pub grounded: HitInfo,
    pub air: HitInfo,
    pub priority: usize,
    pub attack_type: AttackType,
    pub hitbox_id: usize,
    pub hit_type: HitType,
}
impl AttackData {
    pub fn with_same_hitinfo(
        hit_info: HitInfo,
        priority: usize,
        attack_type: AttackType,
        hitbox_id: usize,
        hit_type: HitType,
    ) -> Self {
        Self {
            grounded: hit_info.clone(),
            air: hit_info,
            priority,
            attack_type,
            hitbox_id,
            hit_type,
        }
    }
}
#[derive(Clone, Copy)]
pub enum AttackType {
    High,
    Mid,
    Low,
}
#[derive(Debug, Clone)]
pub enum HitEffect {
    Pushback(f32),
    Launcher(Vector2, KnockdownType),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KnockdownType {
    Hard,
    Soft,
}

#[derive(Clone, Copy)]
pub enum HitConnection {
    Hit,
    Blocked,
    Invuln,
}

pub struct Hurtbox {
    pub shape: CollisionShape,
    pub owner: usize,
}
pub struct Hitbox {
    pub shape: CollisionShape,
    pub info: AttackData,
    pub owner: usize,
}

impl CollisionShape {
    pub fn overlaps(&self, other: &CollisionShape) -> bool {
        match (&self, &other) {
            (CollisionShape::Box(b), CollisionShape::Box(b2)) => b.intersects(b2),
            (CollisionShape::Box(b), CollisionShape::Circle(c)) => b.intersects(c),
            (CollisionShape::Circle(c), CollisionShape::Box(b)) => c.intersects(b),
            (CollisionShape::Circle(c), CollisionShape::Circle(c2)) => c.intersects(c2),
        }
    }
    pub fn at_position(self, position: Vector2) -> Self {
        match self {
            CollisionShape::Box(b) => CollisionShape::Box(b.transformed(position)),
            CollisionShape::Circle(c) => CollisionShape::Circle(c.transformed(position)),
        }
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
