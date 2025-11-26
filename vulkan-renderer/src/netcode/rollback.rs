use std::collections::{vec_deque::Iter as VecDequeIter, VecDeque};

use fighting_game::input::{ButtonState, ButtonStates, InputDir, InputState};

const INPUT_HISTORY_LENGTH: usize = 60;

#[derive(Debug)]
pub struct InputHistory {
    player_inputs: VecDeque<FrameState>,
    remote_inputs: VecDeque<FrameState>,
    last_processed_frame: u32,
}
impl Default for InputHistory {
    fn default() -> Self {
        Self {
            player_inputs: VecDeque::from_iter(std::iter::repeat_n(
                FrameState::default(),
                INPUT_HISTORY_LENGTH,
            )),
            remote_inputs: VecDeque::from_iter(std::iter::repeat_n(
                FrameState::default(),
                INPUT_HISTORY_LENGTH,
            )),
            last_processed_frame: 0,
        }
    }
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
        self.remote_inputs.clone().map(|i| i.input_state)
    }
    pub fn frames(&self) -> usize {
        self.frames
    }
    pub fn current_frame(&self) -> usize {
        self.player_inputs.clone().next().unwrap().frame as usize
    }
}

impl InputHistory {
    pub fn most_recent_local(&self) -> Result<InputState, ()> {
        self.player_inputs
            .front()
            .unwrap()
            .input_state
            .to_input_state()
    }
    pub fn local_frame(&self) -> u32 {
        self.player_inputs.front().unwrap().frame
    }
    pub fn most_recent_remote_real(&self) -> Result<InputState, ()> {
        self.remote_inputs
            .get((self.remote_inputs[0].frame - self.last_processed_frame) as usize)
            .ok_or(())?
            .input_state
            .to_input_state()
    }
    pub fn remote_inputs(&self) -> impl Iterator<Item = InputState> + use<'_> {
        self.remote_inputs
            .iter()
            .flat_map(|state| state.input_state.to_input_state())
    }
    pub fn most_recent_remote(&self) -> Result<InputState, ()> {
        self.remote_inputs
            .front()
            .unwrap()
            .input_state
            .to_input_state()
    }
    pub fn reset(&mut self) {
        self.player_inputs
            .iter_mut()
            .for_each(|i| *i = FrameState::default());
        self.remote_inputs
            .iter_mut()
            .for_each(|i| *i = FrameState::default());
    }
    pub fn local_as_packet(&self) -> GamePacket {
        let mut inputs = self.player_inputs.iter().cloned();
        GamePacket {
            states: std::array::from_fn(|_| inputs.next().unwrap()),
        }
    }
    pub fn process_local_input(&mut self, local: InputState, predicted: InputState) {
        let frame = self.player_inputs.front().unwrap().frame + 1;

        let local = FrameState {
            input_state: PacketInputState::from_input_state(&local),
            frame,
        };
        self.player_inputs.pop_back();
        self.player_inputs.push_front(local);

        let predicted = FrameState {
            input_state: PacketInputState::from_input_state(&predicted),
            frame,
        };
        self.remote_inputs.pop_back();
        self.remote_inputs.push_front(predicted);
    }
    pub fn process_remote_input(&mut self, packet: GamePacket) -> Option<Rollback> {
        let remote_frame = packet.states[0].frame;
        let predicted_remote_frame = self.remote_inputs.front().unwrap().frame;

        println!("remote: {remote_frame}, most recent predicted: {predicted_remote_frame}, last processed: {}", self.last_processed_frame);

        assert!(self
            .player_inputs
            .iter()
            .map(|input| input.frame)
            .take_while(|frame| *frame != 0)
            .zip(self.remote_inputs.iter().map(|input| input.frame))
            .zip((0..).map(|i| predicted_remote_frame - i))
            .all(|((player, remote), expected)| remote == expected && player == expected));

        // should have already processed all inputs
        if remote_frame < self.last_processed_frame {
            return None;
        }

        // gets first processable frame, remote may be ahead of local by more than latency
        let remote_frame = remote_frame.min(predicted_remote_frame);
        // trim data to processable
        let packet_states = &packet.states[(packet.states[0].frame - remote_frame) as usize..];

        // number of unprocessed frames, `remote_frame >= self.last_processed_frame`
        let skipped_remote_frames = remote_frame - self.last_processed_frame;
        // gather inputs that have not been processed yet
        let inputs_to_check = &packet_states[..(skipped_remote_frames as usize)];

        // index of `self.last_processed_frame`
        let last_processed_index = predicted_remote_frame - self.last_processed_frame;

        let last_correct_index = self
            .remote_inputs
            .iter()
            .enumerate() // groups input with vec index
            .take(last_processed_index as usize) // takes up to not including last processed
            .rev() // first item is now frame after last processed
            .zip(inputs_to_check.iter().rev()) // frames should be in sync
            .skip_while(|((_, predicted), remote)| {
                assert!(predicted.frame == remote.frame);
                remote == predicted
            })
            .next() // first incorrectly predicted frame
            .map(|((i, _), _)| i + 1); // iter reversed so +1 is prev index (last correct frame)

        for input in inputs_to_check {
            let index = (predicted_remote_frame - input.frame) as usize;
            assert_eq!(self.remote_inputs[index].frame, input.frame);

            // if `last_correct_index.is_none()` everything should be synced
            assert!(!(last_correct_index.is_none() && &self.remote_inputs[index] != input));

            self.remote_inputs[index] = input.clone();
        }

        // re-predicts all future inputs with new most recent input
        for input in self
            .remote_inputs
            .iter_mut()
            .take((predicted_remote_frame - remote_frame) as usize)
        {
            input.input_state = inputs_to_check[0].input_state;
        }

        if inputs_to_check.len() > 0 {
            assert_eq!(remote_frame, inputs_to_check[0].frame);
        }
        self.last_processed_frame = remote_frame;

        assert!(
            packet_states
                .iter()
                .zip(
                    self.remote_inputs
                        .iter()
                        .skip_while(|state| state.frame > remote_frame)
                )
                .all(|(remote, predicted)| remote == predicted),
            "different input histories\n\trmt: {:?}\n\n\tprd: {:?}\n\titc: {:?}",
            &packet_states,
            &self.remote_inputs,
            inputs_to_check,
        );

        // returns `None` if no rollback
        let last_correct_index = last_correct_index?;

        Some(Rollback {
            frames: last_correct_index,
            player_inputs: self.player_inputs.iter().take(last_correct_index).rev(),
            remote_inputs: self.remote_inputs.iter().take(last_correct_index).rev(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GamePacket {
    states: [FrameState; INPUT_HISTORY_LENGTH],
}
impl GamePacket {
    pub fn most_recent_frame(&self) -> u32 {
        self.states[0].frame
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, PartialEq)]
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
                light: button(self.0 & 0b00010000 != 0),
                mid: button(self.0 & 0b00100000 != 0),
                heavy: button(self.0 & 0b01000000 != 0),
                utility: button(self.0 & 0b10000000 != 0),
            },
        })
    }
}
impl Default for PacketInputState {
    fn default() -> Self {
        Self::from_input_state(&InputState::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! input_state {
        (@button u) => {
            ButtonState::Up
        };
        (@button d) => {
            ButtonState::Down
        };
        ($d:literal, $l:ident, $m:ident, $h:ident, $u:ident) => {
            InputState {
                dir: InputDir::from($d),
                button_states: ButtonStates {
                    light: input_state!(@button $l),
                    mid: input_state!(@button $m),
                    heavy: input_state!(@button $h),
                    utility: input_state!(@button $u),
                },
            }
        };
        () => {
            input_state!(5, u, u, u, u)
        };
    }

    #[test]
    fn rollback_test() {
        let mut input_history = InputHistory::default();

        input_history.process_local_input(
            input_state!(),
            input_history.most_recent_remote_real().unwrap(),
        );
        input_history.process_local_input(
            input_state!(6, u, d, u, u),
            input_history.most_recent_remote_real().unwrap(),
        );
        input_history.process_local_input(
            input_state!(5, u, d, u, u),
            input_history.most_recent_remote_real().unwrap(),
        );

        let rollback = input_history
            .process_remote_input(GamePacket {
                states: {
                    let mut iter = [(2, input_state!(6, d, u, u, u)), (1, input_state!())]
                        .into_iter()
                        .chain(std::iter::repeat((0, input_state!())))
                        .map(|(frame, input)| FrameState {
                            frame,
                            input_state: PacketInputState::from_input_state(&input),
                        });
                    std::array::from_fn(|_| iter.next().unwrap())
                },
            })
            .unwrap();

        assert_eq!(rollback.frames, 2);

        assert_eq!(
            rollback.player_inputs().collect::<Vec<_>>(),
            vec![
                PacketInputState::from_input_state(&input_state!(6, u, d, u, u)),
                PacketInputState::from_input_state(&input_state!(5, u, d, u, u))
            ]
        );
        assert_eq!(
            rollback.remote_inputs().collect::<Vec<_>>(),
            vec![
                PacketInputState::from_input_state(&input_state!(6, d, u, u, u)),
                PacketInputState::from_input_state(&input_state!(6, d, u, u, u)),
            ]
        );
    }
}
