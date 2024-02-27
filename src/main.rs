use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;

mod tcp;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> io::Result<()> {
    let mut connections: HashMap<Quad, tcp::State> = Default::default();
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
            Ok(iph) => {
                let iph = iph.header();
                let src = iph.source_addr();
                let dst = iph.destination_addr();
                let proto = iph.protocol();
                if proto.0 != 0x06 {
                    // Not a tcp protocol
                    continue;
                }
                match etherparse::TcpHeaderSlice::from_slice(&buf[4 + iph.slice().len()..nbytes]) {
                    Ok(tcph) => {
                        let datai = 4 + iph.slice().len() + tcph.slice().len();
                        connections
                            .entry(Quad {
                                src: (src, tcph.source_port()),
                                dst: (dst, tcph.destination_port()),
                            })
                            .or_default()
                            .on_packet(iph, tcph, &buf[datai..nbytes]);
                    }
                    Err(_e) => eprintln!("Ignoring weird tcp packet"),
                }
            }
            Err(_e) => eprintln!("Ignoring this packet"),
        }
    }
}
