use super::{
    Damageable, Direction, Entity, Grounded, HasCollider, JumpSquat, OnHit, Position, Velocity,
    WALK_SPEED,
};
use crate::collision::{HitEffect, HitInfo, KnockdownType};
use crate::datatypes::{BoundingBox, Vector2};
use crate::input::{
    directions::{InputDir, Motion},
    Action, Button, InputHandler,
};
use crate::world::World;

use super::{Sol, SolDamageableState};

const GUNFLAME_STARTUP: usize = 18;

pub struct GunFlameStartup(pub usize);
impl Entity for Sol<GunFlameStartup> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.has_hit = false;
        self.state.0 += 1;
        if self.state.0 > GUNFLAME_STARTUP {
            Box::new(self.transition(GunFlame(0)))
        } else {
            self
        }
    }
}
impl SolDamageableState for GunFlameStartup {}

struct GunFlame(usize);
impl Entity for Sol<GunFlame> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        // TODO: spawn projectile
        todo!()
    }
}
impl SolDamageableState for GunFlame {}

const JUMP_MID_STARTUP: usize = 5;
const JUMP_MID_ACTIVE: usize = 5;
const JUMP_MID_ENDLAG: usize = 14;

pub struct JumpMidStartup(pub usize);
impl Entity for Sol<JumpMidStartup> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.has_hit = false;
        self.gravity();
        if self.grounded {
            return self.grounded_actionable_state(input);
        }
        self.state.0 += 1;
        if self.state.0 > JUMP_MID_STARTUP {
            Box::new(self.transition(JumpMid(0)))
        } else {
            self
        }
    }
}
impl SolDamageableState for JumpMidStartup {}

struct JumpMid(usize);
impl Entity for Sol<JumpMid> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.gravity();
        if self.grounded {
            return self.grounded_actionable_state(input);
        }
        self.state.0 += 1;
        world.spawn_hitbox(
            crate::collision::Hitbox {
                shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                    Vector2::ZERO,
                    Vector2::new(20.0, 10.0),
                )),
                owner: self.player,
                info: HitInfo {
                    damage: 30,
                    attack_type: crate::collision::AttackType::High,
                    hit_effect: HitEffect::Launcher(
                        Vector2::new(40.0 * self.dir(), 50.0),
                        KnockdownType::Soft,
                    ),
                    hitstun: 100,
                    priority: 5,
                    blockstun: 15,
                },
            },
            self.position + Vector2::new(10.0 * self.dir(), 0.0),
            1,
        );

        if self.state.0 > JUMP_MID_ACTIVE {
            Box::new(self.transition(JumpMidEndlag(0)))
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<Box<str>> {
        Some("sol_jump_mid".into())
    }
}
impl SolDamageableState for JumpMid {}

struct JumpMidEndlag(usize);
impl Entity for Sol<JumpMidEndlag> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.gravity();
        if self.grounded {
            return self.grounded_actionable_state(input);
        }
        self.state.0 += 1;
        if self.state.0 > JUMP_MID_ENDLAG {
            self.air_actionable_state(input)
        } else {
            self
        }
    }
}
impl SolDamageableState for JumpMidEndlag {}

const CROUCH_HEAVY_STARTUP: usize = 7;
const CROUCH_HEAVY_ACTIVE: usize = 3;
const CROUCH_HEAVY_ENDLAG_CANCEL_FRAMES: usize = 6;
const CROUCH_HEAVY_ENDLAG: usize = 17;

pub struct CrouchHeavyStartup(pub usize);
impl Entity for Sol<CrouchHeavyStartup> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.has_hit = false;
        self.state.0 += 1;
        if self.state.0 > CROUCH_HEAVY_STARTUP {
            Box::new(self.transition(CrouchHeavy(0)))
        } else {
            self
        }
    }
}
impl SolDamageableState for CrouchHeavyStartup {}

struct CrouchHeavy(usize);
impl Entity for Sol<CrouchHeavy> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.state.0 += 1;

        if self.has_hit {
            // NOTE: fafnir
            if input.has_motion_input(
                &Motion::half_circle().direction(self.direction),
                &Action::Pressed(Button::Heavy),
            ) {
                return Box::new(self.transition(FafnirStartup(0)));
            }

            // NOTE: jump cancel
            let move_dir = input.move_dir();
            if move_dir.y == Vector2::UP.y {
                self.velocity.x = move_dir.x * WALK_SPEED;
                return Box::new(self.transition(JumpSquat {
                    frame: 0,
                    direction: move_dir.x,
                }));
            }
        }

        world.spawn_hitbox(
            crate::collision::Hitbox {
                shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                    Vector2::ZERO,
                    Vector2::new(20.0, 10.0),
                )),
                owner: self.player,
                info: HitInfo {
                    damage: 30,
                    attack_type: crate::collision::AttackType::Mid,
                    hit_effect: HitEffect::Launcher(
                        Vector2::new(10.0 * self.dir(), 80.0),
                        KnockdownType::Soft,
                    ),
                    hitstun: 100,
                    priority: 5,
                    blockstun: 11,
                },
            },
            self.position + Vector2::new(10.0 * self.dir(), 0.0),
            1,
        );

        if self.state.0 > CROUCH_HEAVY_ACTIVE {
            Box::new(self.transition(CrouchHeavyEndlag(0)))
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<Box<str>> {
        Some("sol_jump_mid".into())
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for CrouchHeavy {}

struct CrouchHeavyEndlag(usize);
impl Entity for Sol<CrouchHeavyEndlag> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        if self.state.0 < CROUCH_HEAVY_ENDLAG_CANCEL_FRAMES && self.has_hit {
            // NOTE: fafnir
            if input.has_motion_input(
                &Motion::half_circle().direction(self.direction),
                &Action::Pressed(Button::Heavy),
            ) {
                return Box::new(self.transition(FafnirStartup(0)));
            }

            // NOTE: jump cancel
            let move_dir = input.move_dir();
            if move_dir.y == Vector2::UP.y {
                self.velocity.x = move_dir.x * WALK_SPEED;
                return Box::new(self.transition(JumpSquat {
                    frame: 0,
                    direction: move_dir.x,
                }));
            }
        }

        self.state.0 += 1;
        if self.state.0 > CROUCH_HEAVY_ENDLAG {
            self.grounded_actionable_state(input)
        } else {
            self
        }
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for CrouchHeavyEndlag {}

const FAFNIR_STARTUP: usize = 5;
const FAFNIR_DASH: usize = 7;
const FAFNIR_VELOCITY: f32 = 170.0;
const FAFNIR_ACTIVE: usize = 3;
const FAFNIR_STOP_VELOCITY: f32 = 20.0;
const FAFNIR_ENDLAG: usize = 10;

pub struct FafnirStartup(pub usize);
impl Entity for Sol<FafnirStartup> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.state.0 += 1;
        if self.state.0 > FAFNIR_STARTUP {
            Box::new(self.transition(FafnirDash(0)))
        } else {
            self
        }
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for FafnirStartup {}

struct FafnirDash(usize);
impl Entity for Sol<FafnirDash> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.state.0 += 1;
        self.velocity = Vector2::RIGHT * FAFNIR_VELOCITY * self.dir();
        if self.state.0 > FAFNIR_DASH {
            self.velocity = Vector2::RIGHT * FAFNIR_STOP_VELOCITY * self.dir();
            Box::new(self.transition(Fafnir(0)))
        } else {
            self
        }
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for FafnirDash {}

struct Fafnir(usize);
impl Entity for Sol<Fafnir> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.state.0 += 1;

        self.velocity = Vector2::RIGHT * FAFNIR_STOP_VELOCITY * self.dir();

        world.spawn_hitbox(
            crate::collision::Hitbox {
                shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                    Vector2::ZERO,
                    Vector2::new(20.0, 10.0),
                )),
                owner: self.player,
                info: HitInfo {
                    damage: 80,
                    attack_type: crate::collision::AttackType::Mid,
                    hit_effect: HitEffect::Launcher(
                        Vector2::new(80.0 * self.dir(), 10.0),
                        KnockdownType::Hard,
                    ),
                    hitstun: 100,
                    priority: 5,
                    blockstun: 30,
                },
            },
            self.position + Vector2::new(10.0 * self.dir(), 0.0),
            1,
        );

        if self.state.0 > FAFNIR_ACTIVE {
            Box::new(self.transition(FafnirEndlag(0)))
        } else {
            self
        }
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for Fafnir {}

struct FafnirEndlag(usize);
impl Entity for Sol<FafnirEndlag> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.state.0 += 1;

        self.velocity = Vector2::ZERO;

        if self.state.0 > FAFNIR_ENDLAG {
            self.grounded_actionable_state(input)
        } else {
            self
        }
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for FafnirEndlag {}
