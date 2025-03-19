use crate::collision::{HitConnection, HitInfo};
use crate::datatypes::{BoundingBox, Vector2};
use crate::input::InputHandler;
use crate::world::World;

pub mod sol;

pub trait Entity:
    Damageable
    + OnHit
    + Position
    + Velocity
    + HasCollider
    + ColliderWorldSpace
    + Grounded
    + Direction
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
}
pub trait Damageable {
    fn hit(self: Box<Self>, info: &HitInfo) -> (Box<dyn Entity>, HitConnection);
}
pub trait OnHit {
    fn on_hit(&mut self, hit_type: HitConnection);
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
