use fighting_game::datatypes::Vector2;
use macros::my_proc_macro;

pub mod renderer;

#[derive(Clone, Copy)]
pub struct SpriteHandle(usize);

pub struct Image {
    size: (usize, usize),
    data: Box<[u8]>,
}

fn f() {
    my_proc_macro!();
}
