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
