//! Discrete Packet Switching Router & Queue Dynamics Engine
//!
//! Models multi-port store-and-forward routing, IP prefix forwarding tables,
//! port output FIFOs, tail-drop buffer exhaustion, and switching latency.

use crate::em::MacAddress;
use crate::net::stack::{ArpTable, Ipv4Address, Ipv4Header};

/// Router port representing a physical network interface (e.g. Wi-Fi AP or Ethernet).
#[derive(Debug, Clone)]
pub struct RouterPort {
    /// Port identifier index.
    pub port_id: usize,
    /// Port physical hardware MAC address.
    pub mac_address: MacAddress,
    /// Assigned IP address for this subnet interface.
    pub ip_address: Ipv4Address,
    /// Subnet mask.
    pub subnet_mask: Ipv4Address,
    /// Maximum egress buffer capacity in bytes (e.g. 65536 = 64 KB).
    pub max_buffer_bytes: usize,
    /// Current egress buffer occupancy in bytes.
    pub current_buffer_bytes: usize,
    /// Output packet queue.
    pub egress_queue: Vec<Vec<u8>>,
    /// Telemetry: packets forwarded through this port.
    pub packets_forwarded: u64,
    /// Telemetry: packets dropped due to buffer overflow (tail drop).
    pub packets_dropped: u64,
}

impl RouterPort {
    pub fn new(
        port_id: usize,
        mac_address: MacAddress,
        ip_address: Ipv4Address,
        subnet_mask: Ipv4Address,
        max_buffer_bytes: usize,
    ) -> Self {
        Self {
            port_id,
            mac_address,
            ip_address,
            subnet_mask,
            max_buffer_bytes: max_buffer_bytes.max(2048),
            current_buffer_bytes: 0,
            egress_queue: Vec::new(),
            packets_forwarded: 0,
            packets_dropped: 0,
        }
    }

    /// Enqueues a packet into the egress queue, applying tail drop if capacity exceeded.
    pub fn enqueue_packet(&mut self, packet: Vec<u8>) -> bool {
        let size = packet.len();
        if self.current_buffer_bytes + size > self.max_buffer_bytes {
            // Buffer exhaustion: drop packet!
            self.packets_dropped += 1;
            return false;
        }
        self.current_buffer_bytes += size;
        self.egress_queue.push(packet);
        true
    }

    /// Pops the next packet from the egress queue for physical transmission.
    pub fn dequeue_packet(&mut self) -> Option<Vec<u8>> {
        if let Some(pkt) = self.egress_queue.pop() {
            self.current_buffer_bytes = self.current_buffer_bytes.saturating_sub(pkt.len());
            self.packets_forwarded += 1;
            Some(pkt)
        } else {
            None
        }
    }

    /// Instantaneous buffer occupancy ratio $\in [0.0, 1.0]$.
    pub fn buffer_utilization(&self) -> f64 {
        (self.current_buffer_bytes as f64) / (self.max_buffer_bytes as f64)
    }
}

/// Routing Table Entry for IPv4 longest-prefix matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteEntry {
    pub destination_network: Ipv4Address,
    pub subnet_mask: Ipv4Address,
    pub next_hop: Option<Ipv4Address>,
    pub output_port: usize,
}

/// Discrete Packet Switching Router.
#[derive(Debug, Clone)]
pub struct RouterSwitch {
    /// Router hostname / identifier.
    pub name: String,
    /// Physical ports.
    pub ports: Vec<RouterPort>,
    /// Forwarding routing table.
    pub routing_table: Vec<RouteEntry>,
    /// ARP resolution table.
    pub arp_table: ArpTable,
    /// Switching throughput limit in Megabits per second.
    pub switching_capacity_mbps: f64,
}

impl RouterSwitch {
    pub fn new(name: impl Into<String>, switching_capacity_mbps: f64) -> Self {
        Self {
            name: name.into(),
            ports: Vec::new(),
            routing_table: Vec::new(),
            arp_table: ArpTable::new(),
            switching_capacity_mbps: switching_capacity_mbps.max(10.0),
        }
    }

    /// Adds a port to the router.
    pub fn add_port(&mut self, port: RouterPort) -> usize {
        let id = port.port_id;
        self.ports.push(port);
        id
    }

    /// Adds a route to the forwarding table.
    pub fn add_route(
        &mut self,
        destination_network: Ipv4Address,
        subnet_mask: Ipv4Address,
        next_hop: Option<Ipv4Address>,
        output_port: usize,
    ) {
        self.routing_table.push(RouteEntry {
            destination_network,
            subnet_mask,
            next_hop,
            output_port,
        });
    }

    /// Performs longest prefix match forwarding lookup for destination IP.
    pub fn lookup_route(&self, dst_ip: Ipv4Address) -> Option<&RouteEntry> {
        let mut best_match: Option<&RouteEntry> = None;
        let mut best_mask_len = -1i32;

        for entry in &self.routing_table {
            if dst_ip.matches_subnet(entry.destination_network, entry.subnet_mask) {
                // Count mask bits
                let mask_u32 = u32::from_be_bytes(entry.subnet_mask.0);
                let mask_len = mask_u32.count_ones() as i32;
                if mask_len > best_mask_len {
                    best_mask_len = mask_len;
                    best_match = Some(entry);
                }
            }
        }
        best_match
    }

    /// Ingests an incoming raw IPv4 frame received on a specific port and routes it.
    ///
    /// Validates IP header, decrements TTL, recomputes checksum, and enqueues to egress port.
    pub fn route_incoming_packet(&mut self, _ingress_port: usize, raw_packet: Vec<u8>) -> bool {
        if let Some((mut hdr, payload)) = Ipv4Header::parse(&raw_packet) {
            if hdr.ttl <= 1 {
                // TTL expired: drop packet
                return false;
            }

            // Decrement TTL
            hdr.ttl -= 1;
            hdr.checksum = hdr.calculate_checksum();

            // Reassemble packet with updated header
            let mut updated_packet = hdr.serialize_header();
            updated_packet.extend_from_slice(payload);

            // Forwarding decision
            if let Some(route) = self.lookup_route(hdr.dst_ip) {
                let out_port_id = route.output_port;
                if let Some(port) = self.ports.iter_mut().find(|p| p.port_id == out_port_id) {
                    return port.enqueue_packet(updated_packet);
                }
            }
        }
        false
    }
}
