//! Integration Tests for Discrete Router Switching, Routing Tables & Queue Dynamics

use phonon_models::em::MacAddress;
use phonon_models::net::router::{RouterPort, RouterSwitch};
use phonon_models::net::stack::{IpProtocol, Ipv4Address, Ipv4Header, UdpDatagram};

#[test]
fn test_router_port_queue_capacity_and_tail_drop() {
    let mac = MacAddress([0x00, 0x11, 0x22, 0x33, 0x44, 0x00]);
    let ip = Ipv4Address::new(192, 168, 1, 1);
    let mask = Ipv4Address::new(255, 255, 255, 0);

    // 4096 bytes buffer capacity
    let mut port = RouterPort::new(0, mac, ip, mask, 4096);
    assert_eq!(port.current_buffer_bytes, 0);
    assert_eq!(port.buffer_utilization(), 0.0);

    let packet_1kb = vec![0xEE; 1000];

    // Enqueue 4 packets (4000 bytes <= 4096)
    for _ in 0..4 {
        assert!(port.enqueue_packet(packet_1kb.clone()));
    }
    assert_eq!(port.current_buffer_bytes, 4000);
    assert!(port.buffer_utilization() > 0.9);

    // 5th packet exceeds 4096 bytes: tail drop must engage!
    assert!(!port.enqueue_packet(packet_1kb.clone()));
    assert_eq!(port.packets_dropped, 1);
    assert_eq!(port.current_buffer_bytes, 4000);

    // Dequeue packets
    let popped = port.dequeue_packet();
    assert!(popped.is_some());
    assert_eq!(port.current_buffer_bytes, 3000);
    assert_eq!(port.packets_forwarded, 1);
}

#[test]
fn test_router_longest_prefix_match_forwarding() {
    let mac0 = MacAddress([0x00, 0x00, 0x00, 0x00, 0x01, 0x00]);
    let mac1 = MacAddress([0x00, 0x00, 0x00, 0x00, 0x02, 0x00]);
    let ip0 = Ipv4Address::new(10, 0, 1, 1);
    let ip1 = Ipv4Address::new(10, 0, 2, 1);
    let mask = Ipv4Address::new(255, 255, 255, 0);

    let mut router = RouterSwitch::new("EdgeRouter", 1000.0);
    router.add_port(RouterPort::new(0, mac0, ip0, mask, 65536));
    router.add_port(RouterPort::new(1, mac1, ip1, mask, 65536));

    // Routes:
    // 10.0.1.0/24 -> Port 0
    // 10.0.2.0/24 -> Port 1
    // 10.0.0.0/16 -> Port 0 (less specific)
    router.add_route(
        Ipv4Address::new(10, 0, 0, 0),
        Ipv4Address::new(255, 255, 0, 0),
        None,
        0,
    );
    router.add_route(Ipv4Address::new(10, 0, 1, 0), mask, None, 0);
    router.add_route(Ipv4Address::new(10, 0, 2, 0), mask, None, 1);

    // Lookups
    let r1 = router
        .lookup_route(Ipv4Address::new(10, 0, 1, 55))
        .expect("Route 1 not found");
    assert_eq!(r1.output_port, 0);

    let r2 = router
        .lookup_route(Ipv4Address::new(10, 0, 2, 88))
        .expect("Route 2 not found");
    assert_eq!(r2.output_port, 1);

    // Longest prefix match: 10.0.2.88 matches 10.0.2.0/24 (len 24) over 10.0.0.0/16 (len 16)
    assert_eq!(r2.subnet_mask, mask);
}

#[test]
fn test_router_ttl_decrement_and_packet_forwarding() {
    let mac0 = MacAddress([0x00, 0x01, 0x00, 0x00, 0x00, 0x01]);
    let mac1 = MacAddress([0x00, 0x01, 0x00, 0x00, 0x00, 0x02]);
    let ip0 = Ipv4Address::new(192, 168, 1, 1);
    let ip1 = Ipv4Address::new(192, 168, 2, 1);
    let mask = Ipv4Address::new(255, 255, 255, 0);

    let mut router = RouterSwitch::new("CoreSwitch", 1000.0);
    router.add_port(RouterPort::new(0, mac0, ip0, mask, 65536));
    router.add_port(RouterPort::new(1, mac1, ip1, mask, 65536));
    router.add_route(Ipv4Address::new(192, 168, 2, 0), mask, None, 1);

    // Construct packet from 192.168.1.10 to 192.168.2.20 with TTL=64
    let udp = UdpDatagram::new(1234, 5678, b"PacketPayload".to_vec());
    let udp_bytes = udp.serialize();
    let mut hdr = Ipv4Header::new(
        Ipv4Address::new(192, 168, 1, 10),
        Ipv4Address::new(192, 168, 2, 20),
        IpProtocol::Udp,
        udp_bytes.len(),
    );
    hdr.ttl = 64;
    hdr.checksum = hdr.calculate_checksum();

    let mut raw_packet = hdr.serialize_header();
    raw_packet.extend_from_slice(&udp_bytes);

    // Ingest into Port 0 -> routed to Port 1 egress queue
    assert!(router.route_incoming_packet(0, raw_packet));

    let port1 = router.ports.iter_mut().find(|p| p.port_id == 1).unwrap();
    assert_eq!(port1.egress_queue.len(), 1);
    let forwarded_raw = port1.dequeue_packet().expect("No forwarded packet");

    // Verify forwarded packet has TTL=63 and valid checksum!
    let (forwarded_hdr, _) =
        Ipv4Header::parse(&forwarded_raw).expect("Failed to parse forwarded IP");
    assert_eq!(forwarded_hdr.ttl, 63);
    assert!(forwarded_hdr.verify_checksum());

    // Test TTL expiration (TTL=1)
    let mut expired_hdr = hdr;
    expired_hdr.ttl = 1;
    expired_hdr.checksum = expired_hdr.calculate_checksum();
    let mut expired_packet = expired_hdr.serialize_header();
    expired_packet.extend_from_slice(&udp_bytes);

    // Router must drop packet when TTL <= 1
    assert!(!router.route_incoming_packet(0, expired_packet));
}
