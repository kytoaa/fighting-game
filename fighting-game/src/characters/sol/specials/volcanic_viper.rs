use super::*;

const VOLCANIC_VIPER_STARTUP: usize = 9;
const VOLCANIC_VIPER_ACTIVE_1: usize = 5;
const VOLCANIC_VIPER_ACTIVE_2: usize = 11;
const VOLCANIC_VIPER_RECOVERY: usize = 45;
const VOLCANIC_VIPER_DAMAGE_1: u16 = 15;
const VOLCANIC_VIPER_DAMAGE_2: u16 = 22;
const VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_1: u16 = 20;
const VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_2: u16 = 38;

pub struct VolcanicViper;
impl Entity for Sol<VolcanicViper> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        const ACTIVE_FRAME_2: usize = VOLCANIC_VIPER_STARTUP + VOLCANIC_VIPER_ACTIVE_1;
        const RECOVERY_FRAME: usize = ACTIVE_FRAME_2 + VOLCANIC_VIPER_ACTIVE_2;
        const END_FRAME: usize = RECOVERY_FRAME + VOLCANIC_VIPER_RECOVERY;
        const DECEL: f32 = 4.0;
        const LAUNCH_VELOCITY: Vector2 = Vector2::new(30.0, 160.0);

        if self.frame == 0 || self.frame == ACTIVE_FRAME_2 as u8 {
            self.has_hit = false;
        }

        self.frame += 1;

        match self.frame as usize {
            0..VOLCANIC_VIPER_STARTUP => {
                self.velocity.x = self.velocity.y(0.0).move_towards(&Vector2::ZERO, DECEL).x;
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
                    let hit_info = HitInfo {
                        damage: VOLCANIC_VIPER_DAMAGE_1,
                        hitstun: 50 + active_frames_extra_hitstun,
                        blockstun: 12 + active_frames_extra_hitstun,
                        hit_effect: HitEffect::Launcher(
                            Vector2::new(30.0 * self.dir(), LAUNCH_VELOCITY.y),
                            KnockdownType::Soft,
                        ),
                        block_push: 60.0 * self.dir(),
                    };
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                12.0, 24.0,
                            ))),
                            info: AttackData {
                                grounded: hit_info.clone(),
                                air: hit_info.clone(),
                                counterhit: hit_info.clone(),
                                priority: 10,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::Medium,
                            },
                            owner: self.player,
                        },
                        self.position + Vector2::new(8.0 * self.dir(), 6.0),
                        1,
                    );
                    let clean_hit_info = {
                        let mut info = hit_info;
                        info.damage = VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_1;
                        info.hit_effect = HitEffect::Launcher(
                            Vector2::new(30.0 * self.dir(), LAUNCH_VELOCITY.y),
                            KnockdownType::Hard,
                        );
                        info
                    };
                    // NOTE: clean hit
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                2.0, 8.0,
                            ))),
                            info: AttackData {
                                grounded: clean_hit_info.clone(),
                                air: clean_hit_info.clone(),
                                counterhit: clean_hit_info.clone(),
                                priority: 20,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::SuperHeavy,
                            },
                            owner: self.player,
                        },
                        self.position + Vector2::new(4.0 * self.dir(), 6.0),
                        1,
                    );
                }
                self
            }
            ACTIVE_FRAME_2..RECOVERY_FRAME => {
                self.gravity();

                world.spawn_hurtbox(
                    crate::collision::Hurtbox {
                        shape: crate::collision::CollisionShape::Box(BoundingBox::with_size(
                            Vector2::new(16.0, 20.0),
                        )),
                        owner: self.player,
                    },
                    self.position + Vector2::new(-3.0 * self.dir(), 6.0),
                    1,
                );

                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        VOLCANIC_VIPER_ACTIVE_2 - (self.frame as usize - ACTIVE_FRAME_2);
                    let hit_info = HitInfo {
                        damage: VOLCANIC_VIPER_DAMAGE_2,
                        hitstun: 50 + active_frames_extra_hitstun,
                        blockstun: 12 + active_frames_extra_hitstun,
                        hit_effect: HitEffect::Launcher(
                            Vector2::new(50.0 * self.dir(), LAUNCH_VELOCITY.y),
                            KnockdownType::Soft,
                        ),
                        block_push: 60.0 * self.dir(),
                    };
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                18.0, 24.0,
                            ))),
                            info: AttackData {
                                grounded: hit_info.clone(),
                                air: hit_info.clone(),
                                counterhit: hit_info.clone(),
                                priority: 10,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::Heavy,
                            },
                            owner: self.player,
                        },
                        self.position + Vector2::new(8.0 * self.dir(), 12.0),
                        1,
                    );
                    let clean_hit_info = {
                        let mut info = hit_info;
                        info.damage = VOLCANIC_VIPER_CLEAN_HIT_DAMAGE_2;
                        info.hit_effect = HitEffect::Launcher(
                            Vector2::new(50.0 * self.dir(), LAUNCH_VELOCITY.y),
                            KnockdownType::Hard,
                        );
                        info
                    };
                    // NOTE: clean hit
                    world.spawn_hitbox(
                        Hitbox {
                            shape: CollisionShape::Box(BoundingBox::with_size(Vector2::new(
                                4.0, 8.0,
                            ))),
                            info: AttackData {
                                grounded: clean_hit_info.clone(),
                                air: clean_hit_info.clone(),
                                counterhit: clean_hit_info.clone(),
                                priority: 20,
                                attack_type: crate::collision::AttackType::Mid,
                                hitbox_id: 1,
                                hit_type: crate::collision::HitType::SuperHeavy,
                            },
                            owner: self.player,
                        },
                        self.position + Vector2::new(5.0 * self.dir(), 9.0),
                        1,
                    );
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                self.gravity();

                world.spawn_hurtbox(
                    crate::collision::Hurtbox {
                        shape: crate::collision::CollisionShape::Box(BoundingBox::with_size(
                            Vector2::new(16.0, 20.0),
                        )),
                        owner: self.player,
                    },
                    self.position + Vector2::new(-3.0 * self.dir(), 6.0),
                    1,
                );

                self
            }
            _ => {
                world.spawn_hurtbox(
                    crate::collision::Hurtbox {
                        shape: crate::collision::CollisionShape::Box(BoundingBox::with_size(
                            Vector2::new(16.0, 20.0),
                        )),
                        owner: self.player,
                    },
                    self.position + Vector2::new(-3.0 * self.dir(), 6.0),
                    1,
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
}
impl SolDamageableState for VolcanicViper {}
