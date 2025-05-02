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
const VOLCANIC_VIPER_RECOVERY: usize = 30;
const VOLCANIC_VIPER_DAMAGE_1: u16 = 18;
const VOLCANIC_VIPER_DAMAGE_2: u16 = 30;

pub struct VolcanicViper;
impl Entity for Sol<VolcanicViper> {
    fn update(self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        todo!();
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for VolcanicViper {}
