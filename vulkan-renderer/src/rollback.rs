use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};

use fighting_game::input::{ButtonState, ButtonStates, InputDir, InputState};

pub struct FrameData {
    frame_since_start: u32,
    input_state: InputState,
}

pub enum Packet {
    Connection,
    StartGame,
    Quit,
    FrameData(FrameData),
}
impl Packet {
    fn as_str(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Self::Connection => "CONNECTED".into(),
            Self::StartGame => "STARTGAME".into(),
            Self::Quit => "QUIT".into(),
            Self::FrameData(FrameData {
                frame_since_start,
                input_state:
                    InputState {
                        dir,
                        button_states:
                            ButtonStates {
                                light,
                                mid,
                                heavy,
                                utility,
                            },
                    },
            }) => {
                fn state(state: ButtonState) -> &'static str {
                    match state {
                        ButtonState::Up => "UP",
                        ButtonState::Down => "DOWN",
                    }
                }

                format!(
                    "FRAMEDATA\nframe:{frame_since_start}\ninputstate:{},{},{},{},{}",
                    *dir as u8,
                    state(*light),
                    state(*mid),
                    state(*heavy),
                    state(*utility)
                )
                .into()
            }
        }
    }
    fn from_str(&self, s: &str) -> Result<Self, ()> {
        let packet_type = s.lines().next().ok_or(())?.trim();
        match packet_type {
            "CONNECTED" => Ok(Self::Connection),
            "STARTGAME" => Ok(Self::StartGame),
            "QUIT" => Ok(Self::Quit),
            "FRAMEDATA" => {
                fn frame_data(s: &str) -> Option<FrameData> {
                    let mut lines = s.lines().skip(1);
                    let frame = lines.next()?.trim();
                    let input_state = lines.next()?.trim();

                    let frame: u32 = frame.strip_prefix("frame:")?.parse().ok()?;

                    let input_state = input_state.strip_prefix("inputstate:")?.trim();
                    let mut input_data = input_state.split(",");
                    let dir = InputDir::from(input_data.next()?.parse().ok()?);

                    fn button(s: &str) -> Option<ButtonState> {
                        match s.trim() {
                            "UP" => Some(ButtonState::Up),
                            "DOWN" => Some(ButtonState::Down),
                            _ => None,
                        }
                    }

                    let light = button(input_data.next()?)?;
                    let mid = button(input_data.next()?)?;
                    let heavy = button(input_data.next()?)?;
                    let utility = button(input_data.next()?)?;

                    Some(FrameData {
                        frame_since_start: frame,
                        input_state: InputState {
                            dir,
                            button_states: ButtonStates {
                                light,
                                mid,
                                heavy,
                                utility,
                            },
                        },
                    })
                }
                frame_data(s).map(Self::FrameData).ok_or(())
            }
            _ => Err(()),
        }
    }
}

pub struct Host {}

pub fn host(port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port))?;

    let mut stream = None;
    for connection in listener.incoming() {
        let connection = connection?;

        if let Some(s) = handle_connection(connection) {
            _ = stream.insert(s);
            break;
        }
    }
    let mut stream = stream.unwrap();

    _ = stream.write(Packet::Connection.as_str().as_bytes())?;

    todo!();
}

fn handle_connection(stream: TcpStream) -> Option<TcpStream> {
    let buf_reader = BufReader::new(&stream);
    let "CONNECTED" = buf_reader.lines().next()?.ok()?.trim() else {
        return None;
    };

    Some(stream)
}
