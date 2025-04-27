use super::{
    ground_normals::{CloseMid, FarMid, StandHeavy},
    Airdash, Backdash, Entity, Grounded, RunStartState,
};
use crate::collision::{
    AttackData, CollisionShape, HitEffect, HitInfo, Hitbox, Hurtbox, KnockdownType,
};
use crate::datatypes::{BoundingBox, Vector2};
use crate::input::{directions::InputDir, Action, Button, InputHandler};
use crate::world::World;

use super::{Sol, SolDamageableState, BASE_SPRITE_OFFSET, CROUCHING_COLLIDER, DEFAULT_COLLIDER};

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
