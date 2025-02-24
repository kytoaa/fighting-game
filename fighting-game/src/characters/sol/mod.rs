use super::{Damageable, Direction, Entity, Grounded, HasCollider, OnHit, Position, Velocity};
use crate::collision::{HitEffect, HitInfo, KnockdownType};
use crate::datatypes::{BoundingBox, Vector2};
use crate::input::{
    directions::{InputDir, Motion},
    Action, Button, InputHandler,
};
use crate::world::World;

mod attacks;
use attacks::*;

const WALK_SPEED: f32 = 20.0;
const RUN_SPEED: f32 = 70.0;

pub fn initial_state(player: usize) -> Box<dyn Entity> {
    Box::new(Sol {
        player,
        position: Vector2::ZERO,
        velocity: Vector2::ZERO,
        collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(8.0, 12.0)),
        direction: true,
        has_hit: false,
        grounded: true,
        has_air_action: true,
        state: Stand,
    })
}

struct Sol<S> {
    player: usize,
    position: Vector2,
    velocity: Vector2,
    collider: BoundingBox,
    direction: bool,
    has_hit: bool,
    grounded: bool,
    has_air_action: bool,
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
    fn transition<N>(self, new_state: N) -> Sol<N> {
        Sol {
            player: self.player,
            position: self.position,
            velocity: self.velocity,
            collider: self.collider,
            direction: self.direction,
            has_hit: self.has_hit,
            grounded: self.grounded,
            has_air_action: self.has_air_action,
            state: new_state,
        }
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
impl<S> Direction for Sol<S> {
    fn get_direction(&self) -> bool {
        self.direction
    }
    fn set_direction(&mut self, direction: bool) {
        if self.grounded {
            self.direction = direction
        }
    }
}

trait SolDamageableState {}

impl<S> Damageable for Sol<S>
where
    S: SolDamageableState,
{
    fn hit(mut self: Box<Self>, info: &HitInfo) -> Box<dyn Entity> {
        match &info.hit_effect {
            HitEffect::Pushback(force) => {
                self.velocity.x = *force;
                Box::new(self.transition(BasicHitstun {
                    length: info.hitstun,
                    frame: 0,
                }))
            }
            HitEffect::Launcher(force, knockdown) => {
                self.velocity = *force;
                self.grounded = false;
                Box::new(self.transition(Tumble {
                    length: info.hitstun,
                    frame: 0,
                    knockdown: *knockdown,
                }))
            }
        }
    }
}

impl Damageable for Sol<WalkState<true>> {
    fn hit(self: Box<Self>, info: &HitInfo) -> Box<dyn Entity> {
        Box::new(self.transition(BlockStun::<false> {
            length: info.blockstun,
            frame: 0,
        }))
    }
}
impl Damageable for Sol<Crouch<true>> {
    fn hit(self: Box<Self>, info: &HitInfo) -> Box<dyn Entity> {
        Box::new(self.transition(BlockStun::<true> {
            length: info.blockstun,
            frame: 0,
        }))
    }
}
impl Damageable for Sol<Air<true>> {
    fn hit(self: Box<Self>, info: &HitInfo) -> Box<dyn Entity> {
        Box::new(self.transition(AirBlockStun {
            length: info.blockstun,
            frame: 0,
        }))
    }
}
impl Damageable for Sol<AirBlockStun> {
    fn hit(self: Box<Self>, info: &HitInfo) -> Box<dyn Entity> {
        Box::new(self.transition(AirBlockStun {
            length: info.blockstun,
            frame: 0,
        }))
    }
}
impl<const CROUCHING: bool> Damageable for Sol<BlockStun<CROUCHING>> {
    fn hit(self: Box<Self>, info: &HitInfo) -> Box<dyn Entity> {
        Box::new(self.transition(BlockStun::<CROUCHING> {
            length: info.blockstun,
            frame: 0,
        }))
    }
}
impl Damageable for Sol<Backdash> {
    fn hit(self: Box<Self>, _: &HitInfo) -> Box<dyn Entity> {
        self
    }
}

impl<S> OnHit for Sol<S> {
    fn on_hit(&mut self) {
        self.has_hit = true;
    }
}

const GRAVITY: f32 = 7.0;
impl<S> Sol<S>
where
    Sol<S>: Entity + 'static,
{
    fn grounded_actionable_state(self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Entity> {
        if input.has_motion_input(
            &Motion::quarter_circle().direction(self.direction),
            &Action::Pressed(Button::Light),
        ) {
            return Box::new(self.transition(GunFlameStartup(0)));
        }
        if input.has_action(&Action::DoublePress(self.forward_dir())) {
            return Box::new(self.transition(RunState));
        }
        if input.has_action(&Action::DoublePress(self.backward_dir())) {
            return Box::new(self.transition(Backdash(0)));
        }
        self.walk_block_state(input)
    }

    fn walk_block_state(self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Entity> {
        match input.move_dir().into() {
            (0.0, 0.0) => Box::new(self.transition(Stand)),
            (d, -1.0) => {
                if d.round() == -self.dir() {
                    Box::new(self.transition(Crouch::<true>))
                } else {
                    Box::new(self.transition(Crouch::<false>))
                }
            }
            (d, 0.0) => {
                if d.round() == -self.dir() {
                    Box::new(self.transition(WalkState::<true>))
                } else {
                    Box::new(self.transition(WalkState::<false>))
                }
            }
            (d, 1.0) => Box::new(self.transition(JumpSquat {
                direction: d,
                frame: 0,
            })),
            _ => unreachable!(),
        }
    }
    fn air_actionable_state(self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Entity> {
        if input.has_action(&Action::Pressed(Button::Mid)) {
            return Box::new(self.transition(JumpMidStartup(0)));
        }
        self.air_movement_state(input)
    }
    fn air_movement_state(mut self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Entity> {
        let dir = input.move_dir();

        if self.grounded {
            return if dir.x != self.dir() {
                Box::new(self.transition(WalkState::<true>))
            } else {
                Box::new(self.transition(WalkState::<false>))
            };
        }
        if input.has_action(&Action::DoublePress(self.forward_dir())) && self.has_air_action {
            self.has_air_action = false;
            return Box::new(self.transition(Airdash(0)));
        }
        if input.has_action(&Action::DoublePress(self.backward_dir())) && self.has_air_action {
            self.has_air_action = false;
            return Box::new(self.transition(Backdash(0)));
        }

        if (input.has_action(&Action::JumpPress(InputDir::Dir7))
            || input.has_action(&Action::JumpPress(InputDir::Dir8))
            || input.has_action(&Action::JumpPress(InputDir::Dir9)))
            && self.has_air_action
        {
            self.has_air_action = false;
            self.velocity = Vector2::new(
                dir.x * DOUBLE_JUMP_X_FORCE.max(self.velocity.x.abs()),
                DOUBLE_JUMP_FORCE,
            );
        }

        if dir == Vector2::new(-self.dir(), 0.0) {
            Box::new(self.transition(Air::<true>))
        } else {
            Box::new(self.transition(Air::<false>))
        }
    }

    fn gravity(&mut self) {
        if !self.grounded {
            self.velocity += Vector2::DOWN * GRAVITY;
        }
    }
}

struct WalkState<const BLOCKING: bool>;
impl<const BLOCKING: bool> Entity for Sol<WalkState<BLOCKING>>
where
    Sol<WalkState<BLOCKING>>: Damageable,
{
    fn update(mut self: Box<Self>, _: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.velocity = input.move_dir().y(0.0) * WALK_SPEED;
        self.grounded_actionable_state(input)
    }
}
impl SolDamageableState for WalkState<false> {}

struct RunState;
impl Entity for Sol<RunState>
where
    Sol<RunState>: Damageable,
{
    fn update(mut self: Box<Self>, _: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        if input.move_dir() == Vector2::new(self.dir(), 0.0) {
            self.velocity = input.move_dir().y(0.0) * RUN_SPEED;
            self
        } else {
            self.grounded_actionable_state(input)
        }
    }
}
impl SolDamageableState for RunState {}

const BACKDASH_VELOCITY: f32 = 70.0;
const BACKDASH_FRAMES: usize = 5;
const BACKDASH_VULNERABLE: usize = 10;

struct Backdash(usize);
impl Entity for Sol<Backdash> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.velocity = Vector2::new(-self.dir() * BACKDASH_VELOCITY, 0.0);
        self.state.0 += 1;
        if self.state.0 > BACKDASH_FRAMES {
            Box::new(self.transition(BackdashVulnerable(0)))
        } else {
            self
        }
    }
}
struct BackdashVulnerable(usize);
impl Entity for Sol<BackdashVulnerable> {
    fn update(mut self: Box<Self>, _: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.state.0 += 1;
        if self.state.0 > BACKDASH_VULNERABLE {
            if self.grounded {
                self.grounded_actionable_state(input)
            } else {
                self.air_actionable_state(input)
            }
        } else {
            self
        }
    }
}
impl SolDamageableState for BackdashVulnerable {}

struct Stand;
impl Entity for Sol<Stand> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.velocity = Vector2::ZERO;
        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(self.collider.clone()),
                owner: self.player,
            },
            self.position,
            1,
        );

        self.grounded_actionable_state(input)
    }
}
impl SolDamageableState for Stand {}

struct Crouch<const BLOCKING: bool>;
impl<const BLOCKING: bool> Entity for Sol<Crouch<BLOCKING>>
where
    Sol<Crouch<BLOCKING>>: Damageable,
{
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.velocity = Vector2::ZERO;
        self.grounded_actionable_state(input)
    }
}
impl SolDamageableState for Crouch<false> {}

const JUMPSQUAT_FRAMES: usize = 4;
const JUMP_FORCE: f32 = 120.0;
const DOUBLE_JUMP_FORCE: f32 = 100.0;
const DOUBLE_JUMP_X_FORCE: f32 = 40.0;

struct JumpSquat {
    frame: usize,
    direction: f32,
}
impl SolDamageableState for JumpSquat {}
impl Entity for Sol<JumpSquat> {
    fn update(mut self: Box<Self>, _world: &mut World, _input: &InputHandler) -> Box<dyn Entity> {
        self.state.frame += 1;
        if self.state.frame > JUMPSQUAT_FRAMES {
            self.velocity.y = Vector2::UP.y * JUMP_FORCE;
            Box::new(self.transition(Air::<false>))
        } else {
            self
        }
    }
}

struct Air<const BLOCKING: bool>;
impl<const B: bool> Entity for Sol<Air<B>>
where
    Sol<Air<B>>: Damageable,
{
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.gravity();
        self.air_actionable_state(input)
    }
}
impl SolDamageableState for Air<false> {}

const AIRDASH_LENGTH: usize = 15;
const AIRDASH_SPEED: f32 = 110.0;

#[derive(Debug)]
struct Airdash(usize);
impl Entity for Sol<Airdash> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.velocity = Vector2::new(AIRDASH_SPEED * self.dir(), 0.0);
        self.state.0 += 1;
        if self.state.0 > AIRDASH_LENGTH {
            Box::new(self.transition(Air::<false>))
        } else {
            self
        }
    }
}
impl SolDamageableState for Airdash {}

struct BasicHitstun {
    length: usize,
    frame: usize,
}
impl Entity for Sol<BasicHitstun> {
    fn update(mut self: Box<Self>, world: &mut World, _input: &InputHandler) -> Box<dyn Entity> {
        self.state.frame += 1;
        if self.state.frame > self.state.length {
            Box::new(self.transition(Stand))
        } else {
            self
        }
    }
}
impl SolDamageableState for BasicHitstun {}

#[derive(Debug)]
struct Tumble {
    length: usize,
    frame: usize,
    knockdown: KnockdownType,
}
impl Entity for Sol<Tumble> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.gravity();
        println!(
            "grounded: {}, velocity: {:?}, frame: {}",
            self.grounded, self.velocity, self.state.frame
        );
        if self.grounded {
            return match self.state.knockdown {
                KnockdownType::Hard => Box::new(self.transition(HardKnockdown(0))),
                KnockdownType::Soft => Box::new(self.transition(SoftKnockdown(0))),
            };
        }
        self.state.frame += 1;
        if self.state.frame > self.state.length {
            self.air_actionable_state(input)
        } else {
            self
        }
    }
}
impl SolDamageableState for Tumble {}

#[derive(Debug)]
struct BlockStun<const CROUCHING: bool> {
    length: usize,
    frame: usize,
}
impl<const CROUCHING: bool> Entity for Sol<BlockStun<CROUCHING>> {
    fn update(mut self: Box<Self>, world: &mut World, _input: &InputHandler) -> Box<dyn Entity> {
        self.state.frame += 1;
        if self.state.frame >= self.state.length {
            Box::new(self.transition(Stand))
        } else {
            self
        }
    }
}
#[derive(Debug)]
struct AirBlockStun {
    length: usize,
    frame: usize,
}
impl Entity for Sol<AirBlockStun> {
    fn update(mut self: Box<Self>, world: &mut World, _input: &InputHandler) -> Box<dyn Entity> {
        self.state.frame += 1;
        if self.state.frame >= self.state.length {
            Box::new(self.transition(Stand))
        } else {
            self
        }
    }
}

const HARD_KNOCKDOWN_FRAMES: usize = 30;
const SOFT_KNOCKDOWN_FRAMES: usize = 15;

#[derive(Debug)]
struct SoftKnockdown(usize);
impl Entity for Sol<SoftKnockdown> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.state.0 += 1;
        if self.state.0 >= SOFT_KNOCKDOWN_FRAMES {
            self.walk_block_state(input)
        } else {
            self
        }
    }
}
impl Damageable for Sol<SoftKnockdown> {
    fn hit(self: Box<Self>, _: &HitInfo) -> Box<dyn Entity> {
        self
    }
}

#[derive(Debug)]
struct HardKnockdown(usize);
impl Entity for Sol<HardKnockdown> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.state.0 += 1;
        if self.state.0 >= HARD_KNOCKDOWN_FRAMES {
            self.walk_block_state(input)
        } else {
            self
        }
    }
}
impl Damageable for Sol<HardKnockdown> {
    fn hit(self: Box<Self>, _: &HitInfo) -> Box<dyn Entity> {
        self
    }
}
