use super::*;

const GUNFLAME_STARTUP: usize = 11;
const GUNFLAME_DECEL: f32 = 0.9;

/// bool is feint
pub struct GunFlameStartup<const FEINT: bool = false>;
impl GunFlameStartup {
    pub const fn feint() -> GunFlameStartup<true> {
        GunFlameStartup
    }
    pub const fn real() -> GunFlameStartup<false> {
        GunFlameStartup
    }
}
impl<const FEINT: bool> Entity for Sol<GunFlameStartup<FEINT>> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.has_hit = false;
        self.frame += 1;
        if self.velocity.x * self.dir() < 0.0 {
            self.velocity.x = 0.0;
        } else {
            self.velocity.x *= GUNFLAME_DECEL;
        }
        if self.frame > GUNFLAME_STARTUP {
            if FEINT {
                Box::new(self.transition(GunFlameFeint, true))
            } else {
                Box::new(self.transition(GunFlame, true))
            }
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        let mut path: String = "sol/gunflame/gunflame".into();
        path.push(match self.frame {
            0..4 => '1',
            ..8 => '2',
            _ => '3',
        });

        Some((path.into(), BASE_SPRITE_OFFSET))
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl<const FEINT: bool> SolDamageableState for GunFlameStartup<FEINT> {}

struct GunFlame;
impl Entity for Sol<GunFlame> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        // TODO: spawn projectile
        todo!()
    }
}
impl SolDamageableState for GunFlame {}

const GUNFLAME_FEINT_HOLD_LENGTH: usize = 8;
pub struct GunFlameFeint;
impl Entity for Sol<GunFlameFeint> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.frame += 1;

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
        Some(("sol/gunflame/gunflame3".into(), BASE_SPRITE_OFFSET))
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for GunFlameFeint {}
