use super::*;

const GUNFLAME_FEINT_HOLD_LENGTH: usize = 5;
pub struct GunFlameFeint;
impl Player for Sol<GunFlameFeint> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        self.frame += 1;

        world.spawn_hurtbox(
            self.create_hurtbox(CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );

        if self.frame == 3 {
            world.spawn_hitbox(
                self.create_hitbox(
                    CollisionShape::new(BoundingBox::pos_size(
                        Vector2::ZERO,
                        Vector2::new(4.0, 5.0),
                    )),
                    AttackData {
                        attack: HitData::grounded(
                            10,
                            HitEffect::pushback(20.0 * self.dir(), 15).build(),
                            8,
                            Proration::percent(80),
                            HitData::DEFAULT_LEVEL_2_SCALING,
                        )
                        .air_from_grounded(|g| g)
                        .counterhit_from_grounded(|g| g)
                        .meter_gain(HitData::DEFAULT_LEVEL_2_METER_GAIN)
                        .build(),
                        priority: 10,
                        hitbox_id: 1,
                        hit_level: HitLevel::Light,
                        attack_id: "gunflame feint".into(),
                    },
                ),
                self.position + Vector2::new(10.0 * self.dir(), -4.0),
            );
        }
        if self.frame > GUNFLAME_FEINT_HOLD_LENGTH {
            self.grounded_actionable_state(input)
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/specials/gunflame/gunflame3".into(), BASE_SPRITE_OFFSET))
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for GunFlameFeint {}
