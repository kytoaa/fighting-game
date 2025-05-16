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
        if self.frame > GUNFLAME_STARTUP as u8 {
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
                Hitbox {
                    shape: CollisionShape::new(BoundingBox::pos_size(
                        Vector2::ZERO,
                        Vector2::new(4.0, 5.0),
                    )),
                    owner: self.player_id,
                    info: AttackData::with_same_hitinfo(
                        HitInfo {
                            damage: 10,
                            hitstun: 15,
                            blockstun: 8,
                            hit_effect: HitEffect::Pushback(20.0 * self.dir()),
                            block_push: 8.0 * self.dir(),
                        },
                        1,
                        crate::collision::AttackType::Mid,
                        1,
                        crate::collision::HitLevel::Light,
                    ),
                },
                self.position + Vector2::new(10.0 * self.dir(), -4.0),
            );
        }
        if self.frame > GUNFLAME_FEINT_HOLD_LENGTH as u8 {
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
