use super::super::super::*;

#[derive(Clone)]
pub struct BasicHitstun {
    pub length: usize,
    pub wall_pushback_mult: f32,
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
    fn actionable(&self) -> bool {
        false
    }
    fn in_hitstun(&self) -> bool {
        true
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/hitstun".into(), BASE_SPRITE_OFFSET))
    }
}
impl SolDamageableState for BasicHitstun {}

#[derive(Clone)]
pub struct Tumble {
    pub gravity: f32,
    pub knockdown: KnockdownType,
    pub ground_bounce: Option<BounceInfo>,
    pub wall_bounce: Option<BounceInfo>,
    pub wall_pushback_mult: f32,
}
impl Player for Sol<Tumble> {
    fn update(mut self: Box<Self>, world: &mut World, _: &InputHandler) -> Box<dyn Player> {
        if !self.grounded {
            self.velocity += Vector2::DOWN * self.state.gravity;
        }
        self.frame += 1;

        // if wall bounce
        let bounced_off_wall = if let (
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

            // reset frame for animation
            self.frame = 0;

            true
        } else if World::position_in_wall(self.position).is_some() {
            // else if in wall
            let p = self.state.wall_pushback_mult;
            world.player_hit_wall(self.as_mut(), p);

            self.state.wall_bounce = None;

            false
        } else {
            false
        };

        if self.grounded && !bounced_off_wall {
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

                // reset frame for animation
                self.frame = 0;

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
    fn actionable(&self) -> bool {
        false
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(match self.frame {
            0..10 => ("sol/fall/tumble1".into(), BASE_SPRITE_OFFSET),
            _ => ("sol/fall/tumble2".into(), BASE_SPRITE_OFFSET),
        })
    }
}
impl SolDamageableState for Tumble {}

#[derive(Clone)]
pub struct FloatingCrumple {
    pub gravity: f32,
    pub landing_frames: usize,
    pub wall_pushback_mult: f32,
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
    fn actionable(&self) -> bool {
        false
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some((
            "sol/floating_crumple".into(),
            BASE_SPRITE_OFFSET + Vector2::UP * 3.0,
        ))
    }
}
impl SolDamageableState for FloatingCrumple {}

const HARD_KNOCKDOWN_FRAMES: usize = 50;
const SOFT_KNOCKDOWN_FRAMES: usize = 25;

#[derive(Clone)]
pub struct SoftKnockdown;
impl Player for Sol<SoftKnockdown> {
    fn update(mut self: Box<Self>, _world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const DECEL: f32 = 3.0;
        const MIN_SPEED: f32 = 40.0;
        self.velocity = self.velocity.move_towards(
            if World::position_in_wall(self.position).is_some() {
                Vector2::ZERO
            } else {
                Vector2::LEFT * MIN_SPEED * self.velocity.x.signum()
            },
            DECEL,
        );

        self.frame += 1;
        if self.frame >= SOFT_KNOCKDOWN_FRAMES {
            self.walk_block_state(input)
        } else {
            self
        }
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
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(match self.frame {
            0..10 => (
                "sol/hard_knockdown".into(),
                BASE_SPRITE_OFFSET + Vector2::DOWN * 12.0,
            ),
            10..15 => ("sol/normals/2l/2l3".into(), BASE_SPRITE_OFFSET),
            15.. => ("sol/normals/2l/2l1".into(), BASE_SPRITE_OFFSET),
        })
    }
}
impl Damageable for Sol<SoftKnockdown> {
    fn hit(self: Box<Self>, _: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        (self, HitConnectionStatus::Invuln)
    }
}

#[derive(Clone)]
pub struct HardKnockdown;
impl Player for Sol<HardKnockdown> {
    fn update(mut self: Box<Self>, _world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const DECEL: f32 = 7.0;
        self.velocity = self.velocity.move_towards(Vector2::ZERO, DECEL);

        self.frame += 1;
        if self.frame >= HARD_KNOCKDOWN_FRAMES {
            self.walk_block_state(input)
        } else {
            self
        }
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
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const STAND_FRAME: usize = HARD_KNOCKDOWN_FRAMES - 6;
        Some(match self.frame {
            0..STAND_FRAME => (
                "sol/hard_knockdown".into(),
                BASE_SPRITE_OFFSET + Vector2::DOWN * 12.0,
            ),
            STAND_FRAME.. => ("sol/normals/2l/2l3".into(), BASE_SPRITE_OFFSET),
        })
    }
}
impl Damageable for Sol<HardKnockdown> {
    fn hit(self: Box<Self>, _: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        (self, HitConnectionStatus::Invuln)
    }
}
#[derive(Clone)]
pub struct ThrownState;
impl Player for Sol<ThrownState> {
    fn update(mut self: Box<Self>, world: &mut World, _: &InputHandler) -> Box<dyn Player> {
        world.spawn_hurtbox(
            self.create_hurtbox(crate::collision::CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );
        // not grounded so dir wont be set
        self.direction =
            world.get_entity_position(self.player_id.other_player()).x - self.position.x > 0.0;
        self
    }
    fn in_hitstun(&self) -> bool {
        true
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/fall/tumble1".into(), BASE_SPRITE_OFFSET))
    }
}
impl SolDamageableState for ThrownState {}

#[derive(Clone)]
pub struct DeadState;
impl Player for Sol<DeadState> {
    fn update(mut self: Box<Self>, _: &mut World, _: &InputHandler) -> Box<dyn Player> {
        self.gravity();
        if self.frame < 10 {
            self.frame += 1;
        }
        self
    }
    fn actionable(&self) -> bool {
        false
    }
    fn in_hitstun(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        false
    }
    fn moveable(&self) -> bool {
        false
    }
    fn throwable(&self) -> bool {
        false
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn should_wall_bounce(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(match self.frame {
            _ if self.grounded => (
                "sol/hard_knockdown".into(),
                BASE_SPRITE_OFFSET + Vector2::DOWN * 12.0,
            ),
            0..10 => ("sol/fall/tumble1".into(), BASE_SPRITE_OFFSET),
            _ => ("sol/fall/tumble2".into(), BASE_SPRITE_OFFSET),
        })
    }
}
