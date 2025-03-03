use fighting_game::collision::{CollisionShape, Hitbox, Hurtbox};
use fighting_game::datatypes::{BoundingBox, Vector2};
use fighting_game::initialization::{create_world, Character};
use fighting_game::input::{Button, ButtonState, InputDir, InputHandler};
use fighting_game::world::World;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;

use device_query::{DeviceQuery, DeviceState, Keycode};

fn main() {
    let mut stdout = std::io::stdout();

    let mut input = Rc::new(Mutex::new(InputHandler::new()));
    let mut world = create_world(
        (Character::Sol, input.clone()),
        (Character::Sol, Mutex::new(InputHandler::new()).into()),
    );

    let mut time = std::time::SystemTime::now();
    let delta = std::time::Duration::from_secs_f64(1.0 / 60.0);
    loop {
        print!("\x1B[2J\x1B[1;1H");
        let (keys, dir) = get_input();
        {
            let mut lock = input.lock().unwrap();
            lock.update(keys, dir);
        }
        world.update();
        print_world(&world, &mut stdout);

        std::thread::sleep(delta);
    }
}

fn get_input() -> (HashMap<Button, ButtonState>, InputDir) {
    let mut map = HashMap::new();
    let device_state = DeviceState::new();
    let keys = device_state.get_keys();

    map.insert(
        Button::Light,
        if keys.contains(&Keycode::J) {
            ButtonState::Down
        } else {
            ButtonState::Up
        },
    );
    map.insert(
        Button::Mid,
        if keys.contains(&Keycode::U) {
            ButtonState::Down
        } else {
            ButtonState::Up
        },
    );
    map.insert(
        Button::Heavy,
        if keys.contains(&Keycode::I) {
            ButtonState::Down
        } else {
            ButtonState::Up
        },
    );
    map.insert(
        Button::Utility,
        if keys.contains(&Keycode::K) {
            ButtonState::Down
        } else {
            ButtonState::Up
        },
    );

    (
        map,
        Vector2::new(
            0.0 + if keys.contains(&Keycode::G) { 1.0 } else { 0.0 }
                + if keys.contains(&Keycode::D) {
                    -1.0
                } else {
                    0.0
                },
            0.0 + if keys.contains(&Keycode::Space) {
                1.0
            } else {
                0.0
            } + if keys.contains(&Keycode::F) {
                -1.0
            } else {
                0.0
            },
        )
        .into(),
    )
}

const PRINT_WIDTH: usize = 220;
const PRINT_HEIGHT: usize = 50;
const ORIGIN: Vector2 = Vector2::new((PRINT_WIDTH / 2) as f32, (PRINT_HEIGHT / 4 * 3) as f32);

fn print_world<W: std::io::Write>(world: &World, buffer: &mut W) {
    let mut buf = String::with_capacity((PRINT_WIDTH + 1) * PRINT_HEIGHT);

    let players = world.get_players();

    for _ in 0..PRINT_HEIGHT {
        for _ in 0..PRINT_WIDTH {
            buf.push('.');
        }
        buf.push('\n');
    }

    for player in players.iter() {
        let collider = player.get_collider_world_space();
        draw_box_to_buf(&mut buf, &collider, '#');
    }
    for hitbox in world.get_hitboxes() {
        if let CollisionShape::Box(aabb) = &hitbox.shape {
            draw_box_to_buf(&mut buf, aabb, '%');
        }
    }

    write!(buffer, "{}", buf).unwrap();
}
fn draw_box_to_buf(buf: &mut String, aabb: &BoundingBox, c: char) {
    //println!("collider at {:?}, {:?}", collider.min, collider.max);
    let min = position_to_buf_pos(aabb.min);
    let max = position_to_buf_pos(aabb.max);
    //println!("buf space: {:?}, {:?}", min, max);
    if min.x >= PRINT_WIDTH as f32 || max.x < 0.0 {
        return;
    }
    if max.y >= PRINT_HEIGHT as f32 || min.y < 0.0 {
        return;
    }
    let start_x = min.x.clamp(0.0, PRINT_WIDTH as f32).round();
    let end_x = max.x.clamp(0.0, PRINT_WIDTH as f32).round();
    let start_y = max.y.clamp(0.0, PRINT_HEIGHT as f32).round();
    let end_y = min.y.clamp(0.0, PRINT_HEIGHT as f32).round();
    //println!("start x: {start_x}, start y: {start_y}, end x: {end_x}, end y: {end_y}");
    for y in (start_y as usize)..(end_y as usize) {
        buf.replace_range(
            buf_pos_index(Vector2::new(start_x, y as f32))
                ..buf_pos_index(Vector2::new(end_x, y as f32)),
            &std::iter::repeat(c)
                .take((end_x - start_x) as usize)
                .collect::<String>(),
        )
    }
}

const fn position_to_buf_pos(pos: Vector2) -> Vector2 {
    Vector2::new(ORIGIN.x + pos.x, ORIGIN.y - pos.y)
}
const fn buf_pos_index(pos: Vector2) -> usize {
    (PRINT_WIDTH + 1) * pos.y as usize + pos.x as usize
}

// x = (-4, 2)
// size = (25, 9)
// o = [13, 5]
// x = [9, 3]
// .........................
// .........................
// ........x................
// .........................
// ............o............
// .........................
// .........................
// .........................
// .........................
