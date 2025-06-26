use super::{CharacterSpecificInitInfo, Damageable, Grounded, HasCancelState, HasID, Player};
use crate::collision::{
    AttackData, BounceInfo, CollisionShape, HitConnectionStatus, HitData, HitEffect, KnockdownType,
    OnHitHitData, Proration,
};
use crate::datatypes::*;
use crate::input::{
    directions::{InputDir, Motion},
    Action, Button, ButtonState, InputHandler,
};
use crate::world::{EntityID, World};

mod core_states;
mod normals;
mod resolvers;
mod specials;
mod trait_impls;

mod states {
    pub use super::core_states::*;
    pub use super::normals::*;
    pub use super::specials::*;
}
use states::*;

const MIN_RUN_FRAMES_BEFORE_CANCEL: usize = 4;
const BASE_DRAG: f32 = 4.0;

const GRAVITY: f32 = 9.0;

const BASE_SPRITE_OFFSET: Vector2 = Vector2::new(0.0, 8.0);
const COLLIDER_SIZE: Vector2 = Vector2::new(8.0, 16.0);

const STANDING_HURTBOX: BoundingBox = BoundingBox::pos_size(
    Vector2::new(0.0, (24.0 - COLLIDER_SIZE.y) / 2.0),
    Vector2::new(12.0, 24.0),
);
const CROUCHING_HURTBOX: BoundingBox = BoundingBox::pos_size(
    Vector2::new(0.0, (16.0 - COLLIDER_SIZE.y) / 2.0),
    Vector2::new(12.0, 16.0),
);

pub const fn initial_state(player: EntityID, position: Vector2) -> impl Player {
    Sol {
        player_id: player,
        position,
        velocity: Vector2::ZERO,
        collider: BoundingBox::pos_size(Vector2::ZERO, COLLIDER_SIZE),
        direction: true,
        has_hit: false,
        grounded: true,
        has_air_action: true,
        distance_from_other_player: f32::MAX,
        frame: 0,
        state: Stand,
    }
}
pub const fn init_info() -> CharacterSpecificInitInfo {
    CharacterSpecificInitInfo { max_health: 650 }
}

struct Sol<S> {
    player_id: EntityID,
    position: Vector2,
    velocity: Vector2,
    collider: BoundingBox,
    direction: bool,
    has_hit: bool,
    grounded: bool,
    has_air_action: bool,
    distance_from_other_player: f32,
    frame: usize,
    state: S,
}
impl<S> Sol<S> {
    const fn dir(&self) -> f32 {
        if self.direction {
            1.0
        } else {
            -1.0
        }
    }
    const fn forward_dir(&self) -> InputDir {
        match self.direction {
            true => InputDir::Dir6,
            false => InputDir::Dir4,
        }
    }
    const fn backward_dir(&self) -> InputDir {
        match self.direction {
            true => InputDir::Dir4,
            false => InputDir::Dir6,
        }
    }
    fn transition<N>(self, new_state: N, reset_frame: bool) -> Sol<N> {
        Sol {
            player_id: self.player_id,
            position: self.position,
            velocity: self.velocity,
            collider: self.collider,
            direction: self.direction,
            has_hit: self.has_hit,
            grounded: self.grounded,
            has_air_action: self.has_air_action,
            distance_from_other_player: self.distance_from_other_player,
            frame: if reset_frame { 0 } else { self.frame },
            state: new_state,
        }
    }
    fn gravity(&mut self) {
        if !self.grounded {
            self.velocity += Vector2::DOWN * GRAVITY;
        }
    }
    fn forward_drag(&mut self, drag: f32) {
        if self.velocity.x * self.dir() > 0.0 {
            self.velocity.x = self.velocity.x.move_towards(0.0, drag);
        }
    }
    #[allow(dead_code)]
    fn backward_drag(&mut self, drag: f32) {
        if self.velocity.x * self.dir() < 0.0 {
            self.velocity.x = self.velocity.x.move_towards(0.0, drag);
        }
    }
    fn drag(&mut self, drag: f32) {
        self.velocity.x = self
            .velocity
            .x
            .move_towards(0.0, if self.has_hit { BASE_DRAG } else { drag });
    }
}

trait SolDamageableState {}

fn has_dash_input(input: &InputHandler, dir: InputDir, allow_up_inputs: bool) -> bool {
    let move_dir = input.move_dir();
    let dir_v = dir.into_vector2();
    (input.has_action(&Action::Pressed(Button::Utility, None))
        || input.has_action(&Action::DoublePress(dir)))
        && move_dir.x == dir_v.x
        && (allow_up_inputs || move_dir.y <= 0.0)
}
