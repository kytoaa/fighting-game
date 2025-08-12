use super::{
    crouching_normals::{CrouchHeavy, CrouchLight, CrouchMid},
    HasID, Player, RunStartState,
};
use crate::collision::{
    AttackData, BounceInfo, CollisionShape, HitData, HitDataExtension, HitEffect, HitLevel,
    KnockdownType, Proration,
};
use crate::datatypes::*;
use crate::input::{Action, Button, InputHandler};
use crate::world::World;

use super::{Sol, SolDamageableState, BASE_SPRITE_OFFSET, STANDING_HURTBOX};

const CLOSE_MID_STARTUP: usize = 7;
const CLOSE_MID_ACTIVE: usize = 6;
const CLOSE_MID_RECOVERY: usize = 10;
const CLOSE_MID_DAMAGE: u32 = 20;

pub struct CloseMid;
impl CloseMid {
    pub const MAX_DISTANCE: f32 = 16.0;
}
impl Player for Sol<CloseMid> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const RECOVERY_FRAME: usize = CLOSE_MID_STARTUP + CLOSE_MID_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + CLOSE_MID_RECOVERY;
        const DECEL: f32 = 8.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;
        self.drag(DECEL);

        world.spawn_hurtbox(
            self.create_hurtbox(CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );

        match self.frame as usize {
            0..CLOSE_MID_STARTUP => self,
            CLOSE_MID_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        CLOSE_MID_ACTIVE - (self.frame as usize - CLOSE_MID_STARTUP);
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(12.0, 18.0),
                            )),
                            AttackData {
                                attack: HitData::grounded(
                                    CLOSE_MID_DAMAGE,
                                    HitEffect::floating_crumple(
                                        Vector2::new(25.0 * self.dir(), 45.0),
                                        4.0,
                                        5,
                                    )
                                    .build(),
                                    14 + active_frames_extra_hitstun,
                                    Proration::percent(100),
                                    HitData::DEFAULT_LEVEL_4_SCALING,
                                )
                                .air_from_grounded(|g| g)
                                .counterhit_ground_from_ground_default()
                                .counterhit_air_from_air_default()
                                .block_pushback(40.0 * self.dir())
                                .wall_pushback_mult(3.0)
                                .add_extension(HitDataExtension::UsagesBeforeScaling(4))
                                .build(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Medium,
                                attack_id: "sol c.m".into(),
                            },
                        ),
                        self.position + Vector2::new(5.0 * self.dir(), 10.0),
                    );

                    world.spawn_hurtbox(
                        self.create_hurtbox(CollisionShape::new(BoundingBox::pos_size(
                            Vector2::ZERO,
                            Vector2::new(14.0, 20.0),
                        ))),
                        self.position + Vector2::new(5.0 * self.dir(), 10.0),
                    );
                } else {
                    self = try_transition!(cancel_options_from_grounded_normal; self, input);

                    match self
                        .grounded_movement_cancel_options_from_attack(input, RunStartState::<15>)
                    {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    if input.input_dir().is_down() {
                        if input.has_action(&Action::Pressed(Button::Light, None)) {
                            return Box::new(self.transition(CrouchLight, true));
                        }
                        if input.has_action(&Action::Pressed(Button::Mid, None)) {
                            return Box::new(self.transition(CrouchMid, true));
                        }
                        if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                            return Box::new(self.transition(CrouchHeavy, true));
                        }
                    }
                    if input.has_action(&Action::Pressed(Button::Light, None)) {
                        return Box::new(self.transition(StandLight, true));
                    }
                    if input.has_action(&Action::Pressed(Button::Mid, None)) {
                        return Box::new(self.transition(FarMid, true));
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(StandHeavy, true));
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                if self.has_hit {
                    self = try_transition!(cancel_options_from_grounded_normal; self, input);

                    match self
                        .grounded_movement_cancel_options_from_attack(input, RunStartState::<15>)
                    {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    if input.input_dir().is_down() {
                        if input.has_action(&Action::Pressed(Button::Light, None)) {
                            return Box::new(self.transition(CrouchLight, true));
                        }
                        if input.has_action(&Action::Pressed(Button::Mid, None)) {
                            return Box::new(self.transition(CrouchMid, true));
                        }
                        if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                            return Box::new(self.transition(CrouchHeavy, true));
                        }
                    }
                    if input.has_action(&Action::Pressed(Button::Light, None)) {
                        return Box::new(self.transition(StandLight, true));
                    }
                    if input.has_action(&Action::Pressed(Button::Mid, None)) {
                        return Box::new(self.transition(FarMid, true));
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(StandHeavy, true));
                    }
                }
                self
            }
            _ => self.grounded_actionable_state(input),
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const CLOSE_MID_STARTUP_SECOND_FRAME: usize = CLOSE_MID_STARTUP / 2;
        Some((
            match self.frame as usize {
                0..CLOSE_MID_STARTUP_SECOND_FRAME => "sol/normals/c.m/c.m1",
                CLOSE_MID_STARTUP_SECOND_FRAME..CLOSE_MID_STARTUP => "sol/normals/c.m/c.m2",
                _ => "sol/normals/c.m/c.m3",
            }
            .into(),
            BASE_SPRITE_OFFSET + Vector2::LEFT * 5.0 * self.dir(),
        ))
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for CloseMid {}

const FAR_MID_STARTUP: usize = 10;
const FAR_MID_ACTIVE: usize = 2;
const FAR_MID_RECOVERY: usize = 13;
const FAR_MID_DAMAGE: u32 = 14;

pub struct FarMid;
impl Player for Sol<FarMid> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const ADVANCE_START: usize = 5;
        const RECOVERY_FRAME: usize = FAR_MID_STARTUP + FAR_MID_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + FAR_MID_RECOVERY;
        const ADVANCE_VELOCITY: f32 = 140.0;
        const KARA_ADVANCE_VELOCITY: f32 = 70.0;
        const DECEL: f32 = ADVANCE_VELOCITY / FAR_MID_ACTIVE as f32;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;

        if self.frame < 10 {
            self = match self.grounded_special_cancel_options(input) {
                Ok(mut state) => {
                    println!("kara!");

                    state.set_velocity(
                        Vector2::RIGHT
                            * if state.get_direction() { 1.0 } else { -1.0 }
                            * KARA_ADVANCE_VELOCITY,
                    );
                    return state;
                }
                Err(s) => s,
            };
        }

        world.spawn_hurtbox(
            self.create_hurtbox(CollisionShape::new(STANDING_HURTBOX)),
            self.position,
        );

        match self.frame as usize {
            0..ADVANCE_START => self,
            ADVANCE_START..FAR_MID_STARTUP => {
                self.velocity = Vector2::RIGHT * ADVANCE_VELOCITY * self.dir();
                self
            }
            FAR_MID_STARTUP..RECOVERY_FRAME => {
                self.forward_drag(DECEL);
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        FAR_MID_ACTIVE - (self.frame as usize - FAR_MID_STARTUP);
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(16.0, 18.0),
                            )),
                            AttackData {
                                attack: HitData::grounded(
                                    FAR_MID_DAMAGE,
                                    HitEffect::pushback(
                                        35.0 * self.dir(),
                                        15 + active_frames_extra_hitstun,
                                    )
                                    .build(),
                                    18 + active_frames_extra_hitstun,
                                    Proration::percent(90),
                                    HitData::DEFAULT_LEVEL_3_SCALING,
                                )
                                .with_air(
                                    HitEffect::launcher(
                                        Vector2::new(40.0 * self.dir(), 10.0),
                                        KnockdownType::Soft,
                                    )
                                    .gravity(5.0)
                                    .build(),
                                )
                                .block_pushback(50.0 * self.dir())
                                .counterhit_ground_from_ground_default()
                                .counterhit_air_from_air_default()
                                .build(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Medium,
                                attack_id: "sol f.m".into(),
                            },
                        ),
                        self.position + Vector2::new(10.0 * self.dir(), 10.0),
                    );

                    world.spawn_hurtbox(
                        self.create_hurtbox(CollisionShape::new(BoundingBox::pos_size(
                            Vector2::ZERO,
                            Vector2::new(18.0, 20.0),
                        ))),
                        self.position + Vector2::new(10.0 * self.dir(), 10.0),
                    );
                } else {
                    self = try_transition!(cancel_options_from_grounded_normal; self, input);

                    match self
                        .grounded_movement_cancel_options_from_attack(input, RunStartState::<15>)
                    {
                        Ok(state) => return state,
                        Err(state) => self = state,
                    }
                    if input.input_dir().is_down() {
                        if input.has_action(&Action::Pressed(Button::Mid, None)) {
                            return Box::new(self.transition(CrouchMid, true));
                        }
                        if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                            return Box::new(self.transition(CrouchHeavy, true));
                        }
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(StandHeavy, true));
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                if self.has_hit {
                    self = try_transition!(cancel_options_from_grounded_normal; self, input);

                    match self.grounded_movement_cancel_options_from_attack(
                        input,
                        RunStartState::dash_cancel(),
                    ) {
                        Ok(state) => return state,
                        Err(state) => self = state,
                    }
                    if input.input_dir().is_down() {
                        if input.has_action(&Action::Pressed(Button::Mid, None)) {
                            return Box::new(self.transition(CrouchMid, true));
                        }
                        if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                            return Box::new(self.transition(CrouchHeavy, true));
                        }
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(StandHeavy, true));
                    }
                }
                self
            }
            _ => self.grounded_actionable_state(input),
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const FAR_MID_STARTUP_SECOND_FRAME: usize = 3;
        const FAR_MID_RECOVER_FRAME: usize = FAR_MID_STARTUP + FAR_MID_ACTIVE + 5;
        Some((
            match self.frame as usize {
                0..FAR_MID_STARTUP_SECOND_FRAME => "sol/normals/f.m/f.m1",
                FAR_MID_STARTUP_SECOND_FRAME..FAR_MID_STARTUP => "sol/normals/f.m/f.m2",
                FAR_MID_STARTUP..FAR_MID_RECOVER_FRAME => "sol/normals/f.m/f.m3",
                _ => "sol/normals/f.m/f.m4",
            }
            .into(),
            BASE_SPRITE_OFFSET + Vector2::RIGHT * 0.0 * self.dir(),
        ))
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for FarMid {}

const STAND_LIGHT_STARTUP: usize = 5;
const STAND_LIGHT_FIRST_ACTIVE: usize = 1;
const STAND_LIGHT_SECOND_ACTIVE: usize = 3;
const STAND_LIGHT_RECOVERY: usize = 15;
const STAND_LIGHT_FIRST_HIT_DAMAGE: u32 = 8;
const STAND_LIGHT_SECOND_HIT_DAMAGE: u32 = 14;

pub struct StandLight;
impl Player for Sol<StandLight> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const SECOND_ACTIVE_FRAME: usize = STAND_LIGHT_STARTUP + STAND_LIGHT_FIRST_ACTIVE;
        const RECOVERY_FRAME: usize = SECOND_ACTIVE_FRAME + STAND_LIGHT_SECOND_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + STAND_LIGHT_RECOVERY;
        const DECEL: f32 = 8.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;

        self.drag(DECEL);

        world.spawn_hurtbox(
            self.create_hurtbox(CollisionShape::new(STANDING_HURTBOX)),
            self.position + Vector2::RIGHT * 6.0 * self.dir(),
        );

        if self.frame as usize == STAND_LIGHT_STARTUP + 1 {
            // NOTE: allows cancelling the first hit
            if self.has_hit {
                match self.grounded_movement_cancel_options_from_attack(
                    input,
                    RunStartState::dash_cancel(),
                ) {
                    Ok(state) => return state,
                    Err(s) => self = s,
                }
                self = try_transition!(cancel_options_from_grounded_normal; self, input);

                if input.input_dir().is_down() {
                    if input.has_action(&Action::Pressed(Button::Mid, None)) {
                        return Box::new(self.transition(CrouchMid, true));
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(CrouchHeavy, true));
                    }
                }
                if input.has_action(&Action::Pressed(Button::Mid, None)) {
                    return Box::new(self.transition(FarMid, true));
                }
                if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                    return Box::new(self.transition(StandHeavy, true));
                }
            }
            self.has_hit = false;
        }

        match self.frame as usize {
            0..STAND_LIGHT_STARTUP => self,
            STAND_LIGHT_STARTUP => {
                world.spawn_hitbox(
                    self.create_hitbox(
                        CollisionShape::new(BoundingBox::pos_size(
                            Vector2::ZERO,
                            Vector2::new(10.0, 12.0),
                        )),
                        AttackData {
                            attack: HitData::level_1(
                                STAND_LIGHT_FIRST_HIT_DAMAGE,
                                Vector2::new(20.0 * self.dir(), 50.0),
                                0,
                            )
                            .counterhit_ground_from_ground_default()
                            .counterhit_air_from_air_default()
                            .build(),
                            priority: 10,
                            hitbox_id: 1,
                            hit_level: HitLevel::Light,
                            attack_id: "sol 5l".into(),
                        },
                    ),
                    self.position + Vector2::new(14.0 * self.dir(), 6.0),
                );

                world.spawn_hurtbox(
                    self.create_hurtbox(CollisionShape::new(BoundingBox::pos_size(
                        Vector2::ZERO,
                        Vector2::new(12.0, 14.0),
                    ))),
                    self.position + Vector2::new(14.0 * self.dir(), 6.0),
                );

                self
            }
            SECOND_ACTIVE_FRAME..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        SECOND_ACTIVE_FRAME - (self.frame as usize - STAND_LIGHT_SECOND_ACTIVE);

                    let attack_data: AttackData = AttackData {
                        attack: HitData::level_1(
                            STAND_LIGHT_SECOND_HIT_DAMAGE,
                            Vector2::new(30.0 * self.dir(), 45.0),
                            active_frames_extra_hitstun,
                        )
                        .with_air(
                            HitEffect::launcher(
                                Vector2::new(30.0 * self.dir(), 85.0),
                                KnockdownType::Soft,
                            )
                            .gravity(6.5)
                            .build(),
                        )
                        .counterhit_ground_from_ground_default()
                        .counterhit_air_from_air_default()
                        .build(),
                        priority: 10,
                        hitbox_id: 1,
                        hit_level: HitLevel::Light,
                        attack_id: "sol 5l2".into(),
                    };

                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(10.0, 12.0),
                            )),
                            attack_data.clone(),
                        ),
                        self.position
                            + Vector2::new(14.0 * self.dir(), 24.0)
                            + Vector2::RIGHT * 6.0 * self.dir(),
                    );
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(8.0, 10.0),
                            )),
                            attack_data.clone(),
                        ),
                        self.position
                            + Vector2::new(9.0 * self.dir(), 17.0)
                            + Vector2::RIGHT * 6.0 * self.dir(),
                    );
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(8.0, 10.0),
                            )),
                            attack_data,
                        ),
                        self.position
                            + Vector2::new(5.0 * self.dir(), 10.0)
                            + Vector2::RIGHT * 6.0 * self.dir(),
                    );

                    world.spawn_hurtbox(
                        self.create_hurtbox(CollisionShape::new(BoundingBox::pos_size(
                            Vector2::ZERO,
                            Vector2::new(10.0, 12.0),
                        ))),
                        self.position
                            + Vector2::new(9.0 * self.dir(), 17.0)
                            + Vector2::RIGHT * 6.0 * self.dir(),
                    );
                    world.spawn_hurtbox(
                        self.create_hurtbox(CollisionShape::new(BoundingBox::pos_size(
                            Vector2::ZERO,
                            Vector2::new(10.0, 12.0),
                        ))),
                        self.position
                            + Vector2::new(5.0 * self.dir(), 10.0)
                            + Vector2::RIGHT * 6.0 * self.dir(),
                    );
                } else {
                    self = try_transition!(cancel_options_from_grounded_normal; self, input);

                    match self.grounded_movement_cancel_options_from_attack(
                        input,
                        RunStartState::dash_cancel(),
                    ) {
                        Ok(state) => return state,
                        Err(s) => self = s,
                    }
                    if input.input_dir().is_down() {
                        if input.has_action(&Action::Pressed(Button::Mid, None)) {
                            return Box::new(self.transition(CrouchMid, true));
                        }
                        if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                            return Box::new(self.transition(CrouchHeavy, true));
                        }
                    }
                    if input.has_action(&Action::Pressed(Button::Mid, None)) {
                        return Box::new(self.transition(FarMid, true));
                    }
                    if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                        return Box::new(self.transition(StandHeavy, true));
                    }
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => self,
            _ => self.grounded_actionable_state(input),
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        const MAIN_ACTIVE_FRAME: usize = STAND_LIGHT_STARTUP + STAND_LIGHT_FIRST_ACTIVE;
        const RECOVERY_START: usize = MAIN_ACTIVE_FRAME + STAND_LIGHT_SECOND_ACTIVE + 7;
        const RECOVERY_END: usize = RECOVERY_START + STAND_LIGHT_RECOVERY - 7;
        Some(match self.frame as usize {
            0..STAND_LIGHT_STARTUP => (
                "sol/normals/5l/5l1".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 5.0 * self.dir(),
            ),
            STAND_LIGHT_STARTUP => (
                "sol/normals/5l/5l2".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 6.0 * self.dir(),
            ),
            MAIN_ACTIVE_FRAME..RECOVERY_START => (
                "sol/normals/5l/5l3".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 7.0 * self.dir(),
            ),
            RECOVERY_START..RECOVERY_END => (
                "sol/normals/5l/5l4".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 7.0 * self.dir(),
            ),
            _ => unreachable!(),
        })
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for StandLight {}

const STAND_HEAVY_STARTUP: usize = 10;
const STAND_HEAVY_ACTIVE: usize = 4;
const STAND_HEAVY_RECOVERY: usize = 20;
const STAND_HEAVY_DAMAGE: u32 = 25;

pub struct StandHeavy;
impl Player for Sol<StandHeavy> {
    fn update(mut self: Box<Self>, world: &mut World, input: &InputHandler) -> Box<dyn Player> {
        const RECOVERY_FRAME: usize = STAND_HEAVY_STARTUP + STAND_HEAVY_ACTIVE;
        const END_FRAME: usize = RECOVERY_FRAME + STAND_HEAVY_RECOVERY;
        const DECEL: f32 = 4.0;

        if self.frame == 0 {
            self.has_hit = false;
        }

        self.frame += 1;

        self.drag(DECEL);

        world.spawn_hurtbox(
            self.create_hurtbox(CollisionShape::new(STANDING_HURTBOX)),
            self.position + Vector2::RIGHT * 6.0 * self.dir(),
        );

        match self.frame as usize {
            0..STAND_HEAVY_STARTUP => self,
            STAND_HEAVY_STARTUP..RECOVERY_FRAME => {
                if !self.has_hit {
                    let active_frames_extra_hitstun =
                        STAND_HEAVY_ACTIVE - (self.frame as usize - STAND_HEAVY_STARTUP);
                    world.spawn_hitbox(
                        self.create_hitbox(
                            CollisionShape::new(BoundingBox::pos_size(
                                Vector2::ZERO,
                                Vector2::new(20.0, 15.0),
                            )),
                            AttackData {
                                attack: HitData::grounded(
                                    STAND_HEAVY_DAMAGE,
                                    HitEffect::pushback(
                                        55.0 * self.dir(),
                                        18 + active_frames_extra_hitstun,
                                    )
                                    .build(),
                                    15 + active_frames_extra_hitstun,
                                    Proration::percent(90),
                                    HitData::DEFAULT_LEVEL_4_SCALING,
                                )
                                .meter_gain(HitData::DEFAULT_LEVEL_4_METER_GAIN)
                                .with_air(
                                    HitEffect::launcher(
                                        Vector2::new(90.0 * self.dir(), 20.0),
                                        KnockdownType::Soft,
                                    )
                                    .momentum_scaling((0.25, 0.60))
                                    .ground_bounce(
                                        BounceInfo::new(Vector2::new(30.0 * self.dir(), 50.0))
                                            .gravity(3.0)
                                            .scaling_x(0.5)
                                            .use_x_vel(true),
                                    )
                                    .wall_bounce(
                                        BounceInfo::new(Vector2::new(20.0, 50.0)).gravity(4.0),
                                    )
                                    .build(),
                                )
                                .counterhit_ground_from_ground_default()
                                .counterhit_air_from_air_default()
                                .build(),
                                priority: 10,
                                hitbox_id: 1,
                                hit_level: HitLevel::Heavy,
                                attack_id: "sol 5h".into(),
                            },
                        ),
                        self.position + Vector2::new(18.0 * self.dir(), 13.0),
                    );
                    world.spawn_hurtbox(
                        self.create_hurtbox(CollisionShape::new(BoundingBox::pos_size(
                            Vector2::ZERO,
                            Vector2::new(12.0, 17.0),
                        ))),
                        self.position + Vector2::new(12.0 * self.dir(), 13.0),
                    );
                } else {
                    self = try_transition!(cancel_options_from_grounded_normal; self, input);
                }
                self
            }
            RECOVERY_FRAME..END_FRAME => {
                if self.has_hit {
                    self = try_transition!(cancel_options_from_grounded_normal; self, input);
                }
                self
            }
            _ => self.grounded_actionable_state(input),
        }
    }
    fn frame_name(&self) -> Option<(Box<str>, Vector2)> {
        Some(match self.frame {
            0..3 => (
                "sol/normals/5h/5h1".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 2.0 * self.dir(),
            ),
            3..7 => (
                "sol/normals/5h/5h2".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 2.0 * self.dir(),
            ),
            7..10 => (
                "sol/normals/5h/5h3".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * self.dir(),
            ),
            10..13 => (
                "sol/normals/5h/5h4".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 7.0 * self.dir(),
            ),
            13..16 => (
                "sol/normals/5h/5h5".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 6.0 * self.dir(),
            ),
            16..22 => (
                "sol/normals/5h/5h6".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 6.0 * self.dir(),
            ),
            22..28 => (
                "sol/normals/5h/5h7".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 6.0 * self.dir(),
            ),
            28.. => (
                "sol/normals/5h/5h8".into(),
                BASE_SPRITE_OFFSET + Vector2::RIGHT * 2.0 * self.dir(),
            ),
        })
    }
    fn actionable(&self) -> bool {
        false
    }
    fn counterhit(&self) -> bool {
        true
    }
}
impl SolDamageableState for StandHeavy {}
