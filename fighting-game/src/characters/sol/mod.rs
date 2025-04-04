use super::{
    Damageable, Direction, DistanceFromOtherPlayer, Entity, Grounded, HasCollider, OnHit, Position,
    Velocity,
};
use crate::collision::{AttackData, HitConnection, HitEffect, HitType, KnockdownType};
use crate::datatypes::{BoundingBox, Vector2};
use crate::input::{
    directions::{InputDir, Motion},
    Action, Button, InputHandler,
};
use crate::world::World;

mod attacks;
mod normals;
use attacks::*;
use normals::*;

const WALK_SPEED: f32 = 30.0;
const RUN_SPEED: f32 = 90.0;
const MIN_RUN_FRAMES_BEFORE_CANCEL: u8 = 4;

const BASE_SPRITE_OFFSET: Vector2 = Vector2::new(0.0, 10.0);
const COLLIDER_SIZE: Vector2 = Vector2::new(8.0, 12.0);

const DEFAULT_COLLIDER: BoundingBox = BoundingBox::pos_size(
    Vector2::new(0.0, (24.0 - COLLIDER_SIZE.y) / 2.0),
    Vector2::new(12.0, 24.0),
);

pub fn initial_state(player: usize) -> Box<dyn Entity> {
    Box::new(Sol {
        player,
        position: Vector2::ZERO,
        velocity: Vector2::ZERO,
        collider: BoundingBox::pos_size(Vector2::ZERO, COLLIDER_SIZE),
        direction: true,
        has_hit: false,
        grounded: true,
        has_air_action: true,
        distance_from_other_player: f32::MAX,
        frame: 0,
        combo_hit_count: 0,
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
    distance_from_other_player: f32,
    frame: u8,
    combo_hit_count: u8,
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
            player: self.player,
            position: self.position,
            velocity: self.velocity,
            collider: self.collider,
            direction: self.direction,
            has_hit: self.has_hit,
            grounded: self.grounded,
            has_air_action: self.has_air_action,
            distance_from_other_player: self.distance_from_other_player,
            frame: if reset_frame { 0 } else { self.frame },
            combo_hit_count: self.combo_hit_count,
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
impl<S> Direction for Sol<S>
where
    Sol<S>: Entity,
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
    Sol<S>: Entity,
{
    fn set_distance(&mut self, distance: f32) {
        self.distance_from_other_player = distance;
    }
}

trait SolDamageableState {}

impl<S> Damageable for Sol<S>
where
    S: SolDamageableState,
{
    fn hit(mut self: Box<Self>, info: &AttackData) -> (Box<dyn Entity>, HitConnection) {
        self.combo_hit_count += 1;
        println!("{} hits", self.combo_hit_count);
        let hit_info = if self.is_grounded() {
            &info.grounded
        } else {
            &info.air
        };
        (
            match &hit_info.hit_effect {
                HitEffect::Pushback(force) => {
                    self.velocity.x = *force;
                    Box::new(self.transition(
                        BasicHitstun {
                            length: hit_info.hitstun,
                        },
                        true,
                    ))
                }
                HitEffect::Launcher(force, knockdown) => {
                    self.velocity = *force;
                    self.grounded = false;
                    Box::new(self.transition(
                        Tumble {
                            length: hit_info.hitstun,
                            knockdown: *knockdown,
                        },
                        true,
                    ))
                }
            },
            HitConnection::Hit,
        )
    }
}

impl Damageable for Sol<WalkState<true>> {
    fn hit(self: Box<Self>, info: &AttackData) -> (Box<dyn Entity>, HitConnection) {
        let hit_info = if self.is_grounded() {
            &info.grounded
        } else {
            &info.air
        };
        (
            Box::new(self.transition(
                BlockStun::<false> {
                    length: hit_info.blockstun,
                },
                true,
            )),
            HitConnection::Blocked,
        )
    }
}
impl Damageable for Sol<Crouch<true>> {
    fn hit(self: Box<Self>, info: &AttackData) -> (Box<dyn Entity>, HitConnection) {
        let hit_info = if self.is_grounded() {
            &info.grounded
        } else {
            &info.air
        };
        (
            Box::new(self.transition(
                BlockStun::<true> {
                    length: hit_info.blockstun,
                },
                true,
            )),
            HitConnection::Blocked,
        )
    }
}
impl Damageable for Sol<Air<true>> {
    fn hit(self: Box<Self>, info: &AttackData) -> (Box<dyn Entity>, HitConnection) {
        let hit_info = if self.is_grounded() {
            &info.grounded
        } else {
            &info.air
        };
        (
            Box::new(self.transition(
                AirBlockStun {
                    length: hit_info.blockstun,
                },
                true,
            )),
            HitConnection::Blocked,
        )
    }
}
impl Damageable for Sol<AirBlockStun> {
    fn hit(self: Box<Self>, info: &AttackData) -> (Box<dyn Entity>, HitConnection) {
        let hit_info = if self.is_grounded() {
            &info.grounded
        } else {
            &info.air
        };
        (
            Box::new(self.transition(
                AirBlockStun {
                    length: hit_info.blockstun,
                },
                true,
            )),
            HitConnection::Blocked,
        )
    }
}
impl<const CROUCHING: bool> Damageable for Sol<BlockStun<CROUCHING>> {
    fn hit(self: Box<Self>, info: &AttackData) -> (Box<dyn Entity>, HitConnection) {
        let hit_info = if self.is_grounded() {
            &info.grounded
        } else {
            &info.air
        };
        (
            Box::new(self.transition(
                BlockStun::<CROUCHING> {
                    length: hit_info.blockstun,
                },
                true,
            )),
            HitConnection::Blocked,
        )
    }
}
impl Damageable for Sol<Backdash> {
    fn hit(self: Box<Self>, _: &AttackData) -> (Box<dyn Entity>, HitConnection) {
        (self, HitConnection::Invuln)
    }
}

impl<S> OnHit for Sol<S> {
    fn on_hit(&mut self, hit_connection: HitConnection) {
        match hit_connection {
            HitConnection::Hit | HitConnection::Blocked => self.has_hit = true,
            _ => (),
        }
    }
}

const GRAVITY: f32 = 9.0;
impl<S> Sol<S>
where
    Sol<S>: Entity + 'static,
{
    fn grounded_actionable_state(mut self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Entity> {
        self.combo_hit_count = 0;
        match self.grounded_cancel_options(input) {
            Ok(state) => state,
            Err(s) => s.walk_block_state(input),
        }
    }
    fn grounded_cancel_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Entity>, Box<Sol<S>>> {
        match self.grounded_attack_options(input) {
            Ok(state) => Ok(state),
            Err(s) => s.grounded_movement_cancel_options(input),
        }
    }
    fn grounded_movement_cancel_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Entity>, Box<Sol<S>>> {
        if input.has_action(&Action::DoublePress(self.forward_dir()))
            && input.move_dir().x == self.dir()
        {
            return Ok(Box::new(self.transition(RunStartState::new(), true)));
        }
        if input.has_action(&Action::DoublePress(self.backward_dir())) {
            return Ok(Box::new(self.transition(Backdash, true)));
        }
        let move_dir = input.move_dir();
        if move_dir.y == Vector2::UP.y {
            return Ok(Box::new(self.transition(
                JumpSquat {
                    direction: move_dir.x,
                },
                true,
            )));
        }
        Err(self)
    }
    fn grounded_movement_cancel_options_from_attack<const FRAMES: u8>(
        self: Box<Sol<S>>,
        input: &InputHandler,
        dash_cancel_state: RunStartState<FRAMES>,
    ) -> Result<Box<dyn Entity>, Box<Sol<S>>> {
        if input.has_action(&Action::DoublePress(self.forward_dir()))
            && input.move_dir().x == self.dir()
        {
            return Ok(Box::new(self.transition(dash_cancel_state, true)));
        }
        if input.has_action(&Action::DoublePress(self.backward_dir())) {
            return Ok(Box::new(self.transition(Backdash, true)));
        }
        let move_dir = input.move_dir();
        if move_dir.y == Vector2::UP.y {
            return Ok(Box::new(self.transition(
                JumpSquat {
                    direction: move_dir.x,
                },
                true,
            )));
        }
        Err(self)
    }
    fn grounded_attack_options(
        mut self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Entity>, Box<Sol<S>>> {
        match self.grounded_special_cancel(input) {
            Ok(state) => return Ok(state),
            Err(s) => self = s,
        }

        // NOTE: 2h
        if input.has_action(&Action::Pressed(Button::Heavy, Some(InputDir::Dir2))) {
            return Ok(Box::new(self.transition(CrouchHeavyStartup(0), true)));
        }

        const CLOSE_MID_DISTANCE: f32 = 16.0;

        // NOTE: c.m and f.m
        if input.has_action(&Action::Pressed(Button::Mid, None)) {
            if self.distance_from_other_player < CLOSE_MID_DISTANCE {
                return Ok(Box::new(self.transition(CloseMid, true)));
            } else {
                return Ok(Box::new(self.transition(FarMid, true)));
            }
        }

        // NOTE: 5l
        if input.has_action(&Action::Pressed(Button::Light, None)) {
            return Ok(Box::new(self.transition(StandLight, true)));
        }

        // NOTE: 5h
        if input.has_action(&Action::Pressed(Button::Heavy, None)) {
            return Ok(Box::new(self.transition(StandHeavy, true)));
        }

        Err(self)
    }

    fn walk_block_state(self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Entity> {
        match input.move_dir().into() {
            (0.0, 0.0) => Box::new(self.transition(Stand, true)),
            (d, -1.0) => {
                if d.round() == -self.dir() {
                    Box::new(self.transition(Crouch::<true>, true))
                } else {
                    Box::new(self.transition(Crouch::<false>, true))
                }
            }
            (d, 0.0) => {
                if d.round() == -self.dir() {
                    Box::new(self.transition(WalkState::<true>, false))
                } else {
                    Box::new(self.transition(WalkState::<false>, false))
                }
            }
            (d, 1.0) => Box::new(self.transition(JumpSquat { direction: d }, true)),
            _ => unreachable!(),
        }
    }
    fn air_actionable_state(mut self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Entity> {
        self.combo_hit_count = 0;
        match self.air_attack_options(input) {
            Ok(s) => s,
            Err(s) => s.air_movement_state(input),
        }
    }
    fn air_attack_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Entity>, Box<Sol<S>>> {
        if input.has_action(&Action::Pressed(Button::Mid, None)) {
            return Ok(Box::new(self.transition(JumpMidStartup, true)));
        }
        Err(self)
    }
    fn air_movement_state(mut self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Entity> {
        let dir = input.move_dir();

        if self.grounded {
            return if dir.x == -self.dir() {
                Box::new(self.transition(WalkState::<true>, true))
            } else if dir.x == self.dir() {
                Box::new(self.transition(WalkState::<false>, true))
            } else {
                Box::new(self.transition(Stand, true))
            };
        }
        if input.has_action(&Action::DoublePress(self.forward_dir())) && self.has_air_action {
            self.has_air_action = false;
            return Box::new(self.transition(Airdash, true));
        }
        if input.has_action(&Action::DoublePress(self.backward_dir())) && self.has_air_action {
            self.has_air_action = false;
            return Box::new(self.transition(Backdash, true));
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
            self.frame = 0;
        }

        if dir == Vector2::new(-self.dir(), 0.0) {
            Box::new(self.transition(Air::<true>, false))
        } else {
            Box::new(self.transition(Air::<false>, false))
        }
    }

    fn gravity(&mut self) {
        if !self.grounded {
            self.velocity += Vector2::DOWN * GRAVITY;
        }
    }

    fn grounded_special_cancel(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Entity>, Box<Sol<S>>> {
        // NOTE: fafnir
        if input.has_motion_input(
            &Motion::half_circle().direction(self.direction),
            &Action::Pressed(Button::Heavy, None),
        ) {
            return Ok(Box::new(self.transition(FafnirStartup(0), true)));
        }

        // NOTE: gun flame
        if input.has_motion_input(
            &Motion::quarter_circle().direction(self.direction),
            &Action::Pressed(Button::Light, None),
        ) {
            return Ok(Box::new(self.transition(GunFlameStartup::real(), true)));
        }
        if input.has_motion_input(
            &Motion::quarter_circle().direction(!self.direction),
            &Action::Pressed(Button::Light, None),
        ) {
            return Ok(Box::new(self.transition(GunFlameStartup::feint(), true)));
        }

        Err(self)
    }
}

const WALK_ANIM_LENGTH: u8 = 4;
const FRAMES_PER_WALK_ANIM_FRAME: u8 = 10;
struct WalkState<const BLOCKING: bool>;
impl<const BLOCKING: bool> Entity for Sol<WalkState<BLOCKING>>
where
    Sol<WalkState<BLOCKING>>: Damageable,
{
    fn update(mut self: Box<Self>, _: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.frame += 1;
        if self.frame >= WALK_ANIM_LENGTH * FRAMES_PER_WALK_ANIM_FRAME {
            self.frame = 0;
        }
        self.velocity = input.move_dir().y(0.0) * WALK_SPEED;
        self.grounded_actionable_state(input)
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        let mut path: String = "sol/walk/walk".into();
        path.push(
            (if !BLOCKING {
                '1' as u8 + (self.frame / FRAMES_PER_WALK_ANIM_FRAME)
            } else {
                '4' as u8 - (self.frame / FRAMES_PER_WALK_ANIM_FRAME)
            }) as char,
        );

        Some((path.into(), BASE_SPRITE_OFFSET))
    }
}
impl SolDamageableState for WalkState<false> {}

const RUN_ANIM_LENGTH: u8 = 6;
const FRAMES_PER_RUN_ANIM_FRAME: u8 = 5;
struct RunState;
impl Entity for Sol<RunState>
where
    Sol<RunState>: Damageable,
{
    fn update(mut self: Box<Self>, _: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.frame += 1;
        if self.frame >= RUN_ANIM_LENGTH * FRAMES_PER_RUN_ANIM_FRAME {
            self.frame = 0;
        }
        if input.move_dir() == Vector2::new(self.dir(), 0.0) {
            self.velocity = Vector2::RIGHT * RUN_SPEED * self.dir();
            self.grounded_attack_options(input).unwrap_or_else(|s| s)
        } else {
            self.grounded_actionable_state(input)
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        let mut path: String = "sol/run/run".into();
        path.push(('1' as u8 + (self.frame / FRAMES_PER_RUN_ANIM_FRAME)) as char);

        Some((path.into(), BASE_SPRITE_OFFSET))
    }
}
impl SolDamageableState for RunState {}

struct RunStartState<const FRAMES: u8 = MIN_RUN_FRAMES_BEFORE_CANCEL>;
impl RunStartState {
    const fn dash_cancel() -> RunStartState<10> {
        RunStartState
    }
    const fn new() -> Self {
        RunStartState
    }
}
impl<const FRAMES: u8> Entity for Sol<RunStartState<FRAMES>>
where
    Sol<RunStartState<FRAMES>>: Damageable,
{
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        // NOTE: stops instantly cancelling dash into something, adds a little commitment and stops
        // dash cancel cancels
        self.frame += 1;
        if self.frame < FRAMES {
            self.velocity = Vector2::RIGHT * RUN_SPEED * self.dir();
            return self;
        } else {
            self.velocity = Vector2::RIGHT * RUN_SPEED * self.dir();
            Box::new(self.transition(RunState, false))
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        let mut path: String = "sol/run/run".into();
        path.push(('1' as u8 + (self.frame / FRAMES_PER_RUN_ANIM_FRAME)) as char);

        Some((path.into(), BASE_SPRITE_OFFSET))
    }
}
impl<const FRAMES: u8> SolDamageableState for RunStartState<FRAMES> {}

const BACKDASH_VELOCITY: f32 = 70.0;
const BACKDASH_FRAMES: usize = 6;
const BACKDASH_VULNERABLE: usize = 9;

struct Backdash;
impl Entity for Sol<Backdash> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.velocity = Vector2::new(-self.dir() * BACKDASH_VELOCITY, 0.0);
        self.frame += 1;
        if self.frame > BACKDASH_FRAMES as u8 {
            Box::new(self.transition(BackdashVulnerable, true))
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/backdash/backdash1".into(), BASE_SPRITE_OFFSET))
    }
}
struct BackdashVulnerable;
impl Entity for Sol<BackdashVulnerable> {
    fn update(mut self: Box<Self>, _: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.frame += 1;
        if self.frame > BACKDASH_VULNERABLE as u8 {
            if self.grounded {
                self.grounded_actionable_state(input)
            } else {
                self.air_actionable_state(input)
            }
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/backdash/backdash2".into(), BASE_SPRITE_OFFSET))
    }
}
impl SolDamageableState for BackdashVulnerable {}

const DECEL_RATE: f32 = 12.0;

struct Stand;
impl Entity for Sol<Stand> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.velocity = self.velocity.move_towards(&Vector2::ZERO, DECEL_RATE);
        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(DEFAULT_COLLIDER),
                owner: self.player,
            },
            self.position,
            1,
        );

        self.grounded_actionable_state(input)
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        if self.velocity.x * self.dir() > 5.0 {
            Some(("sol/run/run_stop".into(), BASE_SPRITE_OFFSET))
        } else {
            Some(("sol/idle".into(), BASE_SPRITE_OFFSET))
        }
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
const JUMP_FORCE: f32 = 175.0;
const DOUBLE_JUMP_FORCE: f32 = 150.0;
const DOUBLE_JUMP_X_FORCE: f32 = 40.0;

struct JumpSquat {
    direction: f32,
}
impl SolDamageableState for JumpSquat {}
impl Entity for Sol<JumpSquat> {
    fn update(mut self: Box<Self>, _world: &mut World, _input: &InputHandler) -> Box<dyn Entity> {
        self.frame += 1;
        if self.frame > JUMPSQUAT_FRAMES as u8 {
            if self.velocity.x.abs() < 1.0 {
                self.velocity.x = WALK_SPEED * self.state.direction;
            }
            self.velocity.y = Vector2::UP.y * JUMP_FORCE;
            Box::new(self.transition(Air::<false>, true))
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/fall/fall1".into(), BASE_SPRITE_OFFSET))
    }
}

struct Air<const BLOCKING: bool>;
impl<const B: bool> Entity for Sol<Air<B>>
where
    Sol<Air<B>>: Damageable,
{
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.frame += 1;
        self.gravity();
        self.air_actionable_state(input)
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some((
            match self.frame {
                0..10 => "sol/fall/fall2",
                10..30 => "sol/fall/fall3",
                _ => "sol/fall/fall2",
            }
            .into(),
            BASE_SPRITE_OFFSET,
        ))
    }
}
impl SolDamageableState for Air<false> {}

const AIRDASH_LENGTH: usize = 12;
const AIRDASH_SPEED: f32 = 140.0;
const AIRDASH_ACTIONABLE_FRAME: usize = 4;

#[derive(Debug)]
struct Airdash;
impl Entity for Sol<Airdash> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.velocity = Vector2::new(AIRDASH_SPEED * self.dir(), 0.0);
        self.frame += 1;
        if self.frame > AIRDASH_LENGTH as u8 {
            Box::new(self.transition(Air::<false>, true))
        } else {
            if self.frame > AIRDASH_ACTIONABLE_FRAME as u8 {
                match self.air_attack_options(input) {
                    Ok(s) => s,
                    Err(s) => s,
                }
            } else {
                self
            }
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some((
            match self.frame {
                0 => "sol/fall/fall1",
                1 => "sol/fall/fall2",
                _ => "sol/airdash",
            }
            .into(),
            BASE_SPRITE_OFFSET,
        ))
    }
}
impl SolDamageableState for Airdash {}

struct BasicHitstun {
    length: usize,
}
impl Entity for Sol<BasicHitstun> {
    fn update(mut self: Box<Self>, world: &mut World, _input: &InputHandler) -> Box<dyn Entity> {
        self.frame += 1;
        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(DEFAULT_COLLIDER),
                owner: self.player,
            },
            self.position,
            1,
        );
        if self.frame > self.state.length as u8 {
            Box::new(self.transition(Stand, true))
        } else {
            self
        }
    }
}
impl SolDamageableState for BasicHitstun {}

#[derive(Debug)]
struct Tumble {
    length: usize,
    knockdown: KnockdownType,
}
impl Entity for Sol<Tumble> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.gravity();
        /*println!(
            "grounded: {}, velocity: {:?}, frame: {}",
            self.grounded, self.velocity, self.state.frame
        );*/
        self.frame += 1;
        if self.grounded {
            let length = self
                .state
                .length
                .checked_sub(self.frame as usize)
                .unwrap_or(3); // NOTE: 3 frames of hitstun minimum
            return match self.state.knockdown {
                KnockdownType::Hard => Box::new(self.transition(HardKnockdown, true)),
                KnockdownType::Soft => Box::new(self.transition(SoftKnockdown, true)),
                KnockdownType::None => Box::new(self.transition(BasicHitstun { length }, true)),
            };
        }

        // TODO: maybe remove this in future
        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(self.collider.clone()),
                owner: self.player,
            },
            self.position,
            1,
        );

        self
    }
}
impl SolDamageableState for Tumble {}

#[derive(Debug)]
struct BlockStun<const CROUCHING: bool> {
    length: usize,
}
impl<const CROUCHING: bool> Entity for Sol<BlockStun<CROUCHING>> {
    fn update(mut self: Box<Self>, world: &mut World, _input: &InputHandler) -> Box<dyn Entity> {
        self.frame += 1;
        if self.frame >= self.state.length as u8 {
            Box::new(self.transition(Stand, true))
        } else {
            self
        }
    }
}
#[derive(Debug)]
struct AirBlockStun {
    length: usize,
}
impl Entity for Sol<AirBlockStun> {
    fn update(mut self: Box<Self>, world: &mut World, _input: &InputHandler) -> Box<dyn Entity> {
        self.frame += 1;
        if self.frame >= self.state.length as u8 {
            Box::new(self.transition(Stand, true))
        } else {
            self
        }
    }
}

const HARD_KNOCKDOWN_FRAMES: usize = 30;
const SOFT_KNOCKDOWN_FRAMES: usize = 15;

#[derive(Debug)]
struct SoftKnockdown;
impl Entity for Sol<SoftKnockdown> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.frame += 1;
        if self.frame >= SOFT_KNOCKDOWN_FRAMES as u8 {
            self.walk_block_state(input)
        } else {
            self
        }
    }
}
impl Damageable for Sol<SoftKnockdown> {
    fn hit(self: Box<Self>, _: &AttackData) -> (Box<dyn Entity>, HitConnection) {
        (self, HitConnection::Invuln)
    }
}

#[derive(Debug)]
struct HardKnockdown;
impl Entity for Sol<HardKnockdown> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.frame += 1;
        if self.frame >= HARD_KNOCKDOWN_FRAMES as u8 {
            self.walk_block_state(input)
        } else {
            self
        }
    }
}
impl Damageable for Sol<HardKnockdown> {
    fn hit(self: Box<Self>, _: &AttackData) -> (Box<dyn Entity>, HitConnection) {
        (self, HitConnection::Invuln)
    }
}
