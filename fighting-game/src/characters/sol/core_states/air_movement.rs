use super::super::*;

const JUMPSQUAT_FRAMES: usize = 4;
const JUMP_FORCE: f32 = 175.0;

#[derive(Clone)]
pub struct JumpSquat {
    pub direction: f32,
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

#[derive(Clone)]
pub struct Air<const BLOCKING: bool>;
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

#[derive(Clone)]
pub struct Airdash;
impl Player for Sol<Airdash> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.velocity = Vector2::new(AIRDASH_SPEED * self.dir(), 0.0);
        self.frame += 1;

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::new(STANDING_HURTBOX),
                owner: self.player_id,
            },
            self.position,
        );

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
    fn actionable(&self) -> bool {
        false
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
