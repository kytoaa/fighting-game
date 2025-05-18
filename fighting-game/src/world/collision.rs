use super::World;
use crate::collision::HitLevel;

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
                    acc
                },
            )
            .into_values()
            .collect();

        for (hitbox_index, hurtbox_index) in collisions {
            let hitbox = &mut self.hitboxes[hitbox_index];
            let hurtbox = &mut self.hurtboxes[hurtbox_index];

            println!("collision");
            if hitbox.0.owner == hurtbox.0.owner {
                println!("owner same");
                continue;
            }
            let other_player_position = self.players[hitbox.0.owner.id()]
                .as_ref()
                .unwrap()
                .position();

            let hit_status = super::damaging::hit_player(
                &mut self.players[hurtbox.0.owner.id()],
                &mut self.player_data[hurtbox.0.owner.id()],
                &mut self.combo,
                &hitbox.0.attack_data.attack,
                other_player_position,
            );

            println!("{:?}", hit_status);

            if hitbox.0.owner.is_player() {
                self.players
                    .get_mut(hitbox.0.owner.id())
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .on_hit(hit_status);
            }

            let hitstop_frames = match hit_status {
                crate::collision::HitConnectionStatus::Hit => {
                    hitbox.0.attack_data.hit_level.get_hitstop_frames()
                }
                crate::collision::HitConnectionStatus::Blocked => HitLevel::BLOCKED_HITSTOP_FRAMES,
                crate::collision::HitConnectionStatus::Invuln => 0,
            };

            self.trigger_hitstop(hitstop_frames);
        }

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
}
