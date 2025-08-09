use super::{Grounded, HasID, Player};
use crate::collision::{
    AttackData, AttackType, BounceInfo, CollisionShape, HitData, HitDataExtension, HitEffect,
    HitLevel, KnockdownType, Proration,
};
use crate::datatypes::{BoundingBox, Vector2};
use crate::input::{Action, Button, InputHandler};
use crate::world::World;

use super::{Sol, SolDamageableState, BASE_SPRITE_OFFSET, STANDING_HURTBOX};

const LARGE_SPRITE_BASE_OFFSET: Vector2 = BASE_SPRITE_OFFSET;

const AIR_LIGHT_STARTUP: usize = 10;
const AIR_LIGHT_ACTIVE: usize = 3;
const AIR_LIGHT_RECOVERY: usize = 23;
const AIR_LIGHT_DAMAGE: u32 = 12;

pub struct AirLight;
impl Player for Sol<AirLight> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
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
            self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(Vector2::new(
                20.0, 16.0,
            )))),
            self.position + Vector2::new(-3.0 * self.dir(), 6.0),
        );

        match self.frame as usize {
            0..AIR_LIGHT_STARTUP => self,
            AIR_LIGHT_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        AIR_LIGHT_ACTIVE - (self.frame as usize - AIR_LIGHT_STARTUP);
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(18.0, 14.0))),
                            AttackData {
                                attack: HitData::grounded(
                                    AIR_LIGHT_DAMAGE,
                                    HitEffect::pushback(
                                        30.0 * self.dir(),
                                        28 + active_frames_extra_hitstun,
                                    )
                                    .build(),
                                    25 + active_frames_extra_hitstun,
                                    Proration::percent(80),
                                    HitData::DEFAULT_LEVEL_2_SCALING,
                                )
                                .with_air(
                                    HitEffect::launcher(
                                        Vector2::new(30.0 * self.dir(), 85.0),
                                        KnockdownType::Soft,
                                    )
                                    .build(),
                                )
                                .counterhit_ground_from_ground_default()
                                .counterhit_air_from_air_default()
                                .meter_gain(HitData::DEFAULT_LEVEL_2_METER_GAIN)
                                .attack_type(AttackType::High)
                                .build(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Medium,
                                attack_id: "sol j.l".into(),
                            },
                        ),
                        self.position + Vector2::new(5.0 * self.dir(), -6.0),
                    );

                    world.spawn_hurtbox(
                        self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(
                            Vector2::new(20.0, 16.0),
                        ))),
                        self.position + Vector2::new(5.0 * self.dir(), -6.0),
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
const AIR_MID_DAMAGE: u32 = 8;

pub struct AirMid;
impl Player for Sol<AirMid> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const AIR_MID_ACTIVE_2_FRAME: usize = AIR_MID_STARTUP + AIR_MID_ACTIVE_1;
        const END_FRAME: usize = AIR_MID_ACTIVE_2_FRAME + AIR_MID_ACTIVE_2;

        if self.frame == 0 || self.frame == AIR_MID_ACTIVE_2_FRAME {
            self.has_hit = false;
        }

        if self.is_grounded() {
            return self.grounded_actionable_state(input);
        }

        self.frame += 1;

        self.gravity();

        world.spawn_hurtbox(
            self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(Vector2::new(
                20.0, 16.0,
            )))),
            self.position + Vector2::new(-3.0 * self.dir(), 6.0),
        );

        match self.frame as usize {
            0..AIR_MID_STARTUP => self,
            AIR_MID_STARTUP..END_FRAME => {
                let hit_1 = self.frame < AIR_MID_ACTIVE_2_FRAME;
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
                        attack: HitData::grounded(
                            AIR_MID_DAMAGE,
                            HitEffect::pushback(
                                15.0 * self.dir(),
                                16 + active_frames_extra_hitstun,
                            )
                            .build(),
                            13 + active_frames_extra_hitstun,
                            Proration::percent(80),
                            HitData::DEFAULT_LEVEL_2_SCALING,
                        )
                        .with_air(
                            HitEffect::launcher(
                                Vector2::new(30.0 * self.dir(), 50.0),
                                KnockdownType::Soft,
                            )
                            .build(),
                        )
                        .attack_type(AttackType::High)
                        .counterhit_ground_from_ground_default()
                        .counterhit_air_from_air_default()
                        .build(),
                        priority: 10,
                        hitbox_id: 1,
                        hit_level: HitLevel::Light,
                        attack_id: "sol j.m".into(),
                    };

                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(if hit_1 {
                                Vector2::new(14.0, 16.0)
                            } else {
                                Vector2::new(10.0, 20.0)
                            })),
                            info,
                        ),
                        self.position
                            + if hit_1 {
                                Vector2::new(5.0 * self.dir(), 8.0)
                            } else {
                                Vector2::new(5.0 * self.dir(), 4.0)
                            },
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
const AIR_HEAVY_DAMAGE: u32 = 32;

pub struct AirHeavy;
impl Player for Sol<AirHeavy> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
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
            self.create_hurtbox(CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );

        match self.frame as usize {
            0..AIR_HEAVY_STARTUP => self,
            AIR_HEAVY_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        AIR_HEAVY_ACTIVE - (self.frame as usize - AIR_HEAVY_STARTUP);
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(18.0, 14.0))),
                            AttackData {
                                attack: HitData::air(
                                    AIR_HEAVY_DAMAGE,
                                    HitEffect::launcher(
                                        Vector2::new(70.0 * self.dir(), 80.0),
                                        KnockdownType::Soft,
                                    )
                                    .wall_bounce(
                                        BounceInfo::new(Vector2::new(35.0, 50.0)).gravity(6.0),
                                    )
                                    .build(),
                                    18 + active_frames_extra_hitstun,
                                    Proration::percent(80),
                                    HitData::DEFAULT_LEVEL_3_SCALING,
                                )
                                .grounded_from_air(|mut air| {
                                    if let HitEffect::Launcher { knockback, .. } = &mut air {
                                        knockback.y = 150.0;
                                        air
                                    } else {
                                        unreachable!()
                                    }
                                })
                                .counterhit_ground_from_ground_default()
                                .counterhit_air_from_air_default()
                                .meter_gain(HitData::DEFAULT_LEVEL_3_METER_GAIN)
                                .add_extension(HitDataExtension::UsagesBeforeScaling(4))
                                .build(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Heavy,
                                attack_id: "sol j.h".into(),
                            },
                        ),
                        self.position + Vector2::new(10.0 * self.dir(), 12.0),
                    );

                    world.spawn_hurtbox(
                        self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(
                            Vector2::new(12.0, 18.0),
                        ))),
                        self.position + Vector2::new(6.0 * self.dir(), 12.0),
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
