//! Memory-Mapped Virtual Network Interface Controller (Virtual NIC)
//!
//! Provides hardware-level MMIO registers, FIFO ring buffers, DMA transfers,
//! and interrupt generation coupled directly to simulated CPU instruction pipelines.

use crate::em::MacAddress;

/// MMIO Register Offsets from NIC Base Address (e.g. 0x4000_0000).
pub mod nic_reg {
    /// Status Register (Read-Only):
    /// Bit 0: TX_READY (1 = transmitter ready for new packet)
    /// Bit 1: RX_READY (1 = received packet available in RX buffer)
    /// Bit 2: IRQ_ASSERTED (1 = interrupt currently asserted)
    /// Bit 3: TX_BUSY (1 = currently transmitting over the air)
    pub const STATUS: u32 = 0x00;

    /// Control Register (Write-Only):
    /// Bit 0: TX_START (pulse to start transmission of TX buffer)
    /// Bit 1: RX_ACK (clear RX_READY and de-assert IRQ)
    /// Bit 2: IRQ_ENABLE (enable interrupt generation on packet receive)
    /// Bit 3: RESET (soft reset NIC registers and FIFOs)
    pub const CONTROL: u32 = 0x04;

    /// Transmit Packet Length in Bytes (Read/Write, max 2032 bytes).
    pub const TX_LEN: u32 = 0x08;

    /// Received Packet Length in Bytes (Read-Only).
    pub const RX_LEN: u32 = 0x0C;

    /// Transmit FIFO Buffer Range (0x0010 .. 0x07FF, 2032 bytes).
    pub const TX_BUF_START: u32 = 0x10;
    pub const TX_BUF_END: u32 = 0x7FF;

    /// Receive FIFO Buffer Range (0x0800 .. 0x0FFF, 2048 bytes).
    pub const RX_BUF_START: u32 = 0x800;
    pub const RX_BUF_END: u32 = 0xFFF;
}

pub const STATUS_TX_READY: u32 = 1 << 0;
pub const STATUS_RX_READY: u32 = 1 << 1;
pub const STATUS_IRQ_ASSERTED: u32 = 1 << 2;
pub const STATUS_TX_BUSY: u32 = 1 << 3;

pub const CTRL_TX_START: u32 = 1 << 0;
pub const CTRL_RX_ACK: u32 = 1 << 1;
pub const CTRL_IRQ_ENABLE: u32 = 1 << 2;
pub const CTRL_RESET: u32 = 1 << 3;
pub const CTRL_IRQ_DISABLE: u32 = 1 << 4;

pub const TX_BUF_CAPACITY: usize = 2032;
pub const RX_BUF_CAPACITY: usize = 2048;

/// Virtual Network Interface Controller (NIC) with Memory-Mapped I/O.
#[derive(Debug, Clone)]
pub struct VirtualNic {
    /// Hardware 48-bit IEEE 802 MAC address.
    pub mac_address: MacAddress,
    /// Assigned IPv4 address.
    pub ip_address: [u8; 4],
    /// Transmit buffer SRAM.
    tx_buffer: [u8; TX_BUF_CAPACITY],
    /// Receive buffer SRAM.
    rx_buffer: [u8; RX_BUF_CAPACITY],
    /// Length of packet to transmit.
    pub tx_len: u32,
    /// Length of packet currently stored in RX buffer.
    pub rx_len: u32,
    /// Interrupt enable flag.
    pub irq_enabled: bool,
    /// Active interrupt line connected to CPU interrupt controller.
    pub irq_asserted: bool,
    /// Currently transmitting flag.
    pub tx_busy: bool,
    /// Completed outbound packets ready for physical layer dispatch.
    pub tx_outbound_queue: Vec<Vec<u8>>,
    /// Telemetry: total packets transmitted.
    pub packets_transmitted: u64,
    /// Telemetry: total packets received.
    pub packets_received: u64,
    /// Telemetry: total packet dropped due to buffer overflow.
    pub packets_dropped: u64,
}

impl VirtualNic {
    /// Creates a new Virtual NIC with specified MAC and IP addresses.
    pub fn new(mac_address: MacAddress, ip_address: [u8; 4]) -> Self {
        Self {
            mac_address,
            ip_address,
            tx_buffer: [0u8; TX_BUF_CAPACITY],
            rx_buffer: [0u8; RX_BUF_CAPACITY],
            tx_len: 0,
            rx_len: 0,
            irq_enabled: true,
            irq_asserted: false,
            tx_busy: false,
            tx_outbound_queue: Vec::new(),
            packets_transmitted: 0,
            packets_received: 0,
            packets_dropped: 0,
        }
    }

    /// Evaluates current Status Register value.
    pub fn status_reg(&self) -> u32 {
        let mut st = 0u32;
        if !self.tx_busy {
            st |= STATUS_TX_READY;
        } else {
            st |= STATUS_TX_BUSY;
        }
        if self.rx_len > 0 {
            st |= STATUS_RX_READY;
        }
        if self.irq_asserted {
            st |= STATUS_IRQ_ASSERTED;
        }
        st
    }

    /// Reads a 32-bit word from MMIO address space.
    pub fn read_u32(&self, offset: u32) -> u32 {
        match offset {
            nic_reg::STATUS => self.status_reg(),
            nic_reg::CONTROL => {
                if self.irq_enabled {
                    CTRL_IRQ_ENABLE
                } else {
                    0
                }
            }
            nic_reg::TX_LEN => self.tx_len,
            nic_reg::RX_LEN => self.rx_len,
            addr if (nic_reg::TX_BUF_START..=nic_reg::TX_BUF_END).contains(&addr) => {
                let rel = (addr - nic_reg::TX_BUF_START) as usize;
                if rel + 4 <= TX_BUF_CAPACITY {
                    u32::from_le_bytes([
                        self.tx_buffer[rel],
                        self.tx_buffer[rel + 1],
                        self.tx_buffer[rel + 2],
                        self.tx_buffer[rel + 3],
                    ])
                } else {
                    0
                }
            }
            addr if (nic_reg::RX_BUF_START..=nic_reg::RX_BUF_END).contains(&addr) => {
                let rel = (addr - nic_reg::RX_BUF_START) as usize;
                if rel + 4 <= RX_BUF_CAPACITY {
                    u32::from_le_bytes([
                        self.rx_buffer[rel],
                        self.rx_buffer[rel + 1],
                        self.rx_buffer[rel + 2],
                        self.rx_buffer[rel + 3],
                    ])
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// Writes a 32-bit word into MMIO address space.
    pub fn write_u32(&mut self, offset: u32, val: u32) {
        match offset {
            nic_reg::CONTROL => {
                if val & CTRL_RESET != 0 {
                    self.reset();
                    return;
                }
                if val & CTRL_IRQ_ENABLE != 0 {
                    self.irq_enabled = true;
                }
                if val & CTRL_IRQ_DISABLE != 0 {
                    self.irq_enabled = false;
                    self.irq_asserted = false;
                }
                if val & CTRL_RX_ACK != 0 {
                    self.acknowledge_rx();
                }
                if val & CTRL_TX_START != 0 {
                    self.trigger_tx();
                }
            }
            nic_reg::TX_LEN => {
                self.tx_len = val.min(TX_BUF_CAPACITY as u32);
            }
            addr if (nic_reg::TX_BUF_START..=nic_reg::TX_BUF_END).contains(&addr) => {
                let rel = (addr - nic_reg::TX_BUF_START) as usize;
                if rel + 4 <= TX_BUF_CAPACITY {
                    let bytes = val.to_le_bytes();
                    self.tx_buffer[rel..rel + 4].copy_from_slice(&bytes);
                }
            }
            _ => {}
        }
    }

    /// Writes a slice of raw bytes into the TX FIFO buffer.
    pub fn write_tx_bytes(&mut self, offset: usize, bytes: &[u8]) {
        let end = (offset + bytes.len()).min(TX_BUF_CAPACITY);
        if offset < end {
            self.tx_buffer[offset..end].copy_from_slice(&bytes[..end - offset]);
            self.tx_len = self.tx_len.max(end as u32);
        }
    }

    /// Reads received bytes from the RX FIFO buffer.
    pub fn read_rx_bytes(&self) -> Vec<u8> {
        let len = (self.rx_len as usize).min(RX_BUF_CAPACITY);
        self.rx_buffer[..len].to_vec()
    }

    /// Triggers packet transmission from the TX buffer.
    pub fn trigger_tx(&mut self) {
        let len = (self.tx_len as usize).min(TX_BUF_CAPACITY);
        if len > 0 {
            let packet = self.tx_buffer[..len].to_vec();
            self.tx_outbound_queue.push(packet);
            self.packets_transmitted += 1;
            self.tx_len = 0;
            self.tx_busy = false;
        }
    }

    /// Delivers an incoming packet into the NIC RX FIFO buffer.
    ///
    /// Sets `RX_READY`, updates `rx_len`, and asserts hardware interrupt to the CPU.
    pub fn receive_packet(&mut self, packet: &[u8]) -> bool {
        if self.rx_len > 0 {
            // Buffer already occupied: drop packet (RX overflow)
            self.packets_dropped += 1;
            return false;
        }
        let copy_len = packet.len().min(RX_BUF_CAPACITY);
        self.rx_buffer[..copy_len].copy_from_slice(&packet[..copy_len]);
        self.rx_len = copy_len as u32;
        self.packets_received += 1;

        if self.irq_enabled {
            self.irq_asserted = true;
        }
        true
    }

    /// Acknowledges reception, freeing the RX FIFO buffer and lowering the IRQ line.
    pub fn acknowledge_rx(&mut self) {
        self.rx_len = 0;
        self.irq_asserted = false;
    }

    /// Soft resets the Virtual NIC.
    pub fn reset(&mut self) {
        self.tx_buffer.fill(0);
        self.rx_buffer.fill(0);
        self.tx_len = 0;
        self.rx_len = 0;
        self.irq_asserted = false;
        self.tx_busy = false;
        self.tx_outbound_queue.clear();
    }
}
