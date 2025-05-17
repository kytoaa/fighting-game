use super::*;

const FAFNIR_STARTUP: usize = 20;
const FAFNIR_ACTIVE: usize = 4;
const FAFNIR_RECOVERY: usize = 18;
const FAFNIR_DAMAGE: u32 = 35;

pub struct Fafnir;
impl Entity for Sol<Fafnir> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const STOP_FRAME: usize = FAFNIR_STARTUP - 7;
        const RECOVERY_FRAME: usize = FAFNIR_STARTUP + FAFNIR_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + FAFNIR_RECOVERY;

        const MAX_X_VEL: f32 = 150.0;
        const DECEL: f32 = 25.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;

        world.spawn_hurtbox(
            self.create_hurtbox(CollisionShape::new(BoundingBox::pos_size(
                STANDING_HURTBOX.position(),
                STANDING_HURTBOX.size().x(22.0),
            ))),
            self.position,
        );

        match self.frame as usize {
            0..4 => self,
            4..STOP_FRAME => {
                self.velocity = Vector2::new(MAX_X_VEL * self.dir(), 0.0);
                self
            }
            STOP_FRAME..FAFNIR_STARTUP => {
                self.velocity = self.velocity.move_towards(Vector2::ZERO, DECEL);
                self
            }
            FAFNIR_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        FAFNIR_ACTIVE - (self.frame as usize - FAFNIR_STARTUP);
                    let attack_data = HitData::grounded(
                        FAFNIR_DAMAGE,
                        HitEffect::launcher(
                            Vector2::new(100.0 * self.dir(), 90.0),
                            KnockdownType::Soft,
                        )
                        .wall_bounce_velocity(Vector2::new(60.0, 80.0))
                        .gravity(7.5)
                        .build(),
                        18 + active_frames_extra_hitstun,
                        Proration::percent(80),
                        HitData::DEFAULT_LEVEL_4_SCALING,
                    )
                    .with_air(
                        HitEffect::launcher(
                            Vector2::new(100.0 * self.dir(), 120.0),
                            KnockdownType::Hard,
                        )
                        .wall_bounce_velocity(Vector2::new(60.0, 70.0))
                        .wall_bounce_gravity(5.0)
                        .build(),
                    )
                    .wall_pushback_mult(0.6)
                    .counterhit_from_air(|a| a)
                    .build();

                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(18.0, 10.0))),
                            AttackData {
                                attack: attack_data.clone(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Heavy,
                            },
                        ),
                        self.position + Vector2::new(14.0 * self.dir(), 16.0),
                    );
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(8.0, 16.0))),
                            AttackData {
                                attack: attack_data,
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Heavy,
                            },
                        ),
                        self.position + Vector2::new(6.0 * self.dir(), 6.0),
                    );
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => self,
            _ => self.grounded_actionable_state(input),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn moveable(&self) -> bool {
        if self.frame < FAFNIR_STARTUP {
            false
        } else {
            true
        }
    }
}
impl SolDamageableState for Fafnir {}
