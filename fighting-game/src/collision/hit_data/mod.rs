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
    pub const fn scale_damage(&self, value: u32) -> u32 {
        value * self.0 / 100
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

    pub(crate) damage: u32,
    pub(crate) attack_type: AttackType,
    pub(crate) block_pushback: f32,
    pub(crate) blockstun: usize,

    /// proration is a percentage, calculated by doing `damage * proration / 100`
    pub(crate) proration: Proration,

    /// increases scaling on hit, decreases on block, negative scaling reduces scaling of next combo
    pub(crate) scaling: i32,

    /// amount of meter gained on hit, on block is half
    pub(crate) meter_gain: u32,
    /// changes rate of meter gain, 1000 being default rate of gain
    pub(crate) meter_gain_modifier: i32,

    pub(crate) minimum_damage: u32,

    pub(crate) extensions: Box<[HitDataExtensions]>,
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
    pub hit_effect: HitEffect,

    pub damage: u32,
    pub attack_type: AttackType,
    pub block_pushback: f32,
    pub blockstun: usize,

    _p: std::marker::PhantomData<()>,
}

impl HitData {
    pub fn as_on_hit_hitdata(
        &self,
        grounded: bool,
        counterhit: bool,
        damage_scaling: impl FnOnce(u32) -> u32,
    ) -> OnHitHitData {
        OnHitHitData {
            hit_effect: match (grounded, counterhit) {
                (_, true) => self.counterhit.clone(),
                (false, false) => self.air.clone(),
                (true, false) => self.grounded.clone(),
            },
            damage: (damage_scaling)(self.damage),
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            _p: std::marker::PhantomData,
        }
    }
}
