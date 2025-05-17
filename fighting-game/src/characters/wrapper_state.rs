use super::*;

pub struct WrapperState<const FRAMES: usize, F>
where
    F: Fn(usize) -> Option<(Box<str>, Vector2)> + 'static,
{
    entity: Box<dyn Entity>,
    frame: usize,
    anim: F,
}
impl<const FRAMES: usize, F> Entity for WrapperState<FRAMES, F>
where
    F: Fn(usize) -> Option<(Box<str>, Vector2)> + 'static,
{
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.entity = self.entity.update(world, input);
        if self.frame >= FRAMES {
            self.entity
        } else {
            self.frame += 1;
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        (self.anim)(self.frame)
    }
    fn actionable(&self) -> bool {
        self.entity.actionable()
    }
}
impl<const FRAMES: usize, F> HasID for WrapperState<FRAMES, F>
where
    F: Fn(usize) -> Option<(Box<str>, Vector2)> + 'static,
{
    fn id(&self) -> EntityID {
        self.entity.id()
    }
}

impl<const FRAMES: usize, F> Damageable for WrapperState<FRAMES, F>
where
    F: Fn(usize) -> Option<(Box<str>, Vector2)> + 'static,
{
    fn hit(self: Box<Self>, info: OnHitHitData) -> (Box<dyn Entity>, HitConnectionStatus) {
        self.entity.hit(info)
    }
}
impl<const FRAMES: usize, F> OnHit for WrapperState<FRAMES, F>
where
    F: Fn(usize) -> Option<(Box<str>, Vector2)> + 'static,
{
    fn on_hit(&mut self, hit_type: HitConnectionStatus) {
        self.entity.on_hit(hit_type)
    }
}
impl<const FRAMES: usize, F> Velocity for WrapperState<FRAMES, F>
where
    F: Fn(usize) -> Option<(Box<str>, Vector2)> + 'static,
{
    fn velocity(&self) -> Vector2 {
        self.entity.velocity()
    }
    fn add_velocity(&mut self, velocity: Vector2) {
        self.entity.add_velocity(velocity)
    }
    fn set_velocity(&mut self, velocity: Vector2) {
        self.entity.set_velocity(velocity)
    }
}
impl<const FRAMES: usize, F> HasCollider for WrapperState<FRAMES, F>
where
    F: Fn(usize) -> Option<(Box<str>, Vector2)> + 'static,
{
    fn get_collider(&self) -> &BoundingBox {
        self.entity.get_collider()
    }
}
impl<const FRAMES: usize, F> Grounded for WrapperState<FRAMES, F>
where
    F: Fn(usize) -> Option<(Box<str>, Vector2)> + 'static,
{
    fn is_grounded(&self) -> bool {
        self.entity.is_grounded()
    }
    fn set_grounded(&mut self, grounded: bool) {
        self.entity.set_grounded(grounded)
    }
}
impl<const FRAMES: usize, F> Position for WrapperState<FRAMES, F>
where
    F: Fn(usize) -> Option<(Box<str>, Vector2)> + 'static,
{
    fn position(&self) -> Vector2 {
        self.entity.position()
    }
    fn move_by(&mut self, distance: Vector2) {
        self.entity.move_by(distance)
    }
    fn set_position(&mut self, position: Vector2) {
        self.entity.set_position(position)
    }
}
impl<const FRAMES: usize, F> Direction for WrapperState<FRAMES, F>
where
    F: Fn(usize) -> Option<(Box<str>, Vector2)> + 'static,
{
    fn get_direction(&self) -> bool {
        self.entity.get_direction()
    }
    fn set_direction(&mut self, direction: bool) {
        self.entity.set_direction(direction)
    }
}
impl<const FRAMES: usize, F> DistanceFromOtherPlayer for WrapperState<FRAMES, F>
where
    F: Fn(usize) -> Option<(Box<str>, Vector2)> + 'static,
{
    fn set_distance(&mut self, distance: f32) {
        self.entity.set_distance(distance)
    }
}
