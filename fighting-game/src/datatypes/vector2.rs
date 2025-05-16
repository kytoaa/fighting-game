use super::MoveTowards;

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
    pub fn distance(&self, other: impl Into<Vector2>) -> f32 {
        self.to_const(other.into()).magnitude()
    }
    pub fn rounded(&self) -> Self {
        Vector2::new(self.x.round(), self.y.round())
    }
    pub const fn x(&self, value: f32) -> Self {
        Vector2::new(value, self.y)
    }
    pub const fn y(&self, value: f32) -> Self {
        Vector2::new(self.x, value)
    }
    pub const fn flip_x(&self) -> Self {
        Vector2::new(-self.x, self.y)
    }
    pub const fn flip_y(&self) -> Self {
        Vector2::new(self.x, -self.y)
    }
}
impl Vector2 {
    pub const fn dot_const(&self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }
    pub const fn to_const(&self, other: Self) -> Self {
        (other).sub(*self)
    }
}
impl Vector2 {
    pub fn to(&self, other: impl Into<Vector2>) -> Self {
        self.to_const(other.into())
    }
    pub fn dot<T>(&self, rhs: impl Into<Vector2>) -> f32 {
        self.dot_const(rhs.into())
    }
}

impl Into<Vector2> for &Vector2 {
    fn into(self) -> Vector2 {
        *self
    }
}

impl<T, D> MoveTowards<T, D> for Vector2
where
    T: Into<Vector2>,
    D: Into<f32>,
{
    fn move_towards(self, value: T, delta: D) -> Self {
        let (value, delta) = (value.into(), delta.into());
        let difference = self.to(value);
        if difference.magnitude() < delta {
            value
        } else {
            self + (difference.normalized() * delta)
        }
    }
}

impl Vector2 {
    pub const fn from_floats((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
    pub const fn into_floats(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

impl From<(f32, f32)> for Vector2 {
    fn from((x, y): (f32, f32)) -> Self {
        Self::from_floats((x, y))
    }
}
impl Into<(f32, f32)> for Vector2 {
    fn into(self) -> (f32, f32) {
        self.into_floats()
    }
}

// ------------------------------------
// ------------ operations ------------
// ------------------------------------

impl Vector2 {
    pub const fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
    pub const fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
    pub const fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
    pub const fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
    pub const fn div(self, rhs: f32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
    pub const fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
    pub const fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
    pub const fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
    pub const fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl std::ops::Add for Vector2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}
impl std::ops::Sub for Vector2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self.sub(rhs)
    }
}
impl std::ops::Neg for Vector2 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self.neg()
    }
}
impl std::ops::Mul<f32> for Vector2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        self.mul(rhs)
    }
}
impl std::ops::Div<f32> for Vector2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        self.div(rhs)
    }
}

impl std::ops::AddAssign for Vector2 {
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign(rhs)
    }
}
impl std::ops::SubAssign for Vector2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_assign(rhs)
    }
}
impl std::ops::MulAssign<f32> for Vector2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.mul_assign(rhs)
    }
}
impl std::ops::DivAssign<f32> for Vector2 {
    fn div_assign(&mut self, rhs: f32) {
        self.div_assign(rhs)
    }
}
