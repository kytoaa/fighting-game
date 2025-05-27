use super::*;

pub struct WildThrow {
    success: std::rc::Rc<std::cell::Cell<bool>>,
}
impl WildThrow {
    pub fn new() -> Self {
        Self {
            success: std::cell::Cell::new(false).into(),
        }
    }
}
impl Player for Sol<WildThrow> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.velocity = Vector2::ZERO;

        world.spawn_hurtbox(
            self.create_hurtbox(crate::collision::CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );

        self.frame += 1;

        if self.frame == 3 {
            world.spawn_throwbox(
                crate::collision::ThrowBox {
                    shape: crate::collision::CollisionShape::new(BoundingBox::with_size(
                        Vector2::new(10.0, 20.0),
                    )),
                    owner: self.player_id,
                    throw_success: {
                        let success = self.state.success.clone();
                        let success_state = Box::new(Sol {
                            player_id: self.player_id,
                            position: self.position,
                            velocity: Vector2::ZERO,
                            collider: self.collider.clone(),
                            direction: self.direction,
                            has_hit: false,
                            grounded: true,
                            has_air_action: true,
                            distance_from_other_player: self.distance_from_other_player,
                            frame: 0,
                            state: WildThrowSuccess,
                        });
                        Box::new(move || {
                            success.set(true);
                            println!("successful throw");
                            success_state
                        })
                    },
                },
                self.position + Vector2::new(9.0 * self.dir(), 6.0),
            );
        }

        if self.frame < 50 {
            self
        } else {
            self.grounded_actionable_state(input)
        }
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for WildThrow {}

const GROUND_THROW_DAMAGE: u32 = 30;

struct WildThrowSuccess;
impl Player for Sol<WildThrowSuccess> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        if self.frame == 0 {
            self.has_hit = false;
        }
        if self.frame == 28 {
            self.direction = !self.direction;
        }

        self.velocity = Vector2::ZERO;

        self.frame += 1;

        match self.frame {
            0..40 => {
                world.set_entity_velocity(self.player_id.other_player(), Vector2::ZERO);
                world.set_entity_position(
                    self.player_id.other_player(),
                    self.position + Vector2::new(8.0 * self.dir(), 5.0),
                );
                if let Some(overlap) = World::position_in_wall(
                    world.get_entity_position(self.player_id.other_player()),
                ) {
                    self.position.x += overlap;

                    world.set_entity_position(
                        self.player_id.other_player(),
                        self.position + Vector2::new(8.0 * self.dir(), 5.0),
                    );
                }
                self
            }
            40 => {
                world.spawn_hitbox(
                    self.create_hitbox(
                        CollisionShape::new(BoundingBox::with_size(Vector2::new(20.0, 20.0))),
                        AttackData {
                            attack: HitData::grounded(
                                GROUND_THROW_DAMAGE,
                                HitEffect::launcher(
                                    Vector2::new(10.0 * self.dir(), 0.0),
                                    KnockdownType::Hard,
                                )
                                .ground_bounce(
                                    BounceInfo::new(Vector2::new(10.0, 120.0)).gravity(6.0),
                                )
                                .build(),
                                0,
                                Proration::percent(80),
                                HitData::DEFAULT_LEVEL_1_SCALING,
                            )
                            .air_from_grounded(|g| g)
                            .counterhit_ground_from_ground_default()
                            .counterhit_air_from_air_default()
                            .build(),
                            hitbox_id: 1,
                            attack_id: "sol wild throw".into(),
                            priority: 10,
                            hit_level: crate::collision::HitLevel::Light,
                        },
                    ),
                    self.position + Vector2::new(15.0 * self.dir(), 5.0),
                );
                self
            }
            40..50 => self,
            _ => self.grounded_actionable_state(input),
        }
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        // TODO: wild throw sprites
        Some(("sol/throw".into(), BASE_SPRITE_OFFSET))
    }
}
impl SolDamageableState for WildThrowSuccess {}
