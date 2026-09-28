//! End-to-End CPU-to-Router Network Modeling Subsystem
//!
//! Provides memory-mapped Virtual NICs, full-stack packet framing (IPv4, ARP, UDP, TCP),
//! discrete packet switching Router models, and simulated CPU execution nodes.

pub mod cpu_node;
pub mod nic;
pub mod router;
pub mod stack;

pub use cpu_node::{CpuInstruction, SimulatedCpuNode};
pub use nic::{nic_reg, VirtualNic};
pub use router::{RouteEntry, RouterPort, RouterSwitch};
pub use stack::{
    compute_internet_checksum, ArpOperation, ArpPacket, ArpTable, IpProtocol, Ipv4Address,
    Ipv4Header, UdpDatagram,
};
