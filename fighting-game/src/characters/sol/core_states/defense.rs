use super::super::*;

const BLOCKSTUN_DRAG: f32 = 2.0;

pub struct BlockStun<const CROUCHING: bool> {
    pub length: usize,
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
    fn throwable(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(if CROUCHING {
            ("sol/crouch_block".into(), BASE_SPRITE_OFFSET)
        } else {
            ("sol/standing_block".into(), BASE_SPRITE_OFFSET)
        })
    }
}
pub struct AirBlockStun {
    pub length: usize,
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
    fn throwable(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/air_block".into(), BASE_SPRITE_OFFSET))
    }
}

const BACKDASH_VELOCITY: f32 = 70.0;
const BACKDASH_FRAMES: usize = 6;
const BACKDASH_VULNERABLE: usize = 9;

pub struct Backdash;
impl Player for Sol<Backdash> {
    fn update(mut self: Box<Self>, _world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const BACKTHROW_FRAMES: usize = 2;
        if self.is_grounded() && self.frame < BACKTHROW_FRAMES {
            if input.get_state(Button::Utility) == ButtonState::Down
                && input.get_state(Button::Light) == ButtonState::Down
                && input.has_action(&Action::Pressed(
                    Button::Light,
                    Some(InputDir::Dir4.dir(self.direction)),
                ))
            {
                return Box::new(self.transition(
                    GroundThrow::<false> {
                        success: std::cell::Cell::new(true).into(),
                    },
                    true,
                ));
            }
        }

        self.velocity = Vector2::new(-self.dir() * BACKDASH_VELOCITY, 0.0);
        self.frame += 1;
        if self.frame > BACKDASH_FRAMES {
            Box::new(self.transition(BackdashVulnerable, true))
        } else {
            self
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/backdash/backdash1".into(), BASE_SPRITE_OFFSET))
    }
}
pub struct BackdashVulnerable;
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
    fn actionable(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/backdash/backdash2".into(), BASE_SPRITE_OFFSET))
    }
}
impl SolDamageableState for BackdashVulnerable {}
