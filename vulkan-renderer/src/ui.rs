use super::{renderer, Vector2};

pub fn progress_bar(
    percent: f32,
    max_len: f32,
    flipped: bool,
    height: f32,
    position: Vector2,
    color: (f32, f32, f32, f32),
) -> renderer::Primative {
    let width = percent * max_len / 100.0;
    renderer::Primative {
        pos: if flipped {
            position - Vector2::new(width, 0.0)
        } else {
            position
        },
        size: Vector2::new(width, height),
        color,
    }
}
