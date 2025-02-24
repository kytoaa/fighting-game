use super::{Damageable, Direction, Entity, Grounded, HasCollider, OnHit, Position, Velocity};
use crate::collision::{HitEffect, HitInfo, KnockdownType};
use crate::datatypes::{BoundingBox, Vector2};
use crate::input::{
    directions::{InputDir, Motion},
    Action, Button, InputHandler,
};
use crate::world::World;

use super::{Sol, SolDamageableState};

const GUNFLAME_STARTUP: usize = 18;

pub struct GunFlameStartup(pub usize);
impl Entity for Sol<GunFlameStartup> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.state.0 += 1;
        if self.state.0 > GUNFLAME_STARTUP {
            Box::new(self.transition(GunFlame(0)))
        } else {
            self
        }
    }
}
impl SolDamageableState for GunFlameStartup {}

struct GunFlame(usize);
impl Entity for Sol<GunFlame> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        // TODO: spawn projectile
        todo!()
    }
}
impl SolDamageableState for GunFlame {}

const JUMP_MID_STARTUP: usize = 5;
const JUMP_MID_ACTIVE: usize = 5;

pub struct JumpMidStartup(pub usize);
impl Entity for Sol<JumpMidStartup> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.gravity();
        if self.grounded {
            return self.grounded_actionable_state(input);
        }
        self.state.0 += 1;
        if self.state.0 > JUMP_MID_STARTUP {
            Box::new(self.transition(JumpMid(0)))
        } else {
            self
        }
    }
}
impl SolDamageableState for JumpMidStartup {}

struct JumpMid(usize);
impl Entity for Sol<JumpMid> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.gravity();
        if self.grounded {
            return self.grounded_actionable_state(input);
        }
        self.state.0 += 1;
        world.spawn_hitbox(
            crate::collision::Hitbox {
                shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                    Vector2::ZERO,
                    Vector2::new(20.0, 10.0),
                )),
                owner: self.player,
                info: HitInfo {
                    damage: 30,
                    attack_type: crate::collision::AttackType::High,
                    hit_effect: HitEffect::Launcher(
                        Vector2::new(40.0 * self.dir(), 70.0),
                        KnockdownType::Soft,
                    ),
                    hitstun: 100,
                    priority: 5,
                    blockstun: 15,
                },
            },
            self.position + Vector2::new(10.0 * self.dir(), 0.0),
            1,
        );

        if self.state.0 > JUMP_MID_STARTUP {
            self.air_actionable_state(input)
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<Box<str>> {
        Some("sol_jump_mid".into())
    }
}
impl SolDamageableState for JumpMid {}
