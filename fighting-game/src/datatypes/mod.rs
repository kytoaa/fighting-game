mod bounding_box;
mod bounding_circle;
mod vector2;

#[macro_export]
macro_rules! vector {
    ($x:expr, $y:expr) => {{
        use crate::datatypes::Vector2;
        Vector2::new($x, $y)
    }};
}

pub trait BoundingShape<S> {
    fn intersects(&self, other: &S) -> bool;
}

pub type Point = Vector2;

pub use bounding_box::*;
pub use bounding_circle::*;
pub use vector2::*;

#[cfg(test)]
mod tests;
