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
pub(crate) mod sprite_entity;

pub(crate) struct CharacterSpecificInitInfo {
    pub(crate) max_health: u32,
}
impl CharacterSpecificInitInfo {
    pub fn into_init_info(self, burst: u32) -> CharacterInitInfo {
        CharacterInitInfo {
            max_health: self.max_health,
            burst,
        }
    }
}

pub(crate) struct CharacterInitInfo {
    pub(crate) max_health: u32,
    pub(crate) burst: u32,
}

#[allow(dead_code)]
pub enum EntityUpdateResult {
    Continue,
    Remove,
    ReplaceWith(Box<dyn NonPlayerEntity>),
}

pub trait NonPlayerEntity: HasID + OnHit + Position + AsAny + NonPlayerClone {
    fn update(&mut self, world: &mut World, input: Option<&InputHandler>) -> EntityUpdateResult;

    fn dir(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        None
    }
    fn draw_behind_players(&self) -> bool {
        false
    }
}

pub(crate) trait Player:
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
    + HasDeadState
    + AsAny
    + PlayerClone
{
    fn update(self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player>;
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        None
    }
    fn actionable(&self) -> bool {
        true
    }
    fn counterhit(&self) -> bool {
        false
    }
    #[allow(dead_code)]
    fn should_wall_bounce(&self) -> bool {
        false
    }
    fn in_hitstun(&self) -> bool {
        false
    }
    fn can_cancel(&self) -> bool {
        !self.in_hitstun()
    }
    fn throwable(&self) -> bool {
        true
    }
    fn moveable(&self) -> bool {
        true
    }
}
pub trait PlayerClone {
    fn clone(&self) -> Box<dyn Player>;
}
pub trait NonPlayerClone {
    fn clone(&self) -> Box<dyn NonPlayerEntity>;
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

#[allow(dead_code)]
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

pub trait HasDeadState {
    fn dead_state(self: Box<Self>) -> Box<dyn Player>;
}
