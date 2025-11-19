use super::super::*;

pub const WALK_SPEED: f32 = 30.0;
const RUN_SPEED: f32 = 90.0;
const DECEL_RATE: f32 = 12.0;

#[derive(Clone)]
pub struct Stand;
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
        if self.velocity.x.abs() > 10.0 {
            Some(("sol/run/run_stop".into(), BASE_SPRITE_OFFSET))
        } else {
            Some(("sol/idle".into(), BASE_SPRITE_OFFSET))
        }
    }
}
impl SolDamageableState for Stand {}

#[derive(Clone)]
pub struct Crouch<const BLOCKING: bool>;
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

pub const WALK_ANIM_LENGTH: usize = 4;
pub const FRAMES_PER_WALK_ANIM_FRAME: usize = 10;

#[derive(Clone)]
pub struct WalkState<const BLOCKING: bool>;

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
#[derive(Clone)]
pub struct RunState;
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

        if (input.move_dir().x == self.dir()
            || input.get_state(Button::Utility) == ButtonState::Down)
            && input.move_dir().y <= 0.0
        {
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

#[derive(Clone)]
pub struct RunStartState<const FRAMES: usize = MIN_RUN_FRAMES_BEFORE_CANCEL>;
impl RunStartState {
    pub const fn dash_cancel() -> RunStartState<10> {
        RunStartState
    }
    pub const fn new() -> Self {
        RunStartState
    }
}
impl<const FRAMES: usize> Player for Sol<RunStartState<FRAMES>>
where
    Sol<RunStartState<FRAMES>>: Damageable,
{
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const THROW_FRAMES: usize = 2;
        if self.frame < THROW_FRAMES {
            if input.get_state(Button::Utility) == ButtonState::Down
                && input.get_state(Button::Light) == ButtonState::Down
                && input.has_action(&Action::Pressed(
                    Button::Light,
                    Some(InputDir::Dir4.dir(self.direction)),
                ))
            {
                return Box::new(self.transition(GroundThrow::<false>, true));
            }
        }

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
