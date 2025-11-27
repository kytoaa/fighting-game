use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};

mod connection;
mod rollback;

pub use rollback::{GamePacket, InputHistory, Rollback};

#[derive(Debug, PartialEq)]
pub struct TimeoutError;

#[derive(Debug)]
pub enum ConnectionError {
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    Timeout(TimeoutError),
    InvalidAddress,
    NotPacket,
}

pub fn ask_connection_type() -> Result<ConnectionType, ConnectionError> {
    let mut buf = String::new();
    loop {
        print!("[0] offline (default)\n[1] host\n[2] join\n> ");
        _ = std::io::stdout().flush();

        std::io::stdin()
            .read_line(&mut buf)
            .expect("failed to get input");

        return match buf.trim() {
            "0" | "" => Ok(ConnectionType::Offline),
            "1" => {
                print!("\n[HOST] -- enter address\n> ");
                _ = std::io::stdout().flush();
                buf.clear();

                std::io::stdin()
                    .read_line(&mut buf)
                    .map_err(ConnectionError::Io)?;

                let Some(addr) = buf
                    .trim()
                    .to_socket_addrs()
                    .map_err(|_| ConnectionError::InvalidAddress)
                    .map(|mut a| a.next())?
                else {
                    continue;
                };

                println!("[HOST] -- hosting at {}", addr.to_string());

                Ok(ConnectionType::Host(addr))
            }
            "2" => {
                print!("\n[JOIN] -- enter address\n> ");
                _ = std::io::stdout().flush();
                buf.clear();

                std::io::stdin()
                    .read_line(&mut buf)
                    .map_err(ConnectionError::Io)?;

                let Some(addr) = buf
                    .trim()
                    .to_socket_addrs()
                    .map_err(|_| ConnectionError::InvalidAddress)
                    .map(|mut a| a.next())?
                else {
                    continue;
                };

                println!("[JOIN] -- trying to join {}", addr);

                Ok(ConnectionType::Client(addr))
            }
            _ => {
                buf.clear();
                continue;
            }
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    Host(SocketAddr),
    Client(SocketAddr),
    Offline,
}

pub const MAX_ROLLBACK_FRAMES: usize = 20;

fn create_game_connection(addr: &ConnectionAddr) -> Result<UdpSocket, ConnectionError> {
    let connection = UdpSocket::bind(addr.local).map_err(ConnectionError::Io)?;

    connection
        .set_nonblocking(true)
        .map_err(ConnectionError::Io)?;

    connection
        .connect(addr.remote)
        .map_err(ConnectionError::Io)?;

    Ok(connection)
}

#[derive(Clone)]
pub struct ConnectionAddr {
    local: SocketAddr,
    remote: SocketAddr,
}
impl ConnectionAddr {
    pub fn local(&self) -> SocketAddr {
        self.local.clone()
    }
    pub fn remote(&self) -> SocketAddr {
        self.remote.clone()
    }
}
enum CharacterSelectConnectionData {
    Host {
        has_received_start_request: bool,
        should_start: bool,
    },
    Client {
        has_requested_start: bool,
        should_start: bool,
    },
}

pub struct CharacterSelectConnection {
    addr: ConnectionAddr,
    connection: TcpStream,
    connection_data: CharacterSelectConnectionData,
}

impl CharacterSelectConnection {
    pub fn host(addr: SocketAddr) -> Result<CharacterSelectConnection, ConnectionError> {
        let connection::Connection {
            stream,
            addr: remote,
        } = connection::host(&addr)?;

        let addr = ConnectionAddr {
            local: addr,
            remote,
        };

        println!("[HOST] -- connected to {}", addr.remote);

        Ok(CharacterSelectConnection {
            addr,
            connection: stream,
            connection_data: CharacterSelectConnectionData::Host {
                has_received_start_request: false,
                should_start: false,
            },
        })
    }
    pub fn join(remote: SocketAddr) -> Result<CharacterSelectConnection, ConnectionError> {
        let connection::Connection { stream, addr } =
            connection::connect_to(&remote).expect("connection error");

        let addr = ConnectionAddr {
            local: addr,
            remote,
        };

        println!("[JOIN] -- connected to {}", addr.remote);

        Ok(CharacterSelectConnection {
            addr,
            connection: stream,
            connection_data: CharacterSelectConnectionData::Client {
                has_requested_start: false,
                should_start: false,
            },
        })
    }
    pub fn has_start_been_requested(&mut self) -> bool {
        match &self.connection_data {
            CharacterSelectConnectionData::Host {
                has_received_start_request,
                ..
            } if *has_received_start_request => return true,
            CharacterSelectConnectionData::Client { should_start, .. } if *should_start => {
                return true
            }
            _ => (),
        }

        match self.connection.peek(&mut []) {
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return false,
            _ => (),
        }

        let mut buf = std::io::BufReader::new(&self.connection);

        let mut buffer = Vec::with_capacity(64);

        loop {
            match buf.read_until(
                connection::ConnectionPacket::PACKET_END_CHAR as u8,
                &mut buffer,
            ) {
                Err(_) => continue,
                Ok(0) => return false,
                _ => (),
            }
            let Ok(s) = std::str::from_utf8(&buffer) else {
                continue;
            };

            let Ok(packet) = connection::ConnectionPacket::from_str(s) else {
                continue;
            };

            match packet {
                connection::ConnectionPacket::RequestStartGame => {
                    if let CharacterSelectConnectionData::Host {
                        has_received_start_request,
                        ..
                    } = &mut self.connection_data
                    {
                        *has_received_start_request = true;
                        return true;
                    }
                }
                connection::ConnectionPacket::StartGame => {
                    if let CharacterSelectConnectionData::Client { should_start, .. } =
                        &mut self.connection_data
                    {
                        *should_start = true;
                        return true;
                    }
                }
                _ => continue,
            }
        }
    }
    fn send_start(&mut self) {
        self.connection
            .write_all(connection::ConnectionPacket::StartGame.as_str().as_bytes())
            .expect("network error");

        self.connection.flush().expect("network error");
    }
    pub fn request_start(&mut self) {
        match &mut self.connection_data {
            CharacterSelectConnectionData::Host {
                has_received_start_request,
                should_start,
            } => {
                if *has_received_start_request && !*should_start {
                    *should_start = true;
                    self.send_start();
                }
            }
            CharacterSelectConnectionData::Client { .. } => {
                _ = self.connection.write_all(
                    connection::ConnectionPacket::RequestStartGame
                        .as_str()
                        .as_bytes(),
                );

                _ = self.connection.flush();
            }
        }
    }
    pub fn should_start(&self) -> bool {
        match &self.connection_data {
            CharacterSelectConnectionData::Host { should_start, .. }
            | CharacterSelectConnectionData::Client { should_start, .. } => *should_start,
        }
    }
    pub fn into_game_connection(self) -> Result<GameConnection, ConnectionError> {
        let connection = create_game_connection(&self.addr)?;

        Ok(GameConnection {
            addr: self.addr,
            connection,
            frames_to_wait: 0,
            most_recent_packet: None.into(),
            most_recent_sent_frame: 0.into(),
        })
    }
}

pub struct GameConnection {
    addr: ConnectionAddr,
    connection: UdpSocket,
    frames_to_wait: usize,
    most_recent_packet: std::cell::Cell<Option<GamePacket>>,
    most_recent_sent_frame: std::cell::Cell<u32>,
}

impl GameConnection {
    pub fn get_packet(&self) -> Option<GamePacket> {
        let mut buf = [0_u8; size_of::<GamePacket>()];

        let mut most_recent_packet = None;

        loop {
            match self.connection.recv(&mut buf) {
                Ok(bytes) => {
                    assert_eq!(bytes, buf.len());

                    // SAFETY: the buffer is the length of a `GamePacket` so should be safe to
                    // transmute to a `GamePacket`
                    let packet: GamePacket = unsafe { std::mem::transmute(buf) };

                    _ = most_recent_packet.insert(packet);
                }
                Err(_) => {
                    return most_recent_packet
                        .clone()
                        .or(if most_recent_packet.is_some() {
                            self.most_recent_packet.replace(most_recent_packet)
                        } else {
                            most_recent_packet
                        });
                } //Err(e) => panic!("connection error: {e:?}"),
            }
        }
    }
    pub fn send_packet(&self, packet: GamePacket) {
        self.most_recent_sent_frame.set(packet.most_recent_frame());
        let packet: [u8; size_of::<GamePacket>()] = unsafe { std::mem::transmute(packet) };
        _ = self.connection.send(&packet);
    }
    pub fn set_frames_to_wait(&mut self, frames: usize) {
        self.frames_to_wait = frames;
    }
    pub fn can_continue(&mut self) -> bool {
        if self.frames_to_wait == 0 {
            true
        } else {
            self.frames_to_wait -= 1;
            false
        }
    }
    pub fn current_desync(&mut self) -> u32 {
        self.most_recent_sent_frame.get().saturating_sub(
            self.most_recent_packet
                .get_mut()
                .as_ref()
                .map(|p| p.most_recent_frame())
                .unwrap_or(self.most_recent_sent_frame.get()),
        )
    }
    pub fn current_frame(&self) -> u32 {
        self.most_recent_sent_frame.get()
    }
    pub fn addr(&self) -> ConnectionAddr {
        self.addr.clone()
    }
}
