use super::*;

const BANDIT_REVOLVER_GROUNDED_1_STARTUP: usize = 12;
const BANDIT_REVOLVER_GROUNDED_1_ACTIVE: usize = 6;
const BANDIT_REVOLVER_GROUNDED_1_RECOVERY: usize = 16;
const BANDIT_REVOLVER_GROUNDED_1_LANDING_LAG: usize = 7;
const BANDIT_REVOLVER_GROUNDED_1_DAMAGE: u32 = 11;

pub struct BanditRevolverGrounded;
impl Entity for Sol<BanditRevolverGrounded> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const LAUNCH_FRAME: usize = 6;
        const RECOVERY_FRAME: usize =
            BANDIT_REVOLVER_GROUNDED_1_STARTUP + BANDIT_REVOLVER_GROUNDED_1_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + BANDIT_REVOLVER_GROUNDED_1_RECOVERY;

        const INITIAL_FORCE: Vector2 = Vector2::new(120.0, 100.0);
        const DECEL: f32 = 5.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;

        // NOTE: cancel into hit 2
        {
            const CANCEL_WINDOW_START: usize = BANDIT_REVOLVER_GROUNDED_1_STARTUP + 1;
            const CANCEL_WINDOW_END: usize =
                RECOVERY_FRAME + BANDIT_REVOLVER_GROUNDED_1_RECOVERY / 2;

            if let CANCEL_WINDOW_START..CANCEL_WINDOW_END = self.frame as usize {
                if input.has_action(&Action::Pressed(Button::Mid, None)) {
                    return Box::new(self.transition(BanditRevolverGroundedSecondHit, true));
                }
            }
        }

        match self.frame as usize {
            f @ 0..BANDIT_REVOLVER_GROUNDED_1_STARTUP => {
                if f == LAUNCH_FRAME {
                    self.velocity = Vector2::new(INITIAL_FORCE.x * self.dir(), INITIAL_FORCE.y);
                }
                self.gravity();
                self
            }
            BANDIT_REVOLVER_GROUNDED_1_STARTUP..RECOVERY_FRAME => {
                self.gravity();

                if !self.has_hit {
                    let active_frames_extra_hitstun = BANDIT_REVOLVER_GROUNDED_1_ACTIVE
                        - (self.frame as usize - BANDIT_REVOLVER_GROUNDED_1_STARTUP);
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(14.0, 14.0))),
                            AttackData {
                                attack: HitData::grounded(
                                    BANDIT_REVOLVER_GROUNDED_1_DAMAGE,
                                    HitEffect::pushback(
                                        30.0 * self.dir(),
                                        15 + active_frames_extra_hitstun,
                                    )
                                    .build(),
                                    12 + active_frames_extra_hitstun,
                                    Proration::percent(80),
                                    HitData::DEFAULT_LEVEL_1_SCALING,
                                )
                                .with_air(
                                    HitEffect::launcher(
                                        Vector2::new(50.0 * self.dir(), 60.0),
                                        KnockdownType::Soft,
                                    )
                                    .gravity(6.0)
                                    .build(),
                                )
                                .counterhit_from_air(|a| a)
                                .meter_gain(HitData::DEFAULT_LEVEL_2_METER_GAIN)
                                .build(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Light,
                                attack_id: "bandit revolver".into(),
                            },
                        ),
                        self.position + Vector2::new(10.0 * self.dir(), 6.0),
                    );
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                self.gravity();
                self.velocity.x = self.velocity.y(0.0).move_towards(Vector2::ZERO, DECEL).x;
                self
            }
            _ if !self.is_grounded() => {
                self.gravity();
                self.velocity.x = self.velocity.y(0.0).move_towards(Vector2::ZERO, DECEL).x;
                self
            }
            _ => Box::new(self.transition(
                BanditRevolverGroundedRecovery::<BANDIT_REVOLVER_GROUNDED_1_LANDING_LAG>,
                true,
            )),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn moveable(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(match self.frame {
            0..5 => ("sol/fall/fall1".into(), BASE_SPRITE_OFFSET),
            _ => (
                "sol/specials/bandit_revolver/bandit_revolver1".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * 4.0,
            ),
        })
    }
}
impl SolDamageableState for BanditRevolverGrounded {}

struct BanditRevolverGroundedRecovery<const FRAMES: usize>;
impl<const FRAMES: usize> Entity for Sol<BanditRevolverGroundedRecovery<FRAMES>> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const DECEL: f32 = 5.0;

        self.has_hit = false;
        self.frame += 1;

        world.spawn_hurtbox(
            self.create_hurtbox(CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );

        if (self.frame as usize) < FRAMES {
            self.velocity.x = self.velocity.y(0.0).move_towards(Vector2::ZERO, DECEL).x;
            self
        } else {
            self.grounded_actionable_state(input)
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some((
            "sol/run/run_stop".into(),
            BASE_SPRITE_OFFSET + Vector2::LEFT * 4.0,
        ))
    }
}
impl<const FRAMES: usize> SolDamageableState for BanditRevolverGroundedRecovery<FRAMES> {}

const BANDIT_REVOLVER_GROUNDED_2_STARTUP: usize = 6;
const BANDIT_REVOLVER_GROUNDED_2_ACTIVE: usize = 2;
const BANDIT_REVOLVER_GROUNDED_2_RECOVERY: usize = 8;
const BANDIT_REVOLVER_GROUNDED_2_LANDING_LAG: usize = 15;
const BANDIT_REVOLVER_GROUNDED_2_DAMAGE: u32 = 14;

struct BanditRevolverGroundedSecondHit;
impl Entity for Sol<BanditRevolverGroundedSecondHit> {
    fn update(mut self: Box<Self>, world: &mut World, _: &InputHandler) -> Box<dyn Entity> {
        const RECOVERY_FRAME: usize =
            BANDIT_REVOLVER_GROUNDED_2_STARTUP + BANDIT_REVOLVER_GROUNDED_2_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + BANDIT_REVOLVER_GROUNDED_2_RECOVERY;

        const DECEL: f32 = 5.0;

        if self.frame == 0 {
            self.has_hit = false;
            self.velocity.y = 0.0;
        }

        self.frame += 1;

        match self.frame as usize {
            0..BANDIT_REVOLVER_GROUNDED_2_STARTUP => self,
            BANDIT_REVOLVER_GROUNDED_2_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun = BANDIT_REVOLVER_GROUNDED_2_ACTIVE
                        - (self.frame as usize - BANDIT_REVOLVER_GROUNDED_2_STARTUP);
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(20.0, 14.0))),
                            AttackData {
                                attack: HitData::grounded(
                                    BANDIT_REVOLVER_GROUNDED_2_DAMAGE,
                                    HitEffect::launcher(
                                        Vector2::new(100.0 * self.dir(), 100.0),
                                        KnockdownType::Soft,
                                    )
                                    .build(),
                                    16 + active_frames_extra_hitstun,
                                    Proration::percent(80),
                                    HitData::DEFAULT_LEVEL_1_SCALING,
                                )
                                .air_from_grounded(|g| g)
                                .counterhit_from_grounded(|g| g)
                                .meter_gain(HitData::DEFAULT_LEVEL_2_METER_GAIN)
                                .build(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Light,
                                attack_id: "bandit revolver 2".into(),
                            },
                        ),
                        self.position + Vector2::new(14.0 * self.dir(), 6.0),
                    );
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                self.velocity.x = self.velocity.y(0.0).move_towards(Vector2::ZERO, DECEL).x;
                self.gravity();
                self
            }
            _ if !self.is_grounded() => {
                self.velocity.x = self.velocity.y(0.0).move_towards(Vector2::ZERO, DECEL).x;
                self.gravity();
                self
            }
            _ => Box::new(self.transition(
                BanditRevolverGroundedRecovery::<BANDIT_REVOLVER_GROUNDED_2_LANDING_LAG>,
                true,
            )),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn moveable(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some((
            "sol/specials/bandit_revolver/bandit_revolver2".into(),
            BASE_SPRITE_OFFSET + Vector2::LEFT * 4.0,
        ))
    }
}
impl SolDamageableState for BanditRevolverGroundedSecondHit {}

const BANDIT_REVOLVER_AIR_STARTUP: usize = BANDIT_REVOLVER_GROUNDED_1_STARTUP;
const BANDIT_REVOLVER_AIR_ACTIVE_1: usize = 3;
const BANDIT_REVOLVER_AIR_STARTUP_2: usize = 6;
const BANDIT_REVOLVER_AIR_ACTIVE_2: usize = 2;
const BANDIT_REVOLVER_AIR_RECOVERY: usize = 15;

pub struct BanditRevolverAir;
impl Entity for Sol<BanditRevolverAir> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const STARTUP_2: usize = BANDIT_REVOLVER_AIR_STARTUP + BANDIT_REVOLVER_AIR_ACTIVE_1;
        const HIT_2_FRAME: usize = STARTUP_2 + BANDIT_REVOLVER_AIR_STARTUP_2;
        const RECOVERY_FRAME: usize = HIT_2_FRAME + BANDIT_REVOLVER_AIR_ACTIVE_2;
        const END_FRAME: usize = RECOVERY_FRAME + BANDIT_REVOLVER_AIR_RECOVERY;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;

        match self.frame {
            0..BANDIT_REVOLVER_AIR_STARTUP => {
                self.velocity = Vector2::new(70.0 * self.dir(), 55.0);
                self
            }
            BANDIT_REVOLVER_AIR_STARTUP..STARTUP_2 => {
                //
                self
            }
            STARTUP_2..HIT_2_FRAME => {
                //
                self
            }
            HIT_2_FRAME..RECOVERY_FRAME => {
                self.gravity();
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                self.gravity();
                self
            }
            _ => self.air_actionable_state(input),
        }
    }
}
impl SolDamageableState for BanditRevolverAir {}
