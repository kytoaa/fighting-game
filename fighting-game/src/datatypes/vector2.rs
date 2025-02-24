#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const LEFT: Self = Self::new(-1.0, 0.0);
    pub const RIGHT: Self = Self::new(1.0, 0.0);
    pub const UP: Self = Self::new(0.0, 1.0);
    pub const DOWN: Self = Self::new(0.0, -1.0);
}
impl Vector2 {
    pub fn normalized(&self) -> Self {
        let m = self.magnitude();
        *self / m
    }
    pub fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
    pub const fn dot(&self, rhs: &Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }
    pub fn to(&self, other: &Self) -> Self {
        *other - *self
    }
    pub fn distance(&self, other: &Self) -> f32 {
        self.to(other).magnitude()
    }
    pub const fn x(&self, value: f32) -> Self {
        Vector2::new(value, self.y)
    }
    pub const fn y(&self, value: f32) -> Self {
        Vector2::new(self.x, value)
    }
}

impl From<(f32, f32)> for Vector2 {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}
impl Into<(f32, f32)> for Vector2 {
    fn into(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

// ------------------------------------
// ------------ operations ------------
// ------------------------------------

impl std::ops::Add for Vector2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl std::ops::Sub for Vector2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl std::ops::Neg for Vector2 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}
impl std::ops::Mul<f32> for Vector2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl std::ops::Div<f32> for Vector2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::ops::AddAssign for Vector2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl std::ops::SubAssign for Vector2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
impl std::ops::MulAssign<f32> for Vector2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}
impl std::ops::DivAssign<f32> for Vector2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
