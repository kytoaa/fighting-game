use crate::datatypes::Vector2;

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! motion {
        [ frames: $frames:expr; $($v:literal),+ ] => {
            Motion {
                directions: vec![$(InputDir::from($v)),+].into(),
                fail_directions: [].into(),
                frames: $frames
            }
        };
        ( frames: $frames:expr; [$($v:expr),+]; [$($f:expr),+] ) => {
            Motion {
                directions: vec![$(InputDir::from($v)),+].into(),
                fail_directions: vec![$(InputDir::from($f)),+].into(),
                frames: $frames
            }
        };
    }
}

pub struct Motion {
    pub directions: Box<[InputDir]>,
    pub fail_directions: Box<[InputDir]>,
    pub frames: usize,
}
impl Motion {
    pub fn reverse(self) -> Self {
        Self {
            directions: IntoIterator::into_iter(self.directions)
                .map(|d| d.invert())
                .collect(),
            fail_directions: IntoIterator::into_iter(self.fail_directions)
                .map(|d| d.invert())
                .collect(),
            frames: self.frames,
        }
    }
    pub fn direction(self, facing_right: bool) -> Self {
        if facing_right {
            self
        } else {
            self.reverse()
        }
    }
    pub fn quarter_circle() -> Self {
        motion![frames: 10; 2, 3, 6]
    }
    pub fn half_circle() -> Self {
        motion![frames: 18; 4, 1, 2, 3, 6]
    }
    pub fn dp() -> Self {
        motion![frames: 20; 6, 2, 3]
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputDir {
    Dir1 = 1,
    Dir2 = 2,
    Dir3 = 3,
    Dir4 = 4,
    Dir5 = 5,
    Dir6 = 6,
    Dir7 = 7,
    Dir8 = 8,
    Dir9 = 9,
}
impl InputDir {
    pub const fn from(v: u8) -> InputDir {
        match v {
            1 => InputDir::Dir1,
            2 => InputDir::Dir2,
            3 => InputDir::Dir3,
            4 => InputDir::Dir4,
            5 => InputDir::Dir5,
            6 => InputDir::Dir6,
            7 => InputDir::Dir7,
            8 => InputDir::Dir8,
            9 => InputDir::Dir9,
            _ => panic!("not a valid direction"),
        }
    }
    pub const fn invert(self) -> InputDir {
        match self {
            InputDir::Dir1 => InputDir::Dir3,
            InputDir::Dir2 => InputDir::Dir2,
            InputDir::Dir3 => InputDir::Dir1,

            InputDir::Dir4 => InputDir::Dir6,
            InputDir::Dir5 => InputDir::Dir5,
            InputDir::Dir6 => InputDir::Dir4,

            InputDir::Dir7 => InputDir::Dir9,
            InputDir::Dir8 => InputDir::Dir8,
            InputDir::Dir9 => InputDir::Dir7,
        }
    }
}
impl From<Vector2> for InputDir {
    fn from(value: Vector2) -> Self {
        match value.into() {
            (-1.0, -1.0) => InputDir::Dir1,
            (0.0, -1.0) => InputDir::Dir2,
            (1.0, -1.0) => InputDir::Dir3,
            (-1.0, 0.0) => InputDir::Dir4,
            (0.0, 0.0) => InputDir::Dir5,
            (1.0, 0.0) => InputDir::Dir6,
            (-1.0, 1.0) => InputDir::Dir7,
            (0.0, 1.0) => InputDir::Dir8,
            (1.0, 1.0) => InputDir::Dir9,
            _ => panic!("not a valid input direction"),
        }
    }
}
impl Into<Vector2> for InputDir {
    fn into(self) -> Vector2 {
        match self {
            InputDir::Dir1 => Vector2::new(-1.0, -1.0),
            InputDir::Dir2 => Vector2::new(0.0, -1.0),
            InputDir::Dir3 => Vector2::new(1.0, -1.0),
            InputDir::Dir4 => Vector2::new(-1.0, 0.0),
            InputDir::Dir5 => Vector2::new(0.0, 0.0),
            InputDir::Dir6 => Vector2::new(1.0, 0.0),
            InputDir::Dir7 => Vector2::new(-1.0, 1.0),
            InputDir::Dir8 => Vector2::new(0.0, 1.0),
            InputDir::Dir9 => Vector2::new(1.0, 1.0),
        }
    }
}
impl Into<Vector2> for &InputDir {
    fn into(self) -> Vector2 {
        (*self).into()
    }
}
impl Default for InputDir {
    fn default() -> Self {
        InputDir::Dir5
    }
}
