use super::{EntityID, World, BORDER_X, DELTA};
use crate::characters::{HasID, Position, Velocity};
use crate::datatypes::*;

impl World {
    pub(super) fn move_players(&mut self) {
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
            let mut moveable_1 = self.players[0].as_ref().unwrap().moveable();

            let mut velocity_2 = self.players[1].as_ref().unwrap().velocity();
            let mut moveable_2 = self.players[1].as_ref().unwrap().moveable();

            if collider_1.position().x.abs() >= BORDER_X {
                velocity_1.x = -velocity_2.x;
                velocity_2.x = 0.0;
                moveable_1 = false;
            }
            if collider_2.position().x.abs() >= BORDER_X {
                velocity_2.x = -velocity_1.x;
                velocity_1.x = 0.0;
                moveable_2 = false;
            }

            let overlap = collider_1.overlap(&collider_2);
            match (
                velocity_1.x.abs() > 1.0 && moveable_2,
                velocity_2.x.abs() > 1.0 && moveable_1,
            ) {
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

        // NOTE: moves the players
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

            if pos.x.abs() > BORDER_X {
                let side = pos.x.signum();
                player.set_position(pos.x(BORDER_X * side));
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
    pub fn player_hit_wall<P>(&mut self, player: &mut P)
    where
        P: Position + Velocity + HasID,
    {
        let other = (player.id().id() + 1) % 2;
        let player_vel = player.velocity();

        println!("hit wall with {:?} velocity", player.velocity());
        let other_player = self.players[other].as_mut().unwrap();
        if !other_player.actionable() {
            let other_player_vel = other_player.velocity();

            if other_player_vel.x * player_vel.x.signum() > -player_vel.x.abs()
                && other_player.moveable()
            {
                other_player.add_velocity(Vector2::new(
                    -player.velocity().x * 1.2,
                    other_player.velocity().y,
                ));
            }
        }

        player.set_velocity(player_vel.x(0.0));
    }

    /// if player colliding with wall return direction away from the wall
    pub fn position_colliding_with_wall(&self, position: Vector2) -> Option<f32> {
        match position.x {
            x if x >= BORDER_X - 1.0 => Some(-1.0),
            x if x <= -BORDER_X + 1.0 => Some(1.0),
            _ => None,
        }
    }
}
