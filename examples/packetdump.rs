extern crate smoltcp;
extern crate znet;

use smoltcp::wire;

#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "linux"))]
use znet::raw_socket::{LinkLayer, RawSocket, BufferReader};

use std::env;


fn handle_ip_packet(packet: &[u8]) {
    match wire::IpVersion::of_packet(&packet) {
        Ok(version) => match version {
            wire::IpVersion::Ipv4 => {
                println!("{}", &wire::PrettyPrinter::<wire::Ipv4Packet<&[u8]>>::new("", &packet));
            },
            wire::IpVersion::Ipv6 => {
                println!("{}", &wire::PrettyPrinter::<wire::Ipv6Packet<&[u8]>>::new("", &packet));
            },
            _ => { }
        },
        Err(_) => { }
    }
}

fn handle_ethernet_frame(packet: &[u8]) {
    println!("{}", &wire::PrettyPrinter::<wire::EthernetFrame<&[u8]>>::new("", &packet));
}

#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "linux"))]
fn main() {
    let mut args = env::args();
    if args.len() < 2 {
        println!("Usage:\n    $ sudo target/debug/examples/packetdump <interface name>");
        return ();
    }

    let ifname = args.nth(1).unwrap().clone();

    let mut raw_socket = RawSocket::with_ifname(&ifname).unwrap();
    let mut buffer = vec![0u8; raw_socket.blen()];

    let link_layer = raw_socket.link_layer();
    println!("Interface:\n\tname: {}\n\tdatalink: {}\n", ifname, link_layer);

    loop {
        raw_socket.wait(None).unwrap();
        match raw_socket.recv(&mut buffer) {
            Ok(len) => {
                for (start, end) in BufferReader::new(&buffer, len) {
                    match link_layer {
                        LinkLayer::IpWithPI(prefix_len) => {
                            let packet = &buffer[start+prefix_len..end];
                            handle_ip_packet(&packet);
                        },
                        LinkLayer::Eth => {
                            let packet = &buffer[start..end];
                            handle_ethernet_frame(&packet);
                        },
                        LinkLayer::Ip => {
                            let packet = &buffer[start..end];
                            handle_ip_packet(&packet);
                        }
                    }
                }
            }
            Err(e) => {
                println!("[ERROR] {:?}", e);
            }
        }
    }
}

