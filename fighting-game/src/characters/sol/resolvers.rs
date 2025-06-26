use super::*;

const DOUBLE_JUMP_FORCE: f32 = 150.0;
const DOUBLE_JUMP_X_FORCE: f32 = 40.0;

impl<S> Sol<S>
where
    Sol<S>: Player + 'static,
{
    pub fn grounded_actionable_state(self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Player> {
        match self.grounded_cancel_options(input) {
            Ok(state) => state,
            Err(s) => s.walk_block_state(input),
        }
    }
    pub fn grounded_cancel_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        match self.grounded_attack_options(input) {
            Ok(state) => Ok(state),
            Err(s) => s.grounded_movement_cancel_options(input),
        }
    }
    pub fn grounded_movement_cancel_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        if has_dash_input(input, self.forward_dir(), false) {
            return Ok(Box::new(self.transition(RunStartState::new(), true)));
        }
        if has_dash_input(input, self.backward_dir(), false) {
            return Ok(Box::new(self.transition(Backdash, true)));
        }
        let move_dir = input.move_dir();
        if move_dir.y == Vector2::UP.y {
            return Ok(Box::new(self.transition(
                JumpSquat {
                    direction: move_dir.x,
                },
                true,
            )));
        }
        Err(self)
    }
    pub fn grounded_movement_cancel_options_from_attack<const FRAMES: usize>(
        self: Box<Sol<S>>,
        input: &InputHandler,
        dash_cancel_state: RunStartState<FRAMES>,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        if has_dash_input(input, self.forward_dir(), false) {
            return Ok(Box::new(self.transition(dash_cancel_state, true)));
        }
        if has_dash_input(input, self.backward_dir(), false) {
            return Ok(Box::new(self.transition(Backdash, true)));
        }
        let move_dir = input.move_dir();
        if move_dir.y == Vector2::UP.y {
            return Ok(Box::new(self.transition(
                JumpSquat {
                    direction: move_dir.x,
                },
                true,
            )));
        }
        Err(self)
    }
    pub fn grounded_attack_options(
        mut self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        match self.cancel_options_from_grounded_normal(input) {
            Ok(state) => return Ok(state),
            Err(s) => self = s,
        }
        match self.grounded_command_normal_cancel(input) {
            Ok(state) => return Ok(state),
            Err(s) => self = s,
        }

        // NOTE: ground throw forward
        if input.get_state(Button::Utility) == ButtonState::Down
            && input.has_action(&Action::Pressed(
                Button::Light,
                Some(InputDir::Dir6.dir(self.direction)),
            ))
        {
            return Ok(Box::new(self.transition(
                GroundThrow::<true> {
                    success: std::cell::Cell::new(true).into(),
                },
                true,
            )));
        }
        // NOTE: ground throw backward
        if input.get_state(Button::Utility) == ButtonState::Down
            && input.get_state(Button::Light) == ButtonState::Down
            && input.has_action(&Action::Pressed(
                Button::Light,
                Some(InputDir::Dir4.dir(self.direction)),
            ))
        {
            return Ok(Box::new(self.transition(
                GroundThrow::<false> {
                    success: std::cell::Cell::new(true).into(),
                },
                true,
            )));
        }

        if input.input_dir().is_down() {
            // NOTE: 2l
            if input.has_action(&Action::Pressed(Button::Light, None)) {
                return Ok(Box::new(self.transition(CrouchLight, true)));
            }

            // NOTE: 2m
            if input.has_action(&Action::Pressed(Button::Mid, None)) {
                return Ok(Box::new(self.transition(CrouchMid, true)));
            }

            // NOTE: 2h
            if input.has_action(&Action::Pressed(Button::Heavy, None)) {
                return Ok(Box::new(self.transition(CrouchHeavy, true)));
            }
        }

        // NOTE: c.m and f.m
        if input.has_action(&Action::Pressed(Button::Mid, None)) {
            if self.distance_from_other_player < CloseMid::MAX_DISTANCE {
                return Ok(Box::new(self.transition(CloseMid, true)));
            } else {
                return Ok(Box::new(self.transition(FarMid, true)));
            }
        }

        // NOTE: 5l
        if input.has_action(&Action::Pressed(Button::Light, None)) {
            return Ok(Box::new(self.transition(StandLight, true)));
        }

        // NOTE: 5h
        if input.has_action(&Action::Pressed(Button::Heavy, None)) {
            return Ok(Box::new(self.transition(StandHeavy, true)));
        }

        Err(self)
    }

    pub fn walk_block_state(self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Player> {
        match input.move_dir().into() {
            (0.0, 0.0) => Box::new(self.transition(Stand, true)),
            (d, -1.0) => {
                if d.round() == -self.dir() {
                    Box::new(self.transition(Crouch::<true>, true))
                } else {
                    Box::new(self.transition(Crouch::<false>, true))
                }
            }
            (d, 0.0) => {
                let reset_frame = self.frame >= WALK_ANIM_LENGTH * FRAMES_PER_WALK_ANIM_FRAME;
                if d.round() == -self.dir() {
                    Box::new(self.transition(WalkState::<true>, reset_frame))
                } else {
                    Box::new(self.transition(WalkState::<false>, reset_frame))
                }
            }
            (d, 1.0) => Box::new(self.transition(JumpSquat { direction: d }, true)),
            _ => unreachable!(),
        }
    }
    pub fn air_actionable_state(self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Player> {
        try_transition!(air_attack_options; self, input).air_movement_state(input)
    }
    pub fn air_attack_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        match self.air_special_cancel_options(input) {
            s @ Ok(_) => s,
            Err(s) => s.air_normal_options(input),
        }
    }
    pub fn air_special_cancel_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        if input.has_motion_input(
            &Motion::dp().direction(self.direction),
            &Action::Pressed(Button::Heavy, None),
        ) {
            return Ok(Box::new(self.transition(VolcanicViper, true)));
        }
        if input.has_motion_input(
            &Motion::quarter_circle().direction(self.direction),
            &Action::Pressed(Button::Mid, None),
        ) {
            return Ok(Box::new(self.transition(BanditRevolverAir, true)));
        }

        Err(self)
    }
    pub fn air_normal_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        if input.has_action(&Action::Pressed(Button::Light, None)) {
            return Ok(Box::new(self.transition(AirLight, true)));
        }
        if input.has_action(&Action::Pressed(Button::Mid, None)) {
            return Ok(Box::new(self.transition(AirMid, true)));
        }
        if input.has_action(&Action::Pressed(Button::Heavy, None)) {
            return Ok(Box::new(self.transition(AirHeavy, true)));
        }
        Err(self)
    }
    pub fn air_movement_cancel_options(
        mut self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        if !self.has_air_action {
            return Err(self);
        }
        if has_dash_input(input, self.forward_dir(), true) {
            self.has_air_action = false;
            return Ok(Box::new(self.transition(Airdash, true)));
        }
        if has_dash_input(input, self.backward_dir(), true) {
            self.has_air_action = false;
            return Ok(Box::new(self.transition(Backdash, true)));
        }
        let move_dir = input.move_dir();
        if input.has_action(&Action::JumpPress(InputDir::Dir7))
            || input.has_action(&Action::JumpPress(InputDir::Dir8))
            || input.has_action(&Action::JumpPress(InputDir::Dir9))
        {
            self.double_jump(move_dir.x);
            return Ok(self.air_actionable_state(input));
        }
        Err(self)
    }
    pub fn air_movement_state(mut self: Box<Sol<S>>, input: &InputHandler) -> Box<dyn Player> {
        let dir = input.move_dir();

        if self.grounded {
            // NOTE: resets frame as lands in case of walking, walking does not reset frame
            self.frame = 0;
            return self.grounded_actionable_state(input);
        }
        self = try_transition!(air_movement_cancel_options; self, input);

        if dir.x == -self.dir() {
            Box::new(self.transition(Air::<true>, false))
        } else {
            Box::new(self.transition(Air::<false>, false))
        }
    }

    pub fn double_jump(&mut self, dir: f32) {
        self.has_air_action = false;
        self.velocity = Vector2::new(
            dir * DOUBLE_JUMP_X_FORCE.max(self.velocity.x.abs()),
            DOUBLE_JUMP_FORCE,
        );
        self.frame = 0;
    }

    pub fn grounded_command_normal_cancel(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        if input.has_action(&Action::Pressed(
            Button::Heavy,
            Some(InputDir::Dir3.dir(self.direction)),
        )) {
            return Ok(Box::new(self.transition(Heavy3, true)));
        }

        Err(self)
    }

    pub fn cancel_options_from_grounded_normal(
        mut self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        self = match self.grounded_special_cancel_options(input) {
            Ok(state) => return Ok(state),
            Err(s) => s,
        };
        self.grounded_command_normal_cancel(input)
    }
    pub fn grounded_special_cancel_options(
        self: Box<Sol<S>>,
        input: &InputHandler,
    ) -> Result<Box<dyn Player>, Box<Sol<S>>> {
        // NOTE: fafnir
        if input.has_motion_input(
            &Motion::half_circle().direction(self.direction),
            &Action::Pressed(Button::Heavy, None),
        ) {
            return Ok(Box::new(self.transition(Fafnir, true)));
        }

        // NOTE: VOLCANIC VIPER!!!!
        if input.has_motion_input(
            &Motion::dp().direction(self.direction),
            &Action::Pressed(Button::Heavy, None),
        ) {
            return Ok(Box::new(self.transition(VolcanicViper, true)));
        }

        // NOTE: ground_viper
        if input.has_motion_input(
            &Motion::quarter_circle().direction(!self.direction),
            &Action::Pressed(Button::Mid, None),
        ) {
            return Ok(Box::new(self.transition(GroundViper::default(), true)));
        }

        // NOTE: wild throw
        if input.has_motion_input(
            &Motion::dp().direction(self.direction),
            &Action::Pressed(Button::Light, None),
        ) {
            return Ok(Box::new(self.transition(WildThrow::new(), true)));
        }

        // NOTE: bandit revolver
        if input.has_motion_input(
            &Motion::quarter_circle().direction(self.direction),
            &Action::Pressed(Button::Mid, None),
        ) {
            return Ok(Box::new(self.transition(BanditRevolverGrounded, true)));
        }

        // NOTE: gun flame
        if input.has_motion_input(
            &Motion::quarter_circle().direction(self.direction),
            &Action::Pressed(Button::Light, None),
        ) {
            return Ok(Box::new(self.transition(GunFlameStartup::real(), true)));
        }
        if input.has_motion_input(
            &Motion::quarter_circle().direction(!self.direction),
            &Action::Pressed(Button::Light, None),
        ) {
            return Ok(Box::new(self.transition(GunFlameStartup::feint(), true)));
        }

        Err(self)
    }
}
