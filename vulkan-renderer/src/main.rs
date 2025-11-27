use std::io::prelude::*;
use std::net::ToSocketAddrs;

use vulkan_renderer::{App, AppConfig, ConnectionError, ConnectionType};

fn main() {
    let connection_type = match ask_connection_type() {
        Ok(c) => c,
        Err(ConnectionError::Io(e)) => {
            println!("io error : [{e}]");
            return;
        }
        Err(ConnectionError::InvalidAddress) => {
            println!("invalid address!\naddress should be in form [x.x.x.x:socket]");
            return;
        }
        _ => unreachable!(),
    };

    let config = AppConfig::default().with_connection(connection_type);

    match App::run(config) {
        Ok(_) => {}
        Err(e) => match e {
            ConnectionError::Io(e) => println!("io error - {e:?}"),
            ConnectionError::Timeout(_) => {
                println!("connection timeout, the opponent may have disconnected!")
            }
            ConnectionError::InvalidAddress => println!("invalid address!"),
            ConnectionError::Utf8(_) | ConnectionError::NotPacket => unreachable!(),
        },
    }
}

fn ask_connection_type() -> Result<ConnectionType, ConnectionError> {
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

                if buf.trim().is_empty() {
                    continue;
                }

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

                if buf.trim().is_empty() {
                    continue;
                }

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
