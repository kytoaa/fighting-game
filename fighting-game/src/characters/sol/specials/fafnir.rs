use super::*;

const FAFNIR_STARTUP: usize = 20;
const FAFNIR_ACTIVE: usize = 4;
const FAFNIR_RECOVERY: usize = 18;
const FAFNIR_DAMAGE: u16 = 35;

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
            crate::collision::Hurtbox {
                shape: CollisionShape::Box(BoundingBox::pos_size(
                    STANDING_HURTBOX.position(),
                    STANDING_HURTBOX.size().x(22.0),
                )),
                owner: self.player,
            },
            self.position,
            1,
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
                    let attack_data = AttackData {
                        grounded: HitInfo {
                            damage: FAFNIR_DAMAGE,
                            hitstun: 60 + active_frames_extra_hitstun,
                            blockstun: 15,
                            hit_effect: HitEffect::Launcher(
                                Vector2::new(100.0 * self.dir(), 90.0),
                                KnockdownType::Soft,
                            ),
                            block_push: 60.0 * self.dir(),
                        },
                        air: HitInfo {
                            damage: FAFNIR_DAMAGE,
                            hitstun: 60 + active_frames_extra_hitstun,
                            blockstun: 15,
                            hit_effect: HitEffect::Launcher(
                                Vector2::new(100.0 * self.dir(), 120.0),
                                KnockdownType::Hard,
                            ),
                            block_push: 80.0 * self.dir(),
                        },
                        counterhit: HitInfo {
                            damage: FAFNIR_DAMAGE,
                            hitstun: 60 + active_frames_extra_hitstun,
                            blockstun: 15,
                            hit_effect: HitEffect::Launcher(
                                Vector2::new(100.0 * self.dir(), 120.0),
                                KnockdownType::Hard,
                            ),
                            block_push: 80.0 * self.dir(),
                        },
                        priority: 10,
                        attack_type: crate::collision::AttackType::Mid,
                        hitbox_id: 1,
                        hit_type: crate::collision::HitLevel::Heavy,
                    };

                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                18.0, 10.0,
                            ))),
                            info: attack_data.clone(),
                            owner: self.player,
                        },
                        self.position + Vector2::new(14.0 * self.dir(), 16.0),
                        1,
                    );
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                8.0, 16.0,
                            ))),
                            info: attack_data.clone(),
                            owner: self.player,
                        },
                        self.position + Vector2::new(6.0 * self.dir(), 6.0),
                        1,
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
}
impl SolDamageableState for Fafnir {}
