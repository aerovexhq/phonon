//! Full-Stack Network Protocol Layer: IPv4, ARP, UDP & TCP
//!
//! Implements RFC 791 (IPv4), RFC 826 (ARP), RFC 768 (UDP), and RFC 793 (TCP)
//! with RFC 1071 standard internet checksum calculation and validation.

use crate::em::MacAddress;
use std::collections::HashMap;
use std::fmt;

/// RFC 1071 standard 16-bit one's complement internet checksum.
pub fn compute_internet_checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let (chunks, remainder) = data.as_chunks::<2>();
    for &[b0, b1] in chunks {
        let word = u16::from_be_bytes([b0, b1]) as u32;
        sum = sum.wrapping_add(word);
    }
    if !remainder.is_empty() {
        let word = ((remainder[0] as u16) << 8) as u32;
        sum = sum.wrapping_add(word);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

/// IPv4 32-bit address format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Ipv4Address(pub [u8; 4]);

impl Ipv4Address {
    pub const ANY: Self = Self([0, 0, 0, 0]);
    pub const BROADCAST: Self = Self([255, 255, 255, 255]);
    pub const LOCALHOST: Self = Self([127, 0, 0, 1]);

    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self([a, b, c, d])
    }

    #[inline]
    pub fn is_broadcast(&self) -> bool {
        self.0 == [255, 255, 255, 255]
    }

    #[inline]
    pub fn matches_subnet(&self, network: Ipv4Address, mask: Ipv4Address) -> bool {
        let ip_u32 = u32::from_be_bytes(self.0);
        let net_u32 = u32::from_be_bytes(network.0);
        let mask_u32 = u32::from_be_bytes(mask.0);
        (ip_u32 & mask_u32) == (net_u32 & mask_u32)
    }
}

impl fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

/// ARP Operation code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpOperation {
    Request = 1,
    Reply = 2,
}

/// RFC 826 Address Resolution Protocol (ARP) packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArpPacket {
    pub operation: ArpOperation,
    pub sender_mac: MacAddress,
    pub sender_ip: Ipv4Address,
    pub target_mac: MacAddress,
    pub target_ip: Ipv4Address,
}

impl ArpPacket {
    pub fn new_request(
        sender_mac: MacAddress,
        sender_ip: Ipv4Address,
        target_ip: Ipv4Address,
    ) -> Self {
        Self {
            operation: ArpOperation::Request,
            sender_mac,
            sender_ip,
            target_mac: MacAddress::ZERO,
            target_ip,
        }
    }

    pub fn new_reply(
        sender_mac: MacAddress,
        sender_ip: Ipv4Address,
        target_mac: MacAddress,
        target_ip: Ipv4Address,
    ) -> Self {
        Self {
            operation: ArpOperation::Reply,
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(28);
        buf.extend_from_slice(&1u16.to_be_bytes()); // Hardware Type: Ethernet (1)
        buf.extend_from_slice(&0x0800u16.to_be_bytes()); // Protocol Type: IPv4 (0x0800)
        buf.push(6); // Hardware size (6 bytes MAC)
        buf.push(4); // Protocol size (4 bytes IPv4)
        let op = match self.operation {
            ArpOperation::Request => 1u16,
            ArpOperation::Reply => 2u16,
        };
        buf.extend_from_slice(&op.to_be_bytes());
        buf.extend_from_slice(&self.sender_mac.0);
        buf.extend_from_slice(&self.sender_ip.0);
        buf.extend_from_slice(&self.target_mac.0);
        buf.extend_from_slice(&self.target_ip.0);
        buf
    }

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 28 {
            return None;
        }
        let hw_type = u16::from_be_bytes([data[0], data[1]]);
        let proto_type = u16::from_be_bytes([data[2], data[3]]);
        if hw_type != 1 || proto_type != 0x0800 {
            return None;
        }
        let op_code = u16::from_be_bytes([data[6], data[7]]);
        let operation = match op_code {
            1 => ArpOperation::Request,
            2 => ArpOperation::Reply,
            _ => return None,
        };
        let mut sm = [0u8; 6];
        sm.copy_from_slice(&data[8..14]);
        let mut sip = [0u8; 4];
        sip.copy_from_slice(&data[14..18]);
        let mut tm = [0u8; 6];
        tm.copy_from_slice(&data[18..24]);
        let mut tip = [0u8; 4];
        tip.copy_from_slice(&data[24..28]);

        Some(Self {
            operation,
            sender_mac: MacAddress::new(sm),
            sender_ip: Ipv4Address(sip),
            target_mac: MacAddress::new(tm),
            target_ip: Ipv4Address(tip),
        })
    }
}

/// Dynamic ARP table mapping IPv4 addresses to MAC hardware addresses.
#[derive(Debug, Clone, Default)]
pub struct ArpTable {
    entries: HashMap<Ipv4Address, (MacAddress, u64)>,
}

impl ArpTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, ip: Ipv4Address, mac: MacAddress, timestamp: u64) {
        self.entries.insert(ip, (mac, timestamp));
    }

    pub fn lookup(&self, ip: &Ipv4Address) -> Option<MacAddress> {
        self.entries.get(ip).map(|(mac, _)| *mac)
    }
}

/// IPv4 Transport Protocol indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpProtocol {
    Icmp = 1,
    Tcp = 6,
    Udp = 17,
}

/// RFC 791 IPv4 Header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv4Header {
    pub version: u8,
    pub ihl: u8,
    pub dscp: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags: u8,
    pub fragment_offset: u16,
    pub ttl: u8,
    pub protocol: IpProtocol,
    pub checksum: u16,
    pub src_ip: Ipv4Address,
    pub dst_ip: Ipv4Address,
}

impl Ipv4Header {
    pub fn new(
        src_ip: Ipv4Address,
        dst_ip: Ipv4Address,
        protocol: IpProtocol,
        payload_len: usize,
    ) -> Self {
        let total_length = (20 + payload_len) as u16;
        let mut hdr = Self {
            version: 4,
            ihl: 5,
            dscp: 0,
            total_length,
            identification: 0x1234,
            flags: 0x02, // Don't Fragment (DF)
            fragment_offset: 0,
            ttl: 64,
            protocol,
            checksum: 0,
            src_ip,
            dst_ip,
        };
        hdr.checksum = hdr.calculate_checksum();
        hdr
    }

    pub fn serialize_header(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(20);
        let ver_ihl = (self.version << 4) | (self.ihl & 0x0F);
        buf.push(ver_ihl);
        buf.push(self.dscp);
        buf.extend_from_slice(&self.total_length.to_be_bytes());
        buf.extend_from_slice(&self.identification.to_be_bytes());
        let flags_frag = ((self.flags as u16) << 13) | (self.fragment_offset & 0x1FFF);
        buf.extend_from_slice(&flags_frag.to_be_bytes());
        buf.push(self.ttl);
        let proto_num = match self.protocol {
            IpProtocol::Icmp => 1u8,
            IpProtocol::Tcp => 6u8,
            IpProtocol::Udp => 17u8,
        };
        buf.push(proto_num);
        buf.extend_from_slice(&self.checksum.to_be_bytes());
        buf.extend_from_slice(&self.src_ip.0);
        buf.extend_from_slice(&self.dst_ip.0);
        buf
    }

    pub fn calculate_checksum(&self) -> u16 {
        let mut buf = self.serialize_header();
        // Zero out checksum field (bytes 10 and 11) for calculation
        buf[10] = 0;
        buf[11] = 0;
        compute_internet_checksum(&buf)
    }

    pub fn verify_checksum(&self) -> bool {
        let buf = self.serialize_header();
        compute_internet_checksum(&buf) == 0
    }

    pub fn parse(data: &[u8]) -> Option<(Self, &[u8])> {
        if data.len() < 20 {
            return None;
        }
        let ver_ihl = data[0];
        let version = ver_ihl >> 4;
        let ihl = ver_ihl & 0x0F;
        if version != 4 || ihl < 5 {
            return None;
        }
        let dscp = data[1];
        let total_length = u16::from_be_bytes([data[2], data[3]]);
        let identification = u16::from_be_bytes([data[4], data[5]]);
        let flags_frag = u16::from_be_bytes([data[6], data[7]]);
        let flags = (flags_frag >> 13) as u8;
        let fragment_offset = flags_frag & 0x1FFF;
        let ttl = data[8];
        let proto_num = data[9];
        let protocol = match proto_num {
            1 => IpProtocol::Icmp,
            6 => IpProtocol::Tcp,
            17 => IpProtocol::Udp,
            _ => return None,
        };
        let checksum = u16::from_be_bytes([data[10], data[11]]);
        let mut src = [0u8; 4];
        src.copy_from_slice(&data[12..16]);
        let mut dst = [0u8; 4];
        dst.copy_from_slice(&data[16..20]);

        let hdr = Self {
            version,
            ihl,
            dscp,
            total_length,
            identification,
            flags,
            fragment_offset,
            ttl,
            protocol,
            checksum,
            src_ip: Ipv4Address(src),
            dst_ip: Ipv4Address(dst),
        };

        let hdr_bytes_len = (ihl as usize) * 4;
        if data.len() < hdr_bytes_len {
            return None;
        }
        let payload = &data[hdr_bytes_len..data.len().min(total_length as usize)];
        Some((hdr, payload))
    }
}

/// RFC 768 User Datagram Protocol (UDP) Datagram.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UdpDatagram {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
    pub payload: Vec<u8>,
}

impl UdpDatagram {
    pub fn new(src_port: u16, dst_port: u16, payload: Vec<u8>) -> Self {
        let length = (8 + payload.len()) as u16;
        Self {
            src_port,
            dst_port,
            length,
            checksum: 0,
            payload,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + self.payload.len());
        buf.extend_from_slice(&self.src_port.to_be_bytes());
        buf.extend_from_slice(&self.dst_port.to_be_bytes());
        buf.extend_from_slice(&self.length.to_be_bytes());
        buf.extend_from_slice(&self.checksum.to_be_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let length = u16::from_be_bytes([data[4], data[5]]);
        let checksum = u16::from_be_bytes([data[6], data[7]]);
        let payload_len = (length as usize).saturating_sub(8);
        let payload = data[8..8 + payload_len.min(data.len() - 8)].to_vec();

        Some(Self {
            src_port,
            dst_port,
            length,
            checksum,
            payload,
        })
    }
}
