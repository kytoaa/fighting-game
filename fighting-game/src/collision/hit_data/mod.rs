use super::*;

mod constructors;

#[derive(Debug, Clone, Copy)]
pub enum AttackType {
    High,
    Mid,
    Low,
    Unblockable,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KnockdownType {
    Hard,
    Soft,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Proration(u32);
impl Proration {
    pub const fn percent(value: u32) -> Proration {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HitEffect {
    Launcher {
        knockback: Vector2,
        gravity: f32,
        knockdown: KnockdownType,
        momentum_scaling: (f32, f32),

        /// momentum off the wall, facing away from it
        ground_bounce_velocity: Option<Vector2>,
        wall_bounce_velocity: Option<Vector2>,
    },
    FloatingCrumple {
        knockback: Vector2,
        gravity: f32,
        landing_frames: usize,
    },
    Pushback {
        force: f32,
        frames: usize,
    },
}
impl HitEffect {
    pub const fn x_force(&self) -> f32 {
        match self {
            Self::Launcher { knockback, .. } => knockback.x,
            Self::FloatingCrumple { knockback, .. } => knockback.x,
            Self::Pushback { force, .. } => *force,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HitData {
    grounded: HitEffect,
    air: HitEffect,
    counterhit: HitEffect,

    damage: u32,
    attack_type: AttackType,
    block_pushback: f32,
    blockstun: usize,

    /// proration is a percentage, calculated by doing `damage * proration / 100`
    proration: Proration,

    /// increases scaling on hit, decreases on block, negative scaling reduces scaling of next combo
    scaling: i32,

    /// amount of meter gained on hit, on block is half
    meter_gain: u32,
    /// changes rate of meter gain, 1000 being default rate of gain
    meter_gain_modifier: i32,

    minimum_damage: u32,

    extensions: Box<[HitDataExtensions]>,
}

#[derive(PartialEq, Debug, Clone)]
enum HitDataExtensions {
    SetScaling(i32),
    SetProration(Proration),
    SetChipDamage(u32),
    CleanHit(BoundingBox, HitEffect),
}

#[derive(Debug, Clone)]
pub struct OnHitHitData {
    hit_effect: HitEffect,

    damage: u32,
    attack_type: AttackType,
    block_pushback: f32,
    blockstun: usize,
}
