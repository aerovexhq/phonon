//! IEEE 802.11 Wi-Fi MAC Protocol Engine & CSMA/CA State Machine
//!
//! Provides first-principles models for wireless local area network medium access:
//! - 48-bit IEEE 802 MAC addressing and standard frame parsing
//! - IEEE 802.3 / 802.11 CRC-32 Frame Check Sequence (FCS) verification
//! - MAC Frame archetypes: Management (Beacon), Control (RTS, CTS, ACK), and Data frames
//! - IEEE 802.11a/b/g/n/ac PHY timing standards (SIFS, DIFS, SlotTime, EIFS)
//! - CSMA/CA contention engine: Clear Channel Assessment (CCA), random backoff counter,
//!   binary exponential backoff on collision/timeout ($CW \in [CW_{min}, CW_{max}]$),
//!   and virtual carrier sensing with Network Allocation Vector (NAV).

use std::fmt;

/// Precomputed CRC-32 lookup table using IEEE 802.3 / 802.11 polynomial `0xEDB88320`.
const CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
};

/// Computes the IEEE 802.3 / 802.11 standard 32-bit Frame Check Sequence (FCS) CRC-32.
pub fn compute_crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        let idx = ((crc ^ (byte as u32)) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[idx];
    }
    crc ^ 0xFFFFFFFFu32
}

/// Standard 48-bit IEEE 802 MAC address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MacAddress(pub [u8; 6]);

impl MacAddress {
    /// Broadcast MAC address `FF:FF:FF:FF:FF:FF`.
    pub const BROADCAST: Self = Self([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);

    /// Zero MAC address `00:00:00:00:00:00`.
    pub const ZERO: Self = Self([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Checks if address is broadcast.
    #[inline]
    pub fn is_broadcast(&self) -> bool {
        self.0 == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    }

    /// Checks if address is multicast (least significant bit of first octet is 1).
    #[inline]
    pub fn is_multicast(&self) -> bool {
        (self.0[0] & 0x01) != 0
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// IEEE 802.11 Frame Type and Subtype classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    /// Management Beacon frame announcing SSID and network parameters.
    Beacon,
    /// Management Probe Request.
    ProbeRequest,
    /// Management Probe Response.
    ProbeResponse,
    /// Control Request-to-Send (RTS) handshake.
    Rts,
    /// Control Clear-to-Send (CTS) handshake.
    Cts,
    /// Control Acknowledgment (ACK).
    Ack,
    /// Standard Data payload frame.
    Data,
    /// Quality of Service (QoS) Data payload frame.
    QosData,
}

impl FrameType {
    #[inline]
    pub const fn is_control(&self) -> bool {
        matches!(self, Self::Rts | Self::Cts | Self::Ack)
    }

    #[inline]
    pub const fn is_data(&self) -> bool {
        matches!(self, Self::Data | Self::QosData)
    }

    #[inline]
    pub const fn is_management(&self) -> bool {
        matches!(
            self,
            Self::Beacon | Self::ProbeRequest | Self::ProbeResponse
        )
    }
}

/// Fully-formed IEEE 802.11 MAC Frame with CRC-32 Frame Check Sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacFrame {
    /// Frame classification.
    pub frame_type: FrameType,
    /// Duration / ID field in microseconds (NAV reservation).
    pub duration_us: u16,
    /// Receiver Address (RA / Destination MAC).
    pub ra: MacAddress,
    /// Transmitter Address (TA / Source MAC).
    pub ta: MacAddress,
    /// Basic Service Set Identifier (BSSID / AP MAC).
    pub bssid: MacAddress,
    /// Sequence Control (Sequence number 0..4095 and fragment number 0..15).
    pub sequence_control: u16,
    /// Frame body payload bytes.
    pub payload: Vec<u8>,
    /// 32-bit CRC-32 Frame Check Sequence (FCS).
    pub fcs: u32,
}

impl MacFrame {
    /// Constructs a standard Data frame and automatically calculates its CRC-32 FCS.
    pub fn new_data(
        ta: MacAddress,
        ra: MacAddress,
        bssid: MacAddress,
        seq_num: u16,
        payload: Vec<u8>,
    ) -> Self {
        let mut frame = Self {
            frame_type: FrameType::Data,
            duration_us: 0,
            ra,
            ta,
            bssid,
            sequence_control: (seq_num & 0x0FFF) << 4,
            payload,
            fcs: 0,
        };
        frame.fcs = frame.calculate_fcs();
        frame
    }

    /// Constructs an immediate Control ACK frame.
    pub fn new_ack(ra: MacAddress, duration_us: u16) -> Self {
        let mut frame = Self {
            frame_type: FrameType::Ack,
            duration_us,
            ra,
            ta: MacAddress::ZERO,
            bssid: MacAddress::ZERO,
            sequence_control: 0,
            payload: Vec::new(),
            fcs: 0,
        };
        frame.fcs = frame.calculate_fcs();
        frame
    }

    /// Constructs a Request to Send (RTS) handshake frame.
    pub fn new_rts(ta: MacAddress, ra: MacAddress, duration_us: u16) -> Self {
        let mut frame = Self {
            frame_type: FrameType::Rts,
            duration_us,
            ra,
            ta,
            bssid: MacAddress::ZERO,
            sequence_control: 0,
            payload: Vec::new(),
            fcs: 0,
        };
        frame.fcs = frame.calculate_fcs();
        frame
    }

    /// Constructs a Clear to Send (CTS) handshake frame.
    pub fn new_cts(ra: MacAddress, duration_us: u16) -> Self {
        let mut frame = Self {
            frame_type: FrameType::Cts,
            duration_us,
            ra,
            ta: MacAddress::ZERO,
            bssid: MacAddress::ZERO,
            sequence_control: 0,
            payload: Vec::new(),
            fcs: 0,
        };
        frame.fcs = frame.calculate_fcs();
        frame
    }

    /// Serializes the frame header and payload into a byte vector for FCS computation.
    pub fn serialize_header_and_payload(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(30 + self.payload.len());
        // Frame Control (2 bytes)
        let fc = match self.frame_type {
            FrameType::Beacon => 0x0080u16,
            FrameType::ProbeRequest => 0x0040u16,
            FrameType::ProbeResponse => 0x0050u16,
            FrameType::Rts => 0x00B4u16,
            FrameType::Cts => 0x00C4u16,
            FrameType::Ack => 0x00D4u16,
            FrameType::Data => 0x0008u16,
            FrameType::QosData => 0x0088u16,
        };
        buf.extend_from_slice(&fc.to_le_bytes());
        buf.extend_from_slice(&self.duration_us.to_le_bytes());
        buf.extend_from_slice(&self.ra.0);
        if !matches!(self.frame_type, FrameType::Ack | FrameType::Cts) {
            buf.extend_from_slice(&self.ta.0);
            buf.extend_from_slice(&self.bssid.0);
            buf.extend_from_slice(&self.sequence_control.to_le_bytes());
        }
        buf.extend_from_slice(&self.payload);
        buf
    }

    /// Computes the CRC-32 FCS over the frame header and payload.
    pub fn calculate_fcs(&self) -> u32 {
        let bytes = self.serialize_header_and_payload();
        compute_crc32(&bytes)
    }

    /// Validates whether the frame's stored FCS matches its payload.
    pub fn verify_fcs(&self) -> bool {
        self.fcs == self.calculate_fcs()
    }

    /// Total over-the-air frame size in bytes including MAC header and 4-byte FCS.
    pub fn total_bytes(&self) -> usize {
        let header_len = match self.frame_type {
            FrameType::Ack | FrameType::Cts => 10,
            FrameType::Rts => 16,
            _ => 24,
        };
        header_len + self.payload.len() + 4 // 4 bytes FCS
    }

    /// Calculates required airtime in microseconds at given PHY data rate in Mbps.
    pub fn airtime_us(&self, phy_rate_mbps: f64, preamble_us: f64) -> f64 {
        let total_bits = (self.total_bytes() * 8) as f64;
        let payload_duration = total_bits / phy_rate_mbps.max(1.0);
        preamble_us + payload_duration
    }
}

/// Supported Wi-Fi PHY Protocol standard profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiPhyStandard {
    /// IEEE 802.11a (5 GHz OFDM, 20 MHz channel).
    Dot11a,
    /// IEEE 802.11b (2.4 GHz DSSS, 20 MHz channel).
    Dot11b,
    /// IEEE 802.11g (2.4 GHz ERP-OFDM, 20 MHz channel).
    Dot11g,
    /// IEEE 802.11n (2.4 / 5 GHz HT-OFDM, 20/40 MHz).
    Dot11n,
    /// IEEE 802.11ac (5 GHz VHT-OFDM, 20/40/80 MHz).
    Dot11ac,
}

/// Timing and Contention Window configuration for CSMA/CA MAC.
#[derive(Debug, Clone, PartialEq)]
pub struct CsmaCaConfig {
    pub standard: WifiPhyStandard,
    /// Duration of one backoff time slot in microseconds (SlotTime).
    pub slot_time_us: f64,
    /// Short Interframe Space in microseconds (SIFS).
    pub sifs_us: f64,
    /// DCF Interframe Space in microseconds: $\text{DIFS} = \text{SIFS} + 2 \times \text{SlotTime}$.
    pub difs_us: f64,
    /// Extended Interframe Space in microseconds (EIFS) used after corrupt frame.
    pub eifs_us: f64,
    /// Minimum contention window $CW_{min}$ (e.g. 15).
    pub cw_min: u32,
    /// Maximum contention window $CW_{max}$ (e.g. 1023).
    pub cw_max: u32,
    /// Maximum retransmission attempts before packet drop.
    pub max_retries: u32,
    /// Threshold packet payload size above which RTS/CTS handshake is engaged.
    pub rts_threshold_bytes: usize,
    /// Clear Channel Assessment (CCA) Energy Detection threshold in dBm.
    pub cca_ed_threshold_dbm: f64,
    /// PLCP Preamble + Header duration in microseconds.
    pub preamble_duration_us: f64,
}

impl CsmaCaConfig {
    /// Standard IEEE 802.11a (5 GHz OFDM) MAC timing configuration.
    pub fn wifi_802_11a() -> Self {
        let slot_time_us = 9.0;
        let sifs_us = 16.0;
        let difs_us = sifs_us + 2.0 * slot_time_us; // 34 us
        Self {
            standard: WifiPhyStandard::Dot11a,
            slot_time_us,
            sifs_us,
            difs_us,
            eifs_us: sifs_us + difs_us + 60.0,
            cw_min: 15,
            cw_max: 1023,
            max_retries: 7,
            rts_threshold_bytes: 2346, // Standard disable threshold
            cca_ed_threshold_dbm: -62.0,
            preamble_duration_us: 20.0,
        }
    }

    /// Standard IEEE 802.11g (2.4 GHz ERP-OFDM) MAC timing configuration.
    pub fn wifi_802_11g() -> Self {
        let slot_time_us = 9.0; // Short slot time
        let sifs_us = 10.0;
        let difs_us = sifs_us + 2.0 * slot_time_us; // 28 us
        Self {
            standard: WifiPhyStandard::Dot11g,
            slot_time_us,
            sifs_us,
            difs_us,
            eifs_us: sifs_us + difs_us + 60.0,
            cw_min: 15,
            cw_max: 1023,
            max_retries: 7,
            rts_threshold_bytes: 2346,
            cca_ed_threshold_dbm: -62.0,
            preamble_duration_us: 20.0,
        }
    }

    /// Standard IEEE 802.11n (2.4 / 5 GHz HT) MAC timing configuration.
    pub fn wifi_802_11n() -> Self {
        let slot_time_us = 9.0;
        let sifs_us = 16.0;
        let difs_us = sifs_us + 2.0 * slot_time_us;
        Self {
            standard: WifiPhyStandard::Dot11n,
            slot_time_us,
            sifs_us,
            difs_us,
            eifs_us: sifs_us + difs_us + 60.0,
            cw_min: 15,
            cw_max: 1023,
            max_retries: 7,
            rts_threshold_bytes: 2346,
            cca_ed_threshold_dbm: -62.0,
            preamble_duration_us: 16.0,
        }
    }
}

/// Operational state of a CSMA/CA Wi-Fi station.
#[derive(Debug, Clone, PartialEq)]
pub enum StationState {
    /// Station has no pending data and is listening to channel.
    Idle,
    /// Channel was sensed busy and became idle; waiting for DIFS deferral.
    DifsWait { remaining_us: f64 },
    /// Station is decrementing its random backoff counter during idle slots.
    Backoff {
        remaining_slots: u32,
        slot_timer_us: f64,
    },
    /// Station is actively transmitting a frame over the air.
    Transmitting { remaining_us: f64, frame: MacFrame },
    /// Frame transmitted; station is waiting for short SIFS and ACK frame.
    AwaitingAck {
        timeout_us: f64,
        retry_count: u32,
        frame: MacFrame,
    },
}

/// Simulated CSMA/CA Station Engine.
#[derive(Debug, Clone)]
pub struct CsmaCaStation {
    /// Node hardware MAC address.
    pub mac_address: MacAddress,
    /// Associated Access Point / BSSID.
    pub bssid: MacAddress,
    /// MAC timing configuration.
    pub config: CsmaCaConfig,
    /// Current internal state.
    pub state: StationState,
    /// Current contention window $CW \in [CW_{min}, CW_{max}]$.
    pub current_cw: u32,
    /// Network Allocation Vector (NAV) in microseconds (virtual carrier sense).
    pub nav_us: f64,
    /// Outbound transmit queue.
    pub tx_queue: Vec<MacFrame>,
    /// Inbound received frames that passed CRC-32 FCS check.
    pub rx_queue: Vec<MacFrame>,
    /// Frame sequence counter.
    pub seq_counter: u16,
    /// Total transmitted frames successfully acknowledged.
    pub total_tx_success: u64,
    /// Total collision / retry events.
    pub total_retries: u64,
    /// Total dropped frames after exceeding max retries.
    pub total_dropped: u64,
}

impl CsmaCaStation {
    pub fn new(mac_address: MacAddress, bssid: MacAddress, config: CsmaCaConfig) -> Self {
        let cw_min = config.cw_min;
        Self {
            mac_address,
            bssid,
            config,
            state: StationState::Idle,
            current_cw: cw_min,
            nav_us: 0.0,
            tx_queue: Vec::new(),
            rx_queue: Vec::new(),
            seq_counter: 0,
            total_tx_success: 0,
            total_retries: 0,
            total_dropped: 0,
        }
    }

    /// Enqueues high-level data payload for transmission to target MAC address.
    pub fn enqueue_data(&mut self, dest: MacAddress, payload: Vec<u8>) {
        let frame = MacFrame::new_data(
            self.mac_address,
            dest,
            self.bssid,
            self.seq_counter,
            payload,
        );
        self.seq_counter = (self.seq_counter + 1) & 0x0FFF;
        self.tx_queue.push(frame);
    }

    /// Resets contention window to $CW_{min}$ after successful transmission.
    pub fn reset_cw(&mut self) {
        self.current_cw = self.config.cw_min;
    }

    /// Multiplies contention window exponentially: $CW \leftarrow \min(2(CW+1)-1, CW_{max})$.
    pub fn double_cw(&mut self) {
        self.current_cw = (self.current_cw * 2 + 1).min(self.config.cw_max);
    }

    /// Checks if channel is physically sensed idle (RSSI below CCA ED threshold) and virtual NAV == 0.
    pub fn is_channel_idle(&self, channel_rssi_dbm: f64) -> bool {
        self.nav_us <= 0.0 && channel_rssi_dbm < self.config.cca_ed_threshold_dbm
    }

    /// Advances the CSMA/CA state machine by time step $\Delta t$ in microseconds.
    ///
    /// Returns `Some(MacFrame)` if a frame starts transmission during this step.
    pub fn step(
        &mut self,
        dt_us: f64,
        channel_rssi_dbm: f64,
        pseudo_random_u32: u32,
    ) -> Option<MacFrame> {
        // Decrement NAV virtual carrier sensing
        if self.nav_us > 0.0 {
            self.nav_us = (self.nav_us - dt_us).max(0.0);
        }

        let channel_idle = self.is_channel_idle(channel_rssi_dbm);

        match &mut self.state {
            StationState::Idle => {
                if !self.tx_queue.is_empty() {
                    if channel_idle {
                        self.state = StationState::DifsWait {
                            remaining_us: self.config.difs_us,
                        };
                    } else {
                        // Channel busy: draw backoff slots immediately
                        let backoff_slots = pseudo_random_u32 % (self.current_cw + 1);
                        self.state = StationState::Backoff {
                            remaining_slots: backoff_slots,
                            slot_timer_us: self.config.slot_time_us,
                        };
                    }
                }
                None
            }
            StationState::DifsWait { remaining_us } => {
                if !channel_idle {
                    // Preempted by channel activity; must wait for channel to clear then backoff
                    let backoff_slots = pseudo_random_u32 % (self.current_cw + 1);
                    self.state = StationState::Backoff {
                        remaining_slots: backoff_slots,
                        slot_timer_us: self.config.slot_time_us,
                    };
                    None
                } else {
                    *remaining_us -= dt_us;
                    if *remaining_us <= 0.0 {
                        // DIFS satisfied! Start transmitting top frame
                        let frame = self.tx_queue.remove(0);
                        let airtime = frame.airtime_us(54.0, self.config.preamble_duration_us);
                        self.state = StationState::Transmitting {
                            remaining_us: airtime,
                            frame: frame.clone(),
                        };
                        Some(frame)
                    } else {
                        None
                    }
                }
            }
            StationState::Backoff {
                remaining_slots,
                slot_timer_us,
            } => {
                if channel_idle {
                    *slot_timer_us -= dt_us;
                    if *slot_timer_us <= 0.0 {
                        *slot_timer_us = self.config.slot_time_us;
                        if *remaining_slots > 0 {
                            *remaining_slots -= 1;
                        }
                    }
                    if *remaining_slots == 0 && !self.tx_queue.is_empty() {
                        let frame = self.tx_queue.remove(0);
                        let airtime = frame.airtime_us(54.0, self.config.preamble_duration_us);
                        self.state = StationState::Transmitting {
                            remaining_us: airtime,
                            frame: frame.clone(),
                        };
                        Some(frame)
                    } else {
                        None
                    }
                } else {
                    // Frozen backoff while channel is busy
                    None
                }
            }
            StationState::Transmitting {
                remaining_us,
                frame,
            } => {
                *remaining_us -= dt_us;
                if *remaining_us <= 0.0 {
                    let transmitted_frame = frame.clone();
                    if transmitted_frame.ra.is_broadcast() {
                        // Broadcast frames do not expect an ACK
                        self.reset_cw();
                        self.total_tx_success += 1;
                        self.state = StationState::Idle;
                    } else {
                        // Unicast: transition to AwaitingAck
                        let ack_timeout_us =
                            self.config.sifs_us + self.config.slot_time_us * 3.0 + 50.0;
                        self.state = StationState::AwaitingAck {
                            timeout_us: ack_timeout_us,
                            retry_count: 0,
                            frame: transmitted_frame,
                        };
                    }
                }
                None
            }
            StationState::AwaitingAck {
                timeout_us,
                retry_count,
                frame,
            } => {
                *timeout_us -= dt_us;
                if *timeout_us <= 0.0 {
                    let retries = *retry_count;
                    let max_retries = self.config.max_retries;
                    let cw_min = self.config.cw_min;
                    let cw_max = self.config.cw_max;
                    let slot_time = self.config.slot_time_us;
                    let retry_frame = frame.clone();

                    self.total_retries += 1;
                    if retries >= max_retries {
                        self.total_dropped += 1;
                        self.current_cw = cw_min;
                        self.state = StationState::Idle;
                    } else {
                        let new_cw = (self.current_cw * 2 + 1).min(cw_max);
                        self.current_cw = new_cw;
                        let backoff_slots = pseudo_random_u32 % (new_cw + 1);
                        self.tx_queue.insert(0, retry_frame);
                        self.state = StationState::Backoff {
                            remaining_slots: backoff_slots,
                            slot_timer_us: slot_time,
                        };
                    }
                }
                None
            }
        }
    }

    /// Handles an incoming over-the-air frame received at this station.
    pub fn handle_received_frame(&mut self, frame: MacFrame) {
        if !frame.verify_fcs() {
            // Corrupt FCS: ignore frame
            return;
        }

        // Update virtual NAV reservation if frame not addressed to us
        if frame.ra != self.mac_address && frame.duration_us > 0 {
            self.nav_us = self.nav_us.max(frame.duration_us as f64);
        }

        // Check if addressed to us or broadcast
        if frame.ra == self.mac_address || frame.ra.is_broadcast() {
            if frame.frame_type == FrameType::Ack {
                // Received expected ACK!
                if let StationState::AwaitingAck { .. } = self.state {
                    self.total_tx_success += 1;
                    self.reset_cw();
                    self.state = StationState::Idle;
                }
            } else if frame.frame_type.is_data() {
                // Received data frame: save to receive queue
                self.rx_queue.push(frame);
            }
        }
    }
}
