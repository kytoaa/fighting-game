use super::*;

pub const TOTAL_FRAMES: usize = ACTIVE_FRAMES_PER_PILLAR * TOTAL_FLAME_PILLARS;
const ACTIVE_FRAMES_PER_PILLAR: usize = 12;
const TOTAL_FLAME_PILLARS: usize = 4;

const GUNFLAME_DAMAGE: u32 = 20;
const GUNFLAME_BLOCKSTUN: usize = 20;

pub struct GunFlameProjectile {
    pub(super) frame: usize,
    pub(super) id: EntityID,
    pub(super) position: Vector2,
    pub(super) dir: f32,
    pub(super) has_hit: bool,
}
impl NonPlayerEntity for GunFlameProjectile {
    fn update(
        &mut self,
        world: &mut World,
        _: Option<&InputHandler>,
    ) -> crate::characters::EntityUpdateResult {
        const OFFSET_PER_PILLAR: f32 = 15.0;

        let pillar_number = self.frame / ACTIVE_FRAMES_PER_PILLAR;

        let position =
            self.position + Vector2::RIGHT * OFFSET_PER_PILLAR * pillar_number as f32 * self.dir;

        if self.frame % ACTIVE_FRAMES_PER_PILLAR == 0 {
            let id =
                world.create_new_entity_id(crate::world::EntityType::Owned(self.id.get_owner()));
            world.spawn_non_player_entity(Box::new(
                crate::characters::sprite_entity::SpriteEntity::new(
                    [
                        (
                            "sol/effects/gun_flame/gun_flame_pillar1".into(),
                            3,
                            Vector2::UP * 5.0,
                        ),
                        (
                            "sol/effects/gun_flame/gun_flame_pillar2".into(),
                            6,
                            Vector2::UP * 5.0,
                        ),
                        (
                            "sol/effects/gun_flame/gun_flame_pillar3".into(),
                            8,
                            Vector2::UP * 5.0,
                        ),
                        (
                            "sol/effects/gun_flame/gun_flame_pillar4".into(),
                            8,
                            Vector2::UP * 5.0,
                        ),
                        (
                            "sol/effects/gun_flame/gun_flame_pillar5".into(),
                            6,
                            Vector2::UP * 5.0,
                        ),
                    ],
                    position,
                    id,
                    self.dir == 1.0,
                    Vector2::ZERO,
                ),
            ));
        }

        if !self.has_hit {
            world.spawn_hitbox(
                self.create_hitbox(
                    CollisionShape::new(BoundingBox::with_size(Vector2::new(15.0, 20.0))),
                    AttackData {
                        attack: HitData::grounded(
                            GUNFLAME_DAMAGE - (2 * pillar_number) as u32,
                            HitEffect::launcher(
                                Vector2::new(10.0 * self.dir, 100.0),
                                KnockdownType::Soft,
                            )
                            .gravity(5.0)
                            .momentum_scaling((0.0, 0.0))
                            .build(),
                            GUNFLAME_BLOCKSTUN,
                            Proration::percent(70),
                            HitData::DEFAULT_LEVEL_2_SCALING,
                        )
                        .with_air(
                            HitEffect::launcher(
                                Vector2::new(10.0 * self.dir, 70.0),
                                KnockdownType::Soft,
                            )
                            .gravity(5.0)
                            .momentum_scaling((0.0, 0.0))
                            .build(),
                        )
                        .counterhit_ground_from_ground_default()
                        .counterhit_air_from_air_default()
                        .meter_gain(HitData::DEFAULT_LEVEL_3_METER_GAIN)
                        .wall_pushback_mult(0.0)
                        .build(),
                        attack_id: "gunflame".into(),
                        hitbox_id: 1,
                        priority: 1,
                        hit_level: HitLevel::Light,
                    },
                ),
                position + Vector2::UP * 2.0,
            );
        }

        self.frame += 1;
        if self.frame > TOTAL_FRAMES {
            EntityUpdateResult::Remove
        } else {
            EntityUpdateResult::Continue
        }
    }
}

impl HasID for GunFlameProjectile {
    fn id(&self) -> EntityID {
        self.id
    }
}
impl Position for GunFlameProjectile {
    fn position(&self) -> Vector2 {
        self.position
    }
    fn set_position(&mut self, position: Vector2) {
        self.position = position;
    }
    fn move_by(&mut self, distance: Vector2) {
        self.position += distance;
    }
}
impl OnHit for GunFlameProjectile {
    fn on_hit(&mut self, _: crate::collision::HitConnectionStatus) {
        self.has_hit = true;
    }
}
