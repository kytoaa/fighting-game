use super::{
    Airdash, Damageable, Direction, Entity, Grounded, HasCollider, JumpSquat, OnHit, Position,
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
                shape: crate::collision::CollisionShape::Box(DEFAULT_COLLIDER),
                owner: self.player,
            },
            self.position,
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
                            shape: CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(18.0, 14.0),
                            )),
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
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::Medium,
                            },
                        },
                        self.position + Vector2::new(5.0 * self.dir(), -6.0),
                        1,
                    );

                    world.spawn_hurtbox(
                        crate::collision::Hurtbox {
                            shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(20.0, 16.0),
                            )),
                            owner: self.player,
                        },
                        self.position + Vector2::new(5.0 * self.dir(), -6.0),
                        1,
                    );
                } else {
                    if self.has_air_action
                        && input.has_action(&Action::DoublePress(self.forward_dir()))
                    {
                        self.has_air_action = false;
                        return Box::new(self.transition(Airdash, true));
                    }
                    if self.has_air_action
                        && (input.has_action(&Action::JumpPress(InputDir::Dir7))
                            || input.has_action(&Action::JumpPress(InputDir::Dir8))
                            || input.has_action(&Action::JumpPress(InputDir::Dir9)))
                    {
                        self.double_jump(input.move_dir().x);
                        return self.air_actionable_state(input);
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => self,
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
