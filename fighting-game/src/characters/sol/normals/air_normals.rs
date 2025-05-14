use super::{Entity, Grounded};
use crate::collision::{AttackData, CollisionShape, HitEffect, HitInfo, Hitbox, KnockdownType};
use crate::datatypes::{BoundingBox, Vector2};
use crate::input::{Action, Button, InputHandler};
use crate::world::World;

use super::{Sol, SolDamageableState, BASE_SPRITE_OFFSET, STANDING_HURTBOX};

const LARGE_SPRITE_BASE_OFFSET: Vector2 = BASE_SPRITE_OFFSET;

const AIR_LIGHT_STARTUP: usize = 10;
const AIR_LIGHT_ACTIVE: usize = 3;
const AIR_LIGHT_RECOVERY: usize = 23;
const AIR_LIGHT_DAMAGE: u16 = 12;

pub struct AirLight;
impl Entity for Sol<AirLight> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const RECOVERY_FRAME: usize = AIR_LIGHT_STARTUP + AIR_LIGHT_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + AIR_LIGHT_RECOVERY;

        if self.frame == 0 {
            self.has_hit = false;
        }

        if self.is_grounded() {
            return self.grounded_actionable_state(input);
        }

        self.frame += 1;

        self.gravity();

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                    20.0, 16.0,
                ))),
                owner: self.player,
            },
            self.position + Vector2::new(-3.0 * self.dir(), 6.0),
            1,
        );

        match self.frame as usize {
            0..AIR_LIGHT_STARTUP => self,
            AIR_LIGHT_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        AIR_LIGHT_ACTIVE - (self.frame as usize - AIR_LIGHT_STARTUP);
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                18.0, 14.0,
                            ))),
                            owner: self.player,
                            info: AttackData {
                                grounded: HitInfo {
                                    damage: AIR_LIGHT_DAMAGE,
                                    hitstun: 28 + active_frames_extra_hitstun,
                                    blockstun: 25 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Pushback(30.0 * self.dir()),
                                    block_push: 8.0 * self.dir(),
                                },
                                air: HitInfo {
                                    damage: AIR_LIGHT_DAMAGE,
                                    hitstun: 28 + active_frames_extra_hitstun,
                                    blockstun: 25 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(30.0 * self.dir(), 85.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 8.0 * self.dir(),
                                },
                                counterhit: HitInfo {
                                    damage: AIR_LIGHT_DAMAGE,
                                    hitstun: 28 + active_frames_extra_hitstun,
                                    blockstun: 25 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(30.0 * self.dir(), 80.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 8.0 * self.dir(),
                                },
                                priority: 10,
                                attack_type: crate::collision::AttackType::High,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitLevel::Medium,
                            },
                        },
                        self.position + Vector2::new(5.0 * self.dir(), -6.0),
                        1,
                    );

                    world.spawn_hurtbox(
                        crate::collision::Hurtbox {
                            shape: crate::collision::CollisionShape::Box(BoundingBox::with_size(
                                Vector2::new(20.0, 16.0),
                            )),
                            owner: self.player,
                        },
                        self.position + Vector2::new(5.0 * self.dir(), -6.0),
                        1,
                    );
                } else {
                    self = try_transition!(air_movement_cancel_options; self, input);
                    self = try_transition!(air_special_cancel_options; self, input);

                    if input.has_action(&Action::Pressed(Button::Mid, None)) {
                        return Box::new(self.transition(AirMid, true));
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(AirHeavy, true));
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                if self.has_hit {
                    self = try_transition!(air_movement_cancel_options; self, input);
                    self = try_transition!(air_special_cancel_options; self, input);

                    if input.has_action(&Action::Pressed(Button::Mid, None)) {
                        return Box::new(self.transition(AirMid, true));
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(AirHeavy, true));
                    }
                }
                self
            }
            _ => self.air_actionable_state(input),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        let active_offset = Vector2::new(-3.0 * self.dir(), -4.0);
        Some(match self.frame {
            0..5 => ("sol/normals/j.l/j.l1".into(), LARGE_SPRITE_BASE_OFFSET),
            5..10 => ("sol/normals/j.l/j.l2".into(), LARGE_SPRITE_BASE_OFFSET),
            10..13 => (
                "sol/normals/j.l/j.l3".into(),
                LARGE_SPRITE_BASE_OFFSET + active_offset,
            ),
            13..16 => (
                "sol/normals/j.l/j.l4".into(),
                LARGE_SPRITE_BASE_OFFSET + Vector2::new(-11.0 * self.dir(), -7.0) + active_offset,
            ),
            16..19 => (
                "sol/normals/j.l/j.l5".into(),
                LARGE_SPRITE_BASE_OFFSET + Vector2::new(-11.0 * self.dir(), -7.0) + active_offset,
            ),
            19.. => (
                "sol/normals/j.l/j.l6".into(),
                LARGE_SPRITE_BASE_OFFSET + Vector2::new(-11.0 * self.dir(), -7.0) + active_offset,
            ),
        })
    }
}
impl SolDamageableState for AirLight {}

const AIR_MID_STARTUP: usize = 11;
const AIR_MID_ACTIVE_1: usize = 4;
const AIR_MID_ACTIVE_2: usize = 8;
const AIR_MID_DAMAGE: u16 = 8;

pub struct AirMid;
impl Entity for Sol<AirMid> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const AIR_MID_ACTIVE_2_FRAME: usize = AIR_MID_STARTUP + AIR_MID_ACTIVE_1;
        const END_FRAME: usize = AIR_MID_ACTIVE_2_FRAME + AIR_MID_ACTIVE_2;

        if self.frame == 0 || self.frame == AIR_MID_ACTIVE_2_FRAME as u8 {
            self.has_hit = false;
        }

        if self.is_grounded() {
            return self.grounded_actionable_state(input);
        }

        self.frame += 1;

        self.gravity();

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                    20.0, 16.0,
                ))),
                owner: self.player,
            },
            self.position + Vector2::new(-3.0 * self.dir(), 6.0),
            1,
        );

        match self.frame as usize {
            0..AIR_MID_STARTUP => self,
            AIR_MID_STARTUP..END_FRAME => {
                let hit_1 = self.frame < AIR_MID_ACTIVE_2_FRAME as u8;
                if !self.has_hit {
                    let active_frames_extra_hitstun = if hit_1 {
                        AIR_MID_ACTIVE_1
                    } else {
                        AIR_MID_ACTIVE_2
                    } - (self.frame as usize
                        - if hit_1 {
                            AIR_MID_STARTUP
                        } else {
                            AIR_MID_ACTIVE_2_FRAME
                        });

                    let info = AttackData {
                        grounded: HitInfo {
                            damage: AIR_MID_DAMAGE,
                            hitstun: 10 + active_frames_extra_hitstun,
                            blockstun: 7 + active_frames_extra_hitstun,
                            hit_effect: HitEffect::Pushback(10.0 * self.dir()),
                            block_push: 8.0 * self.dir(),
                        },
                        air: HitInfo {
                            damage: AIR_MID_DAMAGE,
                            hitstun: 10 + active_frames_extra_hitstun,
                            blockstun: 7 + active_frames_extra_hitstun,
                            hit_effect: HitEffect::Launcher(
                                Vector2::new(30.0 * self.dir(), 50.0),
                                KnockdownType::Soft,
                            ),
                            block_push: 8.0 * self.dir(),
                        },
                        counterhit: HitInfo {
                            damage: AIR_MID_DAMAGE,
                            hitstun: 10 + active_frames_extra_hitstun,
                            blockstun: 7 + active_frames_extra_hitstun,
                            hit_effect: HitEffect::Launcher(
                                Vector2::new(10.0 * self.dir(), 50.0),
                                KnockdownType::Soft,
                            ),
                            block_push: 8.0 * self.dir(),
                        },
                        priority: 10,
                        attack_type: crate::collision::AttackType::High,
                        hitbox_id: 1,
                        hit_type: crate::collision::HitLevel::Medium,
                    };

                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(if hit_1 {
                                Vector2::new(14.0, 16.0)
                            } else {
                                Vector2::new(10.0, 20.0)
                            })),
                            owner: self.player,
                            info,
                        },
                        self.position
                            + if hit_1 {
                                Vector2::new(5.0 * self.dir(), 8.0)
                            } else {
                                Vector2::new(5.0 * self.dir(), 4.0)
                            },
                        1,
                    );
                } else {
                    self = try_transition!(air_movement_cancel_options; self, input);
                    self = try_transition!(air_special_cancel_options; self, input);

                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(AirHeavy, true));
                    }
                }
                self
            }
            _ => self.air_actionable_state(input),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const AIR_MID_ACTIVE_2_FRAME: usize = AIR_MID_STARTUP + AIR_MID_ACTIVE_1;
        const END_FRAME: usize = AIR_MID_ACTIVE_2_FRAME + AIR_MID_ACTIVE_2;

        Some(match self.frame as usize {
            0..AIR_MID_STARTUP => ("sol/normals/j.m/j.m1".into(), LARGE_SPRITE_BASE_OFFSET),
            AIR_MID_STARTUP..AIR_MID_ACTIVE_2_FRAME => {
                ("sol/normals/j.m/j.m2".into(), LARGE_SPRITE_BASE_OFFSET)
            }
            AIR_MID_ACTIVE_2_FRAME..END_FRAME => {
                ("sol/normals/j.m/j.m3".into(), LARGE_SPRITE_BASE_OFFSET)
            }
            _ => unreachable!(),
        })
    }
}
impl SolDamageableState for AirMid {}

const AIR_HEAVY_STARTUP: usize = 9;
const AIR_HEAVY_ACTIVE: usize = 3;
const AIR_HEAVY_RECOVERY: usize = 15;
const AIR_HEAVY_DAMAGE: u16 = 20;

pub struct AirHeavy;
impl Entity for Sol<AirHeavy> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const RECOVERY_FRAME: usize = AIR_HEAVY_STARTUP + AIR_HEAVY_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + AIR_HEAVY_RECOVERY;

        if self.frame == 0 {
            self.has_hit = false;
        }

        if self.is_grounded() {
            return self.grounded_actionable_state(input);
        }

        self.frame += 1;

        self.gravity();

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(STANDING_HURTBOX),
                owner: self.player,
            },
            self.position,
            1,
        );

        match self.frame as usize {
            0..AIR_HEAVY_STARTUP => self,
            AIR_HEAVY_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        AIR_HEAVY_ACTIVE - (self.frame as usize - AIR_HEAVY_STARTUP);
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                18.0, 14.0,
                            ))),
                            owner: self.player,
                            info: AttackData {
                                grounded: HitInfo {
                                    damage: AIR_HEAVY_DAMAGE,
                                    hitstun: 41 + active_frames_extra_hitstun,
                                    blockstun: 21 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(70.0 * self.dir(), 90.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 40.0 * self.dir(),
                                },
                                air: HitInfo {
                                    damage: AIR_HEAVY_DAMAGE,
                                    hitstun: 41 + active_frames_extra_hitstun,
                                    blockstun: 21 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(70.0 * self.dir(), 120.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 40.0 * self.dir(),
                                },
                                counterhit: HitInfo {
                                    damage: AIR_HEAVY_DAMAGE,
                                    hitstun: 41 + active_frames_extra_hitstun,
                                    blockstun: 21 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(70.0 * self.dir(), 90.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 40.0 * self.dir(),
                                },
                                priority: 10,
                                attack_type: crate::collision::AttackType::High,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitLevel::Heavy,
                            },
                        },
                        self.position + Vector2::new(10.0 * self.dir(), 12.0),
                        1,
                    );

                    world.spawn_hurtbox(
                        crate::collision::Hurtbox {
                            shape: crate::collision::CollisionShape::Box(BoundingBox::with_size(
                                Vector2::new(12.0, 18.0),
                            )),
                            owner: self.player,
                        },
                        self.position + Vector2::new(6.0 * self.dir(), 12.0),
                        1,
                    )
                } else {
                    self = try_transition!(air_movement_cancel_options; self, input);
                    self = try_transition!(air_special_cancel_options; self, input);

                    // NOTE: add special cancels
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                if self.has_hit {
                    self = try_transition!(air_movement_cancel_options; self, input);
                    self = try_transition!(air_special_cancel_options; self, input);
                }
                self
            }
            _ => self.air_actionable_state(input),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const RECOVERY_FRAME: usize = AIR_HEAVY_STARTUP + AIR_HEAVY_ACTIVE + 10;

        Some(match self.frame as usize {
            0..AIR_HEAVY_STARTUP => ("sol/normals/j.h/j.h1".into(), LARGE_SPRITE_BASE_OFFSET),
            AIR_HEAVY_STARTUP..RECOVERY_FRAME => {
                ("sol/normals/j.h/j.h2".into(), LARGE_SPRITE_BASE_OFFSET)
            }
            RECOVERY_FRAME.. => ("sol/normals/j.h/j.h1".into(), LARGE_SPRITE_BASE_OFFSET),
        })
    }
}
impl SolDamageableState for AirHeavy {}
