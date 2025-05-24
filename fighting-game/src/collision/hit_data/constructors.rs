#![allow(private_bounds)]

use super::*;
use macros::Builder;
use std::marker::PhantomData;

#[derive(Builder)]
pub struct LauncherBuilder {
    knockback: Vector2,
    gravity: f32,
    knockdown: KnockdownType,
    momentum_scaling: (f32, f32),
    #[no_builder]
    ground_bounce: Option<BounceInfo>,
    #[no_builder]
    wall_bounce: Option<BounceInfo>,
}
impl LauncherBuilder {
    pub const fn ground_bounce(mut self, value: BounceInfo) -> Self {
        self.ground_bounce = Some(value);
        self
    }
    pub const fn wall_bounce(mut self, value: BounceInfo) -> Self {
        self.wall_bounce = Some(value);
        self
    }
    pub const fn build(self) -> HitEffect {
        HitEffect::Launcher {
            knockback: self.knockback,
            gravity: self.gravity,
            knockdown: self.knockdown,
            momentum_scaling: self.momentum_scaling,
            ground_bounce: self.ground_bounce,
            wall_bounce: self.wall_bounce,
        }
    }
}

#[derive(Builder)]
pub struct FloatingCrumpleBuilder {
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
pub struct PushbackBuilder {
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
            gravity: DEFAULT_GRAVITY,
            momentum_scaling: (0.15, 0.15),
            ground_bounce: None,
            wall_bounce: None,
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

pub struct Grounded(());
pub struct Air(());
pub struct CounterhitGround(());
pub struct CounterhitAir(());
pub struct Undefined(());
pub struct Default(());
trait Changeable {}
impl Changeable for Undefined {}
impl Changeable for Default {}
trait Confirmable {}
impl Confirmable for Grounded {}
impl Confirmable for Air {}
impl Confirmable for CounterhitGround {}
impl Confirmable for CounterhitAir {}
impl Confirmable for Default {}

pub struct HitDataBuilder<G, A, CG, CA> {
    grounded: Option<HitEffect>,
    air: Option<HitEffect>,
    counterhit_ground: Option<HitEffect>,
    counterhit_air: Option<HitEffect>,

    damage: u32,
    attack_type: AttackType,
    block_pushback: f32,
    blockstun: usize,

    wall_pushback_mult: f32,

    proration: Proration,
    scaling: i32,
    scaling_on_block_mult: u32,
    meter_gain: u32,
    meter_gain_modifier: i32,
    minimum_damage: u32,
    extensions: Vec<HitDataExtension>,
    _pd: PhantomData<(G, A, CG, CA)>,
}
impl<G, A, CG, CA> HitDataBuilder<G, A, CG, CA> {
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
    pub const fn proration(mut self, value: Proration) -> Self {
        self.proration = value;
        self
    }
    pub const fn wall_pushback_mult(mut self, value: f32) -> Self {
        self.wall_pushback_mult = value;
        self
    }
    pub const fn scaling_on_block_mult(mut self, value: u32) -> Self {
        self.scaling_on_block_mult = value;
        self
    }
    pub fn add_extension(mut self, value: HitDataExtension) -> Self {
        self.extensions.push(value);
        self
    }
}
impl<A, CG, CA, G: Changeable> HitDataBuilder<G, A, CG, CA> {
    pub fn with_grounded(self, effect: HitEffect) -> HitDataBuilder<Grounded, A, CG, CA> {
        HitDataBuilder::<Grounded, A, CG, CA> {
            grounded: Some(effect),
            air: self.air,
            counterhit_ground: self.counterhit_ground,
            counterhit_air: self.counterhit_air,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G, CG, CA, A: Changeable> HitDataBuilder<G, A, CG, CA> {
    pub fn with_air(self, effect: HitEffect) -> HitDataBuilder<G, Air, CG, CA> {
        HitDataBuilder::<G, Air, CG, CA> {
            grounded: self.grounded,
            air: Some(effect),
            counterhit_ground: self.counterhit_ground,
            counterhit_air: self.counterhit_air,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G, A, CA, CG: Changeable> HitDataBuilder<G, A, CG, CA> {
    pub fn with_counterhit_ground(
        self,
        effect: HitEffect,
    ) -> HitDataBuilder<G, A, CounterhitGround, CA> {
        HitDataBuilder::<G, A, CounterhitGround, CA> {
            grounded: self.grounded,
            air: self.air,
            counterhit_ground: Some(effect),
            counterhit_air: self.counterhit_air,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G, A, CG, CA: Changeable> HitDataBuilder<G, A, CG, CA> {
    pub fn with_counterhit_air(self, effect: HitEffect) -> HitDataBuilder<G, A, CG, CounterhitAir> {
        HitDataBuilder::<G, A, CG, CounterhitAir> {
            grounded: self.grounded,
            air: self.air,
            counterhit_ground: self.counterhit_ground,
            counterhit_air: Some(effect),

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G, A, CG: Changeable, CA: Changeable> HitDataBuilder<G, A, CG, CA> {
    pub fn with_counterhit_ground_and_air(
        self,
        effect: HitEffect,
    ) -> HitDataBuilder<G, A, CounterhitGround, CounterhitAir> {
        HitDataBuilder::<G, A, CounterhitGround, CounterhitAir> {
            grounded: self.grounded,
            air: self.air,
            counterhit_ground: Some(effect.clone()),
            counterhit_air: Some(effect),

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G: Confirmable, A, CG: Changeable, CA> HitDataBuilder<G, A, CG, CA> {
    pub fn counterhit_ground_from_grounded(
        self,
        f: impl Fn(HitEffect) -> HitEffect,
    ) -> HitDataBuilder<G, A, CounterhitGround, CA> {
        HitDataBuilder::<G, A, CounterhitGround, CA> {
            grounded: self.grounded.clone(),
            air: self.air,
            counterhit_ground: Some((f)(self.grounded.unwrap())),
            counterhit_air: self.counterhit_air,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
    pub fn counterhit_ground_from_ground_default(
        self,
    ) -> HitDataBuilder<G, A, CounterhitGround, CA> {
        HitDataBuilder::<G, A, CounterhitGround, CA> {
            grounded: self.grounded.clone(),
            air: self.air,
            counterhit_ground: Some(self.grounded.unwrap().as_counterhit()),
            counterhit_air: self.counterhit_air,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G, A: Confirmable, CG: Changeable, CA> HitDataBuilder<G, A, CG, CA> {
    pub fn counterhit_ground_from_air(
        self,
        f: impl Fn(HitEffect) -> HitEffect,
    ) -> HitDataBuilder<G, A, CounterhitGround, CA> {
        HitDataBuilder::<G, A, CounterhitGround, CA> {
            grounded: self.grounded,
            air: self.air.clone(),
            counterhit_ground: Some((f)(self.air.unwrap())),
            counterhit_air: self.counterhit_air,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
    pub fn counterhit_ground_from_air_default(self) -> HitDataBuilder<G, A, CounterhitGround, CA> {
        HitDataBuilder::<G, A, CounterhitGround, CA> {
            grounded: self.grounded,
            air: self.air.clone(),
            counterhit_ground: Some(self.air.unwrap().as_counterhit()),
            counterhit_air: self.counterhit_air,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G, A: Confirmable, CG, CA: Changeable> HitDataBuilder<G, A, CG, CA> {
    pub fn counterhit_air_from_air(
        self,
        f: impl Fn(HitEffect) -> HitEffect,
    ) -> HitDataBuilder<G, A, CG, CounterhitAir> {
        HitDataBuilder::<G, A, CG, CounterhitAir> {
            grounded: self.grounded,
            air: self.air.clone(),
            counterhit_ground: self.counterhit_ground,
            counterhit_air: Some((f)(self.air.unwrap())),

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
    pub fn counterhit_air_from_air_default(self) -> HitDataBuilder<G, A, CG, CounterhitAir> {
        HitDataBuilder::<G, A, CG, CounterhitAir> {
            grounded: self.grounded,
            air: self.air.clone(),
            counterhit_ground: self.counterhit_ground,
            counterhit_air: Some(self.air.unwrap().as_counterhit()),

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G: Confirmable, A: Changeable, CG, CA> HitDataBuilder<G, A, CG, CA> {
    pub fn air_from_grounded(
        self,
        f: impl Fn(HitEffect) -> HitEffect,
    ) -> HitDataBuilder<G, Air, CG, CA> {
        HitDataBuilder::<G, Air, CG, CA> {
            grounded: self.grounded.clone(),
            air: Some((f)(self.grounded.unwrap())),
            counterhit_ground: self.counterhit_ground,
            counterhit_air: self.counterhit_air,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G: Changeable, A: Confirmable, CG, CA> HitDataBuilder<G, A, CG, CA> {
    pub fn grounded_from_air(
        self,
        f: impl Fn(HitEffect) -> HitEffect,
    ) -> HitDataBuilder<Grounded, A, CG, CA> {
        HitDataBuilder::<Grounded, A, CG, CA> {
            grounded: Some((f)(self.air.clone().unwrap())),
            air: self.air,
            counterhit_ground: self.counterhit_ground,
            counterhit_air: self.counterhit_air,

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
            meter_gain: self.meter_gain,
            meter_gain_modifier: self.meter_gain_modifier,
            minimum_damage: self.minimum_damage,
            extensions: self.extensions,
            _pd: PhantomData,
        }
    }
}
impl<G: Confirmable, A: Confirmable, CG: Confirmable, CA: Confirmable>
    HitDataBuilder<G, A, CG, CA>
{
    pub fn build(self) -> HitData {
        HitData {
            grounded: self.grounded.unwrap(),
            air: self.air.unwrap(),
            counterhit_ground: self.counterhit_ground.unwrap(),
            counterhit_air: self.counterhit_air.unwrap(),

            damage: self.damage,
            attack_type: self.attack_type,
            block_pushback: self.block_pushback,
            blockstun: self.blockstun,

            wall_pushback_mult: self.wall_pushback_mult,

            proration: self.proration,
            scaling: self.scaling,
            scaling_on_block_mult: self.scaling_on_block_mult,
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
    ) -> HitDataBuilder<Grounded, Undefined, Undefined, Undefined> {
        let dir = effect.x_force().signum();
        HitDataBuilder {
            grounded: Some(effect),
            air: None,
            counterhit_ground: None,
            counterhit_air: None,

            damage,
            attack_type: AttackType::Mid,
            block_pushback: 60.0 * dir,
            blockstun,

            wall_pushback_mult: 1.0,

            proration,
            scaling,
            scaling_on_block_mult: 50,
            meter_gain: Self::DEFAULT_LEVEL_2_METER_GAIN,
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
    ) -> HitDataBuilder<Undefined, Air, Undefined, Undefined> {
        let dir = effect.x_force().signum();
        HitDataBuilder {
            grounded: None,
            air: Some(effect),
            counterhit_ground: None,
            counterhit_air: None,

            damage,
            attack_type: AttackType::Mid,
            block_pushback: 60.0 * dir,
            blockstun,

            wall_pushback_mult: 1.0,

            proration,
            scaling,
            scaling_on_block_mult: 200,
            meter_gain: Self::DEFAULT_LEVEL_2_METER_GAIN,
            meter_gain_modifier: 0,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }

    pub const DEFAULT_LEVEL_1_SCALING: i32 = 800;
    pub const DEFAULT_LEVEL_1_METER_GAIN: u32 = 100;
    pub fn level_1(
        damage: u32,
        launch_force: Vector2,
        extra_hitstun: usize,
    ) -> HitDataBuilder<Default, Default, Undefined, Undefined> {
        let dir = launch_force.x.signum();
        HitDataBuilder {
            grounded: Some(HitEffect::pushback(50.0 * dir, 14 + extra_hitstun).build()),
            air: Some(HitEffect::launcher(launch_force, KnockdownType::Soft).build()),
            counterhit_ground: None,
            counterhit_air: None,

            damage,
            attack_type: AttackType::Mid,
            block_pushback: 40.0 * dir,
            blockstun: 11 + extra_hitstun,

            wall_pushback_mult: 1.0,

            proration: Proration::percent(70),
            scaling: Self::DEFAULT_LEVEL_1_SCALING,
            scaling_on_block_mult: 200,
            meter_gain: Self::DEFAULT_LEVEL_1_METER_GAIN,
            meter_gain_modifier: 0,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }
    pub const DEFAULT_LEVEL_2_SCALING: i32 = 1200;
    pub const DEFAULT_LEVEL_2_METER_GAIN: u32 = 150;
    pub fn level_2(
        damage: u32,
        launch_force: Vector2,
        extra_hitstun: usize,
    ) -> HitDataBuilder<Default, Default, Undefined, Undefined> {
        let dir = launch_force.x.signum();
        HitDataBuilder {
            grounded: Some(HitEffect::pushback(55.0 * dir, 16 + extra_hitstun).build()),
            air: Some(HitEffect::launcher(launch_force, KnockdownType::Soft).build()),
            counterhit_ground: None,
            counterhit_air: None,

            damage,
            attack_type: AttackType::Mid,
            block_pushback: 40.0 * dir,
            blockstun: 13 + extra_hitstun,

            wall_pushback_mult: 1.0,

            proration: Proration::percent(70),
            scaling: Self::DEFAULT_LEVEL_2_SCALING,
            scaling_on_block_mult: 200,
            meter_gain: Self::DEFAULT_LEVEL_2_METER_GAIN,
            meter_gain_modifier: 0,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }
    pub const DEFAULT_LEVEL_3_SCALING: i32 = 1500;
    pub const DEFAULT_LEVEL_3_METER_GAIN: u32 = 200;
    pub fn level_3(
        damage: u32,
        launch_force: Vector2,
        extra_hitstun: usize,
    ) -> HitDataBuilder<Default, Default, Undefined, Undefined> {
        let dir = launch_force.x.signum();
        HitDataBuilder {
            grounded: Some(HitEffect::pushback(60.0 * dir, 19 + extra_hitstun).build()),
            air: Some(HitEffect::launcher(launch_force, KnockdownType::Soft).build()),
            counterhit_ground: None,
            counterhit_air: None,

            damage,
            attack_type: AttackType::Mid,
            block_pushback: 40.0 * dir,
            blockstun: 16 + extra_hitstun,

            wall_pushback_mult: 1.0,

            proration: Proration::percent(70),
            scaling: Self::DEFAULT_LEVEL_3_SCALING,
            scaling_on_block_mult: 200,
            meter_gain: Self::DEFAULT_LEVEL_3_METER_GAIN,
            meter_gain_modifier: 0,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }
    pub const DEFAULT_LEVEL_4_SCALING: i32 = 2000;
    pub const DEFAULT_LEVEL_4_METER_GAIN: u32 = 250;
    pub fn level_4(
        damage: u32,
        launch_force: Vector2,
        extra_hitstun: usize,
    ) -> HitDataBuilder<Default, Default, Undefined, Undefined> {
        let dir = launch_force.x.signum();
        HitDataBuilder {
            grounded: Some(HitEffect::pushback(65.0 * dir, 21 + extra_hitstun).build()),
            air: Some(HitEffect::launcher(launch_force, KnockdownType::Soft).build()),
            counterhit_ground: None,
            counterhit_air: None,

            damage,
            attack_type: AttackType::Mid,
            block_pushback: 40.0 * dir,
            blockstun: 18 + extra_hitstun,

            wall_pushback_mult: 1.0,

            proration: Proration::percent(70),
            scaling: Self::DEFAULT_LEVEL_4_SCALING,
            scaling_on_block_mult: 200,
            meter_gain: Self::DEFAULT_LEVEL_4_METER_GAIN,
            meter_gain_modifier: 0,
            minimum_damage: 1,
            extensions: vec![],
            _pd: PhantomData,
        }
    }
}
