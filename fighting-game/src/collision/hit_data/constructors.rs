use super::*;
use macros::Builder;
use std::marker::PhantomData;

#[derive(Builder)]
struct LauncherBuilder {
    knockback: Vector2,
    gravity: f32,
    knockdown: KnockdownType,
    momentum_scaling: (f32, f32),
    #[no_builder]
    ground_bounce_velocity: Option<Vector2>,
    #[no_builder]
    wall_bounce_velocity: Option<Vector2>,
}
impl LauncherBuilder {
    pub const fn ground_bounce_velocity(mut self, value: Vector2) -> Self {
        self.ground_bounce_velocity = Some(value);
        self
    }
    pub const fn wall_bounce_velocity(mut self, value: Vector2) -> Self {
        self.wall_bounce_velocity = Some(value);
        self
    }
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
            gravity: 9.0,
            momentum_scaling: (0.15, 0.15),
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
    pub const fn pushback(force: f32, frames: usize) -> PushbackBuilder {
        PushbackBuilder { force, frames }
    }
}

struct Grounded;
struct Air;
struct Counterhit;
struct Undefined;
struct Default;
trait Changeable {}
impl Changeable for Undefined {}
impl Changeable for Default {}
trait Confirmable {}
impl Confirmable for Grounded {}
impl Confirmable for Air {}
impl Confirmable for Counterhit {}
impl Confirmable for Default {}

struct HitDataBuilder<G, A, C> {
    grounded: Option<HitEffect>,
    air: Option<HitEffect>,
    counterhit: Option<HitEffect>,

    damage: u32,
    attack_type: AttackType,
    block_pushback: f32,
    blockstun: usize,

    proration: Proration,
    scaling: i32,
    meter_gain: u32,
    meter_gain_modifier: i32,
    minimum_damage: u32,
    extensions: Vec<HitDataExtensions>,
    _pd: PhantomData<(G, A, C)>,
}
impl<G, A, C> HitDataBuilder<G, A, C> {
    pub const fn attack_type(mut self, value: AttackType) -> Self {
        self.attack_type = value;
        self
    }
    pub const fn block_pushback(mut self, value: f32) -> Self {
        self.block_pushback = value;
        self
    }
    pub const fn meter_gain(mut self, value: u32) -> Self {
        self.meter_gain = value;
        self
    }
    pub const fn meter_gain_modifier(mut self, value: i32) -> Self {
        self.meter_gain_modifier = value;
        self
    }
    pub const fn minimum_damage(mut self, value: u32) -> Self {
        self.minimum_damage = value;
        self
    }
    pub fn add_extension(mut self, value: HitDataExtensions) -> Self {
        self.extensions.push(value);
        self
    }
}
impl<A, C, U: Changeable> HitDataBuilder<U, A, C> {
    pub fn with_grounded(self, effect: HitEffect) -> HitDataBuilder<Grounded, A, C> {
        HitDataBuilder::<Grounded, A, C> {
            grounded: Some(effect),
            air: self.air,
            counterhit: self.counterhit,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            proration: self.proration,
            scaling: self.scaling,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G, C, U: Changeable> HitDataBuilder<G, U, C> {
    pub fn with_air(self, effect: HitEffect) -> HitDataBuilder<G, Air, C> {
        HitDataBuilder::<G, Air, C> {
            grounded: self.grounded,
            air: Some(effect),
            counterhit: self.counterhit,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            proration: self.proration,
            scaling: self.scaling,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G, A, U: Changeable> HitDataBuilder<G, A, U> {
    pub fn with_counterhit(self, effect: HitEffect) -> HitDataBuilder<G, A, Counterhit> {
        HitDataBuilder::<G, A, Counterhit> {
            grounded: self.grounded,
            air: self.air,
            counterhit: Some(effect),

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            proration: self.proration,
            scaling: self.scaling,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<A, U: Changeable> HitDataBuilder<Grounded, A, U> {
    pub fn counterhit_from_grounded<F>(
        self,
        f: impl Fn(HitEffect) -> HitEffect,
    ) -> HitDataBuilder<Grounded, A, Counterhit> {
        HitDataBuilder::<Grounded, A, Counterhit> {
            grounded: self.grounded.clone(),
            air: self.air,
            counterhit: Some((f)(self.grounded.unwrap())),

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            proration: self.proration,
            scaling: self.scaling,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G, U: Changeable> HitDataBuilder<G, Air, U> {
    pub fn counterhit_from_air<F>(
        self,
        f: impl Fn(HitEffect) -> HitEffect,
    ) -> HitDataBuilder<G, Air, Counterhit> {
        HitDataBuilder::<G, Air, Counterhit> {
            grounded: self.grounded,
            air: self.air.clone(),
            counterhit: Some((f)(self.air.unwrap())),

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            proration: self.proration,
            scaling: self.scaling,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<C, U: Changeable> HitDataBuilder<Grounded, U, C> {
    pub fn air_from_grounded<F>(
        self,
        f: impl Fn(HitEffect) -> HitEffect,
    ) -> HitDataBuilder<Grounded, Air, C> {
        HitDataBuilder::<Grounded, Air, C> {
            grounded: self.grounded.clone(),
            air: Some((f)(self.grounded.unwrap())),
            counterhit: self.counterhit,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            proration: self.proration,
            scaling: self.scaling,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<C, U: Changeable> HitDataBuilder<U, Air, C> {
    pub fn grounded_from_air<F>(
        self,
        f: impl Fn(HitEffect) -> HitEffect,
    ) -> HitDataBuilder<Grounded, Air, C> {
        HitDataBuilder::<Grounded, Air, C> {
            grounded: Some((f)(self.air.clone().unwrap())),
            air: self.air,
            counterhit: self.counterhit,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            proration: self.proration,
            scaling: self.scaling,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G: Confirmable, A: Confirmable, C: Confirmable> HitDataBuilder<G, A, C> {
    pub fn build(self) -> HitData {
        HitData {
            grounded: self.grounded.unwrap(),
            air: self.air.unwrap(),
            counterhit: self.counterhit.unwrap(),

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            proration: self.proration,
            scaling: self.scaling,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions.into(),
        }
    }
}

impl HitData {
    pub fn grounded(
        damage: u32,
        effect: HitEffect,
        blockstun: usize,
        proration: Proration,
        scaling: i32,
    ) -> HitDataBuilder<Grounded, Undefined, Undefined> {
        let dir = effect.x_force().signum();
        HitDataBuilder {
            grounded: Some(effect),
            air: None,
            counterhit: None,

            damage,
            attack_type: AttackType::Mid,
            block_pushback: 60.0 * dir,
            blockstun,

            proration,
            scaling,
            meter_gain: 100,
            meter_gain_modifier: 0,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }
    pub fn air(
        damage: u32,
        effect: HitEffect,
        blockstun: usize,
        proration: Proration,
        scaling: i32,
    ) -> HitDataBuilder<Undefined, Air, Undefined> {
        let dir = effect.x_force().signum();
        HitDataBuilder {
            grounded: Some(effect),
            air: None,
            counterhit: None,

            damage,
            attack_type: AttackType::Mid,
            block_pushback: 60.0 * dir,
            blockstun,

            proration,
            scaling,
            meter_gain: 100,
            meter_gain_modifier: 0,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }
    pub fn level_1(
        damage: u32,
        launch_force: Vector2,
    ) -> HitDataBuilder<Default, Default, Undefined> {
        HitDataBuilder {
            grounded: Some(HitEffect::pushback(50.0, 14).build()),
            air: Some(HitEffect::launcher(launch_force, KnockdownType::Soft).build()),
            counterhit: None,

            damage,
            attack_type: AttackType::Mid,
            block_pushback: 40.0,
            blockstun: 11,

            proration: Proration::percent(90),
            scaling: 1500,
            meter_gain: 100,
            meter_gain_modifier: 0,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }
    pub fn level_2(
        damage: u32,
        launch_force: Vector2,
    ) -> HitDataBuilder<Default, Default, Undefined> {
        HitDataBuilder {
            grounded: Some(HitEffect::pushback(55.0, 16).build()),
            air: Some(HitEffect::launcher(launch_force, KnockdownType::Soft).build()),
            counterhit: None,

            damage,
            attack_type: AttackType::Mid,
            block_pushback: 40.0,
            blockstun: 13,

            proration: Proration::percent(90),
            scaling: 2000,
            meter_gain: 150,
            meter_gain_modifier: 0,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }
    pub fn level_3(
        damage: u32,
        launch_force: Vector2,
    ) -> HitDataBuilder<Default, Default, Undefined> {
        HitDataBuilder {
            grounded: Some(HitEffect::pushback(60.0, 19).build()),
            air: Some(HitEffect::launcher(launch_force, KnockdownType::Soft).build()),
            counterhit: None,

            damage,
            attack_type: AttackType::Mid,
            block_pushback: 40.0,
            blockstun: 16,

            proration: Proration::percent(90),
            scaling: 2500,
            meter_gain: 200,
            meter_gain_modifier: 0,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }
    pub fn level_4(
        damage: u32,
        launch_force: Vector2,
    ) -> HitDataBuilder<Default, Default, Undefined> {
        HitDataBuilder {
            grounded: Some(HitEffect::pushback(65.0, 21).build()),
            air: Some(HitEffect::launcher(launch_force, KnockdownType::Soft).build()),
            counterhit: None,

            damage,
            attack_type: AttackType::Mid,
            block_pushback: 40.0,
            blockstun: 18,

            proration: Proration::percent(90),
            scaling: 3000,
            meter_gain: 250,
            meter_gain_modifier: 0,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }
}
