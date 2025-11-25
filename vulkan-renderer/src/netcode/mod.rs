use std::io::prelude::*;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream, ToSocketAddrs, UdpSocket};

mod connection;
mod rollback;

pub use rollback::{GamePacket, InputHistory, Rollback};

pub fn ask_connection_type() -> ConnectionType {
    let mut buf = String::new();
    loop {
        print!("[0] offline (default)\n[1] host\n[2] join\n> ");
        _ = std::io::stdout().flush();

        std::io::stdin()
            .read_line(&mut buf)
            .expect("failed to get input");

        return match buf.trim() {
            "0" | "" => ConnectionType::Offline,
            "1" => {
                print!("\n[HOST] -- enter address\n> ");
                _ = std::io::stdout().flush();
                buf.clear();

                std::io::stdin()
                    .read_line(&mut buf)
                    .expect("failed to get input");

                let addr = buf
                    .trim()
                    .to_socket_addrs()
                    .map(|mut a| a.next())
                    .ok()
                    .flatten()
                    .expect("invalid address");

                /*let addr = SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::LOCALHOST,
                    buf.trim().parse::<u16>().ok().expect("invalid address"),
                ));*/

                println!("[HOST] -- hosting at {}", addr.to_string());

                ConnectionType::Host(addr)
            }
            "2" => {
                print!("\n[JOIN] -- enter address\n> ");
                _ = std::io::stdout().flush();
                buf.clear();

                std::io::stdin()
                    .read_line(&mut buf)
                    .expect("failed to get input");

                let addr = buf
                    .trim()
                    .to_socket_addrs()
                    .map(|mut a| a.next())
                    .ok()
                    .flatten()
                    .expect("invalid address");

                println!("[JOIN] -- trying to join {}", addr);

                ConnectionType::Client(addr)
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

#[derive(Debug)]
pub enum GameConnectionError {
    Io(std::io::Error),
}
fn create_game_connection(addr: &ConnectionAddr) -> Result<UdpSocket, GameConnectionError> {
    let connection = UdpSocket::bind(addr.local).map_err(GameConnectionError::Io)?;

    connection
        .set_nonblocking(true)
        .map_err(GameConnectionError::Io)?;

    connection
        .connect(addr.remote)
        .map_err(GameConnectionError::Io)?;

    Ok(connection)
}

struct ConnectionAddr {
    local: SocketAddr,
    remote: SocketAddr,
}

pub struct CharacterSelectConnection {
    addr: ConnectionAddr,
    connection: TcpStream,
    has_requested_start: bool,
    is_host: bool,
}

impl CharacterSelectConnection {
    pub fn host(addr: SocketAddr) -> Result<CharacterSelectConnection, GameConnectionError> {
        let connection::Connection {
            stream,
            addr: remote,
        } = connection::host(&addr).map_err(GameConnectionError::Io)?;

        let addr = ConnectionAddr {
            local: addr,
            remote,
        };

        println!("[HOST] -- connected to {}", addr.remote);

        Ok(CharacterSelectConnection {
            addr,
            connection: stream,
            has_requested_start: false,
            is_host: true,
        })
    }
    pub fn join(remote: SocketAddr) -> Result<CharacterSelectConnection, GameConnectionError> {
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
            has_requested_start: false,
            is_host: false,
        })
    }
    pub fn requested_start(&mut self) -> bool {
        if self.has_requested_start {
            return true;
        }

        match self.connection.peek(&mut []) {
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return false,
            _ => (),
        }

        let mut buf = std::io::BufReader::new(&self.connection);

        let mut buffer = Vec::with_capacity(64);

        if let Err(_) = buf.read_until(
            connection::ConnectionPacket::PACKET_END_CHAR as u8,
            &mut buffer,
        ) {
            return false;
        }

        if self.is_host {
            match std::str::from_utf8(&buffer).map(connection::ConnectionPacket::from_str) {
                Ok(Ok(connection::ConnectionPacket::RequestStartGame)) => {
                    self.has_requested_start = true;
                    true
                }
                _ => false,
            }
        } else {
            println!("received {:?}", &buffer);
            match std::str::from_utf8(&buffer).map(connection::ConnectionPacket::from_str) {
                Ok(Ok(connection::ConnectionPacket::StartGame)) => {
                    self.has_requested_start = true;
                    true
                }
                _ => false,
            }
        }
    }
    pub fn send_start(&mut self) {
        println!("sent start");
        self.connection
            .write(connection::ConnectionPacket::StartGame.as_str().as_bytes())
            .expect("network error");

        self.connection.flush().expect("network error");
    }
    pub fn request_start(&mut self) {
        _ = self.connection.write(
            connection::ConnectionPacket::RequestStartGame
                .as_str()
                .as_bytes(),
        );

        _ = self.connection.flush();
    }
    pub fn into_game_connection(self) -> Result<GameConnection, GameConnectionError> {
        /*self.connection
        .shutdown(std::net::Shutdown::Both)
        .map_err(GameConnectionError::Io)?;*/

        let connection = create_game_connection(&self.addr)?;

        Ok(GameConnection {
            addr: self.addr,
            connection,
            frames_to_wait: 0,
            most_recent_packet: None.into(),
        })
    }
}

pub struct GameConnection {
    addr: ConnectionAddr,
    connection: UdpSocket,
    frames_to_wait: usize,
    most_recent_packet: std::cell::Cell<Option<GamePacket>>,
}

impl GameConnection {
    pub fn get_packet(&self) -> Option<GamePacket> {
        let mut buf = [0_u8; size_of::<GamePacket>()];

        let mut most_recent_packet = None;

        loop {
            match self.connection.recv(&mut buf) {
                Ok(bytes) => {
                    assert_eq!(bytes, buf.len());

                    println!("got packet {:?}", unsafe {
                        std::mem::transmute_copy::<
                            [u8; size_of::<GamePacket>()],
                            [rollback::FrameState; 2],
                        >(&buf)
                    });

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
}
