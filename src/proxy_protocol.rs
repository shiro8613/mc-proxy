use std::{io::{Cursor, Write}, net::SocketAddr};

pub fn create_proxy_header(c_addr: SocketAddr, d_addr :SocketAddr) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    let _ = buffer.write_all(&[0x0D, 0x0A, 0x0D, 0x0A, 0x00, 0x0D, 0x0A, 0x51, 0x55, 0x49, 0x54, 0x0A, 0x21]);

    let (family, len) : (u8, u16) = if c_addr.is_ipv4() {
        (0x11, 12)
    } else {
        (0x21, 36)
    };

    let _ = buffer.write(&[family]);
    let _ = buffer.write_all(&len.to_be_bytes());

    match c_addr {
        SocketAddr::V4(v4) => {
            let _ = buffer.write_all(&v4.ip().octets());
        },
        SocketAddr::V6(v6) => {
            let _ = buffer.write_all(&v6.ip().octets());
        }
    } 

    match d_addr {
        SocketAddr::V4(v4) => {
            let _ = buffer.write_all(&v4.ip().octets());
        },
        SocketAddr::V6(v6) => {
            let _ = buffer.write_all(&v6.ip().octets());
        }
    }

    let _ = buffer.write_all(&c_addr.port().to_be_bytes());
    let _ = buffer.write_all(&d_addr.port().to_be_bytes());

    buffer.get_mut().to_vec()
}