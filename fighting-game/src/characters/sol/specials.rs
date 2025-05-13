use super::{
    Damageable, Direction, Entity, Grounded, HasCollider, JumpSquat, OnHit, Position,
    RunStartState, Velocity, WALK_SPEED,
};
use crate::collision::{AttackData, CollisionShape, HitEffect, HitInfo, Hitbox, KnockdownType};
use crate::datatypes::{BoundingBox, Vector2};
use crate::input::{
    directions::{InputDir, Motion},
    Action, Button, InputHandler,
};
use crate::world::World;

use super::{Sol, SolDamageableState, BASE_SPRITE_OFFSET, STANDING_HURTBOX};

const VOLCANIC_VIPER_STARTUP: usize = 9;
const VOLCANIC_VIPER_ACTIVE_1: usize = 5;
const VOLCANIC_VIPER_ACTIVE_2: usize = 11;
const VOLCANIC_VIPER_RECOVERY: usize = 45;
const VOLCANIC_VIPER_DAMAGE_1: u16 = 15;
const VOLCANIC_VIPER_DAMAGE_2: u16 = 22;
const VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_1: u16 = 20;
const VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_2: u16 = 38;

pub struct VolcanicViper;
impl Entity for Sol<VolcanicViper> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const ACTIVE_FRAME_2: usize = VOLCANIC_VIPER_STARTUP + VOLCANIC_VIPER_ACTIVE_1;
        const RECOVERY_FRAME: usize = ACTIVE_FRAME_2 + VOLCANIC_VIPER_ACTIVE_2;
        const END_FRAME: usize = RECOVERY_FRAME + VOLCANIC_VIPER_RECOVERY;
        const DECEL: f32 = 4.0;
        const LAUNCH_VELOCITY: Vector2 = Vector2::new(30.0, 160.0);

        if self.frame == 0 || self.frame == ACTIVE_FRAME_2 as u8 {
            self.has_hit = false;
        }

        self.frame += 1;

        match self.frame as usize {
            0..VOLCANIC_VIPER_STARTUP => {
                self.velocity.x = self.velocity.y(0.0).move_towards(&Vector2::ZERO, DECEL).x;
                self
            }
            VOLCANIC_VIPER_STARTUP..ACTIVE_FRAME_2 => {
                if self.frame as usize == VOLCANIC_VIPER_STARTUP + VOLCANIC_VIPER_ACTIVE_1 - 1 {
                    self.velocity =
                        Vector2::new(LAUNCH_VELOCITY.x * self.dir(), LAUNCH_VELOCITY.y * 1.3);
                }

                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        VOLCANIC_VIPER_ACTIVE_1 - (self.frame as usize - VOLCANIC_VIPER_STARTUP);
                    let hit_info = HitInfo {
                        damage: VOLCANIC_VIPER_DAMAGE_1,
                        hitstun: 50 + active_frames_extra_hitstun,
                        blockstun: 12 + active_frames_extra_hitstun,
                        hit_effect: HitEffect::Launcher(
                            Vector2::new(30.0 * self.dir(), LAUNCH_VELOCITY.y),
                            KnockdownType::Soft,
                        ),
                        block_push: 60.0 * self.dir(),
                    };
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                12.0, 24.0,
                            ))),
                            info: AttackData {
                                grounded: hit_info.clone(),
                                air: hit_info.clone(),
                                counterhit: hit_info.clone(),
                                priority: 10,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::Medium,
                            },
                            owner: self.player,
                        },
                        self.position + Vector2::new(8.0 * self.dir(), 6.0),
                        1,
                    );
                    let clean_hit_info = {
                        let mut info = hit_info;
                        info.damage = VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_1;
                        info.hit_effect = HitEffect::Launcher(
                            Vector2::new(30.0 * self.dir(), LAUNCH_VELOCITY.y),
                            KnockdownType::Hard,
                        );
                        info
                    };
                    // NOTE: clean hit
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                2.0, 8.0,
                            ))),
                            info: AttackData {
                                grounded: clean_hit_info.clone(),
                                air: clean_hit_info.clone(),
                                counterhit: clean_hit_info.clone(),
                                priority: 20,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::SuperHeavy,
                            },
                            owner: self.player,
                        },
                        self.position + Vector2::new(4.0 * self.dir(), 6.0),
                        1,
                    );
                }
                self
            }
            ACTIVE_FRAME_2..RECOVERY_FRAME => {
                self.gravity();

                world.spawn_hurtbox(
                    crate::collision::Hurtbox {
                        shape: crate::collision::CollisionShape::Box(BoundingBox::with_size(
                            Vector2::new(16.0, 20.0),
                        )),
                        owner: self.player,
                    },
                    self.position + Vector2::new(-3.0 * self.dir(), 6.0),
                    1,
                );

                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        VOLCANIC_VIPER_ACTIVE_2 - (self.frame as usize - ACTIVE_FRAME_2);
                    let hit_info = HitInfo {
                        damage: VOLCANIC_VIPER_DAMAGE_2,
                        hitstun: 50 + active_frames_extra_hitstun,
                        blockstun: 12 + active_frames_extra_hitstun,
                        hit_effect: HitEffect::Launcher(
                            Vector2::new(50.0 * self.dir(), LAUNCH_VELOCITY.y),
                            KnockdownType::Soft,
                        ),
                        block_push: 60.0 * self.dir(),
                    };
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                18.0, 24.0,
                            ))),
                            info: AttackData {
                                grounded: hit_info.clone(),
                                air: hit_info.clone(),
                                counterhit: hit_info.clone(),
                                priority: 10,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::Heavy,
                            },
                            owner: self.player,
                        },
                        self.position + Vector2::new(8.0 * self.dir(), 12.0),
                        1,
                    );
                    let clean_hit_info = {
                        let mut info = hit_info;
                        info.damage = VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_2;
                        info.hit_effect = HitEffect::Launcher(
                            Vector2::new(50.0 * self.dir(), LAUNCH_VELOCITY.y),
                            KnockdownType::Hard,
                        );
                        info
                    };
                    // NOTE: clean hit
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                4.0, 8.0,
                            ))),
                            info: AttackData {
                                grounded: clean_hit_info.clone(),
                                air: clean_hit_info.clone(),
                                counterhit: clean_hit_info.clone(),
                                priority: 20,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::SuperHeavy,
                            },
                            owner: self.player,
                        },
                        self.position + Vector2::new(5.0 * self.dir(), 9.0),
                        1,
                    );
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                self.gravity();

                world.spawn_hurtbox(
                    crate::collision::Hurtbox {
                        shape: crate::collision::CollisionShape::Box(BoundingBox::with_size(
                            Vector2::new(16.0, 20.0),
                        )),
                        owner: self.player,
                    },
                    self.position + Vector2::new(-3.0 * self.dir(), 6.0),
                    1,
                );

                self
            }
            _ => {
                world.spawn_hurtbox(
                    crate::collision::Hurtbox {
                        shape: crate::collision::CollisionShape::Box(BoundingBox::with_size(
                            Vector2::new(16.0, 20.0),
                        )),
                        owner: self.player,
                    },
                    self.position + Vector2::new(-3.0 * self.dir(), 6.0),
                    1,
                );

                self.air_actionable_state(input)
            }
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const START_FRAME: usize = VOLCANIC_VIPER_STARTUP - 2;
        const ACTIVE_FRAME_2: usize = VOLCANIC_VIPER_STARTUP + VOLCANIC_VIPER_ACTIVE_1;
        const RECOVERY_FRAME: usize = ACTIVE_FRAME_2 + VOLCANIC_VIPER_ACTIVE_2;

        const OFFSET: Vector2 = BASE_SPRITE_OFFSET;

        Some(match self.frame as usize {
            0..START_FRAME => (
                "sol/specials/volcanic_viper/volcanic_viper1".into(),
                OFFSET + Vector2::DOWN * 8.0,
            ),
            START_FRAME..ACTIVE_FRAME_2 => (
                "sol/specials/volcanic_viper/volcanic_viper2".into(),
                OFFSET + Vector2::DOWN * 1.0,
            ),
            ACTIVE_FRAME_2..RECOVERY_FRAME => (
                "sol/specials/volcanic_viper/volcanic_viper3".into(),
                OFFSET + Vector2::UP * 5.0,
            ),
            _ if self.grounded => ("sol/run/run_stop".into(), BASE_SPRITE_OFFSET),
            _ => ("sol/specials/volcanic_viper/volcanic_viper4".into(), OFFSET),
        })
    }
}
impl SolDamageableState for VolcanicViper {}

const GUNFLAME_STARTUP: usize = 11;
const GUNFLAME_DECEL: f32 = 0.9;

/// bool is feint
pub struct GunFlameStartup<const FEINT: bool = false>;
impl GunFlameStartup {
    pub const fn feint() -> GunFlameStartup<true> {
        GunFlameStartup
    }
    pub const fn real() -> GunFlameStartup<false> {
        GunFlameStartup
    }
}
impl<const FEINT: bool> Entity for Sol<GunFlameStartup<FEINT>> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.has_hit = false;
        self.frame += 1;
        if self.velocity.x * self.dir() < 0.0 {
            self.velocity.x = 0.0;
        } else {
            self.velocity.x *= GUNFLAME_DECEL;
        }
        if self.frame > GUNFLAME_STARTUP as u8 {
            if FEINT {
                Box::new(self.transition(GunFlameFeint, true))
            } else {
                Box::new(self.transition(GunFlame, true))
            }
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        let mut path: String = "sol/gunflame/gunflame".into();
        path.push(match self.frame {
            0..4 => '1',
            ..8 => '2',
            _ => '3',
        });

        Some((path.into(), BASE_SPRITE_OFFSET))
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl<const FEINT: bool> SolDamageableState for GunFlameStartup<FEINT> {}

struct GunFlame;
impl Entity for Sol<GunFlame> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        // TODO: spawn projectile
        todo!()
    }
}
impl SolDamageableState for GunFlame {}

const GUNFLAME_FEINT_HOLD_LENGTH: usize = 8;
pub struct GunFlameFeint;
impl Entity for Sol<GunFlameFeint> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.frame += 1;

        if self.frame == 3 {
            world.spawn_hitbox(
                Hitbox {
                    shape: CollisionShape::Box(BoundingBox::pos_size(
                        Vector2::ZERO,
                        Vector2::new(4.0, 5.0),
                    )),
                    owner: self.player,
                    info: AttackData::with_same_hitinfo(
                        HitInfo {
                            damage: 10,
                            hitstun: 15,
                            blockstun: 8,
                            hit_effect: HitEffect::Pushback(20.0 * self.dir()),
                            block_push: 8.0 * self.dir(),
                        },
                        1,
                        crate::collision::AttackType::Mid,
                        1,
                        crate::collision::HitType::Light,
                    ),
                },
                self.position + Vector2::new(10.0 * self.dir(), -4.0),
                1,
            );
        }
        if self.frame > GUNFLAME_FEINT_HOLD_LENGTH as u8 {
            self.grounded_actionable_state(input)
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/gunflame/gunflame3".into(), BASE_SPRITE_OFFSET))
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for GunFlameFeint {}

const BANDIT_REVOLVER_GROUNDED_1_STARTUP: usize = 12;
const BANDIT_REVOLVER_GROUNDED_1_ACTIVE: usize = 6;
const BANDIT_REVOLVER_GROUNDED_1_RECOVERY: usize = 16;
const BANDIT_REVOLVER_GROUNDED_1_LANDING_LAG: usize = 4;
const BANDIT_REVOLVER_GROUNDED_1_DAMAGE: u16 = 11;

pub struct BanditRevolverGrounded;
impl Entity for Sol<BanditRevolverGrounded> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const LAUNCH_FRAME: usize = 6;
        const RECOVERY_FRAME: usize =
            BANDIT_REVOLVER_GROUNDED_1_STARTUP + BANDIT_REVOLVER_GROUNDED_1_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + BANDIT_REVOLVER_GROUNDED_1_RECOVERY;

        const INITIAL_FORCE: Vector2 = Vector2::new(120.0, 100.0);
        const DECEL: f32 = 5.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;

        // NOTE: cancel into hit 2
        {
            const CANCEL_WINDOW_START: usize = BANDIT_REVOLVER_GROUNDED_1_STARTUP + 1;
            const CANCEL_WINDOW_END: usize =
                RECOVERY_FRAME + BANDIT_REVOLVER_GROUNDED_1_RECOVERY / 2;

            if let CANCEL_WINDOW_START..CANCEL_WINDOW_END = self.frame as usize {
                if input.has_action(&Action::Pressed(Button::Mid, None)) {
                    return Box::new(self.transition(BanditRevolverGroundedSecondHit, true));
                }
            }
        }

        match self.frame as usize {
            f @ 0..BANDIT_REVOLVER_GROUNDED_1_STARTUP => {
                if f == LAUNCH_FRAME {
                    self.velocity = Vector2::new(INITIAL_FORCE.x * self.dir(), INITIAL_FORCE.y);
                }
                self.gravity();
                self
            }
            BANDIT_REVOLVER_GROUNDED_1_STARTUP..RECOVERY_FRAME => {
                self.gravity();

                if !self.has_hit {
                    let active_frames_extra_hitstun = BANDIT_REVOLVER_GROUNDED_1_ACTIVE
                        - (self.frame as usize - BANDIT_REVOLVER_GROUNDED_1_STARTUP);
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                14.0, 14.0,
                            ))),
                            info: AttackData {
                                grounded: HitInfo {
                                    damage: BANDIT_REVOLVER_GROUNDED_1_DAMAGE,
                                    hitstun: 11 + active_frames_extra_hitstun,
                                    blockstun: 8 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Pushback(30.0 * self.dir()),
                                    block_push: 60.0 * self.dir(),
                                },
                                air: HitInfo {
                                    damage: BANDIT_REVOLVER_GROUNDED_1_DAMAGE,
                                    hitstun: 11 + active_frames_extra_hitstun,
                                    blockstun: 8 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(50.0 * self.dir(), 60.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 80.0 * self.dir(),
                                },
                                counterhit: HitInfo {
                                    damage: BANDIT_REVOLVER_GROUNDED_1_DAMAGE,
                                    hitstun: 11 + active_frames_extra_hitstun,
                                    blockstun: 8 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(50.0 * self.dir(), 60.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 80.0 * self.dir(),
                                },
                                priority: 10,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::Light,
                            },
                            owner: self.player,
                        },
                        self.position + Vector2::new(10.0 * self.dir(), 6.0),
                        1,
                    );
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                self.gravity();
                self.velocity.x = self.velocity.y(0.0).move_towards(&Vector2::ZERO, DECEL).x;
                self
            }
            _ if !self.is_grounded() => {
                self.gravity();
                self.velocity.x = self.velocity.y(0.0).move_towards(&Vector2::ZERO, DECEL).x;
                self
            }
            _ => Box::new(self.transition(
                BanditRevolverGroundedRecovery::<BANDIT_REVOLVER_GROUNDED_1_LANDING_LAG>,
                true,
            )),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for BanditRevolverGrounded {}

struct BanditRevolverGroundedRecovery<const FRAMES: usize>;
impl<const FRAMES: usize> Entity for Sol<BanditRevolverGroundedRecovery<FRAMES>> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const DECEL: f32 = 5.0;

        self.has_hit = false;
        self.frame += 1;

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(STANDING_HURTBOX),
                owner: self.player,
            },
            self.position,
            1,
        );

        if (self.frame as usize) < FRAMES {
            self.velocity.x = self.velocity.y(0.0).move_towards(&Vector2::ZERO, DECEL).x;
            self
        } else {
            self.grounded_actionable_state(input)
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl<const FRAMES: usize> SolDamageableState for BanditRevolverGroundedRecovery<FRAMES> {}

const BANDIT_REVOLVER_GROUNDED_2_STARTUP: usize = 6;
const BANDIT_REVOLVER_GROUNDED_2_ACTIVE: usize = 2;
const BANDIT_REVOLVER_GROUNDED_2_RECOVERY: usize = 8;
const BANDIT_REVOLVER_GROUNDED_2_LANDING_LAG: usize = 15;
const BANDIT_REVOLVER_GROUNDED_2_DAMAGE: u16 = 14;

struct BanditRevolverGroundedSecondHit;
impl Entity for Sol<BanditRevolverGroundedSecondHit> {
    fn update(mut self: Box<Self>, world: &mut World, _: &InputHandler) -> Box<dyn Entity> {
        const RECOVERY_FRAME: usize =
            BANDIT_REVOLVER_GROUNDED_2_STARTUP + BANDIT_REVOLVER_GROUNDED_2_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + BANDIT_REVOLVER_GROUNDED_2_RECOVERY;

        const DECEL: f32 = 5.0;

        if self.frame == 0 {
            self.has_hit = false;
            self.velocity.y = 0.0;
        }

        self.frame += 1;

        match self.frame as usize {
            0..BANDIT_REVOLVER_GROUNDED_2_STARTUP => self,
            BANDIT_REVOLVER_GROUNDED_2_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun = BANDIT_REVOLVER_GROUNDED_2_ACTIVE
                        - (self.frame as usize - BANDIT_REVOLVER_GROUNDED_2_STARTUP);
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                20.0, 14.0,
                            ))),
                            info: AttackData {
                                grounded: HitInfo {
                                    damage: BANDIT_REVOLVER_GROUNDED_2_DAMAGE,
                                    hitstun: 43 + active_frames_extra_hitstun,
                                    blockstun: 14 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(100.0 * self.dir(), 100.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 60.0 * self.dir(),
                                },
                                air: HitInfo {
                                    damage: BANDIT_REVOLVER_GROUNDED_2_DAMAGE,
                                    hitstun: 43 + active_frames_extra_hitstun,
                                    blockstun: 14 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(100.0 * self.dir(), 100.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 80.0 * self.dir(),
                                },
                                counterhit: HitInfo {
                                    damage: BANDIT_REVOLVER_GROUNDED_2_DAMAGE,
                                    hitstun: 43 + active_frames_extra_hitstun,
                                    blockstun: 14 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(100.0 * self.dir(), 100.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 80.0 * self.dir(),
                                },
                                priority: 10,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::Light,
                            },
                            owner: self.player,
                        },
                        self.position + Vector2::new(14.0 * self.dir(), 6.0),
                        1,
                    );
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                self.velocity.x = self.velocity.y(0.0).move_towards(&Vector2::ZERO, DECEL).x;
                self.gravity();
                self
            }
            _ if !self.is_grounded() => {
                self.velocity.x = self.velocity.y(0.0).move_towards(&Vector2::ZERO, DECEL).x;
                self.gravity();
                self
            }
            _ => Box::new(self.transition(
                BanditRevolverGroundedRecovery::<BANDIT_REVOLVER_GROUNDED_2_LANDING_LAG>,
                true,
            )),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for BanditRevolverGroundedSecondHit {}
