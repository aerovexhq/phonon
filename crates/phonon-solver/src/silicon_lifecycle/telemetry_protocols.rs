#![deny(unsafe_code)]

//! Hardware Telemetry Protocol Emulation Engine.
//!
//! Emulates industrial on-die sensor transport protocols:
//! - JTAG IEEE 1149.1: TAP state machine, instruction register, and serialized scan chain readout.
//! - MIPI I3C: High-speed SDR/HDR frames, Dynamic Address Assignment (ENTDAA), and In-Band Interrupts (IBI).
//! - SMBus / PMBus: Standard 100-400 kHz multi-drop master/slave, Packet Error Checking (PEC), and SMBALERT#.
//! - PCIe MCTP / PLDM: DSP0236 transport encapsulation and DSP0248 Platform Level Data Model sensor monitoring.

use super::sensor_mesh::{OnDieSensor, SensorKind, SensorMesh, SensorStatus};

/// Supported hardware bus protocols for on-die telemetry streaming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    /// JTAG IEEE 1149.1 Test Access Port & Boundary Scan.
    Jtag1149,
    /// MIPI I3C Improved Inter-Integrated Circuit.
    MipiI3C,
    /// System Management Bus / Power Management Bus (SMBus/PMBus).
    SmbusPmbus,
    /// PCIe Management Component Transport Protocol with PLDM payload.
    PcieMctpPldm,
}

impl ProtocolType {
    pub fn name(&self) -> &'static str {
        match self {
            ProtocolType::Jtag1149 => "JTAG IEEE 1149.1",
            ProtocolType::MipiI3C => "MIPI I3C (HDR-DDR)",
            ProtocolType::SmbusPmbus => "SMBus / PMBus 1.3",
            ProtocolType::PcieMctpPldm => "PCIe MCTP / PLDM (DSP0248)",
        }
    }

    /// Theoretical peak serial throughput in bits per second.
    pub fn max_throughput_bps(&self) -> f64 {
        match self {
            ProtocolType::Jtag1149 => 25_000_000.0,    // 25 MHz TCK
            ProtocolType::MipiI3C => 25_000_000.0,     // 25 Mbps HDR-DDR mode
            ProtocolType::SmbusPmbus => 400_000.0,     // 400 kHz Fast Mode
            ProtocolType::PcieMctpPldm => 100_000_000.0, // 100 Mbps OOB / in-band MCTP
        }
    }
}

/// JTAG IEEE 1149.1 TAP Controller 16-State Machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JtagTapState {
    TestLogicReset,
    RunTestIdle,
    SelectDrScan,
    CaptureDr,
    ShiftDr,
    Exit1Dr,
    PauseDr,
    Exit2Dr,
    UpdateDr,
    SelectIrScan,
    CaptureIr,
    ShiftIr,
    Exit1Ir,
    PauseIr,
    Exit2Ir,
    UpdateIr,
}

impl JtagTapState {
    /// Advances the TAP controller state based on TMS input pin logic level.
    pub fn next(&self, tms: bool) -> Self {
        match self {
            JtagTapState::TestLogicReset => {
                if tms { JtagTapState::TestLogicReset } else { JtagTapState::RunTestIdle }
            }
            JtagTapState::RunTestIdle => {
                if tms { JtagTapState::SelectDrScan } else { JtagTapState::RunTestIdle }
            }
            JtagTapState::SelectDrScan => {
                if tms { JtagTapState::SelectIrScan } else { JtagTapState::CaptureDr }
            }
            JtagTapState::CaptureDr => {
                if tms { JtagTapState::Exit1Dr } else { JtagTapState::ShiftDr }
            }
            JtagTapState::ShiftDr => {
                if tms { JtagTapState::Exit1Dr } else { JtagTapState::ShiftDr }
            }
            JtagTapState::Exit1Dr => {
                if tms { JtagTapState::UpdateDr } else { JtagTapState::PauseDr }
            }
            JtagTapState::PauseDr => {
                if tms { JtagTapState::Exit2Dr } else { JtagTapState::PauseDr }
            }
            JtagTapState::Exit2Dr => {
                if tms { JtagTapState::UpdateDr } else { JtagTapState::ShiftDr }
            }
            JtagTapState::UpdateDr => {
                if tms { JtagTapState::SelectDrScan } else { JtagTapState::RunTestIdle }
            }
            JtagTapState::SelectIrScan => {
                if tms { JtagTapState::TestLogicReset } else { JtagTapState::CaptureIr }
            }
            JtagTapState::CaptureIr => {
                if tms { JtagTapState::Exit1Ir } else { JtagTapState::ShiftIr }
            }
            JtagTapState::ShiftIr => {
                if tms { JtagTapState::Exit1Ir } else { JtagTapState::ShiftIr }
            }
            JtagTapState::Exit1Ir => {
                if tms { JtagTapState::UpdateIr } else { JtagTapState::PauseIr }
            }
            JtagTapState::PauseIr => {
                if tms { JtagTapState::Exit2Ir } else { JtagTapState::PauseIr }
            }
            JtagTapState::Exit2Ir => {
                if tms { JtagTapState::UpdateIr } else { JtagTapState::ShiftIr }
            }
            JtagTapState::UpdateIr => {
                if tms { JtagTapState::SelectDrScan } else { JtagTapState::RunTestIdle }
            }
        }
    }
}

/// Serialized telemetry packet emitted across the active bus.
#[derive(Debug, Clone)]
pub struct TelemetryPacket {
    /// Monotonically increasing packet sequence number.
    pub seq_num: u64,
    /// Timestamp relative to simulation start in microseconds.
    pub timestamp_us: f64,
    /// Protocol format.
    pub protocol: ProtocolType,
    /// Source sensor ID.
    pub sensor_id: usize,
    /// Sensor kind.
    pub sensor_kind: SensorKind,
    /// Decoded physical floating point value.
    pub physical_value: f64,
    /// Engineering unit label.
    pub unit: &'static str,
    /// Packet size in total bits on wire (including headers, payload, CRC).
    pub size_bits: usize,
    /// Hexadecimal frame dump string.
    pub hex_dump: String,
    /// Human-readable protocol breakdown.
    pub decoded_summary: String,
    /// Does this packet represent an interrupt or threshold alarm?
    pub is_alarm_event: bool,
}

/// Statistics and bus health metrics for the telemetry channel.
#[derive(Debug, Clone)]
pub struct TelemetryBusMetrics {
    pub protocol: ProtocolType,
    pub total_packets_sent: u64,
    pub total_bytes_transferred: u64,
    pub current_throughput_kbps: f64,
    pub bus_utilization_pct: f64,
    pub avg_packet_latency_us: f64,
    pub crc_error_count: u32,
    pub in_band_interrupt_count: u32,
}

/// Calculates standard SMBus / MIPI CRC-8 (polynomial 0x07: x^8 + x^2 + x + 1).
pub fn calculate_crc8(data: &[u8]) -> u8 {
    let mut crc = 0x00u8;
    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if (crc & 0x80) != 0 {
                crc = (crc << 1) ^ 0x07;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Calculates MCTP / Ethernet CRC-32 (polynomial 0xEDB88320).
pub fn calculate_crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// Encapsulates a sensor reading into a JTAG Scan Chain bitstream frame.
pub fn encode_jtag_packet(seq: u64, time_us: f64, sensor: &OnDieSensor) -> TelemetryPacket {
    // 32-bit JTAG User Telemetry DR:
    // Bits 31..28: Preamble (0xA)
    // Bits 27..22: Sensor ID (6 bits, 0..63)
    // Bits 21..20: Sensor Kind (2 bits)
    // Bits 19..4:  Quantized 16-bit raw ADC reading
    // Bits 3..2:   Status (00=Normal, 01=Warn, 10=Crit)
    // Bits 1..0:   Parity / Stop bits
    let kind_bits = match sensor.kind {
        SensorKind::ThermalDiode => 0b00u32,
        SensorKind::RingOscillator => 0b01u32,
        SensorKind::SupplyDroopDetector => 0b10u32,
        SensorKind::CriticalPathMonitor => 0b11u32,
    };

    let raw_val = (sensor.value * 16.0).max(0.0) as u32 & 0xFFFF;
    let status_bits = match sensor.status {
        SensorStatus::Normal => 0b00u32,
        SensorStatus::Warning => 0b01u32,
        SensorStatus::Critical => 0b10u32,
    };

    let word: u32 = (0xAu32 << 28)
        | ((sensor.id as u32 & 0x3F) << 22)
        | (kind_bits << 20)
        | (raw_val << 4)
        | (status_bits << 2)
        | 0b01;

    let hex_dump = format!("0x{:08X}", word);
    let is_alarm = sensor.status != SensorStatus::Normal;

    TelemetryPacket {
        seq_num: seq,
        timestamp_us: time_us,
        protocol: ProtocolType::Jtag1149,
        sensor_id: sensor.id,
        sensor_kind: sensor.kind,
        physical_value: sensor.value,
        unit: sensor.kind.unit_str(),
        size_bits: 32,
        hex_dump,
        decoded_summary: format!(
            "JTAG_DR[ID:{}, {}: {:.2} {}, Status:{:?}]",
            sensor.id,
            sensor.kind.unit_str(),
            sensor.value,
            sensor.kind.unit_str(),
            sensor.status
        ),
        is_alarm_event: is_alarm,
    }
}

/// Encapsulates a sensor reading into a MIPI I3C SDR/HDR data frame.
pub fn encode_i3c_packet(seq: u64, time_us: f64, sensor: &OnDieSensor) -> TelemetryPacket {
    // I3C Frame:
    // Byte 0: Slave Dynamic Address (7-bit e.g. 0x2A << 1 | W/R)
    // Byte 1: Sub-Command / Target Register (0x10 = Telemetry Stream)
    // Byte 2: Sensor ID
    // Byte 3..4: 16-bit Payload (Big-Endian)
    // Byte 5: Alarm Flags (Bit 7: IBI active, Bit 0: Warning, Bit 1: Critical)
    // Byte 6: CRC-8
    let addr = 0x2Au8 << 1;
    let cmd = 0x10u8;
    let s_id = (sensor.id & 0xFF) as u8;
    let raw_val = (sensor.value * 64.0).max(0.0) as u16;
    let val_hi = (raw_val >> 8) as u8;
    let val_lo = (raw_val & 0xFF) as u8;

    let is_alarm = sensor.status != SensorStatus::Normal;
    let flags = match sensor.status {
        SensorStatus::Normal => 0x00u8,
        SensorStatus::Warning => 0x81u8,  // Set IBI request bit + Warning
        SensorStatus::Critical => 0x82u8, // Set IBI request bit + Critical
    };

    let payload = [addr, cmd, s_id, val_hi, val_lo, flags];
    let crc = calculate_crc8(&payload);

    let hex_dump = format!(
        "{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} [CRC:{:02X}]",
        payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], crc
    );

    TelemetryPacket {
        seq_num: seq,
        timestamp_us: time_us,
        protocol: ProtocolType::MipiI3C,
        sensor_id: sensor.id,
        sensor_kind: sensor.kind,
        physical_value: sensor.value,
        unit: sensor.kind.unit_str(),
        size_bits: 7 * 8, // 56 bits
        hex_dump,
        decoded_summary: format!(
            "I3C_DDR[ADDR:0x2A, SID:{}, VAL:{:.2} {}, IBI:{}]",
            sensor.id,
            sensor.value,
            sensor.kind.unit_str(),
            if is_alarm { "YES" } else { "NO" }
        ),
        is_alarm_event: is_alarm,
    }
}

/// Encapsulates a sensor reading into an SMBus/PMBus transaction.
pub fn encode_smbus_packet(seq: u64, time_us: f64, sensor: &OnDieSensor) -> TelemetryPacket {
    // PMBus Read Word Protocol with Packet Error Checking (PEC):
    // Byte 0: Slave Address + Write (0xB0)
    // Byte 1: Command Code: 0x8D (READ_TEMPERATURE_1), 0x8B (READ_VOUT), etc.
    // Byte 2: Slave Address + Read (0xB1)
    // Byte 3: Data Byte Low
    // Byte 4: Data Byte High
    // Byte 5: PEC (CRC-8)
    let cmd = match sensor.kind {
        SensorKind::ThermalDiode => 0x8Du8,         // READ_TEMPERATURE_1
        SensorKind::RingOscillator => 0xD0u8,       // MFR_SPECIFIC_FREQ
        SensorKind::SupplyDroopDetector => 0x8Bu8,  // READ_VOUT
        SensorKind::CriticalPathMonitor => 0xD1u8,  // MFR_TIMING_MARGIN
    };

    let raw_val = (sensor.value * 100.0).max(0.0) as u16;
    let lo = (raw_val & 0xFF) as u8;
    let hi = (raw_val >> 8) as u8;

    let transaction = [0xB0u8, cmd, 0xB1u8, lo, hi];
    let pec = calculate_crc8(&transaction);

    let hex_dump = format!(
        "{:02X} {:02X} {:02X} {:02X} {:02X} [PEC:{:02X}]",
        transaction[0], transaction[1], transaction[2], transaction[3], transaction[4], pec
    );

    let is_alarm = sensor.status != SensorStatus::Normal;

    TelemetryPacket {
        seq_num: seq,
        timestamp_us: time_us,
        protocol: ProtocolType::SmbusPmbus,
        sensor_id: sensor.id,
        sensor_kind: sensor.kind,
        physical_value: sensor.value,
        unit: sensor.kind.unit_str(),
        size_bits: 6 * 8, // 48 bits
        hex_dump,
        decoded_summary: format!(
            "PMBus_READ[CMD:0x{:02X}, SID:{}, VAL:{:.2} {}, Alert:{}]",
            cmd,
            sensor.id,
            sensor.value,
            sensor.kind.unit_str(),
            if is_alarm { "SMBALERT#" } else { "NONE" }
        ),
        is_alarm_event: is_alarm,
    }
}

/// Encapsulates a sensor reading into a PCIe MCTP / PLDM (DSP0248) packet.
pub fn encode_mctp_packet(seq: u64, time_us: f64, sensor: &OnDieSensor) -> TelemetryPacket {
    // MCTP Base Packet (DSP0236):
    // Byte 0: Header Version (0x01)
    // Byte 1: Destination Endpoint ID (EID 0x08 = Baseboard Management Controller)
    // Byte 2: Source Endpoint ID (EID 0x14 = Silicon Die 0)
    // Byte 3: SOM(1) | EOM(1) | Seq(0..3) | Tag(0) -> 0xC0
    // Byte 4: MCTP Message Type: 0x01 (PLDM)
    // PLDM DSP0248 Header & Numeric Sensor Reading:
    // Byte 5: PLDM Type 0x02 (Platform Monitoring)
    // Byte 6: Command 0x11 (GetSensorReading)
    // Byte 7: Sensor ID (Low)
    // Byte 8: Sensor ID (High)
    // Byte 9..12: 32-bit IEEE 754 float reading
    // Byte 13: Operational State (0=Normal, 1=Warning, 2=Critical)
    // Byte 14..17: CRC-32 Trailer
    let mut pkt_bytes = vec![
        0x01u8, // Version
        0x08u8, // Dest EID (BMC)
        0x14u8, // Src EID (Die 0)
        0xC0u8 | ((seq as u8) & 0x03) << 4,
        0x01u8, // Message Type = PLDM
        0x02u8, // PLDM Monitoring
        0x11u8, // GetSensorReading response
        (sensor.id & 0xFF) as u8,
        ((sensor.id >> 8) & 0xFF) as u8,
    ];

    let float_bytes = (sensor.value as f32).to_le_bytes();
    pkt_bytes.extend_from_slice(&float_bytes);

    let state_byte = match sensor.status {
        SensorStatus::Normal => 0x00u8,
        SensorStatus::Warning => 0x01u8,
        SensorStatus::Critical => 0x02u8,
    };
    pkt_bytes.push(state_byte);

    let crc = calculate_crc32(&pkt_bytes);
    let crc_bytes = crc.to_le_bytes();
    pkt_bytes.extend_from_slice(&crc_bytes);

    let hex_dump = pkt_bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");

    let is_alarm = sensor.status != SensorStatus::Normal;

    TelemetryPacket {
        seq_num: seq,
        timestamp_us: time_us,
        protocol: ProtocolType::PcieMctpPldm,
        sensor_id: sensor.id,
        sensor_kind: sensor.kind,
        physical_value: sensor.value,
        unit: sensor.kind.unit_str(),
        size_bits: pkt_bytes.len() * 8,
        hex_dump,
        decoded_summary: format!(
            "MCTP/PLDM[Src:0x14->Dest:0x08, SID:{}, Val:{:.2} {}, Status:{:?}]",
            sensor.id,
            sensor.value,
            sensor.kind.unit_str(),
            sensor.status
        ),
        is_alarm_event: is_alarm,
    }
}

/// Telemetry stream engine managing packet generation, protocol conversions, and bus statistics.
#[derive(Debug, Clone)]
pub struct TelemetryStreamEngine {
    pub active_protocol: ProtocolType,
    pub packet_history: Vec<TelemetryPacket>,
    pub max_history_size: usize,
    pub tap_controller: JtagTapState,
    pub total_packets: u64,
    pub bus_metrics: TelemetryBusMetrics,
    pub simulated_time_us: f64,
}

impl Default for TelemetryStreamEngine {
    fn default() -> Self {
        Self::new(ProtocolType::MipiI3C)
    }
}

impl TelemetryStreamEngine {
    pub fn new(protocol: ProtocolType) -> Self {
        Self {
            active_protocol: protocol,
            packet_history: Vec::new(),
            max_history_size: 200,
            tap_controller: JtagTapState::RunTestIdle,
            total_packets: 0,
            bus_metrics: TelemetryBusMetrics {
                protocol,
                total_packets_sent: 0,
                total_bytes_transferred: 0,
                current_throughput_kbps: 0.0,
                bus_utilization_pct: 0.0,
                avg_packet_latency_us: 1.2,
                crc_error_count: 0,
                in_band_interrupt_count: 0,
            },
            simulated_time_us: 0.0,
        }
    }

    /// Emits a burst of packets for all active sensors in the mesh.
    pub fn stream_mesh_telemetry(&mut self, mesh: &SensorMesh) {
        self.simulated_time_us += 10.0; // 10 us sampling interval

        let mut bits_in_burst = 0;
        let mut alert_count = 0;

        for sensor in &mesh.sensors {
            self.total_packets += 1;
            let pkt = match self.active_protocol {
                ProtocolType::Jtag1149 => {
                    encode_jtag_packet(self.total_packets, self.simulated_time_us, sensor)
                }
                ProtocolType::MipiI3C => {
                    encode_i3c_packet(self.total_packets, self.simulated_time_us, sensor)
                }
                ProtocolType::SmbusPmbus => {
                    encode_smbus_packet(self.total_packets, self.simulated_time_us, sensor)
                }
                ProtocolType::PcieMctpPldm => {
                    encode_mctp_packet(self.total_packets, self.simulated_time_us, sensor)
                }
            };

            bits_in_burst += pkt.size_bits;
            if pkt.is_alarm_event {
                alert_count += 1;
            }

            if self.packet_history.len() >= self.max_history_size {
                self.packet_history.remove(0);
            }
            self.packet_history.push(pkt);
        }

        // Update bus throughput metrics
        self.bus_metrics.total_packets_sent = self.total_packets;
        let bytes = (bits_in_burst / 8) as u64;
        self.bus_metrics.total_bytes_transferred += bytes;
        if alert_count > 0 {
            self.bus_metrics.in_band_interrupt_count += alert_count as u32;
        }

        let time_sec = 0.000010; // 10 us burst window
        let bps = bits_in_burst as f64 / time_sec;
        self.bus_metrics.current_throughput_kbps = bps / 1000.0;
        let max_bps = self.active_protocol.max_throughput_bps();
        self.bus_metrics.bus_utilization_pct = (bps / max_bps * 100.0).clamp(0.1, 99.9);

        // Average packet latency
        let base_lat = match self.active_protocol {
            ProtocolType::Jtag1149 => 32.0 / 25.0,     // 1.28 us
            ProtocolType::MipiI3C => 56.0 / 25.0,      // 2.24 us
            ProtocolType::SmbusPmbus => 48.0 / 0.40,   // 120.0 us
            ProtocolType::PcieMctpPldm => 144.0 / 100.0, // 1.44 us
        };
        self.bus_metrics.avg_packet_latency_us = base_lat;
    }
}
