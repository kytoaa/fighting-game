use super::{World, BORDER_X, DELTA};
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
            }

            if pos.x > BORDER_X {
                player.set_position(pos.x(BORDER_X));
                let vel = player.velocity();
                if player.should_wall_bounce() && vel.x.abs() > 10.0 {
                    player.set_velocity(Vector2::new(-vel.x * 0.5, vel.y.max(30.0)));
                }
            }
            if pos.x < -BORDER_X {
                player.set_position(pos.x(-BORDER_X));
                let vel = player.velocity();
                if player.should_wall_bounce() && vel.x.abs() > 10.0 {
                    player.set_velocity(Vector2::new(-vel.x * 0.5, vel.y.max(30.0)));
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
}
