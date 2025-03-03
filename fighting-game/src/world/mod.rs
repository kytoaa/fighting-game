use super::collision::{CollisionShape, Hitbox, Hurtbox};
use super::datatypes::{BoundingShape, Vector2};
use super::input::InputHandler;
use std::collections::HashSet;
use std::sync::Mutex;

#[cfg(test)]
mod tests;

const DELTA: f32 = 1.0 / 60.0;

pub struct World {
    input_providers: [std::rc::Rc<Mutex<InputHandler>>; 2],
    players: [Option<Box<dyn crate::characters::Entity>>; 2],
    hurtboxes: Vec<Spawn<Hurtbox>>,
    hitboxes: Vec<Spawn<Hitbox>>,
}
struct Spawn<T>(T, usize);

impl World {
    pub fn new(
        players: [Option<Box<dyn crate::characters::Entity>>; 2],
        input_providers: [std::rc::Rc<Mutex<InputHandler>>; 2],
    ) -> Self {
        Self {
            input_providers,
            players,
            hurtboxes: vec![],
            hitboxes: vec![],
        }
    }
}

impl World {
    pub fn update(&mut self) {
        self.update_hitbox_hurtboxes();

        {
            let player = self.players[0].take().unwrap();
            let input_provider = self.input_providers[0].clone();
            let player = player.update(self, &input_provider.as_ref().lock().unwrap());
            _ = self.players[0].insert(player);
        }
        {
            let player = self.players[1].take().unwrap();
            let input_provider = self.input_providers[1].clone();
            let player = player.update(self, &input_provider.as_ref().lock().unwrap());
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
    pub fn update_hitbox_hurtboxes(&mut self) {
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
        if clanks.len() > 0 {
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
        }

        let mut collisions = vec![];

        for (i, hitbox) in self.hitboxes.iter().enumerate() {
            for (j, other) in self.hurtboxes.iter().enumerate() {
                if hitbox.0.shape.overlaps(&other.0.shape) {
                    collisions.push((i, j));
                }
            }
        }

        for (hitbox_index, hurtbox_index) in collisions {
            let hitbox = &mut self.hitboxes[hitbox_index];
            let hurtbox = &mut self.hurtboxes[hurtbox_index];

            if hitbox.0.owner == hurtbox.0.owner {
                continue;
            }
            let hit_player = self
                .players
                .get_mut(hurtbox.0.owner)
                .unwrap()
                .take()
                .unwrap();
            _ = self
                .players
                .get_mut(hurtbox.0.owner)
                .unwrap()
                .insert(hit_player.hit(&hitbox.0.info));

            self.players
                .get_mut(hitbox.0.owner)
                .unwrap()
                .as_mut()
                .unwrap()
                .on_hit()
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

    pub fn is_grounded(&self, shape: &CollisionShape) -> bool {
        match shape {
            CollisionShape::Box(b) => b.min.y <= 0.01,
            CollisionShape::Circle(c) => c.position.y - c.radius <= 0.01,
        }
    }

    pub fn move_players(&mut self) {
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
            let velocity_1 = self.players[0].as_ref().unwrap().velocity();
            let velocity_2 = self.players[1].as_ref().unwrap().velocity();

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
        }
        {
            let player1_position = self.players[0].as_ref().unwrap().position();
            let player2_position = self.players[1].as_ref().unwrap().position();

            let direction = player2_position.x - player1_position.x > 0.0;

            self.players[0].as_mut().unwrap().set_direction(direction);
            self.players[1].as_mut().unwrap().set_direction(!direction);
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
}
