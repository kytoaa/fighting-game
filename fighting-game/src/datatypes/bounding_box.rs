use super::{BoundingCircle, BoundingShape, Point, Vector2};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    pub min: Vector2,
    pub max: Vector2,
}

impl BoundingBox {
    pub const fn new(min: Vector2, max: Vector2) -> Self {
        let (min_x, min_y) = (fmin(min.x, max.x), fmin(min.y, max.y));
        let (max_x, max_y) = (fmax(min.x, max.x), fmax(min.y, max.y));
        Self {
            min: Vector2::new(min_x, min_y),
            max: Vector2::new(max_x, max_y),
        }
    }
    pub const fn pos_size(pos: Vector2, size: Vector2) -> Self {
        Self::new(
            Vector2::new(pos.x - size.x / 2.0, pos.y - size.y / 2.0),
            Vector2::new(pos.x + size.x / 2.0, pos.y + size.y / 2.0),
        )
    }
    pub const fn with_size(size: Vector2) -> Self {
        Self::pos_size(Vector2::ZERO, size)
    }
    pub fn from_point_cloud<T>(points: T) -> Option<Self>
    where
        T: IntoIterator<Item = Vector2>,
    {
        let mut points = points.into_iter();
        let mut min = points.next()?;
        let mut max = min;
        for point in points {
            if point.x < min.x {
                min.x = point.x;
            } else if point.x > max.x {
                max.x = point.x;
            }
            if point.y < min.y {
                min.y = point.y;
            } else if point.y > max.y {
                max.y = point.y;
            }
        }
        Some(Self { min, max })
    }
}
impl BoundingBox {
    pub const fn size(&self) -> Vector2 {
        Vector2::new(self.max.x - self.min.x, self.max.y - self.min.y)
    }
    pub const fn position(&self) -> Vector2 {
        Vector2::new(
            (self.max.x + self.min.x) / 2.0,
            (self.max.y + self.min.y) / 2.0,
        )
    }
    /// distance from nearest edge to point
    pub const fn distance(&self, other: Point) -> Vector2 {
        Vector2::new(
            match (other.x < self.min.x, other.x > self.max.x) {
                (false, false) => 0.0,
                (true, false) => other.x - self.min.x,
                (false, true) => other.x - self.max.x,
                (true, true) => unreachable!(),
            },
            match (other.y < self.min.y, other.y > self.max.y) {
                (false, false) => 0.0,
                (true, false) => other.y - self.min.y,
                (false, true) => other.y - self.max.y,
                (true, true) => unreachable!(),
            },
        )
    }
    pub const fn transformed(self, distance: Vector2) -> Self {
        Self {
            min: self.min.add(distance),
            max: self.max.add(distance),
        }
    }
    pub fn transform_by(&mut self, distance: Vector2) {
        self.min += distance;
        self.max += distance;
    }
    pub fn overlap(&self, other: &BoundingBox) -> Vector2 {
        Vector2::new(
            0.0f32.max(self.max.x.min(other.max.x) - self.min.x.max(other.min.x)),
            0.0f32.max(self.max.y.min(other.max.y) - self.min.y.max(other.min.y)),
        )
    }
}

impl BoundingShape<Point> for BoundingBox {
    fn intersects(&self, other: &Vector2) -> bool {
        other.x < self.max.x && other.x > self.min.x && other.y < self.max.y && other.y > self.min.y
    }
}
impl BoundingShape<BoundingBox> for BoundingBox {
    fn intersects(&self, other: &BoundingBox) -> bool {
        let distance = other.position() - self.position();
        let size_sum = self.size() / 2.0 + other.size() / 2.0;
        if distance.x.abs() >= size_sum.x {
            return false;
        }
        if distance.y.abs() >= size_sum.y {
            return false;
        }
        true
    }
}
impl BoundingShape<BoundingCircle> for BoundingBox {
    fn intersects(&self, other: &BoundingCircle) -> bool {
        other.intersects(self)
    }
}

const fn fmin(a: f32, b: f32) -> f32 {
    if a > b {
        b
    } else {
        a
    }
}
const fn fmax(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}
