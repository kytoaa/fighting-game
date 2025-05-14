use super::*;
use macros::Builder;
use std::marker::PhantomData;

#[derive(Builder)]
struct LauncherBuilder {
    knockback: Vector2,
    gravity: f32,
    knockdown: KnockdownType,
    momentum_scaling: (f32, f32),
    ground_bounce_velocity: Option<Vector2>,
    wall_bounce_velocity: Option<Vector2>,
}
impl LauncherBuilder {
    pub const fn build(self) -> HitEffect {
        HitEffect::Launcher {
            knockback: self.knockback,
            gravity: self.gravity,
            knockdown: self.knockdown,
            momentum_scaling: self.momentum_scaling,
            ground_bounce_velocity: self.ground_bounce_velocity,
            wall_bounce_velocity: self.wall_bounce_velocity,
        }
    }
}

#[derive(Builder)]
struct FloatingCrumpleBuilder {
    knockback: Vector2,
    gravity: f32,
    landing_frames: usize,
}
impl FloatingCrumpleBuilder {
    pub const fn build(self) -> HitEffect {
        HitEffect::FloatingCrumple {
            knockback: self.knockback,
            gravity: self.gravity,
            landing_frames: self.landing_frames,
        }
    }
}

#[derive(Builder)]
struct PushbackBuilder {
    force: f32,
    frames: usize,
}
impl PushbackBuilder {
    pub const fn build(self) -> HitEffect {
        HitEffect::Pushback {
            force: self.force,
            frames: self.frames,
        }
    }
}

impl HitEffect {
    pub const fn launcher(knockback: Vector2, knockdown: KnockdownType) -> LauncherBuilder {
        LauncherBuilder {
            knockback,
            knockdown,
            gravity: 9.8,
            momentum_scaling: (0.0, 0.0),
            ground_bounce_velocity: None,
            wall_bounce_velocity: None,
        }
    }
    pub const fn floating_crumple(
        knockback: Vector2,
        gravity: f32,
        landing_frames: usize,
    ) -> FloatingCrumpleBuilder {
        FloatingCrumpleBuilder {
            knockback,
            gravity,
            landing_frames,
        }
    }
}

struct Grounded;
struct Air;
struct Counterhit;
struct Undefined;

struct HitDataBuilder<G, A, C> {
    base_effect: HitEffect,
    grounded: Option<HitEffect>,
    air: Option<HitEffect>,
    counterhit: Option<HitEffect>,
    damage: u32,
    block_pushback: f32,
    proration: u8,
    scaling: i32,
    minimum_damage: u32,
    extensions: Vec<HitDataExtensions>,
    _pd: PhantomData<(G, A, C)>,
}
impl<G, A, C> HitDataBuilder<G, A, C> {
    pub const fn minimum_damage(mut self, value: u32) -> Self {
        self.minimum_damage = value;
        self
    }
    pub const fn block_pushback(mut self, value: f32) -> Self {
        self.block_pushback = value;
        self
    }
    pub fn add_extension(mut self, value: HitDataExtensions) -> Self {
        self.extensions.push(value);
        self
    }
}
impl<A, C> HitDataBuilder<Undefined, A, C> {
    pub fn with_grounded(self, effect: HitEffect) -> HitDataBuilder<Grounded, A, C> {
        HitDataBuilder::<Grounded, A, C> {
            base_effect: self.base_effect,
            grounded: Some(effect),
            air: self.air,
            counterhit: self.counterhit,
            damage: self.damage,
            block_pushback: self.block_pushback,
            proration: self.proration,
            scaling: self.scaling,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G, C> HitDataBuilder<G, Undefined, C> {
    pub fn with_air(self, effect: HitEffect) -> HitDataBuilder<G, Air, C> {
        HitDataBuilder::<G, Air, C> {
            base_effect: self.base_effect,
            grounded: self.grounded,
            air: Some(effect),
            counterhit: self.counterhit,
            damage: self.damage,
            block_pushback: self.block_pushback,
            proration: self.proration,
            scaling: self.scaling,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G, A> HitDataBuilder<G, A, Undefined> {
    pub fn with_counterhit(self, effect: HitEffect) -> HitDataBuilder<G, A, Counterhit> {
        HitDataBuilder::<G, A, Counterhit> {
            base_effect: self.base_effect,
            grounded: self.grounded,
            air: self.air,
            counterhit: Some(effect),
            damage: self.damage,
            block_pushback: self.block_pushback,
            proration: self.proration,
            scaling: self.scaling,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}

impl HitData {
    pub fn grounded(
        damage: u32,
        effect: HitEffect,
        proration: u8,
        scaling: i32,
    ) -> HitDataBuilder<Grounded, Undefined, Undefined> {
        let dir = effect.x_force().signum();
        HitDataBuilder {
            base_effect: effect.clone(),
            grounded: Some(effect),
            air: None,
            counterhit: None,
            damage,
            block_pushback: 60.0 * dir,
            proration,
            scaling,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }
    pub fn air(
        damage: u32,
        effect: HitEffect,
        proration: u8,
        scaling: i32,
    ) -> HitDataBuilder<Undefined, Air, Undefined> {
        let dir = effect.x_force().signum();
        HitDataBuilder {
            base_effect: effect.clone(),
            grounded: Some(effect),
            air: None,
            counterhit: None,
            damage,
            block_pushback: 60.0 * dir,
            proration,
            scaling,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }
}
