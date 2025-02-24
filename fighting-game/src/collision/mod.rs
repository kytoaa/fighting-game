use crate::datatypes::{BoundingBox, BoundingCircle, BoundingShape, Vector2};

pub enum CollisionShape {
    Box(BoundingBox),
    Circle(BoundingCircle),
}

pub struct HitInfo {
    pub damage: u16,
    pub priority: usize,
    pub hitstun: usize,
    pub blockstun: usize,
    pub hit_effect: HitEffect,
    pub attack_type: AttackType,
}
#[derive(Clone, Copy)]
pub enum AttackType {
    High,
    Mid,
    Low,
}
pub enum HitEffect {
    Pushback(f32),
    Launcher(Vector2, KnockdownType),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KnockdownType {
    Hard,
    Soft,
}

pub struct Hurtbox {
    pub shape: CollisionShape,
    pub owner: usize,
}
pub struct Hitbox {
    pub shape: CollisionShape,
    pub info: HitInfo,
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
