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

use super::{Sol, SolDamageableState, BASE_SPRITE_OFFSET, DEFAULT_COLLIDER};

const CLOSE_MID_STARTUP: usize = 7;
const CLOSE_MID_ACTIVE: usize = 6;
const CLOSE_MID_RECOVERY: usize = 10;
const CLOSE_MID_DAMAGE: u16 = 20;

pub struct CloseMid;
impl Entity for Sol<CloseMid> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const RECOVERY_FRAME: usize = CLOSE_MID_STARTUP + CLOSE_MID_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + CLOSE_MID_RECOVERY;
        const DECEL: f32 = 8.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;
        self.velocity = self.velocity.move_towards(&Vector2::ZERO, DECEL);

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(DEFAULT_COLLIDER),
                owner: self.player,
            },
            self.position,
            1,
        );

        match self.frame as usize {
            0..CLOSE_MID_STARTUP => self,
            CLOSE_MID_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        CLOSE_MID_ACTIVE - (self.frame as usize - CLOSE_MID_STARTUP);
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(10.0, 18.0),
                            )),
                            owner: self.player,
                            info: AttackData::with_same_hitinfo(
                                HitInfo {
                                    damage: CLOSE_MID_DAMAGE,
                                    hitstun: 14 + active_frames_extra_hitstun,
                                    blockstun: 14 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(25.0 * self.dir(), 85.0),
                                        KnockdownType::None,
                                    ),
                                    block_push: 30.0 * self.dir(),
                                },
                                10,
                                crate::collision::AttackType::Mid,
                                1,
                                crate::collision::HitType::Medium,
                            ),
                        },
                        self.position + Vector2::new(5.0 * self.dir(), 10.0),
                        1,
                    );

                    world.spawn_hurtbox(
                        crate::collision::Hurtbox {
                            shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(12.0, 20.0),
                            )),
                            owner: self.player,
                        },
                        self.position + Vector2::new(5.0 * self.dir(), 10.0),
                        1,
                    );
                } else {
                    match self.grounded_special_cancel(input) {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    match self
                        .grounded_movement_cancel_options_from_attack(input, RunStartState::<15>)
                    {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    if input.has_action(&Action::Pressed(Button::Mid, None)) {
                        return Box::new(self.transition(FarMid, true));
                    }
                    if input.has_action(&Action::Pressed(Button::Light, None)) {
                        return Box::new(self.transition(StandLight, true));
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(StandHeavy, true));
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                if self.frame as usize <= RECOVERY_FRAME + 7 {
                    match self.grounded_special_cancel(input) {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    if input.has_action(&Action::Pressed(Button::Mid, None)) {
                        return Box::new(self.transition(FarMid, true));
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(StandHeavy, true));
                    }
                }
                self
            }
            _ => self.grounded_actionable_state(input),
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const CLOSE_MID_STARTUP_SECOND_FRAME: usize = CLOSE_MID_STARTUP / 2;
        Some((
            match self.frame as usize {
                0..CLOSE_MID_STARTUP_SECOND_FRAME => "sol/normals/c.m/c.m1",
                CLOSE_MID_STARTUP_SECOND_FRAME..CLOSE_MID_STARTUP => "sol/normals/c.m/c.m2",
                _ => "sol/normals/c.m/c.m3",
            }
            .into(),
            BASE_SPRITE_OFFSET + Vector2::LEFT * 5.0 * self.dir(),
        ))
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for CloseMid {}

const FAR_MID_STARTUP: usize = 10;
const FAR_MID_ACTIVE: usize = 2;
const FAR_MID_RECOVERY: usize = 13;
const FAR_MID_DAMAGE: u16 = 14;

pub struct FarMid;
impl Entity for Sol<FarMid> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const RECOVERY_FRAME: usize = FAR_MID_STARTUP + FAR_MID_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + FAR_MID_RECOVERY;
        const ADVANCE_VELOCITY: f32 = 90.0;
        const DECEL: f32 = ADVANCE_VELOCITY / FAR_MID_ACTIVE as f32;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(DEFAULT_COLLIDER),
                owner: self.player,
            },
            self.position,
            1,
        );

        match self.frame as usize {
            0..FAR_MID_STARTUP => {
                self.velocity = Vector2::RIGHT * ADVANCE_VELOCITY * self.dir();
                self
            }
            FAR_MID_STARTUP..RECOVERY_FRAME => {
                self.velocity = self.velocity.move_towards(&Vector2::ZERO, DECEL);
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        FAR_MID_ACTIVE - (self.frame as usize - FAR_MID_STARTUP);
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(16.0, 18.0),
                            )),
                            owner: self.player,
                            info: AttackData {
                                grounded: HitInfo {
                                    damage: FAR_MID_DAMAGE,
                                    hitstun: 20 + active_frames_extra_hitstun,
                                    blockstun: 15 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Pushback(35.0 * self.dir()),
                                    block_push: 20.0 * self.dir(),
                                },
                                air: HitInfo {
                                    damage: FAR_MID_DAMAGE,
                                    hitstun: 20 + active_frames_extra_hitstun,
                                    blockstun: 15 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(80.0 * self.dir(), 35.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 50.0 * self.dir(),
                                },
                                counterhit: HitInfo {
                                    damage: FAR_MID_DAMAGE,
                                    hitstun: 20 + active_frames_extra_hitstun,
                                    blockstun: 15 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(20.0 * self.dir(), 50.0),
                                        KnockdownType::None,
                                    ),
                                    block_push: 20.0 * self.dir(),
                                },
                                priority: 10,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::Medium,
                            },
                        },
                        self.position + Vector2::new(10.0 * self.dir(), 10.0),
                        1,
                    );

                    world.spawn_hurtbox(
                        crate::collision::Hurtbox {
                            shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(18.0, 20.0),
                            )),
                            owner: self.player,
                        },
                        self.position + Vector2::new(10.0 * self.dir(), 10.0),
                        1,
                    );
                } else {
                    match self.grounded_special_cancel(input) {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    match self.grounded_movement_cancel_options_from_attack(
                        input,
                        RunStartState::dash_cancel(),
                    ) {
                        Ok(state) => return state,
                        Err(state) => self = state,
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(StandHeavy, true));
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => self,
            _ => self.grounded_actionable_state(input),
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const FAR_MID_STARTUP_SECOND_FRAME: usize = 3;
        const FAR_MID_RECOVER_FRAME: usize =
            FAR_MID_STARTUP + FAR_MID_ACTIVE + FAR_MID_RECOVERY / 2;
        Some((
            match self.frame as usize {
                0..FAR_MID_STARTUP_SECOND_FRAME => "sol/normals/f.m/f.m1",
                FAR_MID_STARTUP_SECOND_FRAME..FAR_MID_STARTUP => "sol/normals/f.m/f.m2",
                FAR_MID_STARTUP..FAR_MID_RECOVER_FRAME => "sol/normals/f.m/f.m3",
                _ => "sol/normals/f.m/f.m4",
            }
            .into(),
            BASE_SPRITE_OFFSET + Vector2::RIGHT * 0.0 * self.dir(),
        ))
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for FarMid {}

const STAND_LIGHT_STARTUP: usize = 5;
const STAND_LIGHT_FIRST_ACTIVE: usize = 1;
const STAND_LIGHT_SECOND_ACTIVE: usize = 3;
const STAND_LIGHT_RECOVERY: usize = 15;
const STAND_LIGHT_FIRST_HIT_DAMAGE: u16 = 8;
const STAND_LIGHT_SECOND_HIT_DAMAGE: u16 = 14;

pub struct StandLight;
impl Entity for Sol<StandLight> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const SECOND_ACTIVE_FRAME: usize = STAND_LIGHT_STARTUP + STAND_LIGHT_FIRST_ACTIVE;
        const RECOVERY_FRAME: usize = SECOND_ACTIVE_FRAME + STAND_LIGHT_SECOND_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + STAND_LIGHT_RECOVERY;
        const DECEL: f32 = 8.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;

        self.velocity = self.velocity.move_towards(&Vector2::ZERO, DECEL);

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(DEFAULT_COLLIDER),
                owner: self.player,
            },
            self.position + Vector2::RIGHT * 6.0 * self.dir(),
            1,
        );

        if self.has_hit && self.frame as usize == STAND_LIGHT_STARTUP + 1 {
            self.has_hit = false;
        }

        match self.frame as usize {
            0..STAND_LIGHT_STARTUP => self,
            STAND_LIGHT_STARTUP => {
                world.spawn_hitbox(
                    Hitbox {
                        shape: CollisionShape::Box(BoundingBox::pos_size(
                            Vector2::ZERO,
                            Vector2::new(10.0, 12.0),
                        )),
                        owner: self.player,
                        info: AttackData::with_same_hitinfo(
                            HitInfo {
                                damage: STAND_LIGHT_FIRST_HIT_DAMAGE,
                                hitstun: 6,
                                blockstun: 5,
                                hit_effect: HitEffect::Pushback(20.0 * self.dir()),
                                block_push: 25.0 * self.dir(),
                            },
                            10,
                            crate::collision::AttackType::Mid,
                            1,
                            crate::collision::HitType::Light,
                        ),
                    },
                    self.position
                        + Vector2::new(8.0 * self.dir(), 8.0)
                        + Vector2::RIGHT * 6.0 * self.dir(),
                    1,
                );

                world.spawn_hurtbox(
                    crate::collision::Hurtbox {
                        shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                            Vector2::ZERO,
                            Vector2::new(12.0, 14.0),
                        )),
                        owner: self.player,
                    },
                    self.position
                        + Vector2::new(8.0 * self.dir(), 8.0)
                        + Vector2::RIGHT * 6.0 * self.dir(),
                    1,
                );

                self
            }
            SECOND_ACTIVE_FRAME..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        SECOND_ACTIVE_FRAME - (self.frame as usize - STAND_LIGHT_SECOND_ACTIVE);

                    let attack_data: AttackData = AttackData {
                        grounded: HitInfo {
                            damage: STAND_LIGHT_SECOND_HIT_DAMAGE,
                            hitstun: 9 + active_frames_extra_hitstun,
                            blockstun: 8 + active_frames_extra_hitstun,
                            hit_effect: HitEffect::Pushback(10.0 * self.dir()),
                            block_push: 8.0 * self.dir(),
                        },
                        air: HitInfo {
                            damage: STAND_LIGHT_SECOND_HIT_DAMAGE,
                            hitstun: 9 + active_frames_extra_hitstun,
                            blockstun: 8 + active_frames_extra_hitstun,
                            hit_effect: HitEffect::Launcher(
                                Vector2::new(30.0 * self.dir(), 60.0),
                                KnockdownType::Soft,
                            ),
                            block_push: 25.0 * self.dir(),
                        },
                        counterhit: HitInfo {
                            damage: STAND_LIGHT_SECOND_HIT_DAMAGE,
                            hitstun: 9 + active_frames_extra_hitstun,
                            blockstun: 8 + active_frames_extra_hitstun,
                            hit_effect: HitEffect::Launcher(
                                Vector2::new(20.0 * self.dir(), 50.0),
                                KnockdownType::None,
                            ),
                            block_push: 0.0 * self.dir(),
                        },
                        priority: 10,
                        attack_type: crate::collision::AttackType::Mid,
                        hitbox_id: 1,
                        hit_type: crate::collision::HitType::Light,
                    };

                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(10.0, 12.0),
                            )),
                            owner: self.player,
                            info: attack_data.clone(),
                        },
                        self.position
                            + Vector2::new(14.0 * self.dir(), 24.0)
                            + Vector2::RIGHT * 6.0 * self.dir(),
                        1,
                    );
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(8.0, 10.0),
                            )),
                            owner: self.player,
                            info: attack_data.clone(),
                        },
                        self.position
                            + Vector2::new(9.0 * self.dir(), 17.0)
                            + Vector2::RIGHT * 6.0 * self.dir(),
                        1,
                    );
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(8.0, 10.0),
                            )),
                            owner: self.player,
                            info: attack_data,
                        },
                        self.position
                            + Vector2::new(5.0 * self.dir(), 10.0)
                            + Vector2::RIGHT * 6.0 * self.dir(),
                        1,
                    );

                    world.spawn_hurtbox(
                        crate::collision::Hurtbox {
                            shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(10.0, 12.0),
                            )),
                            owner: self.player,
                        },
                        self.position
                            + Vector2::new(9.0 * self.dir(), 17.0)
                            + Vector2::RIGHT * 6.0 * self.dir(),
                        1,
                    );
                    world.spawn_hurtbox(
                        crate::collision::Hurtbox {
                            shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(10.0, 12.0),
                            )),
                            owner: self.player,
                        },
                        self.position
                            + Vector2::new(5.0 * self.dir(), 10.0)
                            + Vector2::RIGHT * 6.0 * self.dir(),
                        1,
                    );
                } else {
                    match self.grounded_special_cancel(input) {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    match self.grounded_movement_cancel_options_from_attack(
                        input,
                        RunStartState::dash_cancel(),
                    ) {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    if input.has_action(&Action::Pressed(Button::Mid, None)) {
                        return Box::new(self.transition(FarMid, true));
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(StandHeavy, true));
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => self,
            _ => self.grounded_actionable_state(input),
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const MAIN_ACTIVE_FRAME: usize = STAND_LIGHT_STARTUP + STAND_LIGHT_FIRST_ACTIVE;
        const RECOVERY_START: usize = MAIN_ACTIVE_FRAME + STAND_LIGHT_SECOND_ACTIVE + 7;
        const RECOVERY_END: usize = RECOVERY_START + STAND_LIGHT_RECOVERY - 7;
        Some(match self.frame as usize {
            0..STAND_LIGHT_STARTUP => (
                "sol/normals/5l/5l1".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 5.0 * self.dir(),
            ),
            STAND_LIGHT_STARTUP => (
                "sol/normals/5l/5l2".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 6.0 * self.dir(),
            ),
            MAIN_ACTIVE_FRAME..RECOVERY_START => (
                "sol/normals/5l/5l3".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 7.0 * self.dir(),
            ),
            RECOVERY_START..RECOVERY_END => (
                "sol/normals/5l/5l4".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 7.0 * self.dir(),
            ),
            _ => unreachable!(),
        })
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for StandLight {}

const STAND_HEAVY_STARTUP: usize = 10;
const STAND_HEAVY_ACTIVE: usize = 4;
const STAND_HEAVY_RECOVERY: usize = 20;
const STAND_HEAVY_DAMAGE: u16 = 25;

pub struct StandHeavy;
impl Entity for Sol<StandHeavy> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const RECOVERY_FRAME: usize = STAND_HEAVY_STARTUP + STAND_HEAVY_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + STAND_HEAVY_RECOVERY;
        const DECEL: f32 = 4.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;

        self.velocity = self.velocity.move_towards(&Vector2::ZERO, DECEL);

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(DEFAULT_COLLIDER),
                owner: self.player,
            },
            self.position + Vector2::RIGHT * 6.0 * self.dir(),
            1,
        );

        match self.frame as usize {
            0..STAND_HEAVY_STARTUP => self,
            STAND_HEAVY_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        STAND_HEAVY_ACTIVE - (self.frame as usize - STAND_HEAVY_STARTUP);
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(20.0, 15.0),
                            )),
                            owner: self.player,
                            info: AttackData {
                                grounded: HitInfo {
                                    damage: STAND_HEAVY_DAMAGE,
                                    hitstun: 15 + active_frames_extra_hitstun,
                                    blockstun: 18 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Pushback(55.0 * self.dir()),
                                    block_push: 60.0 * self.dir(),
                                },
                                air: HitInfo {
                                    damage: FAR_MID_DAMAGE,
                                    hitstun: 20 + active_frames_extra_hitstun,
                                    blockstun: 15 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(80.0 * self.dir(), 35.0),
                                        KnockdownType::Hard,
                                    ),
                                    block_push: 60.0 * self.dir(),
                                },
                                counterhit: HitInfo {
                                    damage: FAR_MID_DAMAGE,
                                    hitstun: 20 + active_frames_extra_hitstun,
                                    blockstun: 15 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(40.0 * self.dir(), 70.0),
                                        KnockdownType::None,
                                    ),
                                    block_push: 50.0 * self.dir(),
                                },
                                priority: 10,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::Heavy,
                            },
                        },
                        self.position + Vector2::new(18.0 * self.dir(), 13.0),
                        1,
                    );
                    world.spawn_hurtbox(
                        crate::collision::Hurtbox {
                            shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(12.0, 17.0),
                            )),
                            owner: self.player,
                        },
                        self.position + Vector2::new(12.0 * self.dir(), 13.0),
                        1,
                    );
                } else {
                    match self.grounded_special_cancel(input) {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                if self.frame < (RECOVERY_FRAME + 5) as u8 && self.has_hit {
                    match self.grounded_special_cancel(input) {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                }
                self
            }
            _ => self.grounded_actionable_state(input),
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(match self.frame {
            0..3 => (
                "sol/normals/5h/5h1".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 2.0 * self.dir(),
            ),
            3..7 => (
                "sol/normals/5h/5h2".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 2.0 * self.dir(),
            ),
            7..10 => (
                "sol/normals/5h/5h3".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * self.dir(),
            ),
            10..13 => (
                "sol/normals/5h/5h4".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 7.0 * self.dir(),
            ),
            13..16 => (
                "sol/normals/5h/5h5".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 6.0 * self.dir(),
            ),
            16..22 => (
                "sol/normals/5h/5h6".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 6.0 * self.dir(),
            ),
            22..28 => (
                "sol/normals/5h/5h7".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 6.0 * self.dir(),
            ),
            28.. => (
                "sol/normals/5h/5h8".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 2.0 * self.dir(),
            ),
        })
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for StandHeavy {}
