use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};

mod connection;
mod rollback;

pub use rollback::{FrameState, GamePacket, InputHistory, Rollback};

pub fn ask_connection_type() -> ConnectionType {
    print!("[0] offline (default)\n[1] host\n[2] join\n> ");
    _ = std::io::stdout().flush();

    let mut buf = String::new();
    loop {
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

                ConnectionType::Host(
                    buf.trim()
                        .to_socket_addrs()
                        .map(|mut a| a.next())
                        .ok()
                        .flatten()
                        .expect("invalid address"),
                )
            }
            "2" => {
                print!("\n[JOIN] -- enter address\n> ");
                _ = std::io::stdout().flush();
                buf.clear();

                std::io::stdin()
                    .read_line(&mut buf)
                    .expect("failed to get input");

                ConnectionType::Client(
                    buf.trim()
                        .to_socket_addrs()
                        .map(|mut a| a.next())
                        .ok()
                        .flatten()
                        .expect("invalid address"),
                )
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

pub const MAX_ROLLBACK_FRAMES: usize = 10;

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
}

impl CharacterSelectConnection {
    pub fn into_game_connection(self) -> Result<GameConnection, GameConnectionError> {
        self.connection
            .shutdown(std::net::Shutdown::Both)
            .map_err(GameConnectionError::Io)?;

        let connection = create_game_connection(&self.addr)?;

        Ok(GameConnection {
            addr: self.addr,
            connection,
        })
    }
}

pub struct GameConnection {
    addr: ConnectionAddr,
    connection: UdpSocket,
}

impl GameConnection {
    pub fn get_packet(&self) -> Option<GamePacket> {
        let mut buf = [0_u8; size_of::<GamePacket>()];

        match self.connection.recv(&mut buf) {
            Ok(bytes) => {
                assert_eq!(bytes, buf.len());

                // SAFETY: the buffer is the length of a `GamePacket` so should be safe to
                // transmute to a `GamePacket`
                let packet: GamePacket = unsafe { std::mem::transmute(buf) };

                Some(packet)
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
            Err(_) => panic!("connection error"),
        }
    }
}
