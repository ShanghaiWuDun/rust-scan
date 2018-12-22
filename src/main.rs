#![feature(duration_as_u128, test)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate smoltcp;
extern crate net2;
extern crate rand;

#[cfg(test)]
extern crate test;


use net2::raw_socket::{ LinkLayer, RawSocket, BufferReader };
use smoltcp::wire::{
    self,
    EthernetFrame, EthernetProtocol,
    IpVersion, IpProtocol,
    Ipv4Address, Ipv4Packet,
};
use crate::smoltcp::phy::{ Checksum, ChecksumCapabilities };


use std::io;
use std::env;
use std::time;
use std::thread;



pub struct SynPkt {
    pub buffer: [u8; 1500],
    pub len: usize,
}

pub fn mk_syn_packet(src_mac: [u8; 6],
                 dst_mac: [u8; 6],
                 src_ip: [u8; 4],
                 dst_ip: [u8; 4],
                 src_port: u16,
                 dst_port: u16) -> SynPkt {
    let mut ether_buffer = [0u8; 1500];

    let mut ether_frame = wire::EthernetFrame::new_unchecked(&mut ether_buffer[..]);

    let ether_repr = wire::EthernetRepr {
        src_addr: wire::EthernetAddress(src_mac),
        dst_addr: wire::EthernetAddress(dst_mac),
        ethertype: wire::EthernetProtocol::Ipv4,
    };
    ether_repr.emit(&mut ether_frame);
    
    let mut checksum_cap = ChecksumCapabilities::default();
    checksum_cap.ipv4 = Checksum::Both;
    checksum_cap.tcp = Checksum::Both;
    checksum_cap.udp = Checksum::None;

    let tcp_repr = wire::TcpRepr {
        src_port: src_port,
        dst_port: dst_port,
        control: wire::TcpControl::Syn,
        seq_number: wire::TcpSeqNumber(rand::random::<i32>()),
        ack_number: None,
        window_len: 1024,
        window_scale: None,
        max_seg_size: None,
        payload: &[],
    };

    let tcp_msg_header_len = tcp_repr.mss_header_len();
    
    let mut ipv4_packet = wire::Ipv4Packet::new_unchecked(&mut ether_frame.payload_mut()[..]);
    
    let src_ip = wire::Ipv4Address(src_ip);
    let dst_ip = wire::Ipv4Address(dst_ip);
    
    let ipv4_repr = wire::Ipv4Repr {
        src_addr: src_ip,
        dst_addr: dst_ip,
        protocol: wire::IpProtocol::Tcp,
        payload_len: tcp_msg_header_len,
        hop_limit: 0,
    };
    
    ipv4_repr.emit(&mut ipv4_packet, &checksum_cap);

    let mut tcp_packet = wire::TcpPacket::new_unchecked(&mut ipv4_packet.payload_mut()[..] );
    tcp_repr.emit(&mut tcp_packet,
                  &wire::IpAddress::Ipv4(src_ip),
                  &wire::IpAddress::Ipv4(dst_ip),
                  &checksum_cap);
    
    ipv4_repr.emit(&mut ipv4_packet, &checksum_cap);
    
    let pkt_len = EthernetFrame::<&[u8]>::buffer_len(ipv4_packet.total_len() as usize);
    
    SynPkt {
        buffer: ether_buffer,
        len: pkt_len,
    }

    // src_ip  : 25, 26, 27, 28
    // dst_ip  : 29, 30, 31, 32
    // src_port: 33, 34
    // dst_port: 35, 36
    // seq_num : 37, 38, 39, 40
    // ack_num : 41, 42, 43, 44

    // let mut data = [
    //     // dst mac
    //     // 204, 45, 224, 235, 119, 49, 
    //     0xd4, 0xee, 0x7, 0x5a, 0x67, 0x40,
    //     // src mac
    //     24, 101, 144, 221, 76, 149, 
    //     8, 0, 
    //     69, 0, 0, 40, 0, 0, 64, 0, 0, 6, 200, 58, 
    //         // src ip
    //         192, 168, 199, 200, 
    //         // dst ip
    //         192, 168, 199, 1, 
    //     // src
    //     221, 90, 0, 1, 

    //         // seq num , ack num
    //         0, 10, 44, 42, 0, 0, 0, 0, 
    //         80, 2, 0, 0, 243, 188, 0, 0, 
    // ];
}



#[bench]
fn name(b: &mut test::Bencher) {
    b.bytes = 54;
    b.iter(|| {
        let _ = mk_syn_packet(
            [0x18u8, 0x65, 0x90, 0xdd, 0x4c, 0x95], [0xd4u8, 0xee, 0x7, 0x5a, 0x67, 0x40],
            [192u8, 168, 199, 200], [192u8, 168, 199, 1], 
            10000u16, 81u16,
        );
    })
}

fn main() {
    env::set_var("RUST_LOG", "scan=trace");
    env_logger::init();

    let mut args = env::args();
    if args.len() < 2 {
        println!("Usage:\n    $ sudo target/debug/scan <interface name>");
        return ();
    }
    let ifname = args.nth(1).unwrap().clone();

    let mut raw_socket = RawSocket::with_ifname(&ifname).unwrap();
    let mut buffer = vec![0u8; raw_socket.blen()];
    let link_layer = raw_socket.link_layer();

    let now = time::Instant::now();

    for dst_port in 1..1000u16 {
        if dst_port % 20 == 0 {
            thread::sleep(std::time::Duration::from_millis(500));
        }
        // dst_mac: [0xcc, 0x2d, 0xe0, 0xeb, 0x77, 0x31]
        let pkt = mk_syn_packet(
            [0x18u8, 0x65, 0x90, 0xdd, 0x4c, 0x95], [0xd4u8, 0xee, 0x7, 0x5a, 0x67, 0x40],
            [192u8, 168, 199, 200], [192u8, 168, 199, 1], 
            10000u16, dst_port,
        );
        
        raw_socket.send(&pkt.buffer[..pkt.len]).unwrap();
    }
    
    println!("duration: {:?}", now.elapsed().as_millis());
    
    loop {
        break;
    }
}
