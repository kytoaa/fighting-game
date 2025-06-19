use super::*;

const VOLCANIC_VIPER_STARTUP: usize = 9;
const VOLCANIC_VIPER_ACTIVE_1: usize = 5;
const VOLCANIC_VIPER_ACTIVE_2: usize = 11;
const VOLCANIC_VIPER_RECOVERY: usize = 45;
const VOLCANIC_VIPER_DAMAGE_1: u32 = 15;
const VOLCANIC_VIPER_DAMAGE_2: u32 = 22;
const VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_1: u32 = 20;
const VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_2: u32 = 38;

const KNOCKDOWN_STARTUP: usize = 18;
const KNOCKDOWN_ACTIVE: usize = 3;
const KNOCKDOWN_LANDING_LAG: usize = 16;
const KNOCKDOWN_DAMAGE: u32 = 12;

pub struct VolcanicViper;
impl Player for Sol<VolcanicViper> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const ACTIVE_FRAME_2: usize = VOLCANIC_VIPER_STARTUP + VOLCANIC_VIPER_ACTIVE_1;
        const RECOVERY_FRAME: usize = ACTIVE_FRAME_2 + VOLCANIC_VIPER_ACTIVE_2;
        const END_FRAME: usize = RECOVERY_FRAME + VOLCANIC_VIPER_RECOVERY;
        const DECEL: f32 = 4.0;
        const LAUNCH_VELOCITY: Vector2 = Vector2::new(30.0, 160.0);

        if self.frame == 0 || self.frame == ACTIVE_FRAME_2 {
            self.has_hit = false;
        }

        self.frame += 1;

        match self.frame as usize {
            0..VOLCANIC_VIPER_STARTUP => {
                self.velocity.x = self.velocity.y(0.0).move_towards(Vector2::ZERO, DECEL).x;
                self
            }
            VOLCANIC_VIPER_STARTUP..ACTIVE_FRAME_2 => {
                if self.frame as usize == VOLCANIC_VIPER_STARTUP + VOLCANIC_VIPER_ACTIVE_1 - 1 {
                    self.velocity =
                        Vector2::new(LAUNCH_VELOCITY.x * self.dir(), LAUNCH_VELOCITY.y * 1.3);
                }

                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        VOLCANIC_VIPER_ACTIVE_1 - (self.frame as usize - VOLCANIC_VIPER_STARTUP);
                    let hit_info = HitData::grounded(
                        VOLCANIC_VIPER_DAMAGE_1,
                        HitEffect::launcher(
                            Vector2::new(30.0 * self.dir(), LAUNCH_VELOCITY.y),
                            KnockdownType::Soft,
                        )
                        .build(),
                        12 + active_frames_extra_hitstun,
                        Proration::percent(70),
                        HitData::DEFAULT_LEVEL_1_SCALING,
                    )
                    .air_from_grounded(|g| g)
                    .counterhit_ground_from_ground_default()
                    .counterhit_air_from_air_default()
                    .meter_gain(HitData::DEFAULT_LEVEL_4_METER_GAIN)
                    .build();

                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(12.0, 24.0))),
                            AttackData {
                                attack: hit_info.clone(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Medium,
                                attack_id: "volcanic viper".into(),
                            },
                        ),
                        self.position + Vector2::new(8.0 * self.dir(), 6.0),
                    );
                    let clean_hit_info = HitData::grounded(
                        VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_1,
                        HitEffect::launcher(
                            Vector2::new(30.0 * self.dir(), LAUNCH_VELOCITY.y),
                            KnockdownType::Hard,
                        )
                        .build(),
                        12 + active_frames_extra_hitstun,
                        Proration::percent(70),
                        HitData::DEFAULT_LEVEL_1_SCALING,
                    )
                    .air_from_grounded(|g| g)
                    .counterhit_ground_from_ground_default()
                    .counterhit_air_from_air_default()
                    .meter_gain(HitData::DEFAULT_LEVEL_4_METER_GAIN)
                    .add_extension(HitDataExtension::SetScaling(
                        HitData::DEFAULT_LEVEL_1_SCALING,
                    ))
                    .build();

                    // NOTE: clean hit
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(2.0, 8.0))),
                            AttackData {
                                attack: clean_hit_info,
                                priority: 20,
                                hitbox_id: 1,
                                hit_level: HitLevel::SuperHeavy,
                                attack_id: "volcanic viper".into(),
                            },
                        ),
                        self.position + Vector2::new(4.0 * self.dir(), 6.0),
                    );
                }
                self
            }
            ACTIVE_FRAME_2..RECOVERY_FRAME => {
                self.gravity();

                world.spawn_hurtbox(
                    self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(Vector2::new(
                        16.0, 20.0,
                    )))),
                    self.position + Vector2::new(-3.0 * self.dir(), 6.0),
                );

                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        VOLCANIC_VIPER_ACTIVE_2 - (self.frame as usize - ACTIVE_FRAME_2);
                    let hit_info = HitData::grounded(
                        VOLCANIC_VIPER_DAMAGE_2,
                        HitEffect::launcher(
                            Vector2::new(50.0 * self.dir(), LAUNCH_VELOCITY.y),
                            KnockdownType::Soft,
                        )
                        .build(),
                        12 + active_frames_extra_hitstun,
                        Proration::percent(70),
                        HitData::DEFAULT_LEVEL_1_SCALING,
                    )
                    .air_from_grounded(|g| g)
                    .counterhit_ground_from_ground_default()
                    .counterhit_air_from_air_default()
                    .meter_gain(HitData::DEFAULT_LEVEL_4_METER_GAIN)
                    .build();

                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(18.0, 24.0))),
                            AttackData {
                                attack: hit_info,
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Heavy,
                                attack_id: "volcanic viper".into(),
                            },
                        ),
                        self.position + Vector2::new(8.0 * self.dir(), 12.0),
                    );

                    let clean_hit_info = HitData::grounded(
                        VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_2,
                        HitEffect::launcher(
                            Vector2::new(50.0 * self.dir(), LAUNCH_VELOCITY.y),
                            KnockdownType::Hard,
                        )
                        .build(),
                        12 + active_frames_extra_hitstun,
                        Proration::percent(70),
                        HitData::DEFAULT_LEVEL_1_SCALING,
                    )
                    .air_from_grounded(|g| g)
                    .counterhit_ground_from_ground_default()
                    .counterhit_air_from_air_default()
                    .meter_gain(HitData::DEFAULT_LEVEL_4_METER_GAIN)
                    .add_extension(HitDataExtension::SetScaling(
                        HitData::DEFAULT_LEVEL_1_SCALING,
                    ))
                    .build();

                    // NOTE: clean hit
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(4.0, 8.0))),
                            AttackData {
                                attack: clean_hit_info,
                                priority: 20,
                                hitbox_id: 1,
                                hit_level: HitLevel::SuperHeavy,
                                attack_id: "volcanic viper".into(),
                            },
                        ),
                        self.position + Vector2::new(5.0 * self.dir(), 9.0),
                    );
                }

                if input.has_motion_input(
                    &crate::input::directions::Motion::quarter_circle()
                        .direction(!self.direction)
                        .frames(40),
                    &Action::Pressed(Button::Light, None),
                ) {
                    Box::new(self.transition(Knockdown, true))
                } else {
                    self
                }
            }
            RECOVERY_FRAME..END_FRAME => {
                self.gravity();

                world.spawn_hurtbox(
                    self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(Vector2::new(
                        16.0, 20.0,
                    )))),
                    self.position + Vector2::new(-3.0 * self.dir(), 6.0),
                );

                if input.has_motion_input(
                    &crate::input::directions::Motion::quarter_circle()
                        .direction(!self.direction)
                        .frames(40),
                    &Action::Pressed(Button::Light, None),
                ) {
                    Box::new(self.transition(Knockdown, true))
                } else {
                    self
                }
            }
            _ => {
                world.spawn_hurtbox(
                    self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(Vector2::new(
                        16.0, 20.0,
                    )))),
                    self.position + Vector2::new(-3.0 * self.dir(), 6.0),
                );

                self.air_actionable_state(input)
            }
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn can_cancel(&self) -> bool {
        self.has_hit
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const START_FRAME: usize = VOLCANIC_VIPER_STARTUP - 2;
        const ACTIVE_FRAME_2: usize = VOLCANIC_VIPER_STARTUP + VOLCANIC_VIPER_ACTIVE_1;
        const RECOVERY_FRAME: usize = ACTIVE_FRAME_2 + VOLCANIC_VIPER_ACTIVE_2;
        const FLAME_END_FRAME: usize = RECOVERY_FRAME + 4;

        const OFFSET: Vector2 = BASE_SPRITE_OFFSET;

        Some(match self.frame as usize {
            0..START_FRAME => (
                "sol/specials/volcanic_viper/volcanic_viper1".into(),
                OFFSET + Vector2::UP * 0.0,
            ),
            START_FRAME..ACTIVE_FRAME_2 => (
                "sol/specials/volcanic_viper/volcanic_viper2".into(),
                OFFSET + Vector2::UP * 7.0,
            ),
            ACTIVE_FRAME_2..RECOVERY_FRAME => (
                "sol/specials/volcanic_viper/volcanic_viper3".into(),
                OFFSET + Vector2::UP * 8.0,
            ),
            RECOVERY_FRAME..FLAME_END_FRAME => (
                "sol/specials/volcanic_viper/volcanic_viper4".into(),
                OFFSET + Vector2::UP * 8.0,
            ),
            _ if self.grounded => ("sol/run/run_stop".into(), BASE_SPRITE_OFFSET),
            _ => (
                "sol/specials/volcanic_viper/volcanic_viper5".into(),
                OFFSET + Vector2::UP * 8.0,
            ),
        })
    }
    fn moveable(&self) -> bool {
        false
    }
}
impl SolDamageableState for VolcanicViper {}

struct Knockdown;
impl Player for Sol<Knockdown> {
    fn update(mut self: Box<Self>, world: &mut World, _input: &InputHandler) -> Box<dyn Player> {
        const RECOVERY_FRAME: usize = KNOCKDOWN_STARTUP + KNOCKDOWN_ACTIVE;
        if self.frame == 0 {
            self.has_hit = false;
        }
        self.frame += 1;

        world.spawn_hurtbox(
            self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(Vector2::new(
                16.0, 20.0,
            )))),
            self.position + Vector2::new(-3.0 * self.dir(), 6.0),
        );

        match self.frame {
            0..KNOCKDOWN_STARTUP => {
                self.gravity();
                self
            }
            KNOCKDOWN_STARTUP..RECOVERY_FRAME => {
                self.velocity.y = 0.0;

                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        KNOCKDOWN_ACTIVE - (self.frame as usize - KNOCKDOWN_STARTUP);

                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::with_size(Vector2::new(25.0, 20.0))),
                            AttackData {
                                attack: HitData::grounded(
                                    KNOCKDOWN_DAMAGE,
                                    HitEffect::pushback(
                                        20.0 * self.dir(),
                                        16 + active_frames_extra_hitstun,
                                    )
                                    .build(),
                                    16 + active_frames_extra_hitstun,
                                    Proration::percent(80),
                                    HitData::DEFAULT_LEVEL_1_SCALING,
                                )
                                .with_air(
                                    HitEffect::launcher(
                                        Vector2::new(30.0 * self.dir(), -100.0),
                                        KnockdownType::Hard,
                                    )
                                    .momentum_scaling((0.0, 0.0))
                                    .build(),
                                )
                                .counterhit_ground_from_ground_default()
                                .counterhit_air_from_air_default()
                                .meter_gain(HitData::DEFAULT_LEVEL_2_METER_GAIN)
                                .build(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Light,
                                attack_id: "vv knockdown".into(),
                            },
                        ),
                        self.position + Vector2::new(12.0 * self.dir(), 4.0),
                    );
                }
                self
            }
            f @ RECOVERY_FRAME.. => {
                if f > RECOVERY_FRAME + 10 && self.has_hit {
                    self.velocity.y = -110.0;
                    self.gravity();
                    self.gravity();
                }
                self.gravity();

                if self.is_grounded() {
                    Box::new(self.transition(
                        BanditRevolverGroundedRecovery::<KNOCKDOWN_LANDING_LAG>,
                        true,
                    ))
                } else {
                    self
                }
            }
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn moveable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const ANIM_START_FRAME: usize = KNOCKDOWN_STARTUP - 8;
        const ANIM_RECOVERY_FRAME: usize = KNOCKDOWN_STARTUP + KNOCKDOWN_ACTIVE + 10;

        Some(match self.frame {
            0..ANIM_START_FRAME => ("sol/fall/fall2".into(), BASE_SPRITE_OFFSET),
            ANIM_START_FRAME..KNOCKDOWN_STARTUP => (
                "sol/specials/bandit_revolver/bandit_revolver1".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * 4.0 * self.dir(),
            ),
            KNOCKDOWN_STARTUP..ANIM_RECOVERY_FRAME => (
                "sol/specials/bandit_revolver/bandit_revolver2".into(),
                BASE_SPRITE_OFFSET + Vector2::LEFT * 4.0 * self.dir(),
            ),
            _ => ("sol/fall/fall2".into(), BASE_SPRITE_OFFSET),
        })
    }
}
impl SolDamageableState for Knockdown {}
