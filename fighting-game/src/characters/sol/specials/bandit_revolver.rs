use super::*;

const BANDIT_REVOLVER_GROUNDED_1_STARTUP: usize = 12;
const BANDIT_REVOLVER_GROUNDED_1_ACTIVE: usize = 6;
const BANDIT_REVOLVER_GROUNDED_1_RECOVERY: usize = 16;
const BANDIT_REVOLVER_GROUNDED_1_LANDING_LAG: usize = 4;
const BANDIT_REVOLVER_GROUNDED_1_DAMAGE: u16 = 11;

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
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                14.0, 14.0,
                            ))),
                            info: AttackData {
                                grounded: HitInfo {
                                    damage: BANDIT_REVOLVER_GROUNDED_1_DAMAGE,
                                    hitstun: 11 + active_frames_extra_hitstun,
                                    blockstun: 8 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Pushback(30.0 * self.dir()),
                                    block_push: 60.0 * self.dir(),
                                },
                                air: HitInfo {
                                    damage: BANDIT_REVOLVER_GROUNDED_1_DAMAGE,
                                    hitstun: 11 + active_frames_extra_hitstun,
                                    blockstun: 8 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(50.0 * self.dir(), 60.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 80.0 * self.dir(),
                                },
                                counterhit: HitInfo {
                                    damage: BANDIT_REVOLVER_GROUNDED_1_DAMAGE,
                                    hitstun: 11 + active_frames_extra_hitstun,
                                    blockstun: 8 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(50.0 * self.dir(), 60.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 80.0 * self.dir(),
                                },
                                priority: 10,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitLevel::Light,
                            },
                            owner: self.player,
                        },
                        self.position + Vector2::new(10.0 * self.dir(), 6.0),
                        1,
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
}
impl SolDamageableState for BanditRevolverGrounded {}

struct BanditRevolverGroundedRecovery<const FRAMES: usize>;
impl<const FRAMES: usize> Entity for Sol<BanditRevolverGroundedRecovery<FRAMES>> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const DECEL: f32 = 5.0;

        self.has_hit = false;
        self.frame += 1;

        world.spawn_hurtbox(
            crate::collision::Hurtbox {
                shape: crate::collision::CollisionShape::Box(STANDING_HURTBOX),
                owner: self.player,
            },
            self.position,
            1,
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
}
impl<const FRAMES: usize> SolDamageableState for BanditRevolverGroundedRecovery<FRAMES> {}

const BANDIT_REVOLVER_GROUNDED_2_STARTUP: usize = 6;
const BANDIT_REVOLVER_GROUNDED_2_ACTIVE: usize = 2;
const BANDIT_REVOLVER_GROUNDED_2_RECOVERY: usize = 8;
const BANDIT_REVOLVER_GROUNDED_2_LANDING_LAG: usize = 15;
const BANDIT_REVOLVER_GROUNDED_2_DAMAGE: u16 = 14;

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
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                20.0, 14.0,
                            ))),
                            info: AttackData {
                                grounded: HitInfo {
                                    damage: BANDIT_REVOLVER_GROUNDED_2_DAMAGE,
                                    hitstun: 43 + active_frames_extra_hitstun,
                                    blockstun: 14 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(100.0 * self.dir(), 100.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 60.0 * self.dir(),
                                },
                                air: HitInfo {
                                    damage: BANDIT_REVOLVER_GROUNDED_2_DAMAGE,
                                    hitstun: 43 + active_frames_extra_hitstun,
                                    blockstun: 14 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(100.0 * self.dir(), 100.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 80.0 * self.dir(),
                                },
                                counterhit: HitInfo {
                                    damage: BANDIT_REVOLVER_GROUNDED_2_DAMAGE,
                                    hitstun: 43 + active_frames_extra_hitstun,
                                    blockstun: 14 + active_frames_extra_hitstun,
                                    hit_effect: HitEffect::Launcher(
                                        Vector2::new(100.0 * self.dir(), 100.0),
                                        KnockdownType::Soft,
                                    ),
                                    block_push: 80.0 * self.dir(),
                                },
                                priority: 10,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitLevel::Light,
                            },
                            owner: self.player,
                        },
                        self.position + Vector2::new(14.0 * self.dir(), 6.0),
                        1,
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
}
impl SolDamageableState for BanditRevolverGroundedSecondHit {}
