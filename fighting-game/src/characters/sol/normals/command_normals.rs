use super::Entity;
use crate::collision::{
    AttackData, CollisionShape, HitEffect, HitInfo, Hitbox, Hurtbox, KnockdownType,
};
use crate::datatypes::*;
use crate::input::{directions::InputDir, Action, Button, InputHandler};
use crate::world::World;

use super::{Sol, SolDamageableState, BASE_SPRITE_OFFSET, CROUCHING_HURTBOX, STANDING_HURTBOX};

const HEAVY_3_STARTUP: usize = 8;
const HEAVY_3_ACTIVE: usize = 3;
const HEAVY_3_RECOVERY: usize = 18;
const HEAVY_3_DAMAGE: u16 = 14;

pub struct Heavy3;
impl Entity for Sol<Heavy3> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const RECOVERY_FRAME: usize = HEAVY_3_STARTUP + HEAVY_3_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + HEAVY_3_RECOVERY;
        const DECEL: f32 = 6.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;
        self.velocity = self.velocity.move_towards(Vector2::ZERO, DECEL);

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                    26.0, 10.0,
                ))),
                owner: self.player,
            },
            self.position + Vector2::new(4.0 * self.dir(), -1.0),
            1,
        );

        match self.frame as usize {
            0..HEAVY_3_STARTUP => self,
            HEAVY_3_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        HEAVY_3_ACTIVE - (self.frame as usize - HEAVY_3_STARTUP);

                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                20.0, 8.0,
                            ))),
                            owner: self.player,
                            info: AttackData {
                                grounded: HitInfo {
                                    damage: HEAVY_3_DAMAGE,
                                    hitstun: 12 + active_frames_extra_hitstun,
                                    blockstun: 9 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(30.0 * self.dir(), 80.0),
                                        KnockdownType::Hard,
                                    ),
                                    block_push: 60.0 * self.dir(),
                                },
                                air: HitInfo {
                                    damage: HEAVY_3_DAMAGE,
                                    hitstun: 12 + active_frames_extra_hitstun,
                                    blockstun: 9 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(30.0 * self.dir(), 60.0),
                                        KnockdownType::Hard,
                                    ),
                                    block_push: 80.0 * self.dir(),
                                },
                                counterhit: HitInfo {
                                    damage: HEAVY_3_DAMAGE,
                                    hitstun: 12 + active_frames_extra_hitstun,
                                    blockstun: 9 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(30.0 * self.dir(), 100.0),
                                        KnockdownType::Hard,
                                    ),
                                    block_push: 60.0 * self.dir(),
                                },
                                priority: 10,
                                attack_type: crate::collision::AttackType::Low,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::Medium,
                            },
                        },
                        self.position + Vector2::new(6.0 * self.dir(), -1.0),
                        1,
                    );
                } else {
                    self = try_transition!(grounded_special_cancel_options; self, input);
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                if self.has_hit && (self.frame as usize) < RECOVERY_FRAME + 2 {
                    self = try_transition!(grounded_special_cancel_options; self, input);
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
        const RECOVERY_FRAME: usize = HEAVY_3_STARTUP + HEAVY_3_ACTIVE;
        const FINAL_FRAME: usize = RECOVERY_FRAME + HEAVY_3_RECOVERY / 2;

        Some(match self.frame as usize {
            0..HEAVY_3_STARTUP => (
                "sol/command_normals/3h/3h1".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * 4.0 * self.dir(),
            ),
            HEAVY_3_STARTUP..RECOVERY_FRAME => (
                "sol/command_normals/3h/3h2".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 4.0 * self.dir(),
            ),
            RECOVERY_FRAME..FINAL_FRAME => (
                "sol/command_normals/3h/3h3".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * 4.0 * self.dir(),
            ),
            FINAL_FRAME.. => (
                "sol/command_normals/3h/3h4".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * 4.0 * self.dir(),
            ),
        })
    }
}
impl SolDamageableState for Heavy3 {}

const HEAVY_6_STARTUP: usize = 23;
const HEAVY_6_ACTIVE: usize = 3;
const HEAVY_6_RECOVERY: usize = 20;
const HEAVY_6_DAMAGE: u16 = 30;

pub struct Heavy6;
impl Entity for Sol<Heavy6> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const RECOVERY_FRAME: usize = HEAVY_6_STARTUP + HEAVY_6_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + HEAVY_6_RECOVERY;
        const DECEL: f32 = 12.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;

        self.velocity = self.velocity.move_towards(Vector2::ZERO, DECEL);

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(STANDING_HURTBOX),
                owner: self.player,
            },
            self.position + Vector2::RIGHT * 6.0 * self.dir(),
            1,
        );

        match self.frame as usize {
            0..HEAVY_6_STARTUP => self,
            HEAVY_6_STARTUP..RECOVERY_FRAME => todo!(),
            _ => todo!(),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for Heavy6 {}
