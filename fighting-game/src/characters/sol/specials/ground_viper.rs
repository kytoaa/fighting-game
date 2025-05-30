use super::*;

const GROUND_VIPER_STARTUP: usize = 4;
const MIN_GROUND_VIPER_SLIDE_FRAMES: usize = 10;
const MAX_GROUND_VIPER_SLIDE_FRAMES: usize = 32;
const GROUND_VIPER_ACTIVE: usize = 2;
const GROUND_VIPER_RECOVERY: usize = 30;
const GROUND_VIPER_DAMAGE: u32 = 20;

#[derive(Default)]
pub struct GroundViper {
    release_frame: usize,
}
impl Player for Sol<GroundViper> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const MIN_SLIDE_END: usize = GROUND_VIPER_STARTUP + MIN_GROUND_VIPER_SLIDE_FRAMES;
        const RECOVERY_FRAME: usize = MIN_SLIDE_END + GROUND_VIPER_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + GROUND_VIPER_RECOVERY;
        const SPEED: f32 = 120.0;
        const DECEL: f32 = 20.0;

        if self.frame == 0 {
            self.has_hit = false;
        }
        /*if input.has_action(&Action::Released(Button::Mid)) {
            self.state.release_frame = self.frame;
        }*/
        self.frame += 1;

        if self.frame >= MIN_SLIDE_END - 2 {
            self.velocity = self.velocity.move_towards(Vector2::ZERO, DECEL);
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
            GROUND_VIPER_STARTUP..MIN_SLIDE_END => {
                self.velocity = Vector2::new(SPEED * self.dir(), 0.0);

                world.spawn_hurtbox(
                    self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(Vector2::new(
                        26.0, 8.0,
                    )))),
                    self.position + Vector2::new(-2.0 * self.dir(), -4.0),
                );
                self
            }
            MIN_SLIDE_END..RECOVERY_FRAME => {
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
                        GROUND_VIPER_ACTIVE - (self.frame as usize - MIN_SLIDE_END);
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(10.0, 24.0))),
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
                                    .gravity(6.5)
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
                            },
                        ),
                        self.position + Vector2::new(8.0 * self.dir(), 14.0),
                    );
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
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
}
impl SolDamageableState for GroundViper {}
