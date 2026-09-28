//! Synchronized CPU-to-Router Co-Simulation Engine & Multi-Physics Physical RF Coupling
//!
//! Synchronizes CPU cycle-stepped instruction pipelines, memory-mapped NIC FIFOs,
//! discrete Router packet switching queues, and physical over-the-air RF propagation.

use crate::em::rf_tier_engine::RfRealismTier;
use crate::em::wifi_protocol_solver::WifiLinkSimulator;
use phonon_models::em::{ChannelRng, DielectricWall, MacAddress, ModulationScheme, Vector3D};
use phonon_models::net::cpu_node::SimulatedCpuNode;
use phonon_models::net::router::{RouterPort, RouterSwitch};
use phonon_models::net::stack::Ipv4Address;

/// Report produced on each co-simulation tick.
#[derive(Debug, Clone, PartialEq)]
pub struct CoSimStepReport {
    /// CPU-A total clock cycles.
    pub cpu_a_cycles: u64,
    /// CPU-B total clock cycles.
    pub cpu_b_cycles: u64,
    /// Number of packets transmitted over Hop 1 (CPU-A -> Router).
    pub hop1_packets_sent: usize,
    /// Number of packets successfully received by Router on Hop 1.
    pub hop1_packets_delivered: usize,
    /// Number of packets transmitted over Hop 2 (Router -> CPU-B).
    pub hop2_packets_sent: usize,
    /// Number of packets successfully delivered to CPU-B on Hop 2.
    pub hop2_packets_delivered: usize,
    /// Router Port 1 egress queue utilization ratio [0.0, 1.0].
    pub router_egress_utilization: f64,
    /// True if CPU-B received the application payload during this step.
    pub payload_received_by_cpu_b: bool,
}

/// End-to-End Network Co-Simulator coupling dual CPUs, Router switch, and RF wireless links.
#[derive(Debug, Clone)]
pub struct NetworkCoSimulator {
    /// Transmitting simulated CPU node (CPU-A).
    pub cpu_a: SimulatedCpuNode,
    /// Receiving simulated CPU node (CPU-B).
    pub cpu_b: SimulatedCpuNode,
    /// Central discrete packet switching Router.
    pub router: RouterSwitch,
    /// 3D position of CPU-A in meters.
    pub pos_cpu_a: Vector3D,
    /// 3D position of Router in meters.
    pub pos_router: Vector3D,
    /// 3D position of CPU-B in meters.
    pub pos_cpu_b: Vector3D,
    /// Hop 1 RF wireless link: CPU-A <-> Router.
    pub link_hop1: WifiLinkSimulator,
    /// Hop 2 RF wireless link: Router <-> CPU-B.
    pub link_hop2: WifiLinkSimulator,
}

impl NetworkCoSimulator {
    /// Creates a complete end-to-end co-simulation environment with dual CPUs and an intermediate Router.
    pub fn new(
        pos_cpu_a: Vector3D,
        pos_router: Vector3D,
        pos_cpu_b: Vector3D,
        tier: RfRealismTier,
        modulation: ModulationScheme,
    ) -> Self {
        let mac_a = MacAddress([0x00, 0x11, 0x22, 0x33, 0x01, 0x01]);
        let mac_router_p0 = MacAddress([0x00, 0x11, 0x22, 0x33, 0x00, 0x01]);
        let mac_router_p1 = MacAddress([0x00, 0x11, 0x22, 0x33, 0x00, 0x02]);
        let mac_b = MacAddress([0x00, 0x11, 0x22, 0x33, 0x02, 0x01]);

        let ip_a = [192, 168, 1, 10];
        let ip_router_p0 = Ipv4Address::new(192, 168, 1, 1);
        let ip_router_p1 = Ipv4Address::new(192, 168, 2, 1);
        let ip_b = [192, 168, 2, 20];
        let netmask = Ipv4Address::new(255, 255, 255, 0);

        let cpu_a = SimulatedCpuNode::new(1, "CPU-A", mac_a, ip_a);
        let cpu_b = SimulatedCpuNode::new(2, "CPU-B", mac_b, ip_b);

        // Configure Router Switch with two ports (Port 0: Subnet 1, Port 1: Subnet 2)
        let mut router = RouterSwitch::new("Gateway-Router", 1000.0);
        let port0 = RouterPort::new(0, mac_router_p0, ip_router_p0, netmask, 65536);
        let port1 = RouterPort::new(1, mac_router_p1, ip_router_p1, netmask, 65536);
        router.add_port(port0);
        router.add_port(port1);

        // Routing table: Subnet 192.168.1.0/24 -> Port 0; Subnet 192.168.2.0/24 -> Port 1
        router.add_route(Ipv4Address::new(192, 168, 1, 0), netmask, None, 0);
        router.add_route(Ipv4Address::new(192, 168, 2, 0), netmask, None, 1);

        // Hop 1 RF wireless link: CPU-A <-> Router Port 0
        let mut link_hop1 =
            WifiLinkSimulator::new(mac_a, mac_router_p0, pos_cpu_a, pos_router, modulation);
        link_hop1.set_tier(tier);

        // Hop 2 RF wireless link: Router Port 1 <-> CPU-B
        let mut link_hop2 =
            WifiLinkSimulator::new(mac_router_p1, mac_b, pos_router, pos_cpu_b, modulation);
        link_hop2.set_tier(tier);

        Self {
            cpu_a,
            cpu_b,
            router,
            pos_cpu_a,
            pos_router,
            pos_cpu_b,
            link_hop1,
            link_hop2,
        }
    }

    /// Adds a physical dielectric wall to the propagation environment.
    pub fn add_wall(&mut self, wall: DielectricWall) {
        self.link_hop1.add_wall(wall.clone());
        self.link_hop2.add_wall(wall);
    }

    /// Advances the co-simulation environment by one discrete cycle.
    pub fn step(&mut self, rng: &mut ChannelRng) -> CoSimStepReport {
        // 1. Step CPU-A instruction datapath
        self.cpu_a.step_cycle();

        let mut hop1_sent = 0;
        let mut hop1_delivered = 0;
        let mut hop2_sent = 0;
        let mut hop2_delivered = 0;

        // 2. Process CPU-A Virtual NIC outbound packets over Hop 1 RF link
        while let Some(raw_packet) = self.cpu_a.nic.tx_outbound_queue.pop() {
            hop1_sent += 1;
            // Transmit packet through physical multi-tier RF wireless channel
            let tx_result = self.link_hop1.transmit_payload(&raw_packet, rng);
            if tx_result.success {
                hop1_delivered += 1;
                // Deliver to Router Port 0
                self.router.route_incoming_packet(0, raw_packet);
            }
        }

        // 3. Process Router egress queues over Hop 2 RF link
        // Drain Router Port 1 egress queue
        if let Some(port) = self.router.ports.iter_mut().find(|p| p.port_id == 1) {
            while let Some(packet_to_b) = port.dequeue_packet() {
                hop2_sent += 1;
                // Transmit through Hop 2 physical RF wireless channel
                let tx_result = self.link_hop2.transmit_payload(&packet_to_b, rng);
                if tx_result.success {
                    hop2_delivered += 1;
                    // Deliver into CPU-B Virtual NIC RX buffer and trigger hardware IRQ
                    self.cpu_b.nic.receive_packet(&packet_to_b);
                }
            }
        }

        // 4. Step CPU-B instruction datapath (handles network IRQ and processes payload)
        self.cpu_b.step_cycle();

        let router_util = self
            .router
            .ports
            .iter()
            .find(|p| p.port_id == 1)
            .map(|p| p.buffer_utilization())
            .unwrap_or(0.0);

        let received = self.cpu_b.last_received_payload.is_some();

        CoSimStepReport {
            cpu_a_cycles: self.cpu_a.clock_cycles,
            cpu_b_cycles: self.cpu_b.clock_cycles,
            hop1_packets_sent: hop1_sent,
            hop1_packets_delivered: hop1_delivered,
            hop2_packets_sent: hop2_sent,
            hop2_packets_delivered: hop2_delivered,
            router_egress_utilization: router_util,
            payload_received_by_cpu_b: received,
        }
    }
}
