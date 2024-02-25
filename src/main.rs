use std::{io, u16};

fn main() -> io::Result<()> {
    let nic = tun_tap::Iface::new("tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1504];

    loop {
        let nbytes = nic.recv(&mut buf[..])?;
        let _eth_flags: u16 = u16::from_be_bytes([buf[0], buf[1]]);
        let eth_proto: u16 = u16::from_be_bytes([buf[2], buf[3]]);

        if eth_proto != 0x0800 {
            // Not an IPv4 protocol
            continue;
        }

        match etherparse::Ipv4Slice::from_slice(&buf[4..nbytes]) {
            Ok(p) => {
                let p = p.header();
                let src = p.source_addr();
                let dst = p.destination_addr();
                let proto = p.protocol();
                if proto.0 != 0x06 {
                    // Not a tcp protocol
                    continue;
                }
                eprintln!(
                    "{} -> {}, {}b of protocol {}",
                    src,
                    dst,
                    p.payload_len().unwrap(),
                    proto.0
                );
            }
            Err(_e) => {
                eprintln!("Ignoring this packet");
            }
        }
    }
}
