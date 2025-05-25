use crate::collision::{AttackData, HitConnectionStatus, OnHitHitData};
use crate::datatypes::{BoundingBox, Vector2};
use crate::input::InputHandler;
use crate::world::{EntityID, World};

macro_rules! try_transition {
    ($f:ident; $($arg:expr),*) => {
        try_transition!(Self::$f; $($arg),*)
    };
    ($f:path; $($arg:expr),*) => {
        match $f($($arg),*) {
            Ok(state) => return state,
            Err(e) => e,
        }
    };
}

pub mod sol;
mod wrapper_state;

#[allow(unused_imports)]
pub(crate) use wrapper_state::WrapperState;

pub struct CharacterInitInfo {
    pub(crate) max_health: u32,
}

pub enum EntityUpdateResult {
    Continue,
    Remove,
    ReplaceWith(Box<dyn NonPlayerEntity>),
}

pub trait NonPlayerEntity: HasID + OnHit + Position + AsAny {
    fn update(&mut self, world: &mut World, input: Option<&InputHandler>) -> EntityUpdateResult;
}

pub trait Player:
    HasID
    + Damageable
    + OnHit
    + Position
    + Velocity
    + HasCollider
    + ColliderWorldSpace
    + Grounded
    + Direction
    + DistanceFromOtherPlayer
    + HasCancelState
    + HasThrownState
    + AsAny
{
    fn update(self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player>;
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/idle".into(), Vector2::UP * 8.0))
        //None
    }
    fn actionable(&self) -> bool {
        true
    }
    fn counterhit(&self) -> bool {
        false
    }
    fn should_wall_bounce(&self) -> bool {
        false
    }
    fn in_hitstun(&self) -> bool {
        false
    }
    fn can_cancel(&self) -> bool {
        !self.in_hitstun()
    }
    fn moveable(&self) -> bool {
        true
    }
}
pub trait Damageable {
    fn hit(self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus);
}
pub trait OnHit {
    fn on_hit(&mut self, hit_type: HitConnectionStatus);
}
pub trait Position {
    fn position(&self) -> Vector2;
    fn move_by(&mut self, distance: Vector2);
    fn set_position(&mut self, position: Vector2);
}
pub trait Velocity {
    fn velocity(&self) -> Vector2;
    fn add_velocity(&mut self, velocity: Vector2);
    fn set_velocity(&mut self, velocity: Vector2);
}
pub trait HasCollider {
    fn get_collider(&self) -> &BoundingBox;
}
pub trait ColliderWorldSpace: Position + HasCollider {
    fn get_collider_world_space(&self) -> BoundingBox {
        self.get_collider().clone().transformed(self.position())
    }
}
pub trait Direction {
    fn get_direction(&self) -> bool;
    fn set_direction(&mut self, direction: bool);
}
pub trait Grounded: Position {
    fn set_grounded(&mut self, grounded: bool);
    fn is_grounded(&self) -> bool;
}
pub trait DistanceFromOtherPlayer {
    fn set_distance(&mut self, distance: f32);
}
pub trait HasCancelState {
    fn cancel_state(self: Box<Self>) -> Box<dyn Player>;
}
pub trait HasThrownState {
    fn thrown(self: Box<Self>) -> Box<dyn Player>;
}
pub trait HasID {
    fn id(&self) -> EntityID;

    fn create_hitbox(
        &self,
        shape: crate::collision::CollisionShape,
        attack_data: AttackData,
    ) -> crate::collision::Hitbox {
        crate::collision::Hitbox {
            shape,
            owner: self.id(),
            attack_data,
        }
    }
    fn create_hurtbox(&self, shape: crate::collision::CollisionShape) -> crate::collision::Hurtbox {
        crate::collision::Hurtbox {
            shape,
            owner: self.id(),
        }
    }
}

impl<T> ColliderWorldSpace for T where T: Position + HasCollider {}

pub trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}
impl<T> AsAny for T
where
    T: 'static,
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
