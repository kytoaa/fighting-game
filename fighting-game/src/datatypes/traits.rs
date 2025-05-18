pub trait MoveTowards<T, D> {
    fn move_towards(self, value: T, delta: D) -> Self;
}

impl MoveTowards<f32, f32> for f32 {
    fn move_towards(self, value: f32, delta: f32) -> Self {
        let diff: f32 = value - self;
        if diff.abs() < delta {
            value
        } else {
            self + diff.signum() * delta
        }
    }
}
impl<T, D> MoveTowards<T, D> for i32
where
    i32: From<T>,
    i32: From<D>,
{
    fn move_towards(self, value: T, delta: D) -> Self {
        let (value, delta) = (value.into(), delta.into());
        let diff: i32 = value - self;
        if diff.abs() < delta {
            value
        } else {
            self + diff.signum() * delta
        }
    }
}
