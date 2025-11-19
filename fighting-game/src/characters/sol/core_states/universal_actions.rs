use super::super::*;

#[derive(Clone)]
pub struct GroundThrow<const FACING_RIGHT: bool>;
impl<const FORWARD: bool> Player for Sol<GroundThrow<FORWARD>> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.velocity = Vector2::ZERO;

        world.spawn_hurtbox(
            self.create_hurtbox(crate::collision::CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );

        self.frame += 1;

        if self.frame == 4 {
            world.spawn_throwbox(
                crate::collision::ThrowBox {
                    shape: crate::collision::CollisionShape::new(BoundingBox::with_size(
                        Vector2::new(10.0, 20.0),
                    )),
                    owner: self.player_id,
                    throw_success: Box::new(Sol {
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
                        state: GroundThrowSuccess::<FORWARD>,
                    }),
                },
                self.position + Vector2::new(5.0 * self.dir(), 6.0),
            );
        }

        if self.frame < 35 {
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
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/throw".into(), BASE_SPRITE_OFFSET))
    }
}
impl<const FORWARD: bool> SolDamageableState for GroundThrow<FORWARD> {}

const GROUND_THROW_DAMAGE: u32 = 25;

#[derive(Clone)]
pub struct GroundThrowSuccess<const FORWARD: bool>;
impl<const FORWARD: bool> Player for Sol<GroundThrowSuccess<FORWARD>> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        if self.frame == 0 {
            self.direction = if FORWARD {
                self.direction
            } else {
                !self.direction
            };
        }

        if self.frame < 18 {
            world.set_entity_velocity(self.player_id.other_player(), Vector2::ZERO);
            world.set_entity_position(
                self.player_id.other_player(),
                self.position + Vector2::new(10.0 * self.dir(), 7.0),
            );
            if let Some(overlap) =
                World::position_in_wall(world.get_entity_position(self.player_id.other_player()))
            {
                self.position.x += overlap;

                world.set_entity_position(
                    self.player_id.other_player(),
                    self.position + Vector2::new(10.0 * self.dir(), 7.0),
                );
            }
        }
        self.velocity = Vector2::ZERO;

        self.frame += 1;

        match self.frame {
            0..28 => self,
            28 => {
                world.spawn_hitbox(
                    self.create_hitbox(
                        CollisionShape::new(BoundingBox::with_size(Vector2::new(20.0, 20.0))),
                        AttackData {
                            attack: HitData::grounded(
                                GROUND_THROW_DAMAGE,
                                HitEffect::launcher(
                                    Vector2::new(50.0 * self.dir(), 50.0),
                                    KnockdownType::Hard,
                                )
                                .build(),
                                0,
                                Proration::percent(100),
                                90,
                            )
                            .air_from_grounded(|g| g)
                            .counterhit_ground_from_ground_default()
                            .counterhit_air_from_air_default()
                            .build(),
                            hitbox_id: 1,
                            attack_id: "sol ground throw".into(),
                            priority: 10,
                            hit_level: crate::collision::HitLevel::Light,
                        },
                    ),
                    self.position + Vector2::new(8.0 * self.dir(), 5.0),
                );
                self
            }
            18..50 => self,
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
        Some(("sol/throw".into(), BASE_SPRITE_OFFSET))
    }
}
impl<const FACING_RIGHT: bool> SolDamageableState for GroundThrowSuccess<FACING_RIGHT> {}

impl<S> HasCancelState for Sol<S>
where
    Sol<S>: Player,
{
    fn cancel_state(self: Box<Sol<S>>) -> Box<dyn Player> {
        Box::new(self.transition(SolCancelState, true))
    }
}
#[derive(Clone)]
pub struct SolCancelState;
impl Player for Sol<SolCancelState> {
    fn update(self: Box<Self>, _: &mut World, _: &InputHandler) -> Box<dyn Player> {
        if self.grounded {
            Box::new(self.transition(Stand, true))
        } else {
            Box::new(self.transition(Air::<false>, true))
        }
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn actionable(&self) -> bool {
        false
    }
    fn throwable(&self) -> bool {
        false
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/cancel_state".into(), BASE_SPRITE_OFFSET))
    }
}
