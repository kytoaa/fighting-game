use super::*;

const GROUND_VIPER_STARTUP: usize = 4;
const MIN_GROUND_VIPER_SLIDE_FRAMES: usize = 10;
const MAX_GROUND_VIPER_SLIDE_FRAMES: usize = 22;
const GROUND_VIPER_ACTIVE: usize = 2;
const GROUND_VIPER_RECOVERY: usize = 30;
const GROUND_VIPER_DAMAGE: u32 = 20;
const GROUND_VIPER_CHARGED_DAMAGE: u32 = 25;

#[derive(Default, Clone)]
pub struct GroundViper {
    frames_sliding: usize,
    released: bool,
}
impl Player for Sol<GroundViper> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const SLIDE_HOLD_FRAME: usize = GROUND_VIPER_STARTUP; // stay on slide frame until release
        const ACTIVE_FRAME: usize = SLIDE_HOLD_FRAME + MIN_GROUND_VIPER_SLIDE_FRAMES;
        const RECOVERY_FRAME: usize = ACTIVE_FRAME + GROUND_VIPER_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + GROUND_VIPER_RECOVERY;
        const SPEED: f32 = 130.0;
        const DECEL: f32 = 20.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        if self.frame != SLIDE_HOLD_FRAME || self.state.released {
            self.frame += 1;
        }

        if input.has_action(&Action::Released(Button::Mid)) {
            self.state.released = true;
        }

        match self.frame {
            0..GROUND_VIPER_STARTUP => {
                world.spawn_hurtbox(
                    self.create_hurtbox(CollisionShape::new(BoundingBox::pos_size(
                        STANDING_HURTBOX.position(),
                        STANDING_HURTBOX.size(),
                    ))),
                    self.position,
                );
                self
            }
            SLIDE_HOLD_FRAME..ACTIVE_FRAME => {
                if self.frame == SLIDE_HOLD_FRAME {
                    if !self.state.released {
                        if self.state.frames_sliding < MAX_GROUND_VIPER_SLIDE_FRAMES {
                            self.state.frames_sliding += 1;
                        } else {
                            self.state.released = true;
                        }
                    }
                }

                self.velocity = Vector2::new(SPEED * self.dir(), 0.0);

                world.spawn_hurtbox(
                    self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(Vector2::new(
                        26.0, 8.0,
                    )))),
                    self.position + Vector2::new(-2.0 * self.dir(), -4.0),
                );

                self
            }
            ACTIVE_FRAME..RECOVERY_FRAME => {
                self.velocity = self.velocity.move_towards(Vector2::ZERO, DECEL);

                world.spawn_hurtbox(
                    self.create_hurtbox(CollisionShape::new(BoundingBox::pos_size(
                        STANDING_HURTBOX.position(),
                        STANDING_HURTBOX.size(),
                    ))),
                    self.position,
                );
                world.spawn_hurtbox(
                    self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(Vector2::new(
                        14.0, 20.0,
                    )))),
                    self.position + Vector2::new(8.0 * self.dir(), 10.0),
                );

                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        GROUND_VIPER_ACTIVE - (self.frame as usize - ACTIVE_FRAME);

                    let attack_data = if self.state.frames_sliding == MAX_GROUND_VIPER_SLIDE_FRAMES
                    {
                        AttackData {
                            attack: HitData::grounded(
                                GROUND_VIPER_CHARGED_DAMAGE,
                                HitEffect::launcher(
                                    Vector2::new(8.0 * self.dir(), 150.0),
                                    KnockdownType::Hard,
                                )
                                .gravity(7.5)
                                .build(),
                                18 + active_frames_extra_hitstun,
                                Proration::percent(85),
                                HitData::DEFAULT_LEVEL_3_SCALING,
                            )
                            .with_air(
                                HitEffect::launcher(
                                    Vector2::new(8.0 * self.dir(), 160.0),
                                    KnockdownType::Hard,
                                )
                                .gravity(7.5)
                                .build(),
                            )
                            .with_counterhit_ground(
                                HitEffect::launcher(
                                    Vector2::new(8.0 * self.dir(), 130.0),
                                    KnockdownType::Hard,
                                )
                                .gravity(4.5)
                                .build(),
                            )
                            .with_counterhit_air(
                                HitEffect::launcher(
                                    Vector2::new(8.0 * self.dir(), 150.0),
                                    KnockdownType::Hard,
                                )
                                .gravity(4.5)
                                .build(),
                            )
                            .build(),
                            priority: 10,
                            hitbox_id: 1,
                            hit_level: HitLevel::Heavy,
                            attack_id: "ground viper charged".into(),
                        }
                    } else {
                        AttackData {
                            attack: HitData::grounded(
                                GROUND_VIPER_DAMAGE,
                                HitEffect::launcher(
                                    Vector2::new(8.0 * self.dir(), 90.0),
                                    KnockdownType::Soft,
                                )
                                .gravity(4.5)
                                .build(),
                                15 + active_frames_extra_hitstun,
                                Proration::percent(80),
                                HitData::DEFAULT_LEVEL_2_SCALING,
                            )
                            .with_air(
                                HitEffect::launcher(
                                    Vector2::new(8.0 * self.dir(), 100.0),
                                    KnockdownType::Soft,
                                )
                                .gravity(5.3)
                                .build(),
                            )
                            .with_counterhit_ground(
                                HitEffect::launcher(
                                    Vector2::new(8.0 * self.dir(), 125.0),
                                    KnockdownType::Soft,
                                )
                                .gravity(4.5)
                                .build(),
                            )
                            .with_counterhit_air(
                                HitEffect::launcher(
                                    Vector2::new(8.0 * self.dir(), 145.0),
                                    KnockdownType::Soft,
                                )
                                .gravity(4.5)
                                .build(),
                            )
                            .build(),
                            priority: 10,
                            hitbox_id: 1,
                            hit_level: HitLevel::Medium,
                            attack_id: "ground viper".into(),
                        }
                    };
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(10.0, 24.0))),
                            attack_data,
                        ),
                        self.position + Vector2::new(8.0 * self.dir(), 14.0),
                    );
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                self.velocity = self.velocity.move_towards(Vector2::ZERO, DECEL);

                world.spawn_hurtbox(
                    self.create_hurtbox(CollisionShape::new(BoundingBox::pos_size(
                        STANDING_HURTBOX.position(),
                        STANDING_HURTBOX.size(),
                    ))),
                    self.position,
                );

                self
            }
            _ => {
                world.spawn_hurtbox(
                    self.create_hurtbox(CollisionShape::new(BoundingBox::pos_size(
                        STANDING_HURTBOX.position(),
                        STANDING_HURTBOX.size(),
                    ))),
                    self.position,
                );
                self.grounded_actionable_state(input)
            }
        }
    }

    fn moveable(&self) -> bool {
        false
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const SLIDE_HOLD_FRAME: usize = GROUND_VIPER_STARTUP; // stay on slide frame until release
        const ACTIVE_FRAME: usize = SLIDE_HOLD_FRAME + MIN_GROUND_VIPER_SLIDE_FRAMES;
        Some(match self.frame {
            0..GROUND_VIPER_STARTUP => (
                "sol/specials/ground_viper/ground_viper1".into(),
                BASE_SPRITE_OFFSET,
            ),
            SLIDE_HOLD_FRAME..ACTIVE_FRAME => (
                "sol/specials/ground_viper/ground_viper2".into(),
                BASE_SPRITE_OFFSET + Vector2::new(-4.0 * self.dir(), -4.0),
            ),
            ACTIVE_FRAME.. => (
                "sol/specials/ground_viper/ground_viper3".into(),
                BASE_SPRITE_OFFSET + Vector2::new(-3.0 * self.dir(), 8.0),
            ),
        })
    }
}
impl SolDamageableState for GroundViper {}
