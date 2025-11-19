use std::io::{prelude::*, BufReader};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};

pub struct Host {
    stream: TcpStream,
}

#[derive(Debug)]
pub enum ConnectionError {
    IO(std::io::Error),
    Utf8(std::str::Utf8Error),
    NotPacket,
}

pub fn host(port: u16) -> std::io::Result<Host> {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port))?;

    let mut stream = None;
    for connection in listener.incoming() {
        let Ok(connection) = connection else {
            continue;
        };

        if let Ok(s) = handle_connection(connection) {
            _ = stream.insert(s);
            break;
        }
    }
    let stream = stream.unwrap();

    Ok(Host { stream })
}

fn handle_connection(mut stream: TcpStream) -> Result<TcpStream, ConnectionError> {
    let mut buf_reader = BufReader::new(&stream);

    let mut buf = Vec::with_capacity(ConnectionPacket::CLIENT_CONNECTION.len());

    buf_reader
        .read_until(ConnectionPacket::PACKET_END_CHAR as u8, &mut buf)
        .map_err(ConnectionError::IO)?;

    let packet = str::from_utf8(&buf).map_err(ConnectionError::Utf8)?;

    let Ok(ConnectionPacket::ClientConnection) = ConnectionPacket::from_str(packet) else {
        return Err(ConnectionError::NotPacket);
    };

    stream
        .write_all(ConnectionPacket::HostConfirmConnection.as_str().as_bytes())
        .map_err(ConnectionError::IO)?;

    Ok(stream)
}

pub struct Client {
    stream: TcpStream,
}

pub fn connect_to<A: std::net::ToSocketAddrs>(addr: A) -> Result<Client, ConnectionError> {
    let mut stream = TcpStream::connect(addr).map_err(ConnectionError::IO)?;
    stream
        .write_all(ConnectionPacket::ClientConnection.as_str().as_bytes())
        .map_err(ConnectionError::IO)?;

    let mut buf = Vec::with_capacity(ConnectionPacket::REQUEST_START_GAME.len());

    let mut buf_reader = BufReader::new(&stream);
    buf_reader
        .read_until(ConnectionPacket::PACKET_END_CHAR as u8, &mut buf)
        .map_err(ConnectionError::IO)?;

    let packet = str::from_utf8(&buf).map_err(ConnectionError::Utf8)?;

    let Ok(ConnectionPacket::HostConfirmConnection) = ConnectionPacket::from_str(packet) else {
        return Err(ConnectionError::NotPacket);
    };

    Ok(Client { stream })
}

enum PacketType {
    ClientConnection,
    HostConfirmConnection,
    Request,
}

pub enum ConnectionPacket {
    ClientConnection,
    HostConfirmConnection,
    RequestStartGame,
    StartGame,
}
impl ConnectionPacket {
    const PACKET_END_CHAR: char = '\0';
    const CLIENT_CONNECTION: &str = "CONNECTED\0";
    const HOST_CONFIRM_CONNECTION: &str = "HOSTCONNECTED\0";
    const REQUEST_START_GAME: &str = "REQUESTSTARTGAME\0";
    const START_GAME: &str = "STARTGAME\0";

    fn as_str(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Self::ClientConnection => Self::CLIENT_CONNECTION.into(),
            Self::HostConfirmConnection => Self::HOST_CONFIRM_CONNECTION.into(),
            Self::RequestStartGame => Self::REQUEST_START_GAME.into(),
            Self::StartGame => Self::START_GAME.into(),
        }
    }
    fn from_str(s: &str) -> Result<Self, ()> {
        let packet_type = s.lines().next().ok_or(())?.trim();
        match packet_type {
            Self::CLIENT_CONNECTION => Ok(Self::ClientConnection),
            Self::HOST_CONFIRM_CONNECTION => Ok(Self::HostConfirmConnection),
            Self::REQUEST_START_GAME => Ok(Self::RequestStartGame),
            Self::START_GAME => Ok(Self::StartGame),
            _ => Err(()),
        }
    }
}
