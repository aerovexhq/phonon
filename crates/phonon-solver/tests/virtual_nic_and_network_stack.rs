//! Integration Tests for Virtual NIC, MMIO Registers & Full-Stack Network Layer

use phonon_models::em::MacAddress;
use phonon_models::net::nic::{
    nic_reg, VirtualNic, CTRL_IRQ_DISABLE, CTRL_IRQ_ENABLE, CTRL_RESET, CTRL_RX_ACK, CTRL_TX_START,
    STATUS_IRQ_ASSERTED, STATUS_RX_READY, STATUS_TX_READY,
};
use phonon_models::net::stack::{
    compute_internet_checksum, ArpOperation, ArpPacket, ArpTable, IpProtocol, Ipv4Address,
    Ipv4Header, UdpDatagram,
};

#[test]
fn test_virtual_nic_mmio_read_write_and_control() {
    let mac = MacAddress([0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E]);
    let ip = [192, 168, 1, 100];
    let mut nic = VirtualNic::new(mac, ip);

    // Initial state: TX is ready, RX empty, IRQ low
    assert_eq!(
        nic.read_u32(nic_reg::STATUS) & STATUS_TX_READY,
        STATUS_TX_READY
    );
    assert_eq!(nic.read_u32(nic_reg::STATUS) & STATUS_RX_READY, 0);
    assert_eq!(nic.read_u32(nic_reg::STATUS) & STATUS_IRQ_ASSERTED, 0);

    // Test IRQ disable & re-enable
    nic.write_u32(nic_reg::CONTROL, CTRL_IRQ_DISABLE);
    assert!(!nic.irq_enabled);
    nic.write_u32(nic_reg::CONTROL, CTRL_IRQ_ENABLE);
    assert!(nic.irq_enabled);

    // Write packet into TX buffer via MMIO
    let tx_payload = [0xAA, 0xBB, 0xCC, 0xDD];
    let word = u32::from_le_bytes(tx_payload);
    nic.write_u32(nic_reg::TX_BUF_START, word);
    nic.write_u32(nic_reg::TX_LEN, 4);

    assert_eq!(nic.read_u32(nic_reg::TX_BUF_START), word);
    assert_eq!(nic.read_u32(nic_reg::TX_LEN), 4);

    // Trigger transmission: CONTROL <- TX_START
    nic.write_u32(nic_reg::CONTROL, CTRL_TX_START);
    assert_eq!(nic.tx_outbound_queue.len(), 1);
    assert_eq!(nic.tx_outbound_queue[0], tx_payload);
    assert_eq!(nic.packets_transmitted, 1);

    // Receive a packet into RX buffer
    let rx_packet = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
    assert!(nic.receive_packet(&rx_packet));

    // Verify RX_READY and IRQ_ASSERTED
    let st = nic.read_u32(nic_reg::STATUS);
    assert_ne!(st & STATUS_RX_READY, 0);
    assert_ne!(st & STATUS_IRQ_ASSERTED, 0);
    assert!(nic.irq_asserted);
    assert_eq!(nic.read_u32(nic_reg::RX_LEN), 6);
    assert_eq!(nic.read_rx_bytes(), rx_packet);

    // Acknowledge RX: CONTROL <- RX_ACK
    nic.write_u32(nic_reg::CONTROL, CTRL_RX_ACK);
    assert_eq!(nic.read_u32(nic_reg::STATUS) & STATUS_RX_READY, 0);
    assert_eq!(nic.read_u32(nic_reg::STATUS) & STATUS_IRQ_ASSERTED, 0);
    assert!(!nic.irq_asserted);

    // Soft reset
    nic.write_u32(nic_reg::CONTROL, CTRL_RESET);
    assert_eq!(nic.read_u32(nic_reg::TX_LEN), 0);
    assert_eq!(nic.read_u32(nic_reg::RX_LEN), 0);
}

#[test]
fn test_arp_packet_framing_and_cache_learning() {
    let sm = MacAddress([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    let sip = Ipv4Address::new(192, 168, 1, 10);
    let tm = MacAddress([0x00, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);
    let tip = Ipv4Address::new(192, 168, 1, 20);

    let arp_req = ArpPacket::new_request(sm, sip, tip);
    assert_eq!(arp_req.operation, ArpOperation::Request);

    let bytes = arp_req.serialize();
    assert_eq!(bytes.len(), 28);

    let parsed = ArpPacket::parse(&bytes).expect("Failed to parse serialized ARP packet");
    assert_eq!(parsed.operation, ArpOperation::Request);
    assert_eq!(parsed.sender_mac, sm);
    assert_eq!(parsed.sender_ip, sip);
    assert_eq!(parsed.target_ip, tip);

    // ARP Table lookup and dynamic entry
    let mut arp_table = ArpTable::new();
    assert!(arp_table.lookup(&tip).is_none());

    // Learn mapping from ARP reply
    let arp_rep = ArpPacket::new_reply(tm, tip, sm, sip);
    arp_table.insert(arp_rep.sender_ip, arp_rep.sender_mac, 100);

    let resolved = arp_table.lookup(&tip);
    assert_eq!(resolved, Some(tm));
}

#[test]
fn test_ipv4_and_udp_header_serialization_and_rfc1071_checksum() {
    let src_ip = Ipv4Address::new(10, 0, 0, 1);
    let dst_ip = Ipv4Address::new(10, 0, 0, 2);
    let payload = b"Phonon UDP Data Payload 2026".to_vec();

    let udp = UdpDatagram::new(5000, 8080, payload.clone());
    let udp_bytes = udp.serialize();
    assert_eq!(udp_bytes.len(), 8 + payload.len());

    let parsed_udp = UdpDatagram::parse(&udp_bytes).expect("Failed to parse UDP");
    assert_eq!(parsed_udp.src_port, 5000);
    assert_eq!(parsed_udp.dst_port, 8080);
    assert_eq!(parsed_udp.payload, payload);

    let ip_hdr = Ipv4Header::new(src_ip, dst_ip, IpProtocol::Udp, udp_bytes.len());
    assert!(
        ip_hdr.verify_checksum(),
        "Initial IPv4 checksum failed verification"
    );

    let ip_bytes = ip_hdr.serialize_header();
    assert_eq!(compute_internet_checksum(&ip_bytes), 0);

    // Parse back
    let mut full_packet = ip_bytes;
    full_packet.extend_from_slice(&udp_bytes);

    let (parsed_ip, extracted_payload) =
        Ipv4Header::parse(&full_packet).expect("Failed to parse IPv4");
    assert_eq!(parsed_ip.src_ip, src_ip);
    assert_eq!(parsed_ip.dst_ip, dst_ip);
    assert_eq!(parsed_ip.protocol, IpProtocol::Udp);
    assert_eq!(extracted_payload, udp_bytes.as_slice());

    // Check corruption detection
    let mut corrupt_ip = full_packet.clone();
    corrupt_ip[12] ^= 0x01; // Corrupt IP src byte
    assert_ne!(compute_internet_checksum(&corrupt_ip[..20]), 0);
}
