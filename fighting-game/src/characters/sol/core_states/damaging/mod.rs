use super::super::*;

pub use states::*;

mod states;
mod trait_impls;

impl<S> Sol<S> {
    pub fn hit(mut self: Box<Self>, info: OnHitHitData) -> (Box<dyn Player>, HitConnectionStatus) {
        (
            match info.hit_effect {
                HitEffect::Pushback { force, frames } => {
                    self.velocity.x = force;
                    Box::new(self.transition(
                        BasicHitstun {
                            length: frames,
                            wall_pushback_mult: info.wall_pushback_mult,
                        },
                        true,
                    ))
                }
                HitEffect::Launcher {
                    knockback,
                    gravity,
                    knockdown,
                    momentum_scaling,
                    ground_bounce,
                    wall_bounce,
                } => {
                    self.velocity = Vector2::new(
                        self.velocity.x * momentum_scaling.0 + knockback.x,
                        self.velocity.y * momentum_scaling.1 + knockback.y,
                    );
                    self.grounded = false;
                    Box::new(self.transition(
                        Tumble {
                            gravity,
                            knockdown,
                            ground_bounce,
                            wall_bounce,
                            wall_pushback_mult: info.wall_pushback_mult,
                        },
                        true,
                    ))
                }
                HitEffect::FloatingCrumple {
                    knockback,
                    gravity,
                    landing_frames,
                } => {
                    self.velocity = knockback;
                    Box::new(self.transition(
                        FloatingCrumple {
                            gravity,
                            landing_frames,
                            wall_pushback_mult: info.wall_pushback_mult,
                        },
                        true,
                    ))
                }
            },
            HitConnectionStatus::Hit,
        )
    }
}
