use super::{
    Damageable, Direction, Entity, Grounded, HasCollider, JumpSquat, OnHit, Position, Velocity,
    WALK_SPEED,
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
                                Vector2::new(12.0, 18.0),
                            )),
                            owner: self.player,
                            info: AttackData::with_same_hitinfo(
                                HitInfo {
                                    damage: CLOSE_MID_DAMAGE,
                                    hitstun: 13 + active_frames_extra_hitstun,
                                    blockstun: 13 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(12.0 * self.dir(), 70.0),
                                        KnockdownType::None,
                                    ),
                                    block_push: 8.0 * self.dir(),
                                },
                                10,
                                crate::collision::AttackType::Mid,
                                1,
                                crate::collision::HitType::Medium,
                            ),
                        },
                        self.position + Vector2::new(7.0 * self.dir(), 10.0),
                        1,
                    );
                } else {
                    match self.grounded_movement_cancel_options(input) {
                        Ok(state) => return state,
                        Err(state) => self = state,
                    }
                    if input.has_action(&Action::Pressed(Button::Mid)) {
                        return Box::new(self.transition(FarMid, true));
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                if self.frame as usize <= RECOVERY_FRAME + 7 {
                    if input.has_action(&Action::Pressed(Button::Mid)) {
                        return Box::new(self.transition(FarMid, true));
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
        const ADVANCE_VELOCITY: f32 = 50.0;
        const DECEL: f32 = ADVANCE_VELOCITY / 2.0;

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
                                Vector2::new(12.0, 18.0),
                            )),
                            owner: self.player,
                            info: AttackData::with_same_hitinfo(
                                HitInfo {
                                    damage: FAR_MID_DAMAGE,
                                    hitstun: 20 + active_frames_extra_hitstun,
                                    blockstun: 15 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Pushback(20.0 * self.dir()),
                                    block_push: 8.0 * self.dir(),
                                },
                                10,
                                crate::collision::AttackType::Mid,
                                1,
                                crate::collision::HitType::Medium,
                            ),
                        },
                        self.position + Vector2::new(7.0 * self.dir(), 10.0),
                        1,
                    );
                } else {
                    match self.grounded_movement_cancel_options(input) {
                        Ok(state) => return state,
                        Err(state) => self = state,
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => self,
            _ => self.grounded_actionable_state(input),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for FarMid {}
