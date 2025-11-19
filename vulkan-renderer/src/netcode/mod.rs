use std::net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};

mod connection;
mod rollback;

pub use rollback::{FrameState, GamePacket, InputHistory, PacketInputState, Rollback};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    Host,
    Client,
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
