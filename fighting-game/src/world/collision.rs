use super::World;
use crate::collision::HitLevel;
use crate::datatypes::Vector2;

impl World {
    pub(super) fn update_hitbox_hurtboxes(&mut self) {
        let mut collisions = vec![];

        for (i, hitbox) in self.hitboxes.iter().enumerate() {
            for (j, other) in self.hurtboxes.iter().enumerate() {
                if hitbox.0.shape.overlaps(&other.0.shape) {
                    collisions.push((i, j));
                }
            }
        }

        collisions = collisions
            .iter()
            .fold(
                std::collections::HashMap::new(),
                |mut acc: std::collections::HashMap<usize, (usize, usize)>,
                 (hitbox_index, hurtbox_index)| {
                    let hitbox = &self.hitboxes[*hitbox_index];
                    let hurtbox = &self.hurtboxes[*hurtbox_index];
                    // filter out self hits
                    if !(hitbox.0.owner == hurtbox.0.owner
                        || hitbox.0.owner.owned_by(&hurtbox.0.owner))
                    {
                        // if owner has hit something
                        if let Some(collision) = acc.get(&hurtbox.0.owner.id()) {
                            // the hitbox in the hashmap
                            let other = &self.hitboxes[collision.0];
                            if other.0.attack_data.priority > hitbox.0.attack_data.priority {
                                // if in hashmap has higher priority, return hashmap
                                return acc;
                            }
                        }
                        // otherwise insert other collision
                        acc.insert(hurtbox.0.owner.id(), (*hitbox_index, *hurtbox_index));
                    }
                    acc
                },
            )
            .into_values()
            .collect();

        let both_hit = collisions
            .iter()
            .map(|(_, hurtbox_index)| &self.hurtboxes[*hurtbox_index].0.owner)
            .filter(|hurtbox| hurtbox.is_player())
            .fold([false, false], |mut acc, hurtbox| {
                acc[hurtbox.id()] = true;
                acc
            })
            == [true, true];

        for (hitbox_index, hurtbox_index) in collisions {
            if both_hit {
                self.combo = None;
            }

            let hit_status = {
                let hitbox = &mut self.hitboxes[hitbox_index];
                let hurtbox = &mut self.hurtboxes[hurtbox_index];

                super::damaging::hit_player(
                    &mut self.players[hurtbox.0.owner.id()],
                    &mut self.player_data,
                    &mut self.combo,
                    &hitbox.0.attack_data.attack,
                    hitbox.0.attack_data.attack_id,
                )
            };

            println!("{:?}", hit_status);

            {
                let hit_effect_id = self.create_new_entity_id(crate::world::EntityType::Unique);

                let overlap = self.hitboxes[hitbox_index]
                    .0
                    .shape
                    .get_bounding_box()
                    .overlap_bb(&self.hurtboxes[hurtbox_index].0.shape.get_bounding_box());

                match hit_status {
                    crate::collision::HitConnectionStatus::Hit => {
                        match self.hitboxes[hitbox_index].0.attack_data.hit_level {
                            HitLevel::Light => {
                                self.spawn_non_player_entity(Box::new(
                                    crate::characters::sprite_entity::SpriteEntity::new(
                                        [
                                            ("effects/light_hit_effect1".into(), 0, Vector2::ZERO),
                                            ("effects/light_hit_effect2".into(), 3, Vector2::ZERO),
                                        ],
                                        overlap.position(),
                                        hit_effect_id,
                                        true,
                                        Vector2::ZERO,
                                    ),
                                ));
                            }
                            HitLevel::Heavy => {
                                self.spawn_non_player_entity(Box::new(
                                    crate::characters::sprite_entity::SpriteEntity::new(
                                        [
                                            ("effects/heavy_hit_effect1".into(), 0, Vector2::ZERO),
                                            ("effects/heavy_hit_effect2".into(), 3, Vector2::ZERO),
                                        ],
                                        overlap.position(),
                                        hit_effect_id,
                                        true,
                                        Vector2::ZERO,
                                    ),
                                ));
                            }
                            HitLevel::SuperHeavy => {
                                let hit_effect_id_2 =
                                    self.create_new_entity_id(crate::world::EntityType::Unique);
                                self.spawn_non_player_entity(Box::new(
                                    crate::characters::sprite_entity::SpriteEntity::new(
                                        [(
                                            "effects/super_heavy_hit_effect".into(),
                                            0,
                                            Vector2::ZERO,
                                        )],
                                        overlap.position() + Vector2::UP * 8.0,
                                        hit_effect_id_2,
                                        (hit_effect_id_2.id() / 4) % 2 == 0,
                                        Vector2::ZERO,
                                    )
                                    .draw_behind_players(),
                                ));
                                self.spawn_non_player_entity(Box::new(
                                    crate::characters::sprite_entity::SpriteEntity::new(
                                        [
                                            ("effects/heavy_hit_effect1".into(), 0, Vector2::ZERO),
                                            ("effects/heavy_hit_effect2".into(), 3, Vector2::ZERO),
                                        ],
                                        overlap.position(),
                                        hit_effect_id,
                                        true,
                                        Vector2::ZERO,
                                    ),
                                ));
                            }
                            _ => {
                                self.spawn_non_player_entity(Box::new(
                                    crate::characters::sprite_entity::SpriteEntity::new(
                                        [
                                            (
                                                "effects/default_hit_effect1".into(),
                                                0,
                                                Vector2::ZERO,
                                            ),
                                            (
                                                "effects/default_hit_effect2".into(),
                                                3,
                                                Vector2::ZERO,
                                            ),
                                        ],
                                        overlap.position(),
                                        hit_effect_id,
                                        true,
                                        Vector2::ZERO,
                                    ),
                                ));
                            }
                        }
                    }
                    crate::collision::HitConnectionStatus::Blocked
                    | crate::collision::HitConnectionStatus::Invuln => {
                        if let HitLevel::Light = self.hitboxes[hitbox_index].0.attack_data.hit_level
                        {
                            self.spawn_non_player_entity(Box::new(
                                crate::characters::sprite_entity::SpriteEntity::new(
                                    [
                                        (
                                            "effects/light_block_hit_effect1".into(),
                                            3,
                                            Vector2::ZERO,
                                        ),
                                        (
                                            "effects/light_block_hit_effect2".into(),
                                            3,
                                            Vector2::ZERO,
                                        ),
                                    ],
                                    overlap.position(),
                                    hit_effect_id,
                                    true,
                                    Vector2::ZERO,
                                ),
                            ));
                        } else {
                            self.spawn_non_player_entity(Box::new(
                                crate::characters::sprite_entity::SpriteEntity::new(
                                    [
                                        ("effects/block_hit_effect1".into(), 3, Vector2::ZERO),
                                        ("effects/block_hit_effect2".into(), 3, Vector2::ZERO),
                                    ],
                                    overlap.position(),
                                    hit_effect_id,
                                    true,
                                    Vector2::ZERO,
                                ),
                            ));
                        }
                    }
                }
            }

            let hitbox = &self.hitboxes[hitbox_index];
            let hurtbox = &self.hurtboxes[hurtbox_index];

            if hitbox.0.owner.is_player() {
                self.players
                    .get_mut(hitbox.0.owner.id())
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .on_hit(hit_status);
            } else {
                self.non_player_entities
                    .as_mut()
                    .unwrap()
                    .get_mut(&hitbox.0.owner.id())
                    .iter_mut()
                    .for_each(|e| e.on_hit(hit_status));
            }

            let hitstop_frames = match hit_status {
                crate::collision::HitConnectionStatus::Hit => {
                    hitbox.0.attack_data.hit_level.get_hitstop_frames()
                }
                crate::collision::HitConnectionStatus::Blocked => HitLevel::BLOCKED_HITSTOP_FRAMES,
                crate::collision::HitConnectionStatus::Invuln => 0,
            };

            println!(
                "hits: {}, damage: {}, scaling: {}",
                self.combo.as_ref().map(|c| c.hits()).unwrap_or(0),
                self.combo.as_ref().map(|c| c.total_damage()).unwrap_or(0),
                self.player_data[hurtbox.0.owner.id()].scaling,
            );

            self.trigger_hitstop(hitstop_frames);
        }

        if both_hit {
            self.combo = None;
        }

        self.decrement_hitbox_hurtbox_frame_timers();
    }
    pub(super) fn decrement_hitbox_hurtbox_frame_timers(&mut self) {
        self.hitboxes = self
            .hitboxes
            .drain(..)
            .into_iter()
            .filter_map(|mut h| {
                h.1 -= 1;
                if h.1 > 0 {
                    Some(h)
                } else {
                    None
                }
            })
            .collect();

        self.hurtboxes = self
            .hurtboxes
            .drain(..)
            .into_iter()
            .filter_map(|mut h| {
                h.1 -= 1;
                if h.1 > 0 {
                    Some(h)
                } else {
                    None
                }
            })
            .collect();
    }
    pub(super) fn update_throw_boxes(&mut self) -> bool {
        if self.throwboxes.len() != 1 {
            self.throwboxes = vec![];
            return false;
        }

        let throw_box = self.throwboxes.remove(0);

        let throw_target = self.hurtboxes.iter().find(|hb| {
            hb.0.owner.is_player()
                && throw_box.owner != hb.0.owner
                && throw_box.shape.overlaps(&hb.0.shape)
        });

        if let Some(hurtbox) = throw_target {
            println!("thrown");
            if {
                let p = self.players[hurtbox.0.owner.id()].as_ref().unwrap();
                p.is_grounded() && !p.in_hitstun() && p.throwable()
            } {
                _ = self.players[throw_box.owner.id()].insert((throw_box.throw_success)());

                let other_player = self.players[hurtbox.0.owner.id()].take().unwrap();
                _ = self.players[hurtbox.0.owner.id()].insert(other_player.thrown());
            }

            self.throwboxes = vec![];
            return true;
        }
        return false;
    }
}
