use super::*;

mod gunflame_feint;
mod gunflame_projectile;

pub use gunflame_feint::*;
pub use gunflame_projectile::*;

const GUNFLAME_STARTUP: usize = 14;
const GUNFLAME_DECEL: f32 = 3.0;

/// bool is feint
#[derive(Clone)]
pub struct GunFlameStartup<const FEINT: bool = false>;
impl GunFlameStartup {
    pub const fn feint() -> GunFlameStartup<true> {
        GunFlameStartup
    }
    pub const fn real() -> GunFlameStartup<false> {
        GunFlameStartup
    }
}
impl<const FEINT: bool> Player for Sol<GunFlameStartup<FEINT>> {
    fn update(mut self: Box<Self>, world: &mut World, _: &InputHandler) -> Box<dyn Player> {
        self.has_hit = false;
        self.frame += 1;

        self.velocity.x = self.velocity.x.move_towards(0.0, GUNFLAME_DECEL);

        world.spawn_hurtbox(
            self.create_hurtbox(CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );

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
        Some(match self.frame {
            0..4 => ("sol/normals/5h/5h1".into(), BASE_SPRITE_OFFSET),
            4..10 => ("sol/specials/gunflame/gunflame1".into(), BASE_SPRITE_OFFSET),
            10..15 => ("sol/specials/gunflame/gunflame2".into(), BASE_SPRITE_OFFSET),
            15.. => ("sol/specials/gunflame/gunflame3".into(), BASE_SPRITE_OFFSET),
        })
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl<const FEINT: bool> SolDamageableState for GunFlameStartup<FEINT> {}

#[derive(Clone)]
struct GunFlame;
impl Player for Sol<GunFlame> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        if self.frame == 0 {
            let id =
                world.create_new_entity_id(crate::world::EntityType::Owned(self.player_id.id()));

            world.spawn_non_player_entity(Box::new(GunFlameProjectile {
                frame: 0,
                id,
                position: self.position + Vector2::RIGHT * 8.0 * self.dir(),
                dir: self.dir(),
                has_hit: false,
            }));
        }

        self.velocity.x = self.velocity.x.move_towards(0.0, GUNFLAME_DECEL);

        world.spawn_hurtbox(
            self.create_hurtbox(CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );

        self.frame += 1;

        const TOTAL: usize = TOTAL_FRAMES - 6;

        match self.frame {
            0..TOTAL => self,
            _ => self.grounded_actionable_state(input),
        }
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/specials/gunflame/gunflame3".into(), BASE_SPRITE_OFFSET))
    }
}
impl SolDamageableState for GunFlame {}
