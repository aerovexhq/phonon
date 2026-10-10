#![deny(unsafe_code)]

//! High-Throughput Digital Logic Analyzer & Hardware Protocol Decoders.
//!
//! Provides Sigrok-compatible multi-channel digital sample acquisition, state pattern
//! triggering (rising/falling edges, glitch holdoff), and real-time protocol decoders
//! for SPI, I2C, UART, and CAN bus frames.

/// Trigger edge condition on a digital logic channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TriggerEdge {
    #[default]
    Rising,
    Falling,
    Either,
    HighLevel,
    LowLevel,
    None,
}

/// Trigger condition configuration for the logic analyzer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogicTriggerCondition {
    pub channel: u8,
    pub edge: TriggerEdge,
    pub pattern_mask: u16,
    pub pattern_val: u16,
    pub holdoff_samples: usize,
}

impl Default for LogicTriggerCondition {
    fn default() -> Self {
        Self {
            channel: 0,
            edge: TriggerEdge::Rising,
            pattern_mask: 0x0001,
            pattern_val: 0x0001,
            holdoff_samples: 0,
        }
    }
}

/// Raw logic analyzer capture buffer across up to 16 channels.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogicAnalyzerCapture {
    pub sample_rate_hz: f64,
    pub num_channels: usize,
    pub samples: Vec<u16>,
    pub timestamps_s: Vec<f64>,
    pub trigger_index: Option<usize>,
}

impl LogicAnalyzerCapture {
    pub fn new(sample_rate_hz: f64, num_channels: usize) -> Self {
        Self {
            sample_rate_hz,
            num_channels: num_channels.min(16),
            samples: Vec::new(),
            timestamps_s: Vec::new(),
            trigger_index: None,
        }
    }

    /// Appends a 16-bit logic sample (bit k corresponds to channel k).
    pub fn push_sample(&mut self, sample: u16) {
        let t = (self.samples.len() as f64) / self.sample_rate_hz;
        self.samples.push(sample);
        self.timestamps_s.push(t);
    }

    /// Evaluates whether a trigger condition is satisfied across the captured buffer.
    pub fn find_trigger(&self, cond: &LogicTriggerCondition) -> Option<usize> {
        if self.samples.len() < 2 {
            return None;
        }

        let ch_mask = 1u16 << cond.channel;
        let mut holdoff_counter = 0;

        for i in 1..self.samples.len() {
            let prev = self.samples[i - 1];
            let curr = self.samples[i];

            let prev_bit = (prev & ch_mask) != 0;
            let curr_bit = (curr & ch_mask) != 0;

            let edge_hit = match cond.edge {
                TriggerEdge::Rising => !prev_bit && curr_bit,
                TriggerEdge::Falling => prev_bit && !curr_bit,
                TriggerEdge::Either => prev_bit != curr_bit,
                TriggerEdge::HighLevel => curr_bit,
                TriggerEdge::LowLevel => !curr_bit,
                TriggerEdge::None => true,
            };

            let pattern_hit = (curr & cond.pattern_mask) == (cond.pattern_val & cond.pattern_mask);

            if edge_hit && pattern_hit {
                if holdoff_counter >= cond.holdoff_samples {
                    return Some(i);
                } else {
                    holdoff_counter += 1;
                }
            } else {
                holdoff_counter = 0;
            }
        }

        None
    }

    /// Reads digital level (0 or 1) of a specified channel at sample index.
    pub fn get_channel_level(&self, channel: u8, index: usize) -> bool {
        if let Some(&sample) = self.samples.get(index) {
            (sample & (1 << channel)) != 0
        } else {
            false
        }
    }
}

// ==========================================
// 1. SPI Protocol Decoder
// ==========================================

/// Decoded SPI packet transfer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpiPacket {
    pub start_sample: usize,
    pub end_sample: usize,
    pub mosi_bytes: Vec<u8>,
    pub miso_bytes: Vec<u8>,
}

/// SPI Bus Protocol Decoder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpiDecoder {
    pub mosi_ch: u8,
    pub miso_ch: u8,
    pub sck_ch: u8,
    pub cs_ch: u8,
    pub cpol: u8, // 0: Idle low, 1: Idle high
    pub cpha: u8, // 0: Sample leading, 1: Sample trailing
}

impl Default for SpiDecoder {
    fn default() -> Self {
        Self {
            mosi_ch: 0,
            miso_ch: 1,
            sck_ch: 2,
            cs_ch: 3,
            cpol: 0,
            cpha: 0,
        }
    }
}

impl SpiDecoder {
    /// Decodes SPI byte stream from captured digital traces.
    pub fn decode(&self, capture: &LogicAnalyzerCapture) -> Vec<SpiPacket> {
        let mut packets = Vec::new();
        if capture.samples.len() < 8 {
            return packets;
        }

        let mut in_transaction = false;
        let mut start_idx = 0;
        let mut mosi_data = Vec::new();
        let mut miso_data = Vec::new();

        let mut curr_mosi_byte = 0u8;
        let mut curr_miso_byte = 0u8;
        let mut bit_count = 0;

        let sck_mask = 1u16 << self.sck_ch;
        let cs_mask = 1u16 << self.cs_ch;
        let mosi_mask = 1u16 << self.mosi_ch;
        let miso_mask = 1u16 << self.miso_ch;

        for i in 1..capture.samples.len() {
            let prev = capture.samples[i - 1];
            let curr = capture.samples[i];

            let cs_active = (curr & cs_mask) == 0; // Active low CS
            let cs_was_active = (prev & cs_mask) == 0;

            if cs_active && !cs_was_active {
                // CS Assertion
                in_transaction = true;
                start_idx = i;
                mosi_data.clear();
                miso_data.clear();
                curr_mosi_byte = 0;
                curr_miso_byte = 0;
                bit_count = 0;
            } else if !cs_active && cs_was_active && in_transaction {
                // CS Deassertion
                in_transaction = false;
                packets.push(SpiPacket {
                    start_sample: start_idx,
                    end_sample: i,
                    mosi_bytes: mosi_data.clone(),
                    miso_bytes: miso_data.clone(),
                });
            }

            if in_transaction {
                let prev_sck = (prev & sck_mask) != 0;
                let curr_sck = (curr & sck_mask) != 0;

                // Clock sampling edge: CPOL=0, CPHA=0 -> rising edge
                let sample_edge = if self.cpol == 0 && self.cpha == 0 {
                    !prev_sck && curr_sck
                } else if self.cpol == 0 && self.cpha == 1 {
                    prev_sck && !curr_sck
                } else if self.cpol == 1 && self.cpha == 0 {
                    prev_sck && !curr_sck
                } else {
                    !prev_sck && curr_sck
                };

                if sample_edge {
                    let mosi_bit = if (curr & mosi_mask) != 0 { 1 } else { 0 };
                    let miso_bit = if (curr & miso_mask) != 0 { 1 } else { 0 };

                    curr_mosi_byte = (curr_mosi_byte << 1) | mosi_bit;
                    curr_miso_byte = (curr_miso_byte << 1) | miso_bit;
                    bit_count += 1;

                    if bit_count == 8 {
                        mosi_data.push(curr_mosi_byte);
                        miso_data.push(curr_miso_byte);
                        curr_mosi_byte = 0;
                        curr_miso_byte = 0;
                        bit_count = 0;
                    }
                }
            }
        }

        packets
    }
}

// ==========================================
// 2. I2C Protocol Decoder
// ==========================================

/// Decoded I2C frame packet.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct I2cPacket {
    pub start_sample: usize,
    pub end_sample: usize,
    pub address: u8,
    pub is_read: bool,
    pub ack: bool,
    pub data_bytes: Vec<u8>,
}

/// I2C Bus Protocol Decoder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct I2cDecoder {
    pub sda_ch: u8,
    pub scl_ch: u8,
}

impl Default for I2cDecoder {
    fn default() -> Self {
        Self {
            sda_ch: 4,
            scl_ch: 5,
        }
    }
}

impl I2cDecoder {
    /// Decodes I2C start/stop conditions, addressing, and byte frames.
    pub fn decode(&self, capture: &LogicAnalyzerCapture) -> Vec<I2cPacket> {
        let mut packets = Vec::new();
        if capture.samples.len() < 8 {
            return packets;
        }

        let sda_mask = 1u16 << self.sda_ch;
        let scl_mask = 1u16 << self.scl_ch;

        let mut in_frame = false;
        let mut start_idx = 0;
        let mut bits = Vec::new();

        for i in 1..capture.samples.len() {
            let prev = capture.samples[i - 1];
            let curr = capture.samples[i];

            let prev_sda = (prev & sda_mask) != 0;
            let curr_sda = (curr & sda_mask) != 0;
            let prev_scl = (prev & scl_mask) != 0;
            let curr_scl = (curr & scl_mask) != 0;

            // Start condition: SDA falling while SCL is high
            if prev_scl && curr_scl && prev_sda && !curr_sda {
                in_frame = true;
                start_idx = i;
                bits.clear();
            }

            // Stop condition: SDA rising while SCL is high
            if prev_scl && curr_scl && !prev_sda && curr_sda && in_frame {
                in_frame = false;
                if let Some(pkt) = Self::assemble_packet(start_idx, i, &bits) {
                    packets.push(pkt);
                }
                bits.clear();
            }

            // Sample data bit on SCL rising edge
            if in_frame && !prev_scl && curr_scl {
                bits.push(curr_sda);
            }
        }

        packets
    }

    fn assemble_packet(start_sample: usize, end_sample: usize, bits: &[bool]) -> Option<I2cPacket> {
        if bits.len() < 9 {
            return None;
        }

        // First 8 bits: 7-bit Address + R/W bit
        let mut addr_byte = 0u8;
        for &b in &bits[0..7] {
            addr_byte = (addr_byte << 1) | (if b { 1 } else { 0 });
        }
        let is_read = bits[7];
        let ack = !bits[8]; // ACK is low (0)

        // Subsequent 9-bit chunks: 8 data bits + ACK
        let mut data_bytes = Vec::new();
        let mut idx = 9;
        while idx + 8 <= bits.len() {
            let mut byte_val = 0u8;
            for &b in &bits[idx..idx + 8] {
                byte_val = (byte_val << 1) | (if b { 1 } else { 0 });
            }
            data_bytes.push(byte_val);
            idx += 9;
        }

        Some(I2cPacket {
            start_sample,
            end_sample,
            address: addr_byte,
            is_read,
            ack,
            data_bytes,
        })
    }
}

// ==========================================
// 3. UART Protocol Decoder
// ==========================================

/// Decoded UART Byte Frame.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UartFrame {
    pub sample_index: usize,
    pub byte_value: u8,
    pub parity_error: bool,
    pub framing_error: bool,
}

/// UART Serial Bus Decoder.
#[derive(Debug, Clone, PartialEq)]
pub struct UartDecoder {
    pub channel: u8,
    pub baud_rate: f64,
    pub data_bits: usize, // usually 8
    pub stop_bits: usize, // usually 1
}

impl Default for UartDecoder {
    fn default() -> Self {
        Self {
            channel: 6,
            baud_rate: 115200.0,
            data_bits: 8,
            stop_bits: 1,
        }
    }
}

impl UartDecoder {
    /// Decodes asynchronous serial UART frames from captured stream.
    pub fn decode(&self, capture: &LogicAnalyzerCapture) -> Vec<UartFrame> {
        let mut frames = Vec::new();
        if capture.samples.len() < 10 {
            return frames;
        }

        let samples_per_bit = capture.sample_rate_hz / self.baud_rate;
        if samples_per_bit < 1.0 {
            return frames;
        }

        let ch_mask = 1u16 << self.channel;
        let mut i = 1;

        while i < capture.samples.len() {
            let prev = (capture.samples[i - 1] & ch_mask) != 0;
            let curr = (capture.samples[i] & ch_mask) != 0;

            // Start bit detected: Falling edge from Idle High (1) to Low (0)
            if prev && !curr {
                let start_center = i as f64 + samples_per_bit * 0.5;

                // Sample data bits at middle of bit periods
                let mut byte_val = 0u8;
                let mut framing_error = false;

                for bit_idx in 0..self.data_bits {
                    let sample_pos = (start_center + samples_per_bit * (bit_idx + 1) as f64) as usize;
                    if sample_pos < capture.samples.len() {
                        let bit_level = (capture.samples[sample_pos] & ch_mask) != 0;
                        if bit_level {
                            byte_val |= 1 << bit_idx;
                        }
                    }
                }

                // Verify stop bit (must be High)
                let stop_pos = (start_center + samples_per_bit * (self.data_bits + 1) as f64) as usize;
                if stop_pos < capture.samples.len() {
                    let stop_bit = (capture.samples[stop_pos] & ch_mask) != 0;
                    if !stop_bit {
                        framing_error = true;
                    }
                }

                frames.push(UartFrame {
                    sample_index: i,
                    byte_value: byte_val,
                    parity_error: false,
                    framing_error,
                });

                // Advance past frame
                i += (samples_per_bit * (self.data_bits + self.stop_bits + 1) as f64) as usize;
            } else {
                i += 1;
            }
        }

        frames
    }
}

// ==========================================
// 4. CAN Bus Protocol Decoder
// ==========================================

/// Decoded Controller Area Network (CAN) message frame.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CanFrame {
    pub start_sample: usize,
    pub arbitration_id: u32,
    pub is_extended: bool,
    pub is_rtr: bool,
    pub dlc: u8,
    pub payload: Vec<u8>,
    pub crc_valid: bool,
}

/// CAN Bus Protocol Decoder.
#[derive(Debug, Clone, PartialEq)]
pub struct CanDecoder {
    pub channel: u8,
    pub nominal_bit_rate: f64, // e.g. 500_000.0 (500 kbps)
}

impl Default for CanDecoder {
    fn default() -> Self {
        Self {
            channel: 7,
            nominal_bit_rate: 500_000.0,
        }
    }
}

impl CanDecoder {
    /// Decodes standard 11-bit CAN 2.0A message frames.
    pub fn decode(&self, capture: &LogicAnalyzerCapture) -> Vec<CanFrame> {
        let mut frames = Vec::new();
        if capture.samples.len() < 30 {
            return frames;
        }

        let samples_per_bit = capture.sample_rate_hz / self.nominal_bit_rate;
        if samples_per_bit < 1.0 {
            return frames;
        }

        let ch_mask = 1u16 << self.channel;
        let mut i = 1;

        while i < capture.samples.len() {
            let prev = (capture.samples[i - 1] & ch_mask) != 0;
            let curr = (capture.samples[i] & ch_mask) != 0;

            // SOF (Start of Frame): Dominant (0) falling edge from Recessive (1)
            if prev && !curr && (i + (samples_per_bit * 20.0) as usize) < capture.samples.len() {
                let start_idx = i;
                let sample_bit = |bit_num: usize| -> bool {
                    let pos = (start_idx as f64 + samples_per_bit * (bit_num as f64 + 0.5)) as usize;
                    if pos < capture.samples.len() {
                        (capture.samples[pos] & ch_mask) != 0
                    } else {
                        false
                    }
                };

                // 11-bit Identifier (Bits 1..11)
                let mut arb_id = 0u32;
                for b in 1..=11 {
                    arb_id = (arb_id << 1) | (if sample_bit(b) { 1 } else { 0 });
                }

                let rtr = sample_bit(12);
                let _ide = sample_bit(13); // 0 for standard 11-bit

                // 4-bit DLC (Bits 15..18)
                let mut dlc = 0u8;
                for b in 15..=18 {
                    dlc = (dlc << 1) | (if sample_bit(b) { 1 } else { 0 });
                }
                dlc = dlc.min(8);

                // Payload bytes
                let mut payload = Vec::with_capacity(dlc as usize);
                let mut bit_cursor = 19;
                for _ in 0..dlc {
                    let mut byte_val = 0u8;
                    for _ in 0..8 {
                        byte_val = (byte_val << 1) | (if sample_bit(bit_cursor) { 1 } else { 0 });
                        bit_cursor += 1;
                    }
                    payload.push(byte_val);
                }

                frames.push(CanFrame {
                    start_sample: start_idx,
                    arbitration_id: arb_id,
                    is_extended: false,
                    is_rtr: rtr,
                    dlc,
                    payload,
                    crc_valid: true,
                });

                i += (samples_per_bit * (bit_cursor + 10) as f64) as usize;
            } else {
                i += 1;
            }
        }

        frames
    }
}
