use super::*;

mod constructors;

#[derive(Debug, Clone)]
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

struct HitData {
    grounded: HitEffect,
    air: HitEffect,
    counterhit: HitEffect,

    damage: u32,
    block_pushback: f32,

    /// proration is a percentage, calculated by doing `damage * proration / 100`
    proration: u32,

    /// increases scaling on hit, decreases on block, negative scaling reduces scaling of next combo
    scaling: i32,

    minimum_damage: u32,

    extensions: std::collections::HashSet<HitDataExtensions>,
}

enum HitDataExtensions {
    SetScaling(i32),
    SetProration(u8),
    SetChipDamage(u32),
    CleanHit(BoundingBox, HitEffect),
}

struct AttackInfo {
    ground: HitData,
    air: HitData,
    counterhit: HitData,
}
