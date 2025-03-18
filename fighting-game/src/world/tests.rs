use super::*;
use crate::characters::{
    Damageable, Direction, Entity, Grounded, HasCollider, OnHit, Position, Velocity,
};
use crate::datatypes::{BoundingBox, Vector2};

#[test]
fn collision_move_1_test() {
    let mut world = World {
        input_providers: [
            std::sync::Arc::new(std::sync::Mutex::new(InputHandler::new())),
            std::sync::Arc::new(std::sync::Mutex::new(InputHandler::new())),
        ],
        players: [
            Some(Box::new(TestPlayer {
                position: Vector2::new(5.0, 0.0),
                velocity: Vector2::new(-60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true,
            })),
            Some(Box::new(TestPlayer {
                position: Vector2::ZERO,
                velocity: Vector2::ZERO,
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true,
            })),
        ],
        hitboxes: vec![],
        hurtboxes: vec![],

        hitstop_frames_left: 0,
    };

    world.move_players();
    assert_eq!(
        world
            .players
            .iter()
            .map(|v| v
                .as_ref()
                .unwrap()
                .as_any()
                .downcast_ref::<TestPlayer>()
                .unwrap())
            .collect::<Vec<_>>(),
        vec![
            &TestPlayer {
                position: Vector2::new(4.0, 0.0),
                velocity: Vector2::new(-60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
            &TestPlayer {
                position: Vector2::ZERO,
                velocity: Vector2::ZERO,
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
        ]
    );

    world.move_players();
    assert_eq!(
        world
            .players
            .iter()
            .map(|v| v
                .as_ref()
                .unwrap()
                .as_any()
                .downcast_ref::<TestPlayer>()
                .unwrap())
            .collect::<Vec<_>>(),
        vec![
            &TestPlayer {
                position: Vector2::new(3.0, 0.0),
                velocity: Vector2::new(-60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
            &TestPlayer {
                position: Vector2::new(-1.0, 0.0),
                velocity: Vector2::ZERO,
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
        ]
    );
}

#[test]
fn collision_move_2_test() {
    let mut world = World {
        input_providers: [
            std::rc::Rc::new(std::sync::Mutex::new(InputHandler::new())),
            std::rc::Rc::new(std::sync::Mutex::new(InputHandler::new())),
        ],
        players: [
            Some(Box::new(TestPlayer {
                position: Vector2::ZERO,
                velocity: Vector2::ZERO,
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true,
            })),
            Some(Box::new(TestPlayer {
                position: Vector2::new(5.0, 0.0),
                velocity: Vector2::new(-60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true,
            })),
        ],
        hitboxes: vec![],
        hurtboxes: vec![],
    };

    world.move_players();
    assert_eq!(
        world
            .players
            .iter()
            .map(|v| v
                .as_ref()
                .unwrap()
                .as_any()
                .downcast_ref::<TestPlayer>()
                .unwrap())
            .collect::<Vec<_>>(),
        vec![
            &TestPlayer {
                position: Vector2::ZERO,
                velocity: Vector2::ZERO,
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
            &TestPlayer {
                position: Vector2::new(4.0, 0.0),
                velocity: Vector2::new(-60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
        ]
    );

    world.move_players();
    assert_eq!(
        world
            .players
            .iter()
            .map(|v| v
                .as_ref()
                .unwrap()
                .as_any()
                .downcast_ref::<TestPlayer>()
                .unwrap())
            .collect::<Vec<_>>(),
        vec![
            &TestPlayer {
                position: Vector2::new(-1.0, 0.0),
                velocity: Vector2::ZERO,
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
            &TestPlayer {
                position: Vector2::new(3.0, 0.0),
                velocity: Vector2::new(-60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
        ]
    );
}

#[test]
fn move_both_test() {
    let mut world = World {
        input_providers: [
            std::rc::Rc::new(std::sync::Mutex::new(InputHandler::new())),
            std::rc::Rc::new(std::sync::Mutex::new(InputHandler::new())),
        ],
        players: [
            Some(Box::new(TestPlayer {
                position: Vector2::ZERO,
                velocity: Vector2::new(60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true,
            })),
            Some(Box::new(TestPlayer {
                position: Vector2::new(5.0, 0.0),
                velocity: Vector2::new(-60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true,
            })),
        ],
        hitboxes: vec![],
        hurtboxes: vec![],
    };

    world.move_players();
    assert_eq!(
        world
            .players
            .iter()
            .map(|v| v
                .as_ref()
                .unwrap()
                .as_any()
                .downcast_ref::<TestPlayer>()
                .unwrap())
            .collect::<Vec<_>>(),
        vec![
            &TestPlayer {
                position: Vector2::new(0.5, 0.0),
                velocity: Vector2::new(60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
            &TestPlayer {
                position: Vector2::new(4.5, 0.0),
                velocity: Vector2::new(-60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
        ]
    );
}

#[test]
fn move_both_same_direction_test() {
    let mut world = World {
        input_providers: [
            std::rc::Rc::new(std::sync::Mutex::new(InputHandler::new())),
            std::rc::Rc::new(std::sync::Mutex::new(InputHandler::new())),
        ],
        players: [
            Some(Box::new(TestPlayer {
                position: Vector2::ZERO,
                velocity: Vector2::new(120.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true,
            })),
            Some(Box::new(TestPlayer {
                position: Vector2::new(5.0, 0.0),
                velocity: Vector2::new(60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true,
            })),
        ],
        hitboxes: vec![],
        hurtboxes: vec![],
    };

    world.move_players();
    assert_eq!(
        world
            .players
            .iter()
            .map(|v| v
                .as_ref()
                .unwrap()
                .as_any()
                .downcast_ref::<TestPlayer>()
                .unwrap())
            .collect::<Vec<_>>(),
        vec![
            &TestPlayer {
                position: Vector2::new(2.0, 0.0),
                velocity: Vector2::new(120.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
            &TestPlayer {
                position: Vector2::new(6.0, 0.0),
                velocity: Vector2::new(60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
        ]
    );

    world.move_players();
    assert_eq!(
        world
            .players
            .iter()
            .map(|v| v
                .as_ref()
                .unwrap()
                .as_any()
                .downcast_ref::<TestPlayer>()
                .unwrap())
            .collect::<Vec<_>>(),
        vec![
            &TestPlayer {
                position: Vector2::new(3.5, 0.0),
                velocity: Vector2::new(120.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
            &TestPlayer {
                position: Vector2::new(7.5, 0.0),
                velocity: Vector2::new(60.0, 0.0),
                collider: BoundingBox::pos_size(Vector2::ZERO, Vector2::new(4.0, 6.0)),
                grounded: true
            },
        ]
    );
}

#[derive(Debug, PartialEq, Clone)]
struct TestPlayer {
    position: Vector2,
    velocity: Vector2,
    collider: BoundingBox,
    grounded: bool,
}
impl Entity for TestPlayer {
    fn update(self: Box<Self>, _: &mut World, _: &crate::input::InputHandler) -> Box<dyn Entity> {
        self
    }
}
impl Position for TestPlayer {
    fn position(&self) -> Vector2 {
        self.position
    }
    fn move_by(&mut self, distance: Vector2) {
        self.position += distance;
    }
    fn set_position(&mut self, position: Vector2) {
        self.position = position;
    }
}
impl Velocity for TestPlayer {
    fn velocity(&self) -> Vector2 {
        self.velocity
    }
    fn add_velocity(&mut self, velocity: Vector2) {
        self.velocity += velocity;
    }
    fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
    }
}
impl Damageable for TestPlayer {
    fn hit(self: Box<Self>, _: &crate::collision::HitInfo) -> Box<dyn Entity> {
        self
    }
}
impl OnHit for TestPlayer {
    fn on_hit(&mut self) {}
}
impl HasCollider for TestPlayer {
    fn get_collider(&self) -> &BoundingBox {
        &self.collider
    }
}
impl Grounded for TestPlayer {
    fn is_grounded(&self) -> bool {
        self.grounded
    }
    fn set_grounded(&mut self, grounded: bool) {
        self.grounded = grounded;
    }
}
impl Direction for TestPlayer {
    fn set_direction(&mut self, _: bool) {}
    fn get_direction(&self) -> bool {
        false
    }
}
