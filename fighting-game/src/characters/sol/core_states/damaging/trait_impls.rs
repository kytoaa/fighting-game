use super::super::super::*;

impl Damageable for Sol<DeadState> {
    fn hit(self: Box<Self>, _: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        (self, HitConnectionStatus::Invuln)
    }
}

impl Damageable for Sol<WalkState<true>> {
    fn hit(mut self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        self.velocity.x = info.block_pushback;

        if let crate::collision::AttackType::Low = info.attack_type {
            return Sol::hit(self, info);
        }
        (
            Box::new(self.transition(
                BlockStun::<false> {
                    length: info.blockstun,
                },
                true,
            )),
            HitConnectionStatus::Blocked,
        )
    }
}
impl Damageable for Sol<Crouch<true>> {
    fn hit(mut self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        self.velocity.x = info.block_pushback;

        if let crate::collision::AttackType::High = info.attack_type {
            return Sol::hit(self, info);
        }
        (
            Box::new(self.transition(
                BlockStun::<true> {
                    length: info.blockstun + 1,
                },
                true,
            )),
            HitConnectionStatus::Blocked,
        )
    }
}
impl Damageable for Sol<Air<true>> {
    fn hit(mut self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        self.velocity.x = info.block_pushback * 2.0;

        if let crate::collision::AttackType::Low = info.attack_type {
            return Sol::hit(self, info);
        }
        (
            Box::new(self.transition(AirBlockStun { length: 15 }, true)),
            HitConnectionStatus::Blocked,
        )
    }
}
impl Damageable for Sol<AirBlockStun> {
    fn hit(mut self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        self.velocity.x = info.block_pushback * 2.0;

        if let crate::collision::AttackType::Low = info.attack_type {
            return Sol::hit(self, info);
        }
        (
            Box::new(self.transition(AirBlockStun { length: 15 }, true)),
            HitConnectionStatus::Blocked,
        )
    }
}
impl<const CROUCHING: bool> Damageable for Sol<BlockStun<CROUCHING>> {
    fn hit(mut self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        self.velocity.x = info.block_pushback;

        match CROUCHING {
            true => {
                if let crate::collision::AttackType::High = info.attack_type {
                    return Sol::hit(self, info);
                }
            }
            false => {
                if let crate::collision::AttackType::Low = info.attack_type {
                    return Sol::hit(self, info);
                }
            }
        }
        (
            Box::new(self.transition(
                BlockStun::<CROUCHING> {
                    length: info.blockstun + if CROUCHING { 1 } else { 0 },
                },
                true,
            )),
            HitConnectionStatus::Blocked,
        )
    }
}
impl Damageable for Sol<Backdash> {
    fn hit(self: Box<Self>, _: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        (self, HitConnectionStatus::Invuln)
    }
}
