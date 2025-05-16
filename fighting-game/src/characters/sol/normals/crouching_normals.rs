use super::{
    ground_normals::{CloseMid, FarMid, StandHeavy},
    Entity, RunStartState,
};
use crate::collision::{
    AttackData, AttackType, CollisionShape, HitData, HitEffect, HitLevel, KnockdownType, Proration,
};
use crate::datatypes::*;
use crate::input::{Action, Button, InputHandler};
use crate::world::World;

use super::{Sol, SolDamageableState, BASE_SPRITE_OFFSET, CROUCHING_HURTBOX, STANDING_HURTBOX};

const CROUCH_LIGHT_STARTUP: usize = 4;
const CROUCH_LIGHT_ACTIVE: usize = 3;
const CROUCH_LIGHT_RECOVERY: usize = 11;
const CROUCH_LIGHT_DAMAGE: u32 = 13;

pub struct CrouchLight;
impl Entity for Sol<CrouchLight> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const RECOVERY_FRAME: usize = CROUCH_LIGHT_STARTUP + CROUCH_LIGHT_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + CROUCH_LIGHT_RECOVERY;
        const DECEL: f32 = 6.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;
        self.drag(DECEL);

        world.spawn_hurtbox(
            self.create_hurtbox(crate::collision::CollisionShape::new(CROUCHING_HURTBOX)),
            self.position + Vector2::new(-3.0 * self.dir(), 0.0),
        );

        match self.frame as usize {
            0..CROUCH_LIGHT_STARTUP => self,
            CROUCH_LIGHT_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        CROUCH_LIGHT_ACTIVE - (self.frame as usize - CROUCH_LIGHT_STARTUP);

                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(20.0, 6.0))),
                            AttackData {
                                attack: HitData::level_1(
                                    CROUCH_LIGHT_DAMAGE,
                                    Vector2::new(30.0 * self.dir(), 60.0),
                                    active_frames_extra_hitstun,
                                )
                                .attack_type(AttackType::Low)
                                .with_grounded(
                                    HitEffect::pushback(
                                        20.0 * self.dir(),
                                        12 + active_frames_extra_hitstun,
                                    )
                                    .build(),
                                )
                                .counterhit_from_air(|a| a)
                                .build(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Light,
                            },
                        ),
                        self.position + Vector2::new(6.0 * self.dir(), -3.0),
                    );

                    world.spawn_hurtbox(
                        self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(
                            Vector2::new(22.0, 8.0),
                        ))),
                        self.position + Vector2::new(6.0 * self.dir(), -2.0),
                    )
                } else {
                    self = try_transition!(cancel_options_from_grounded_normal; self, input);
                    match self
                        .grounded_movement_cancel_options_from_attack(input, RunStartState::<15>)
                    {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    if input.input_dir().is_down() {
                        if input.has_action(&Action::Pressed(Button::Mid, None)) {
                            return Box::new(self.transition(CrouchMid, true));
                        }
                        if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                            return Box::new(self.transition(CrouchHeavy, true));
                        }
                    }
                    if input.has_action(&Action::Pressed(Button::Mid, None)) {
                        if self.distance_from_other_player < CloseMid::MAX_DISTANCE {
                            return Box::new(self.transition(CloseMid, true));
                        } else {
                            return Box::new(self.transition(FarMid, true));
                        }
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                if self.has_hit && (self.frame as usize) < RECOVERY_FRAME + 2 {
                    self = try_transition!(cancel_options_from_grounded_normal; self, input);
                    match self.grounded_movement_cancel_options_from_attack(
                        input,
                        RunStartState::dash_cancel(),
                    ) {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    if input.input_dir().is_down() {
                        if input.has_action(&Action::Pressed(Button::Mid, None)) {
                            return Box::new(self.transition(CrouchMid, true));
                        }
                        if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                            return Box::new(self.transition(CrouchHeavy, true));
                        }
                    }
                    if input.has_action(&Action::Pressed(Button::Mid, None)) {
                        if self.distance_from_other_player < CloseMid::MAX_DISTANCE {
                            return Box::new(self.transition(CloseMid, true));
                        } else {
                            return Box::new(self.transition(FarMid, true));
                        }
                    }
                }
                self
            }
            _ => self.grounded_actionable_state(input),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const RECOVERY_FRAME: usize = CROUCH_LIGHT_STARTUP + CROUCH_LIGHT_ACTIVE + 3;

        Some(match self.frame as usize {
            0..CROUCH_LIGHT_STARTUP => (
                "sol/normals/2l/2l1".into(),
                BASE_SPRITE_OFFSET + Vector2::new(-2.0 * self.dir(), 0.0),
            ),
            CROUCH_LIGHT_STARTUP..RECOVERY_FRAME => {
                ("sol/normals/2l/2l2".into(), BASE_SPRITE_OFFSET)
            }
            RECOVERY_FRAME.. => ("sol/normals/2l/2l3".into(), BASE_SPRITE_OFFSET),
        })
    }
}
impl SolDamageableState for CrouchLight {}

const CROUCH_MID_STARTUP: usize = 10;
const CROUCH_MID_ACTIVE: usize = 6;
const CROUCH_MID_RECOVERY: usize = 12;
const CROUCH_MID_DAMAGE: u32 = 17;

pub struct CrouchMid;
impl Entity for Sol<CrouchMid> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const RECOVERY_FRAME: usize = CROUCH_MID_STARTUP + CROUCH_MID_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + CROUCH_MID_RECOVERY;
        const DECEL: f32 = 3.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;
        self.drag(DECEL);

        world.spawn_hurtbox(
            self.create_hurtbox(crate::collision::CollisionShape::new(CROUCHING_HURTBOX)),
            self.position,
        );

        match self.frame as usize {
            0..CROUCH_MID_STARTUP => self,
            CROUCH_MID_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        CROUCH_MID_ACTIVE - (self.frame as usize - CROUCH_MID_STARTUP);

                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(18.0, 8.0))),
                            AttackData {
                                attack: HitData::level_2(
                                    CROUCH_MID_DAMAGE,
                                    Vector2::new(40.0 * self.dir(), 40.0),
                                    active_frames_extra_hitstun,
                                )
                                .proration(Proration::percent(90))
                                .with_grounded(HitEffect::pushback(40.0 * self.dir(), 16).build())
                                .counterhit_from_air(|a| a)
                                .build(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Medium,
                            },
                        ),
                        self.position + Vector2::new(16.0 * self.dir(), 4.0),
                    );

                    world.spawn_hurtbox(
                        self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(
                            Vector2::new(12.0, 12.0),
                        ))),
                        self.position + Vector2::new(6.0 * self.dir(), 4.0),
                    )
                } else {
                    self = try_transition!(cancel_options_from_grounded_normal; self, input);
                    match self
                        .grounded_movement_cancel_options_from_attack(input, RunStartState::<15>)
                    {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        if input.input_dir().is_down() {
                            return Box::new(self.transition(CrouchHeavy, true));
                        }
                        return Box::new(self.transition(StandHeavy, true));
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                if self.has_hit && (self.frame as usize) <= RECOVERY_FRAME + 5 {
                    self = try_transition!(cancel_options_from_grounded_normal; self, input);
                    match self.grounded_movement_cancel_options_from_attack(
                        input,
                        RunStartState::dash_cancel(),
                    ) {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        if input.input_dir().is_down() {
                            return Box::new(self.transition(CrouchHeavy, true));
                        }
                        return Box::new(self.transition(StandHeavy, true));
                    }
                }
                self
            }
            _ => self.grounded_actionable_state(input),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const STARTUP_FRAME: usize = CROUCH_MID_STARTUP - 4;
        const RECOVERY_FRAME: usize = CROUCH_MID_STARTUP + CROUCH_MID_ACTIVE;
        const FINAL_FRAME: usize = RECOVERY_FRAME + 4;

        Some(match self.frame as usize {
            0..STARTUP_FRAME => ("sol/normals/2m/2m1".into(), BASE_SPRITE_OFFSET),
            STARTUP_FRAME..CROUCH_MID_STARTUP => (
                "sol/normals/2m/2m2".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * 8.0 * self.dir(),
            ),
            CROUCH_MID_STARTUP..RECOVERY_FRAME => (
                "sol/normals/2m/2m3".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 12.0 * self.dir(),
            ),
            RECOVERY_FRAME..FINAL_FRAME => (
                "sol/normals/2m/2m4".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * self.dir(),
            ),
            FINAL_FRAME.. => (
                "sol/normals/2m/2m5".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * self.dir(),
            ),
        })
    }
}
impl SolDamageableState for CrouchMid {}

const CROUCH_HEAVY_STARTUP: usize = 10;
const CROUCH_HEAVY_EARLY_ACTIVE: usize = 3;
const CROUCH_HEAVY_ACTIVE: usize = 3;
const CROUCH_HEAVY_RECOVERY: usize = 25;
const CROUCH_HEAVY_DAMAGE: u32 = 23;

pub struct CrouchHeavy;
impl Entity for Sol<CrouchHeavy> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const SECOND_ACTIVE: usize = CROUCH_HEAVY_STARTUP + CROUCH_HEAVY_EARLY_ACTIVE;
        const RECOVERY_FRAME: usize = SECOND_ACTIVE + CROUCH_HEAVY_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + CROUCH_HEAVY_RECOVERY;
        const DECEL: f32 = 8.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;
        self.drag(DECEL);

        let hitbox_data = if self.frame as usize >= CROUCH_HEAVY_STARTUP
            && (self.frame as usize) < RECOVERY_FRAME
        {
            let active_frames_extra_hitstun = (CROUCH_HEAVY_EARLY_ACTIVE + CROUCH_HEAVY_ACTIVE)
                - (self.frame as usize - CROUCH_HEAVY_STARTUP);

            Some(
                HitData::grounded(
                    CROUCH_HEAVY_DAMAGE,
                    HitEffect::floating_crumple(Vector2::new(30.0 * self.dir(), 70.0), 5.5, 8)
                        .build(),
                    12 + active_frames_extra_hitstun,
                    Proration::percent(90),
                    HitData::DEFAULT_LEVEL_4_SCALING,
                )
                .with_air(
                    HitEffect::launcher(Vector2::new(20.0 * self.dir(), 90.0), KnockdownType::Soft)
                        .build(),
                )
                .counterhit_from_grounded(|g| {
                    if let HitEffect::FloatingCrumple {
                        knockback,
                        gravity,
                        landing_frames,
                    } = &mut g
                    {
                        (*knockback).y += 15.0;
                        *gravity = 5.0;
                        g
                    } else {
                        unreachable!()
                    }
                })
                .build(),
            )
        } else {
            None
        };

        if self.has_hit {
            self = try_transition!(cancel_options_from_grounded_normal; self, input);
            match self
                .grounded_movement_cancel_options_from_attack(input, RunStartState::dash_cancel())
            {
                Ok(state) => return state,
                Err(s) => self = s,
            }
        }

        match self.frame as usize {
            0..CROUCH_HEAVY_STARTUP => {
                world.spawn_hurtbox(
                    self.create_hurtbox(crate::collision::CollisionShape::new(CROUCHING_HURTBOX)),
                    self.position,
                );
                self
            }
            CROUCH_HEAVY_STARTUP..SECOND_ACTIVE => {
                world.spawn_hurtbox(
                    self.create_hurtbox(crate::collision::CollisionShape::new(STANDING_HURTBOX)),
                    self.position,
                );
                world.spawn_hurtbox(
                    self.create_hurtbox(crate::collision::CollisionShape::new(
                        BoundingBox::with_size(Vector2::new(10.0, 14.0)),
                    )),
                    self.position + Vector2::new(6.0 * self.dir(), 8.0),
                );
                if !self.has_hit {
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(10.0, 10.0))),
                            AttackData {
                                attack: hitbox_data.unwrap(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Heavy,
                            },
                        ),
                        self.position + Vector2::new(4.0 * self.dir(), 8.0),
                    );
                }
                self
            }
            SECOND_ACTIVE..RECOVERY_FRAME => {
                world.spawn_hurtbox(
                    self.create_hurtbox(crate::collision::CollisionShape::new(STANDING_HURTBOX)),
                    self.position,
                );
                world.spawn_hurtbox(
                    self.create_hurtbox(crate::collision::CollisionShape::new(
                        BoundingBox::with_size(Vector2::new(10.0, 20.0)),
                    )),
                    self.position + Vector2::new(8.0 * self.dir(), 16.0),
                );
                if !self.has_hit {
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(10.0, 28.0))),
                            AttackData {
                                attack: hitbox_data.unwrap(),
                                hit_level: HitLevel::Heavy,
                                hitbox_id: 1,
                                priority: 10,
                            },
                        ),
                        self.position + Vector2::new(10.0 * self.dir(), 24.0),
                    );
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                world.spawn_hurtbox(
                    self.create_hurtbox(crate::collision::CollisionShape::new(STANDING_HURTBOX)),
                    self.position,
                );
                self
            }
            _ => self.grounded_actionable_state(input),
        }
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn actionable(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const SECOND_ACTIVE: usize = CROUCH_HEAVY_STARTUP + CROUCH_HEAVY_EARLY_ACTIVE;

        Some(match self.frame as usize {
            0..CROUCH_HEAVY_STARTUP => (
                "sol/normals/2h/2h1".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * 9.0 * self.dir(),
            ),
            CROUCH_HEAVY_STARTUP..SECOND_ACTIVE => {
                ("sol/normals/2h/2h2".into(), BASE_SPRITE_OFFSET)
            }
            SECOND_ACTIVE.. => (
                "sol/normals/2h/2h3".into(),
                BASE_SPRITE_OFFSET + Vector2::UP * 8.0,
            ),
        })
    }
}
impl SolDamageableState for CrouchHeavy {}
