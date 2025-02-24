use super::{BoundingBox, BoundingShape, Point, Vector2};

#[derive(Debug, Clone, PartialEq)]
pub struct BoundingCircle {
    pub position: Vector2,
    pub radius: f32,
}

impl BoundingCircle {
    pub const fn new(position: Vector2, radius: f32) -> Self {
        BoundingCircle { position, radius }
    }
}
impl BoundingCircle {
    pub fn transformed(self, distance: Vector2) -> Self {
        Self {
            position: self.position + distance,
            ..self
        }
    }
    pub fn transform_by(&mut self, distance: Vector2) {
        self.position += distance;
    }
}

impl BoundingShape<Point> for BoundingCircle {
    fn intersects(&self, other: &Point) -> bool {
        self.position.distance(&other) < self.radius
    }
}
impl BoundingShape<BoundingCircle> for BoundingCircle {
    fn intersects(&self, other: &BoundingCircle) -> bool {
        self.position.distance(&other.position) < self.radius + other.radius
    }
}
impl BoundingShape<BoundingBox> for BoundingCircle {
    fn intersects(&self, other: &BoundingBox) -> bool {
        other.intersects(&self.position) || other.distance(self.position).magnitude() < self.radius
    }
}
