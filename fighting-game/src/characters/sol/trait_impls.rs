use super::super::{
    Damageable, Direction, DistanceFromOtherPlayer, Grounded, HasCollider, HasDeadState, HasID,
    HasThrownState, OnHit, Player, Position, Velocity,
};

use super::*;

impl<S> HasID for Sol<S> {
    fn id(&self) -> EntityID {
        self.player_id
    }
}
impl<S> Position for Sol<S> {
    fn position(&self) -> Vector2 {
        self.position
    }
    fn move_by(&mut self, distance: Vector2) {
        self.position += distance;
    }
    fn set_position(&mut self, position: Vector2) {
        self.position = position;
    }
}
impl<S> Velocity for Sol<S> {
    fn velocity(&self) -> Vector2 {
        self.velocity
    }
    fn add_velocity(&mut self, velocity: Vector2) {
        self.velocity += velocity;
    }
    fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
    }
}
impl<S> HasCollider for Sol<S> {
    fn get_collider(&self) -> &BoundingBox {
        &self.collider
    }
}
impl<S> Grounded for Sol<S> {
    fn set_grounded(&mut self, grounded: bool) {
        self.grounded = grounded;
        if grounded {
            self.has_air_action = true;
        }
    }
    fn is_grounded(&self) -> bool {
        self.grounded
    }
}
impl<S> Direction for Sol<S>
where
    Sol<S>: Player,
{
    fn get_direction(&self) -> bool {
        self.direction
    }
    fn set_direction(&mut self, direction: bool) {
        if self.actionable() && self.grounded {
            self.direction = direction
        }
    }
}
impl<S> DistanceFromOtherPlayer for Sol<S>
where
    Sol<S>: Player,
{
    fn set_distance(&mut self, distance: f32) {
        self.distance_from_other_player = distance;
    }
}

impl<S> Damageable for Sol<S>
where
    S: SolDamageableState,
{
    fn hit(self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        Sol::hit(self, info)
    }
}

impl Damageable for Sol<SolCancelState> {
    fn hit(self: Box<Self>, _: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        (self, HitConnectionStatus::Invuln)
    }
}

impl<S> HasThrownState for Sol<S>
where
    Sol<S>: Player,
{
    fn thrown(self: Box<Self>) -> Box<dyn Player> {
        Box::new(self.transition(ThrownState, true))
    }
}

impl<S> HasDeadState for Sol<S> {
    fn dead_state(self: Box<Self>) -> Box<dyn Player> {
        Box::new(self.transition(DeadState, true))
    }
}

impl<S> OnHit for Sol<S> {
    fn on_hit(&mut self, hit_connection: HitConnectionStatus) {
        match hit_connection {
            HitConnectionStatus::Hit | HitConnectionStatus::Blocked => self.has_hit = true,
            _ => (),
        }
    }
}
