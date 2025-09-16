use super::*;

const FAFNIR_STARTUP: usize = 20;
const FAFNIR_ACTIVE: usize = 4;
const FAFNIR_RECOVERY: usize = 18;
const FAFNIR_DAMAGE: u32 = 35;

pub struct Fafnir;
impl Player for Sol<Fafnir> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const START_FRAME: usize = 4;
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
            0..START_FRAME => self,
            START_FRAME..STOP_FRAME => {
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
                            Vector2::new(100.0 * self.dir(), 60.0),
                            KnockdownType::Soft,
                        )
                        .gravity(7.0)
                        .build(),
                        18 + active_frames_extra_hitstun,
                        Proration::percent(80),
                        HitData::DEFAULT_LEVEL_4_SCALING,
                    )
                    .with_air(
                        HitEffect::launcher(
                            Vector2::new(100.0 * self.dir(), 50.0),
                            KnockdownType::Soft,
                        )
                        .gravity(7.0)
                        .build(),
                    )
                    .add_extension(HitDataExtension::CleanHit(
                        BoundingBox::pos_size(
                            Vector2::new(10.0 * self.dir(), 5.0),
                            Vector2::new(5.0, 5.0),
                        )
                        .transformed(self.position),
                        HitEffect::launcher(
                            Vector2::new(100.0 * self.dir(), 50.0),
                            KnockdownType::Hard,
                        )
                        .gravity(7.0)
                        .wall_bounce(BounceInfo::new(Vector2::new(60.0, 70.0)).gravity(5.0))
                        .build(),
                    ))
                    .wall_pushback_mult(0.6)
                    .counterhit_ground_from_ground_default()
                    .counterhit_air_from_air_default()
                    .build();

                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(18.0, 10.0))),
                            AttackData {
                                attack: attack_data.clone(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Heavy,
                                attack_id: "fafnir".into(),
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
                                attack_id: "fafnir".into(),
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
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const START_FRAME: usize = 4;
        const RECOVERY_FRAME: usize = FAFNIR_STARTUP + FAFNIR_ACTIVE + FAFNIR_RECOVERY / 2;
        const FLAME_END_FRAME: usize = RECOVERY_FRAME - 4;

        Some(match self.frame {
            0..START_FRAME => ("sol/specials/fafnir/fafnir1".into(), BASE_SPRITE_OFFSET),
            START_FRAME..FAFNIR_STARTUP => (
                "sol/specials/fafnir/fafnir2".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * 8.0 * self.dir(),
            ),
            FAFNIR_STARTUP..RECOVERY_FRAME => (
                "sol/specials/fafnir/fafnir3".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * 8.0 * self.dir(),
            ),
            RECOVERY_FRAME.. => (
                "sol/specials/fafnir/fafnir5".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * 8.0 * self.dir(),
            ),
        })
    }
}
impl SolDamageableState for Fafnir {}
