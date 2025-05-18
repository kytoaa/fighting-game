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

pub(crate) use wrapper_state::WrapperState;

pub(crate) struct CharacterInitInfo {
    pub max_health: u32,
}

pub struct HitstunInfo {
    hit: OnHitHitData,
}

pub trait Entity:
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
    + AsAny
{
    fn update(self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity>;
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
    fn moveable(&self) -> bool {
        true
    }
}
pub trait Damageable {
    fn hit(self: Box<Self>, info: OnHitHitData) -> (Box<dyn Entity>, HitConnectionStatus);
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
    fn cancel_state() -> Box<dyn Entity>;
}
pub trait HasID {
    fn id(&self) -> EntityID;
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
