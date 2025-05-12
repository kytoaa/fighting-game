use super::collision::{CollisionShape, HitType, Hitbox, Hurtbox};
use super::datatypes::{BoundingShape, Vector2};
use super::input::InputHandler;
use std::collections::HashSet;

const DELTA: f32 = 1.0 / 60.0;

const BORDER_X: f32 = 100.0;

pub struct World {
    players: [Option<Box<dyn crate::characters::Entity>>; 2],
    hurtboxes: Vec<Spawn<Hurtbox>>,
    hitboxes: Vec<Spawn<Hitbox>>,

    hitstop_frames_left: usize,
}
struct Spawn<T>(T, usize);

impl World {
    pub fn new(players: [Option<Box<dyn crate::characters::Entity>>; 2]) -> Self {
        Self {
            players,
            hurtboxes: vec![],
            hitboxes: vec![],

            hitstop_frames_left: 0,
        }
    }
}

impl World {
    pub fn update(&mut self, input_providers: &[InputHandler]) {
        self.update_hitbox_hurtboxes();

        if self.hitstop_frames_left > 0 {
            self.hitstop_frames_left -= 1;
            return;
        }

        {
            let player = self.players[0].take().unwrap();
            let input_provider = &input_providers[0];
            let player = player.update(self, &input_provider);
            _ = self.players[0].insert(player);
        }
        {
            let player = self.players[1].take().unwrap();
            let input_provider = &input_providers[1];
            let player = player.update(self, &input_provider);
            _ = self.players[1].insert(player);
        }

        self.move_players();
    }

    pub fn spawn_hurtbox(&mut self, hurtbox: Hurtbox, position: Vector2, frames: usize) {
        self.hurtboxes
            .push(Spawn(hurtbox.at_position(position), frames));
    }
    pub fn spawn_hitbox(&mut self, hitbox: Hitbox, position: Vector2, frames: usize) {
        self.hitboxes
            .push(Spawn(hitbox.at_position(position), frames));
    }
    fn update_hitbox_hurtboxes(&mut self) {
        let mut clanks = HashSet::new();

        for (i, hitbox) in self.hitboxes.iter().enumerate() {
            for (j, other) in self.hitboxes.iter().enumerate() {
                if i == j {
                    continue;
                }
                if hitbox.0.owner != other.0.owner
                    && hitbox.0.info.priority == other.0.info.priority
                    && hitbox.0.shape.overlaps(&other.0.shape)
                {
                    clanks.insert(i);
                    clanks.insert(j);
                }
            }
        }
        /*if clanks.len() > 0 {
            self.hitboxes = self
                .hitboxes
                .drain(0..)
                .enumerate()
                .filter_map(|(i, item)| {
                    if clanks.contains(&i) {
                        None
                    } else {
                        Some(item)
                    }
                })
                .collect();
        }*/

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
                    if let Some(collision) = acc.get(&hurtbox.0.owner) {
                        // the hitbox in the hashmap
                        let other = &self.hitboxes[collision.0];
                        if other.0.info.priority > hitbox.0.info.priority {
                            // if in hashmap has higher priority, return hashmap
                            return acc;
                        }
                    }
                    // otherwise insert other collision
                    acc.insert(hurtbox.0.owner, (*hitbox_index, *hurtbox_index));
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
            let hit_player = self
                .players
                .get_mut(hurtbox.0.owner)
                .unwrap()
                .take()
                .unwrap();

            let in_corner = hit_player.position().x.abs() >= BORDER_X - 10.0;
            let pushback = hitbox.0.info.grounded.hit_effect.x_vel();

            let (hit_state, hit_connection) = hit_player.hit(&hitbox.0.info);
            _ = self
                .players
                .get_mut(hurtbox.0.owner)
                .unwrap()
                .insert(hit_state);

            let hitstop_frames = match hit_connection {
                crate::collision::HitConnection::Hit => hitbox.0.info.hit_type.get_hitstop_frames(),
                crate::collision::HitConnection::Blocked => HitType::BLOCKED_HITSTOP_FRAMES,
                crate::collision::HitConnection::Invuln => 0,
            };

            let player = self
                .players
                .get_mut(hitbox.0.owner)
                .unwrap()
                .as_mut()
                .unwrap();
            if in_corner {
                if player.is_grounded() {
                    player.set_velocity(
                        player
                            .velocity()
                            .x(pushback.abs().max(50.0) * pushback.signum() * -1.5),
                    );
                } else {
                    const AIR_PUSHBACK: f32 = 20.0;
                    player.set_velocity(
                        player
                            .velocity()
                            .x(AIR_PUSHBACK * if player.get_direction() { -1.0 } else { 1.0 }),
                    );
                }
            }
            player.on_hit(hit_connection);

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

    fn is_grounded(&self, shape: &CollisionShape) -> bool {
        match shape {
            CollisionShape::Box(b) => b.min.y <= 0.01,
            CollisionShape::Circle(c) => c.position.y - c.radius <= 0.01,
        }
    }

    fn move_players(&mut self) {
        {
            let velocity_1 = self.players[0].as_ref().unwrap().velocity();
            self.players[0]
                .as_mut()
                .unwrap()
                .move_by(velocity_1 * DELTA);

            let velocity_2 = self.players[1].as_ref().unwrap().velocity();
            self.players[1]
                .as_mut()
                .unwrap()
                .move_by(velocity_2 * DELTA);
        }

        let (collider_1, collider_2) = {
            let player1 = self.players.get(0).unwrap().as_ref().unwrap();
            let player2 = self.players.get(1).unwrap().as_ref().unwrap();
            (
                player1.get_collider_world_space(),
                player2.get_collider_world_space(),
            )
        };

        if collider_1.intersects(&collider_2) {
            let mut velocity_1 = self.players[0].as_ref().unwrap().velocity();
            let mut velocity_2 = self.players[1].as_ref().unwrap().velocity();

            if collider_1.position().x.abs() >= BORDER_X {
                velocity_1.x = -velocity_2.x;
                velocity_2.x = 0.0;
            }
            if collider_2.position().x.abs() >= BORDER_X {
                velocity_2.x = -velocity_1.x;
                velocity_1.x = 0.0;
            }

            let overlap = collider_1.overlap(&collider_2);
            match (velocity_1.x.abs() > 1.0, velocity_2.x.abs() > 1.0) {
                (true, true) | (false, false) => {
                    let distance = self.players[1].as_ref().unwrap().position()
                        - self.players[0].as_ref().unwrap().position();
                    self.players[0]
                        .as_mut()
                        .unwrap()
                        .move_by(overlap.y(0.0) / 2.0 * -distance.x.signum());
                    self.players[1]
                        .as_mut()
                        .unwrap()
                        .move_by(overlap.y(0.0) / 2.0 * distance.x.signum());
                }
                (true, false) => {
                    let distance = self.players[1].as_ref().unwrap().position()
                        - self.players[0].as_ref().unwrap().position();
                    self.players[1]
                        .as_mut()
                        .unwrap()
                        .move_by(overlap.y(0.0) * distance.x.signum());
                    /*println!(
                        "adjusting player 2 by {:?}",
                        overlap.y(0.0) * distance.x.signum()
                    );*/
                }
                (false, true) => {
                    let distance = self.players[0].as_ref().unwrap().position()
                        - self.players[1].as_ref().unwrap().position();
                    self.players[0]
                        .as_mut()
                        .unwrap()
                        .move_by(overlap.y(0.0) * distance.x.signum());
                    /*println!(
                        "adjusting player 1 by {:?}",
                        overlap.y(0.0) * distance.x.signum()
                    );*/
                }
            }
        }
        for i in 0..2 {
            let player = self.players[i].as_mut().unwrap();
            let pos = player.position();
            if pos.y <= 0.0 {
                player.set_grounded(true);
                player.set_position(pos.y(0.0));
            } else {
                player.set_grounded(false);
            }
            let pos = player.position();
            if pos.x > BORDER_X {
                player.set_position(pos.x(BORDER_X));
                let vel = player.velocity();
                if player.should_wall_bounce() && vel.x.abs() > 10.0 {
                    player.set_velocity(vel.x(-vel.x * 0.5));
                }
            }
            if pos.x < -BORDER_X {
                player.set_position(pos.x(-BORDER_X));
                let vel = player.velocity();
                if player.should_wall_bounce() && vel.x.abs() > 10.0 {
                    player.set_velocity(vel.x(-vel.x * 0.5));
                }
            }
        }
        {
            let player1_position = self.players[0].as_ref().unwrap().position();
            let player2_position = self.players[1].as_ref().unwrap().position();

            let distance = player2_position.x - player1_position.x;
            let direction = distance > 0.0;

            self.players[0].as_mut().unwrap().set_direction(direction);
            self.players[1].as_mut().unwrap().set_direction(!direction);

            self.players[0]
                .as_mut()
                .unwrap()
                .set_distance(distance.abs());
            self.players[1]
                .as_mut()
                .unwrap()
                .set_distance(distance.abs());
        }
    }
    pub fn get_players(&self) -> Box<[&dyn crate::characters::Entity; 2]> {
        Box::new([
            self.players[0].as_ref().unwrap().as_ref(),
            self.players[1].as_ref().unwrap().as_ref(),
        ])
    }
    pub fn get_hurtboxes(&self) -> impl Iterator<Item = &Hurtbox> {
        self.hurtboxes.iter().map(|s| &s.0)
    }
    pub fn get_hitboxes(&self) -> impl Iterator<Item = &Hitbox> {
        self.hitboxes.iter().map(|s| &s.0)
    }

    pub fn trigger_hitstop(&mut self, frames: usize) {
        self.hitstop_frames_left = frames.max(self.hitstop_frames_left);
    }

    pub const fn in_hitstop(&self) -> bool {
        self.hitstop_frames_left > 0
    }
}
