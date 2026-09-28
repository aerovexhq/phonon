//! Simulated CPU Execution Node & MMIO NIC Driver
//!
//! Provides a cycle-stepped simulated CPU execution datapath with 32-bit registers,
//! RAM memory controller, memory-mapped I/O decoding for the Virtual NIC, and an
//! Interrupt Service Routine (ISR) triggered by network hardware events.

use crate::em::MacAddress;
use crate::net::nic::VirtualNic;
use crate::net::stack::{IpProtocol, Ipv4Address, Ipv4Header, UdpDatagram};

/// Memory map base addresses.
pub const RAM_BASE: u32 = 0x0000_0000;
pub const RAM_SIZE: usize = 65536; // 64 KB data/code RAM
pub const NIC_MMIO_BASE: u32 = 0x4000_0000;
pub const NIC_MMIO_SIZE: u32 = 0x1000; // 4 KB MMIO window

/// High-level CPU Instruction representation for simulated execution programs.
#[derive(Debug, Clone)]
pub enum CpuInstruction {
    /// NOP instruction: spends 1 cycle.
    Nop,
    /// Transmit UDP packet: constructs IPv4/UDP packet and writes to NIC MMIO.
    SendUdp {
        dst_ip: Ipv4Address,
        src_port: u16,
        dst_port: u16,
        payload: Vec<u8>,
    },
    /// Await and process received packet in ISR.
    ProcessRxPacket,
}

/// Simulated CPU Execution Node.
#[derive(Debug, Clone)]
pub struct SimulatedCpuNode {
    /// Unique node ID.
    pub node_id: usize,
    /// Node human-readable hostname (e.g. "CPU-A", "CPU-B").
    pub name: String,
    /// 32-bit integer register file (R0..R31, where R0 is always 0).
    pub registers: [u32; 32],
    /// Program Counter (PC).
    pub pc: u32,
    /// Data & instruction RAM.
    pub ram: [u8; RAM_SIZE],
    /// Memory-mapped Virtual NIC.
    pub nic: VirtualNic,
    /// Clock cycles elapsed.
    pub clock_cycles: u64,
    /// Instructions retired.
    pub instructions_executed: u64,
    /// Program instruction memory.
    pub program: Vec<CpuInstruction>,
    /// Last received application payload extracted from packet.
    pub last_received_payload: Option<Vec<u8>>,
    /// Counter of handled interrupts.
    pub irq_count: u64,
}

impl SimulatedCpuNode {
    pub fn new(node_id: usize, name: impl Into<String>, mac: MacAddress, ip: [u8; 4]) -> Self {
        Self {
            node_id,
            name: name.into(),
            registers: [0u32; 32],
            pc: 0,
            ram: [0u8; RAM_SIZE],
            nic: VirtualNic::new(mac, ip),
            clock_cycles: 0,
            instructions_executed: 0,
            program: Vec::new(),
            last_received_payload: None,
            irq_count: 0,
        }
    }

    /// Loads a high-level instruction program into the CPU execution queue.
    pub fn load_program(&mut self, program: Vec<CpuInstruction>) {
        self.program = program;
        self.pc = 0;
    }

    /// Reads a 32-bit word from the CPU address bus (dispatching RAM or NIC MMIO).
    pub fn read_u32(&self, addr: u32) -> u32 {
        if (NIC_MMIO_BASE..NIC_MMIO_BASE + NIC_MMIO_SIZE).contains(&addr) {
            self.nic.read_u32(addr - NIC_MMIO_BASE)
        } else if (addr as usize) + 4 <= RAM_SIZE {
            let offset = addr as usize;
            u32::from_le_bytes([
                self.ram[offset],
                self.ram[offset + 1],
                self.ram[offset + 2],
                self.ram[offset + 3],
            ])
        } else {
            0
        }
    }

    /// Writes a 32-bit word onto the CPU address bus.
    pub fn write_u32(&mut self, addr: u32, val: u32) {
        if (NIC_MMIO_BASE..NIC_MMIO_BASE + NIC_MMIO_SIZE).contains(&addr) {
            self.nic.write_u32(addr - NIC_MMIO_BASE, val);
        } else if (addr as usize) + 4 <= RAM_SIZE {
            let offset = addr as usize;
            self.ram[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
        }
    }

    /// Encapsulates application data into an IPv4/UDP packet and transfers via MMIO to NIC.
    pub fn send_udp_datagram(
        &mut self,
        dst_ip: Ipv4Address,
        src_port: u16,
        dst_port: u16,
        payload: &[u8],
    ) {
        let udp = UdpDatagram::new(src_port, dst_port, payload.to_vec());
        let udp_bytes = udp.serialize();
        let src_ip = Ipv4Address(self.nic.ip_address);
        let ip_hdr = Ipv4Header::new(src_ip, dst_ip, IpProtocol::Udp, udp_bytes.len());

        let mut full_packet = ip_hdr.serialize_header();
        full_packet.extend_from_slice(&udp_bytes);

        // Write into NIC TX buffer and initiate transmission
        self.nic.write_tx_bytes(0, &full_packet);
        self.nic.trigger_tx();
    }

    /// Steps the CPU by one clock cycle.
    pub fn step_cycle(&mut self) {
        self.clock_cycles += 1;

        // Check hardware interrupt line from NIC
        if self.nic.irq_asserted {
            self.handle_network_irq();
        }

        // Fetch and execute next program instruction
        if (self.pc as usize) < self.program.len() {
            let inst = self.program[self.pc as usize].clone();
            self.pc += 1;
            self.instructions_executed += 1;

            match inst {
                CpuInstruction::Nop => {}
                CpuInstruction::SendUdp {
                    dst_ip,
                    src_port,
                    dst_port,
                    payload,
                } => {
                    self.send_udp_datagram(dst_ip, src_port, dst_port, &payload);
                }
                CpuInstruction::ProcessRxPacket => {
                    self.handle_network_irq();
                }
            }
        }
    }

    /// Interrupt Service Routine (ISR) triggered by packet arrival in NIC RX FIFO.
    pub fn handle_network_irq(&mut self) {
        if self.nic.rx_len > 0 {
            self.irq_count += 1;
            let rx_bytes = self.nic.read_rx_bytes();
            if let Some((_ip_hdr, ip_payload)) = Ipv4Header::parse(&rx_bytes) {
                if let Some(udp) = UdpDatagram::parse(ip_payload) {
                    self.last_received_payload = Some(udp.payload);
                }
            }
            // Acknowledge interrupt and free RX FIFO
            self.nic.acknowledge_rx();
        }
    }
}
