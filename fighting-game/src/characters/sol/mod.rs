use super::{
    CharacterInitInfo, Damageable, Direction, DistanceFromOtherPlayer, Grounded, HasCancelState,
    HasCollider, HasID, HasThrownState, OnHit, Player, Position, Velocity,
};
use crate::collision::{
    AttackData, BounceInfo, CollisionShape, HitConnectionStatus, HitData, HitEffect, KnockdownType,
    OnHitHitData, Proration,
};
use crate::datatypes::*;
use crate::input::{
    directions::{InputDir, Motion},
    Action, Button, InputHandler,
};
use crate::world::{EntityID, World};

mod normals;
mod specials;
use normals::*;
use specials::*;

const WALK_SPEED: f32 = 30.0;
const RUN_SPEED: f32 = 90.0;
const MIN_RUN_FRAMES_BEFORE_CANCEL: usize = 4;
const BASE_DRAG: f32 = 4.0;

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
pub const fn init_info() -> CharacterInitInfo {
    CharacterInitInfo { max_health: 500 }
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

trait SolDamageableState {}

impl<S> Damageable for Sol<S>
where
    S: SolDamageableState,
{
    fn hit(self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        Sol::hit(self, info)
    }
}

impl<S> HasCancelState for Sol<S>
where
    Sol<S>: Player,
{
    fn cancel_state(self: Box<Sol<S>>) -> Box<dyn Player> {
        if self.grounded {
            Box::new(self.transition(Stand, true))
        } else {
            Box::new(self.transition(Air::<false>, true))
        }
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

impl<S> Sol<S> {
    fn hit(mut self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        (
            match info.hit_effect {
                HitEffect::Pushback { force, frames } => {
                    self.velocity.x = force;
                    Box::new(self.transition(
                        BasicHitstun {
                            length: frames,
                            wall_pushback_mult: info.wall_pushback_mult,
                        },
                        true,
                    ))
                }
                HitEffect::Launcher {
                    knockback,
                    gravity,
                    knockdown,
                    momentum_scaling,
                    ground_bounce,
                    wall_bounce,
                } => {
                    self.velocity = Vector2::new(
                        self.velocity.x * momentum_scaling.0 + knockback.x,
                        self.velocity.y * momentum_scaling.1 + knockback.y,
                    );
                    self.grounded = false;
                    Box::new(self.transition(
                        Tumble {
                            gravity,
                            knockdown,
                            ground_bounce,
                            wall_bounce,
                            wall_pushback_mult: info.wall_pushback_mult,
                        },
                        true,
                    ))
                }
                HitEffect::FloatingCrumple {
                    knockback,
                    gravity,
                    landing_frames,
                } => {
                    self.velocity = knockback;
                    Box::new(self.transition(
                        FloatingCrumple {
                            gravity,
                            landing_frames,
                            wall_pushback_mult: info.wall_pushback_mult,
                        },
                        true,
                    ))
                }
            },
            HitConnectionStatus::Hit,
        )
    }
    fn hit_on_block<const CROUCHING: bool>() {}
}

impl Damageable for Sol<WalkState<true>> {
    fn hit(mut self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        self.velocity.x = info.block_pushback;

        if let crate::collision::AttackType::Low = info.attack_type {
            return Sol::hit(self, info);
        }
        (
            Box::new(self.transition(
                BlockStun::<false> {
                    length: info.blockstun,
                },
                true,
            )),
            HitConnectionStatus::Blocked,
        )
    }
}
impl Damageable for Sol<Crouch<true>> {
    fn hit(mut self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        self.velocity.x =
            info.block_pushback * (1.0 + ((info.wall_pushback_mult - 1.0) / 2.0).clamp(0.0, 2.0));

        if let crate::collision::AttackType::High = info.attack_type {
            return Sol::hit(self, info);
        }
        (
            Box::new(self.transition(
                BlockStun::<true> {
                    length: info.blockstun + 1,
                },
                true,
            )),
            HitConnectionStatus::Blocked,
        )
    }
}
impl Damageable for Sol<Air<true>> {
    fn hit(mut self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        self.velocity.x = info.block_pushback * 2.0;

        if let crate::collision::AttackType::Low = info.attack_type {
            return Sol::hit(self, info);
        }
        (
            Box::new(self.transition(AirBlockStun { length: 15 }, true)),
            HitConnectionStatus::Blocked,
        )
    }
}
impl Damageable for Sol<AirBlockStun> {
    fn hit(mut self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        self.velocity.x = info.block_pushback * 2.0;

        if let crate::collision::AttackType::Low = info.attack_type {
            return Sol::hit(self, info);
        }
        (
            Box::new(self.transition(AirBlockStun { length: 15 }, true)),
            HitConnectionStatus::Blocked,
        )
    }
}
impl<const CROUCHING: bool> Damageable for Sol<BlockStun<CROUCHING>> {
    fn hit(mut self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        self.velocity.x = info.block_pushback;

        match CROUCHING {
            true => {
                if let crate::collision::AttackType::High = info.attack_type {
                    return Sol::hit(self, info);
                }
            }
            false => {
                if let crate::collision::AttackType::Low = info.attack_type {
                    return Sol::hit(self, info);
                }
            }
        }
        (
            Box::new(self.transition(
                BlockStun::<CROUCHING> {
                    length: info.blockstun + if CROUCHING { 1 } else { 0 },
                },
                true,
            )),
            HitConnectionStatus::Blocked,
        )
    }
}
impl Damageable for Sol<Backdash> {
    fn hit(self: Box<Self>, _: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        (self, HitConnectionStatus::Invuln)
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

const GRAVITY: f32 = 9.0;
impl<S> Sol<S>
where
    Sol<S>: Player + 'static,
{
    fn grounded_actionable_state(self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Player> {
        match self.grounded_cancel_options(input) {
            Ok(state) => state,
            Err(s) => s.walk_block_state(input),
        }
    }
    fn grounded_cancel_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        match self.grounded_attack_options(input) {
            Ok(state) => Ok(state),
            Err(s) => s.grounded_movement_cancel_options(input),
        }
    }
    fn grounded_movement_cancel_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
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
    fn grounded_movement_cancel_options_from_attack<const FRAMES: usize>(
        self: Box<Sol<S>>,
        input: &InputHandler,
        dash_cancel_state: RunStartState<FRAMES>,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
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
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        match self.cancel_options_from_grounded_normal(input) {
            Ok(state) => return Ok(state),
            Err(s) => self = s,
        }
        match self.grounded_command_normal_cancel(input) {
            Ok(state) => return Ok(state),
            Err(s) => self = s,
        }

        if input.input_dir().is_down() {
            // NOTE: 2l
            if input.has_action(&Action::Pressed(Button::Light, None)) {
                return Ok(Box::new(self.transition(CrouchLight, true)));
            }

            // NOTE: 2m
            if input.has_action(&Action::Pressed(Button::Mid, None)) {
                return Ok(Box::new(self.transition(CrouchMid, true)));
            }

            // NOTE: 2h
            if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                return Ok(Box::new(self.transition(CrouchHeavy, true)));
            }
        }

        // NOTE: c.m and f.m
        if input.has_action(&Action::Pressed(Button::Mid, None)) {
            if self.distance_from_other_player < CloseMid::MAX_DISTANCE {
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

        // NOTE: ground throw forward
        if input.has_action(&Action::Pressed(
            Button::Utility,
            Some(InputDir::Dir6.dir(self.direction)),
        )) {
            return Ok(Box::new(self.transition(
                GroundThrow::<true> {
                    success: std::cell::Cell::new(true).into(),
                },
                true,
            )));
        }
        // NOTE: ground throw backward
        if input.has_action(&Action::Pressed(
            Button::Utility,
            Some(InputDir::Dir4.dir(self.direction)),
        )) {
            return Ok(Box::new(self.transition(
                GroundThrow::<false> {
                    success: std::cell::Cell::new(true).into(),
                },
                true,
            )));
        }

        Err(self)
    }

    fn walk_block_state(self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Player> {
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
                let reset_frame = self.frame >= WALK_ANIM_LENGTH * FRAMES_PER_WALK_ANIM_FRAME;
                if d.round() == -self.dir() {
                    Box::new(self.transition(WalkState::<true>, reset_frame))
                } else {
                    Box::new(self.transition(WalkState::<false>, reset_frame))
                }
            }
            (d, 1.0) => Box::new(self.transition(JumpSquat { direction: d }, true)),
            _ => unreachable!(),
        }
    }
    fn air_actionable_state(mut self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Player> {
        try_transition!(air_attack_options; self, input).air_movement_state(input)
    }
    fn air_attack_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        match self.air_special_cancel_options(input) {
            s @ Ok(_) => s,
            Err(s) => s.air_normal_options(input),
        }
    }
    fn air_special_cancel_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        if input.has_motion_input(
            &Motion::dp().direction(self.direction),
            &Action::Pressed(Button::Heavy, None),
        ) {
            return Ok(Box::new(self.transition(VolcanicViper, true)));
        }
        if input.has_motion_input(
            &Motion::quarter_circle().direction(self.direction),
            &Action::Pressed(Button::Mid, None),
        ) {
            return Ok(Box::new(self.transition(BanditRevolverAir, true)));
        }

        Err(self)
    }
    fn air_normal_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        if input.has_action(&Action::Pressed(Button::Light, None)) {
            return Ok(Box::new(self.transition(AirLight, true)));
        }
        if input.has_action(&Action::Pressed(Button::Mid, None)) {
            return Ok(Box::new(self.transition(AirMid, true)));
        }
        if input.has_action(&Action::Pressed(Button::Heavy, None)) {
            return Ok(Box::new(self.transition(AirHeavy, true)));
        }
        Err(self)
    }
    fn air_movement_cancel_options(
        mut self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        if !self.has_air_action {
            return Err(self);
        }
        if input.has_action(&Action::DoublePress(self.forward_dir()))
            && input.move_dir().x == self.dir()
        {
            self.has_air_action = false;
            return Ok(Box::new(self.transition(Airdash, true)));
        }
        if input.has_action(&Action::DoublePress(self.backward_dir())) {
            self.has_air_action = false;
            return Ok(Box::new(self.transition(Backdash, true)));
        }
        let move_dir = input.move_dir();
        if input.has_action(&Action::JumpPress(InputDir::Dir7))
            || input.has_action(&Action::JumpPress(InputDir::Dir8))
            || input.has_action(&Action::JumpPress(InputDir::Dir9))
        {
            self.double_jump(move_dir.x);
            return Ok(self.air_actionable_state(input));
        }
        Err(self)
    }
    fn air_movement_state(mut self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Player> {
        let dir = input.move_dir();

        if self.grounded {
            // NOTE: resets frame as lands in case of walking, walking does not reset frame
            self.frame = 0;
            return self.grounded_actionable_state(input);
        }
        self = try_transition!(air_movement_cancel_options; self, input);

        if dir.x == -self.dir() {
            Box::new(self.transition(Air::<true>, false))
        } else {
            Box::new(self.transition(Air::<false>, false))
        }
    }

    fn double_jump(&mut self, dir: f32) {
        self.has_air_action = false;
        self.velocity = Vector2::new(
            dir * DOUBLE_JUMP_X_FORCE.max(self.velocity.x.abs()),
            DOUBLE_JUMP_FORCE,
        );
        self.frame = 0;
    }

    fn grounded_command_normal_cancel(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        if input.has_action(&Action::Pressed(
            Button::Heavy,
            Some(InputDir::Dir3.dir(self.direction)),
        )) {
            return Ok(Box::new(self.transition(Heavy3, true)));
        }

        Err(self)
    }

    fn cancel_options_from_grounded_normal(
        mut self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        self = match self.grounded_special_cancel_options(input) {
            Ok(state) => return Ok(state),
            Err(s) => s,
        };
        self.grounded_command_normal_cancel(input)
    }
    fn grounded_special_cancel_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        // NOTE: fafnir
        if input.has_motion_input(
            &Motion::half_circle().direction(self.direction),
            &Action::Pressed(Button::Heavy, None),
        ) {
            return Ok(Box::new(self.transition(Fafnir, true)));
        }

        // NOTE: VOLCANIC VIPER!!!!
        if input.has_motion_input(
            &Motion::dp().direction(self.direction),
            &Action::Pressed(Button::Heavy, None),
        ) {
            return Ok(Box::new(self.transition(VolcanicViper, true)));
        }

        // NOTE: bandit revolver
        if input.has_motion_input(
            &Motion::quarter_circle().direction(self.direction),
            &Action::Pressed(Button::Mid, None),
        ) {
            return Ok(Box::new(self.transition(BanditRevolverGrounded, true)));
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

const WALK_ANIM_LENGTH: usize = 4;
const FRAMES_PER_WALK_ANIM_FRAME: usize = 10;
struct WalkState<const BLOCKING: bool>;
impl<const BLOCKING: bool> Player for Sol<WalkState<BLOCKING>>
where
    Sol<WalkState<BLOCKING>>: Damageable,
{
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.frame += 1;
        if self.frame >= WALK_ANIM_LENGTH * FRAMES_PER_WALK_ANIM_FRAME {
            self.frame = 0;
        }

        world.spawn_hurtbox(
            self.create_hurtbox(crate::collision::CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );

        self.velocity = input.move_dir().y(0.0) * WALK_SPEED;
        self.grounded_actionable_state(input)
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        let mut path: String = "sol/walk/walk".into();
        path.push(
            char::from_u32(
                (if !BLOCKING {
                    '1' as usize + (self.frame / FRAMES_PER_WALK_ANIM_FRAME)
                } else {
                    '4' as usize - (self.frame / FRAMES_PER_WALK_ANIM_FRAME)
                }) as u32,
            )
            .unwrap(),
        );

        Some((path.into(), BASE_SPRITE_OFFSET))
    }
}
impl SolDamageableState for WalkState<false> {}

const RUN_ANIM_LENGTH: usize = 6;
const FRAMES_PER_RUN_ANIM_FRAME: usize = 5;
struct RunState;
impl Player for Sol<RunState>
where
    Sol<RunState>: Damageable,
{
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.frame += 1;
        if self.frame >= RUN_ANIM_LENGTH * FRAMES_PER_RUN_ANIM_FRAME {
            self.frame = 0;
        }

        world.spawn_hurtbox(
            self.create_hurtbox(crate::collision::CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );

        if input.move_dir() == Vector2::new(self.dir(), 0.0) {
            self.velocity = Vector2::RIGHT * RUN_SPEED * self.dir();
            self.grounded_attack_options(input).unwrap_or_else(|s| s)
        } else {
            self.grounded_actionable_state(input)
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        let mut path: String = "sol/run/run".into();
        path.push(
            char::from_u32(('1' as u8 as usize + (self.frame / FRAMES_PER_RUN_ANIM_FRAME)) as u32)
                .unwrap(),
        );

        Some((path.into(), BASE_SPRITE_OFFSET))
    }
}
impl SolDamageableState for RunState {}

struct RunStartState<const FRAMES: usize = MIN_RUN_FRAMES_BEFORE_CANCEL>;
impl RunStartState {
    const fn dash_cancel() -> RunStartState<10> {
        RunStartState
    }
    const fn new() -> Self {
        RunStartState
    }
}
impl<const FRAMES: usize> Player for Sol<RunStartState<FRAMES>>
where
    Sol<RunStartState<FRAMES>>: Damageable,
{
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        // NOTE: stops instantly cancelling dash into something, adds a little commitment and stops
        // dash cancel cancels
        self.frame += 1;

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::new(STANDING_HURTBOX),
                owner: self.player_id,
            },
            self.position,
        );

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
        path.push(
            char::from_u32(('1' as u8 as usize + (self.frame / FRAMES_PER_RUN_ANIM_FRAME)) as u32)
                .unwrap(),
        );

        Some((path.into(), BASE_SPRITE_OFFSET))
    }
}
impl<const FRAMES: usize> SolDamageableState for RunStartState<FRAMES> {}

const BACKDASH_VELOCITY: f32 = 70.0;
const BACKDASH_FRAMES: usize = 6;
const BACKDASH_VULNERABLE: usize = 9;

struct Backdash;
impl Player for Sol<Backdash> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.velocity = Vector2::new(-self.dir() * BACKDASH_VELOCITY, 0.0);
        self.frame += 1;
        if self.frame > BACKDASH_FRAMES {
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
impl Player for Sol<BackdashVulnerable> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.frame += 1;

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::new(STANDING_HURTBOX),
                owner: self.player_id,
            },
            self.position,
        );

        if self.frame > BACKDASH_VULNERABLE {
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
impl Player for Sol<Stand> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.velocity = self.velocity.move_towards(Vector2::ZERO, DECEL_RATE);

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::new(STANDING_HURTBOX),
                owner: self.player_id,
            },
            self.position,
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
impl<const BLOCKING: bool> Player for Sol<Crouch<BLOCKING>>
where
    Sol<Crouch<BLOCKING>>: Damageable,
{
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const DECEL: f32 = 8.0;
        self.velocity = self.velocity.move_towards(Vector2::ZERO, DECEL);

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::new(CROUCHING_HURTBOX),
                owner: self.player_id,
            },
            self.position,
        );

        self.grounded_actionable_state(input)
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(if BLOCKING {
            // TODO: change to blocking when done
            ("sol/crouch_idle".into(), BASE_SPRITE_OFFSET)
        } else {
            ("sol/crouch_idle".into(), BASE_SPRITE_OFFSET)
        })
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
impl Player for Sol<JumpSquat> {
    fn update(mut self: Box<Self>, world: &mut World, _input: &InputHandler) -> Box<dyn Player> {
        self.frame += 1;

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::new(STANDING_HURTBOX),
                owner: self.player_id,
            },
            self.position,
        );

        if self.frame > JUMPSQUAT_FRAMES {
            if self.velocity.x * self.state.direction < 1.0 {
                self.velocity.x = WALK_SPEED * self.state.direction;
            }
            self.velocity.y = Vector2::UP.y * JUMP_FORCE;
            if self.state.direction == -self.dir() {
                Box::new(self.transition(Air::<true>, true))
            } else {
                Box::new(self.transition(Air::<false>, true))
            }
        } else {
            self
        }
    }
    fn throwable(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/fall/fall1".into(), BASE_SPRITE_OFFSET))
    }
}

struct Air<const BLOCKING: bool>;
impl<const B: bool> Player for Sol<Air<B>>
where
    Sol<Air<B>>: Damageable,
{
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.frame += 1;
        self.gravity();
        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::new(STANDING_HURTBOX),
                owner: self.player_id,
            },
            self.position,
        );
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
impl Player for Sol<Airdash> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.velocity = Vector2::new(AIRDASH_SPEED * self.dir(), 0.0);
        self.frame += 1;
        if self.frame > AIRDASH_LENGTH {
            Box::new(self.transition(Air::<false>, true))
        } else {
            if self.frame > AIRDASH_ACTIONABLE_FRAME {
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
    wall_pushback_mult: f32,
}
impl Player for Sol<BasicHitstun> {
    fn update(mut self: Box<Self>, world: &mut World, _input: &InputHandler) -> Box<dyn Player> {
        self.frame += 1;

        if World::position_in_wall(self.position).is_some() {
            let p = self.state.wall_pushback_mult;
            world.player_hit_wall(self.as_mut(), p);
        }

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::new(STANDING_HURTBOX),
                owner: self.player_id,
            },
            self.position,
        );
        if self.frame > self.state.length {
            Box::new(self.transition(Stand, true))
        } else {
            self
        }
    }
    fn in_hitstun(&self) -> bool {
        true
    }
}
impl SolDamageableState for BasicHitstun {}

#[derive(Debug)]
struct Tumble {
    gravity: f32,
    knockdown: KnockdownType,
    ground_bounce: Option<BounceInfo>,
    wall_bounce: Option<BounceInfo>,
    wall_pushback_mult: f32,
}
impl Player for Sol<Tumble> {
    fn update(mut self: Box<Self>, world: &mut World, _: &InputHandler) -> Box<dyn Player> {
        if !self.grounded {
            self.velocity += Vector2::DOWN * self.state.gravity;
        }
        self.frame += 1;

        // if wall bounce
        if let (
            Some(dir),
            Some(BounceInfo {
                velocity,
                gravity,
                scaling,
                ..
            }),
        ) = (
            World::position_should_bounce_off_wall(self.position),
            self.state.wall_bounce.clone(),
        ) {
            println!("wall bounce");
            let p = self.state.wall_pushback_mult;
            let vel = self.velocity;
            world.player_hit_wall(self.as_mut(), p);

            self.velocity = Vector2::new(
                -vel.x * scaling.0 + velocity.x * dir,
                vel.y * scaling.1 + velocity.y,
            );
            self.state.gravity = gravity;
            self.state.wall_bounce = None;
        } else if World::position_in_wall(self.position).is_some() {
            // else if in wall
            let p = self.state.wall_pushback_mult;
            world.player_hit_wall(self.as_mut(), p);

            self.state.wall_bounce = None;
        }

        if self.grounded {
            if let Some(BounceInfo {
                velocity,
                gravity,
                scaling,
                use_x_vel,
                ..
            }) = self.state.ground_bounce
            {
                println!("ground bounce");
                self.velocity = Vector2::new(
                    self.velocity.x * scaling.0
                        + velocity.x
                            * if use_x_vel {
                                1.0
                            } else {
                                self.velocity.x.signum()
                            },
                    self.velocity.y * scaling.1 + velocity.y,
                );
                self.state.gravity = gravity;

                if World::position_in_wall(self.position).is_some() {
                    self.velocity.x = 0.0;
                }

                self.state.ground_bounce = None;

                world.spawn_hurtbox(
                    crate::collision::Hurtbox {
                        shape: crate::collision::CollisionShape::new(STANDING_HURTBOX),
                        owner: self.player_id,
                    },
                    self.position,
                );
            } else {
                return match self.state.knockdown {
                    KnockdownType::Hard => Box::new(self.transition(HardKnockdown, true)),
                    KnockdownType::Soft => Box::new(self.transition(SoftKnockdown, true)),
                };
            }
        } else {
            world.spawn_hurtbox(
                crate::collision::Hurtbox {
                    shape: crate::collision::CollisionShape::new(STANDING_HURTBOX),
                    owner: self.player_id,
                },
                self.position,
            );
        }

        self
    }

    fn should_wall_bounce(&self) -> bool {
        true
    }
    fn in_hitstun(&self) -> bool {
        true
    }
}
impl SolDamageableState for Tumble {}

struct FloatingCrumple {
    gravity: f32,
    landing_frames: usize,
    wall_pushback_mult: f32,
}
impl Player for Sol<FloatingCrumple> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        if !self.grounded {
            self.velocity += Vector2::DOWN * self.state.gravity;
        }

        if World::position_in_wall(self.position).is_some() {
            let p = self.state.wall_pushback_mult;
            world.player_hit_wall(self.as_mut(), p);
        }

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::new(STANDING_HURTBOX),
                owner: self.player_id,
            },
            self.position,
        );

        if self.grounded {
            self.frame += 1;
            if self.frame >= self.state.landing_frames {
                return self.grounded_actionable_state(input);
            }
        }
        self
    }
    fn in_hitstun(&self) -> bool {
        true
    }
}
impl SolDamageableState for FloatingCrumple {}

const BLOCKSTUN_DRAG: f32 = 2.0;

#[derive(Debug)]
struct BlockStun<const CROUCHING: bool> {
    length: usize,
}
impl<const CROUCHING: bool> Player for Sol<BlockStun<CROUCHING>> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.frame += 1;

        if World::position_in_wall(self.position).is_some() {
            world.player_hit_wall(self.as_mut(), 1.0);
        }

        self.velocity = self.velocity.move_towards(Vector2::ZERO, BLOCKSTUN_DRAG);

        if CROUCHING {
            world.spawn_hurtbox(
                crate::collision::Hurtbox {
                    shape: crate::collision::CollisionShape::new(CROUCHING_HURTBOX),
                    owner: self.player_id,
                },
                self.position,
            );
        } else {
            world.spawn_hurtbox(
                crate::collision::Hurtbox {
                    shape: crate::collision::CollisionShape::new(STANDING_HURTBOX),
                    owner: self.player_id,
                },
                self.position,
            );
        }

        if self.frame >= self.state.length {
            self.grounded_actionable_state(input)
        } else {
            let length = self.state.length;
            if input.input_dir().is_down() {
                Box::new(self.transition(BlockStun::<true> { length }, false))
            } else {
                Box::new(self.transition(BlockStun::<false> { length }, false))
            }
        }
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        // TODO: replace with sprites when theyre done
        Some(if CROUCHING {
            ("sol/crouch_block".into(), BASE_SPRITE_OFFSET)
        } else {
            ("sol/standing_block".into(), BASE_SPRITE_OFFSET)
        })
    }
}
#[derive(Debug)]
struct AirBlockStun {
    length: usize,
}
impl Player for Sol<AirBlockStun> {
    fn update(mut self: Box<Self>, world: &mut World, _input: &InputHandler) -> Box<dyn Player> {
        self.gravity();

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::new(STANDING_HURTBOX),
                owner: self.player_id,
            },
            self.position,
        );

        if self.grounded {
            let length = self.state.length;
            Box::new(self.transition(BlockStun::<false> { length }, true))
        } else {
            self
        }
    }
    fn can_cancel(&self) -> bool {
        false
    }
}

const HARD_KNOCKDOWN_FRAMES: usize = 50;
const SOFT_KNOCKDOWN_FRAMES: usize = 25;

#[derive(Debug)]
struct SoftKnockdown;
impl Player for Sol<SoftKnockdown> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.frame += 1;
        if self.frame >= SOFT_KNOCKDOWN_FRAMES {
            self.walk_block_state(input)
        } else {
            self
        }
    }
}
impl Damageable for Sol<SoftKnockdown> {
    fn hit(self: Box<Self>, _: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        (self, HitConnectionStatus::Invuln)
    }
}

#[derive(Debug)]
struct HardKnockdown;
impl Player for Sol<HardKnockdown> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.frame += 1;
        if self.frame >= HARD_KNOCKDOWN_FRAMES {
            self.walk_block_state(input)
        } else {
            self
        }
    }
}
impl Damageable for Sol<HardKnockdown> {
    fn hit(self: Box<Self>, _: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        (self, HitConnectionStatus::Invuln)
    }
}

struct GroundThrow<const FACING_RIGHT: bool> {
    success: std::rc::Rc<std::cell::Cell<bool>>,
}
impl<const FORWARD: bool> Player for Sol<GroundThrow<FORWARD>> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.velocity = Vector2::ZERO;

        world.spawn_hurtbox(
            self.create_hurtbox(crate::collision::CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );

        self.frame += 1;

        if self.frame == 4 {
            world.spawn_throwbox(
                crate::collision::ThrowBox {
                    shape: crate::collision::CollisionShape::new(BoundingBox::with_size(
                        Vector2::new(10.0, 20.0),
                    )),
                    owner: self.player_id,
                    throw_success: {
                        let success = self.state.success.clone();
                        let success_state = Box::new(Sol {
                            player_id: self.player_id,
                            position: self.position,
                            velocity: Vector2::ZERO,
                            collider: self.collider.clone(),
                            direction: self.direction,
                            has_hit: false,
                            grounded: true,
                            has_air_action: true,
                            distance_from_other_player: self.distance_from_other_player,
                            frame: 0,
                            state: GroundThrowSuccess::<FORWARD>,
                        });
                        Box::new(move || {
                            success.set(true);
                            println!("successful throw");
                            success_state
                        })
                    },
                },
                self.position + Vector2::new(5.0 * self.dir(), 6.0),
            );
        }

        if self.frame < 35 {
            self
        } else {
            self.grounded_actionable_state(input)
        }
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl<const FORWARD: bool> SolDamageableState for GroundThrow<FORWARD> {}

const GROUND_THROW_DAMAGE: u32 = 25;

struct GroundThrowSuccess<const FORWARD: bool>;
impl<const FORWARD: bool> Player for Sol<GroundThrowSuccess<FORWARD>> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        if self.frame == 0 {
            self.direction = if FORWARD {
                self.direction
            } else {
                !self.direction
            };
        }

        if self.frame < 18 {
            world.set_entity_velocity(self.player_id.other_player(), Vector2::ZERO);
            world.set_entity_position(
                self.player_id.other_player(),
                self.position + Vector2::new(10.0 * self.dir(), 7.0),
            );
            if let Some(overlap) =
                World::position_in_wall(world.get_entity_position(self.player_id.other_player()))
            {
                self.position.x += overlap;

                world.set_entity_position(
                    self.player_id.other_player(),
                    self.position + Vector2::new(10.0 * self.dir(), 7.0),
                );
            }
        }
        self.velocity = Vector2::ZERO;

        self.frame += 1;

        match self.frame {
            0..28 => self,
            28 => {
                world.spawn_hitbox(
                    self.create_hitbox(
                        CollisionShape::new(BoundingBox::with_size(Vector2::new(20.0, 20.0))),
                        AttackData {
                            attack: HitData::grounded(
                                GROUND_THROW_DAMAGE,
                                HitEffect::launcher(
                                    Vector2::new(50.0 * self.dir(), 50.0),
                                    KnockdownType::Hard,
                                )
                                .build(),
                                0,
                                Proration::percent(100),
                                90,
                            )
                            .air_from_grounded(|g| g)
                            .counterhit_ground_from_ground_default()
                            .counterhit_air_from_air_default()
                            .build(),
                            hitbox_id: 1,
                            attack_id: "sol ground throw".into(),
                            priority: 10,
                            hit_level: crate::collision::HitLevel::Light,
                        },
                    ),
                    self.position + Vector2::new(8.0 * self.dir(), 5.0),
                );
                self
            }
            18..50 => self,
            _ => self.grounded_actionable_state(input),
        }
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/throw".into(), BASE_SPRITE_OFFSET))
    }
}
impl<const FACING_RIGHT: bool> SolDamageableState for GroundThrowSuccess<FACING_RIGHT> {}

#[derive(Debug)]
struct ThrownState;
impl Player for Sol<ThrownState> {
    fn update(self: Box<Self>, world: &mut World, _: &InputHandler) -> Box<dyn Player> {
        world.spawn_hurtbox(
            self.create_hurtbox(crate::collision::CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );
        self
    }
    fn actionable(&self) -> bool {
        false
    }
    fn in_hitstun(&self) -> bool {
        true
    }
    fn can_cancel(&self) -> bool {
        false
    }
}
impl SolDamageableState for ThrownState {}
