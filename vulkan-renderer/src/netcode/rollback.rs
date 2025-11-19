use std::collections::{vec_deque::Iter as VecDequeIter, VecDeque};

use fighting_game::input::{ButtonState, ButtonStates, InputDir, InputState};

pub struct InputHistory {
    player_inputs: VecDeque<FrameState>,
    remote_inputs: VecDeque<FrameState>,
    last_processed_frame: u32,
}

pub struct Rollback<'a> {
    frames: usize,
    player_inputs: std::iter::Rev<std::iter::Take<VecDequeIter<'a, FrameState>>>,
    remote_inputs: std::iter::Rev<std::iter::Take<VecDequeIter<'a, FrameState>>>,
}
impl Rollback<'_> {
    pub fn player_inputs(&self) -> impl Iterator<Item = PacketInputState> + use<'_> {
        self.player_inputs.clone().map(|i| i.input_state)
    }
    pub fn remote_inputs(&self) -> impl Iterator<Item = PacketInputState> + use<'_> {
        self.player_inputs.clone().map(|i| i.input_state)
    }
}

impl InputHistory {
    pub fn process_local_input(&mut self, local: FrameState, predicted: FrameState) {
        self.player_inputs.pop_back();
        self.player_inputs.push_front(local);

        self.remote_inputs.pop_back();
        self.player_inputs.push_front(predicted);
    }
    pub fn process_remote_input(&mut self, packet: GamePacket) -> Option<Rollback> {
        let remote_frame = packet.states[0].frame;

        if remote_frame < self.last_processed_frame {
            return None;
        }
        let skipped_remote_frames = remote_frame - self.last_processed_frame;
        let inputs_to_check = &packet.states[..(skipped_remote_frames as usize)];

        let last_processed_index = self.remote_inputs.front().unwrap().frame - remote_frame;

        let last_correct_index = self
            .remote_inputs
            .iter()
            .enumerate()
            .rev()
            .skip(self.remote_inputs.len() - last_processed_index as usize)
            .zip(inputs_to_check.iter())
            .skip_while(|((_, predicted), remote)| remote == predicted)
            .next()
            .map(|((i, _), _)| i + 1)
            .unwrap();

        for input in inputs_to_check {
            let index = (self.remote_inputs[0].frame - input.frame) as usize;
            self.remote_inputs[index] = input.clone();
        }

        self.last_processed_frame = remote_frame;

        Some(Rollback {
            frames: last_correct_index - 1,
            player_inputs: self.player_inputs.iter().take(last_correct_index).rev(),
            remote_inputs: self.remote_inputs.iter().take(last_correct_index).rev(),
        })
    }
}

pub struct GamePacket {
    states: [FrameState; 60],
}

#[repr(C)]
#[derive(Default, Clone, PartialEq)]
pub struct FrameState {
    frame: u32,
    input_state: PacketInputState,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketInputState(u32);
impl PacketInputState {
    pub fn from_input_state(state: &InputState) -> PacketInputState {
        let mut inner = state.dir as u8;
        fn button(button: ButtonState) -> bool {
            button == ButtonState::Down
        }
        inner |= (button(state.button_states.light) as u8) << 4;
        inner |= (button(state.button_states.mid) as u8) << 5;
        inner |= (button(state.button_states.heavy) as u8) << 6;
        inner |= (button(state.button_states.utility) as u8) << 7;

        PacketInputState(inner as u32)
    }
    pub fn to_input_state(self) -> Result<InputState, ()> {
        let dir = (self.0 & 0b1111) as u8;

        if dir == 0 || dir > 9 {
            return Err(());
        }

        fn button(state: bool) -> ButtonState {
            match state {
                true => ButtonState::Down,
                false => ButtonState::Up,
            }
        }

        Ok(InputState {
            dir: InputDir::from(dir),
            button_states: ButtonStates {
                light: button(self.0 & 0b00010000 == 1),
                mid: button(self.0 & 0b00100000 == 1),
                heavy: button(self.0 & 0b01000000 == 1),
                utility: button(self.0 & 0b10000000 == 1),
            },
        })
    }
}
impl Default for PacketInputState {
    fn default() -> Self {
        Self::from_input_state(&InputState::default())
    }
}
