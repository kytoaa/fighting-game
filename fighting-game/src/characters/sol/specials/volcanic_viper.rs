use super::*;

const VOLCANIC_VIPER_STARTUP: usize = 9;
const VOLCANIC_VIPER_ACTIVE_1: usize = 5;
const VOLCANIC_VIPER_ACTIVE_2: usize = 11;
const VOLCANIC_VIPER_RECOVERY: usize = 45;
const VOLCANIC_VIPER_DAMAGE_1: u32 = 15;
const VOLCANIC_VIPER_DAMAGE_2: u32 = 22;
const VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_1: u32 = 20;
const VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_2: u32 = 38;

pub struct VolcanicViper;
impl Entity for Sol<VolcanicViper> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
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
                    .counterhit_from_grounded(|g| g)
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
                    .counterhit_from_grounded(|g| g)
                    .meter_gain(HitData::DEFAULT_LEVEL_4_METER_GAIN)
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
                    .counterhit_from_grounded(|g| g)
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
                    .counterhit_from_grounded(|g| g)
                    .meter_gain(HitData::DEFAULT_LEVEL_4_METER_GAIN)
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
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                self.gravity();

                world.spawn_hurtbox(
                    self.create_hurtbox(CollisionShape::new(BoundingBox::with_size(Vector2::new(
                        16.0, 20.0,
                    )))),
                    self.position + Vector2::new(-3.0 * self.dir(), 6.0),
                );

                self
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
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const START_FRAME: usize = VOLCANIC_VIPER_STARTUP - 2;
        const ACTIVE_FRAME_2: usize = VOLCANIC_VIPER_STARTUP + VOLCANIC_VIPER_ACTIVE_1;
        const RECOVERY_FRAME: usize = ACTIVE_FRAME_2 + VOLCANIC_VIPER_ACTIVE_2;

        const OFFSET: Vector2 = BASE_SPRITE_OFFSET;

        Some(match self.frame as usize {
            0..START_FRAME => (
                "sol/specials/volcanic_viper/volcanic_viper1".into(),
                OFFSET + Vector2::DOWN * 8.0,
            ),
            START_FRAME..ACTIVE_FRAME_2 => (
                "sol/specials/volcanic_viper/volcanic_viper2".into(),
                OFFSET + Vector2::DOWN * 1.0,
            ),
            ACTIVE_FRAME_2..RECOVERY_FRAME => (
                "sol/specials/volcanic_viper/volcanic_viper3".into(),
                OFFSET + Vector2::UP * 5.0,
            ),
            _ if self.grounded => ("sol/run/run_stop".into(), BASE_SPRITE_OFFSET),
            _ => ("sol/specials/volcanic_viper/volcanic_viper4".into(), OFFSET),
        })
    }
    fn moveable(&self) -> bool {
        false
    }
}
impl SolDamageableState for VolcanicViper {}
