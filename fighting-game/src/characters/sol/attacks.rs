use super::{
    Damageable, Direction, Entity, Grounded, HasCollider, JumpSquat, OnHit, Position, Velocity,
    WALK_SPEED,
};
use crate::collision::{CollisionShape, HitEffect, HitInfo, Hitbox, KnockdownType};
use crate::datatypes::{BoundingBox, Vector2};
use crate::input::{
    directions::{InputDir, Motion},
    Action, Button, InputHandler,
};
use crate::world::World;

use super::{Sol, SolDamageableState, BASE_SPRITE_OFFSET};

const GUNFLAME_STARTUP: usize = 15;
const GUNFLAME_DECEL: f32 = 0.9;

/// bool is feint
pub struct GunFlameStartup(pub usize, pub bool);
impl Entity for Sol<GunFlameStartup> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.has_hit = false;
        self.state.0 += 1;
        if self.velocity.x * self.dir() < 0.0 {
            self.velocity.x = 0.0;
        } else {
            self.velocity.x *= GUNFLAME_DECEL;
        }
        if self.state.0 > GUNFLAME_STARTUP {
            if self.state.1 {
                Box::new(self.transition(GunFlameFeint(0)))
            } else {
                Box::new(self.transition(GunFlame(0)))
            }
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        let mut path: String = "sol/gunflame/gunflame".into();
        path.push(match self.state.0 {
            0..6 => '1',
            ..11 => '2',
            _ => '3',
        });

        Some((path.into(), BASE_SPRITE_OFFSET))
    }
    fn actionable(&self) -> bool {
        false
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

const GUNFLAME_FEINT_HOLD_LENGTH: usize = 8;
pub struct GunFlameFeint(usize);
impl Entity for Sol<GunFlameFeint> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.state.0 += 1;

        if self.state.0 == 3 {
            world.spawn_hitbox(
                Hitbox {
                    shape: CollisionShape::Box(BoundingBox::pos_size(
                        Vector2::ZERO,
                        Vector2::new(4.0, 5.0),
                    )),
                    owner: self.player,
                    info: HitInfo {
                        damage: 10,
                        priority: 1,
                        hitstun: 15,
                        blockstun: 8,
                        hit_effect: HitEffect::Pushback(20.0 * self.dir()),
                        attack_type: crate::collision::AttackType::Mid,
                        hitbox_id: 1,
                        hit_type: crate::collision::HitType::Light,
                    },
                },
                self.position + Vector2::new(10.0 * self.dir(), -4.0),
                1,
            );
        }
        if self.state.0 > GUNFLAME_FEINT_HOLD_LENGTH {
            self.grounded_actionable_state(input)
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/gunflame/gunflame3".into(), BASE_SPRITE_OFFSET))
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for GunFlameFeint {}

const JUMP_MID_STARTUP: usize = 5;
const JUMP_MID_ACTIVE: usize = 5;
const JUMP_MID_RECOVERY: usize = 14;

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
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/jump_mid".into(), BASE_SPRITE_OFFSET))
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
        if !self.has_hit {
            world.spawn_hitbox(
                crate::collision::Hitbox {
                    shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                        Vector2::ZERO,
                        Vector2::new(20.0, 5.0),
                    )),
                    owner: self.player,
                    info: HitInfo {
                        damage: 30,
                        attack_type: crate::collision::AttackType::High,
                        hit_effect: HitEffect::Launcher(
                            Vector2::new(80.0 * self.dir(), 50.0),
                            KnockdownType::Soft,
                        ),
                        hitstun: 100,
                        priority: 5,
                        blockstun: 15,
                        hitbox_id: 1,
                        hit_type: crate::collision::HitType::Medium,
                    },
                },
                self.position + Vector2::new(10.0 * self.dir(), 4.0),
                1,
            );
        }

        if self.state.0 > JUMP_MID_ACTIVE {
            Box::new(self.transition(JumpMidRecovery(0)))
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/jump_mid".into(), BASE_SPRITE_OFFSET))
    }
}
impl SolDamageableState for JumpMid {}

struct JumpMidRecovery(usize);
impl Entity for Sol<JumpMidRecovery> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.gravity();
        if self.grounded {
            return self.grounded_actionable_state(input);
        }
        self.state.0 += 1;
        if self.state.0 > JUMP_MID_RECOVERY {
            self.air_actionable_state(input)
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/jump_mid".into(), BASE_SPRITE_OFFSET))
    }
}
impl SolDamageableState for JumpMidRecovery {}

const CROUCH_HEAVY_STARTUP: usize = 7;
const CROUCH_HEAVY_ACTIVE: usize = 3;
const CROUCH_HEAVY_RECOVERY_CANCEL_FRAMES: usize = 6;
const CROUCH_HEAVY_RECOVERY: usize = 17;

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

        if !self.has_hit {
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
                            Vector2::new(10.0 * self.dir(), 90.0),
                            KnockdownType::Soft,
                        ),
                        hitstun: 100,
                        priority: 5,
                        blockstun: 11,
                        hitbox_id: 1,
                        hit_type: crate::collision::HitType::Heavy,
                    },
                },
                self.position + Vector2::new(10.0 * self.dir(), 0.0),
                1,
            );
        }

        if self.state.0 > CROUCH_HEAVY_ACTIVE {
            Box::new(self.transition(CrouchHeavyRecovery(0)))
        } else {
            self
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(("sol/jump_mid".into(), BASE_SPRITE_OFFSET))
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for CrouchHeavy {}

struct CrouchHeavyRecovery(usize);
impl Entity for Sol<CrouchHeavyRecovery> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        if self.state.0 < CROUCH_HEAVY_RECOVERY_CANCEL_FRAMES && self.has_hit {
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
        if self.state.0 > CROUCH_HEAVY_RECOVERY {
            self.grounded_actionable_state(input)
        } else {
            self
        }
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for CrouchHeavyRecovery {}

const FAFNIR_STARTUP: usize = 12;
const FAFNIR_DASH: usize = 9;
const FAFNIR_VELOCITY: f32 = 200.0;
const FAFNIR_ACTIVE: usize = 3;
const FAFNIR_STOP_VELOCITY: f32 = 20.0;
const FAFNIR_RECOVERY: usize = 12;

pub struct FafnirStartup(pub usize);
impl Entity for Sol<FafnirStartup> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.velocity = Vector2::ZERO;
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

        if !self.has_hit {
            world.spawn_hitbox(
                crate::collision::Hitbox {
                    shape: crate::collision::CollisionShape::Box(BoundingBox::pos_size(
                        Vector2::ZERO,
                        Vector2::new(20.0, 5.0),
                    )),
                    owner: self.player,
                    info: HitInfo {
                        damage: 80,
                        attack_type: crate::collision::AttackType::Mid,
                        hit_effect: HitEffect::Launcher(
                            Vector2::new(100.0 * self.dir(), 10.0),
                            KnockdownType::Hard,
                        ),
                        hitstun: 100,
                        priority: 5,
                        blockstun: 20,
                        hitbox_id: 1,
                        hit_type: crate::collision::HitType::SuperHeavy,
                    },
                },
                self.position + Vector2::new(10.0 * self.dir(), 0.0),
                1,
            );
        }

        if self.state.0 > FAFNIR_ACTIVE {
            Box::new(self.transition(FafnirRecovery(0)))
        } else {
            self
        }
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for Fafnir {}

struct FafnirRecovery(usize);
impl Entity for Sol<FafnirRecovery> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Entity> {
        self.state.0 += 1;

        self.velocity = Vector2::ZERO;

        if self.state.0 > FAFNIR_RECOVERY {
            self.grounded_actionable_state(input)
        } else {
            self
        }
    }
    fn actionable(&self) -> bool {
        false
    }
}
impl SolDamageableState for FafnirRecovery {}
