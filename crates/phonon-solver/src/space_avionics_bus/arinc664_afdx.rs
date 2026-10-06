#![deny(unsafe_code)]

//! ARINC 664 Part 7 / AFDX (Avionics Full-Duplex Switched Ethernet) Co-Simulator.
//!
//! Models deterministic Virtual Link (VL) bandwidth reservation, Bandwidth Allocation Gap (BAG)
//! token-bucket policing ($1\text{--}128\text{ ms}$), redundant dual-network (Network A/B) First-Valid
//! sequence number de-duplication ($SN \in [0, 255]$), switch technical latency $T_{\text{tech}}$,
//! output queue contention, and deterministic worst-case end-to-end latency and jitter bounding.

/// ARINC 664 Virtual Link (VL) configuration and status.
#[derive(Debug, Clone)]
pub struct AfdxVirtualLink {
    /// Virtual Link identifier (1 to 65535).
    pub vl_id: u16,
    /// Bandwidth Allocation Gap (BAG) in milliseconds (standard: 1, 2, 4, 8, 16, 32, 64, 128 ms).
    pub bag_ms: f64,
    /// Maximum allowed Ethernet frame length in bytes (64 to 1518 bytes).
    pub l_max_bytes: usize,
    /// Maximum allowed transmission jitter allowance in microseconds (standard <= 500 us).
    pub max_jitter_us: f64,
    /// Whether redundant dual-rail transmission (Network A + Network B) is enabled.
    pub redundancy_enabled: bool,
    /// Last transmission timestamp in milliseconds (for BAG policing).
    pub last_tx_timestamp_ms: f64,
    /// Cumulative compliant frames transmitted.
    pub compliant_frames_tx: u64,
    /// Cumulative non-compliant frames dropped by BAG regulator policing.
    pub dropped_by_policing: u64,
    /// Next sequence number expected to send (0 to 255).
    pub next_tx_seq_num: u8,
    /// Last received sequence number accepted at receiver.
    pub last_rx_seq_num: Option<u8>,
    /// Cumulative frames accepted on Network A.
    pub accepted_net_a: u64,
    /// Cumulative frames accepted on Network B.
    pub accepted_net_b: u64,
    /// Cumulative duplicate frames successfully rejected.
    pub duplicates_rejected: u64,
    /// Cumulative integrity errors (out-of-sequence / skipped frames).
    pub integrity_errors: u64,
}

impl Default for AfdxVirtualLink {
    fn default() -> Self {
        Self {
            vl_id: 101,
            bag_ms: 8.0, // 8 ms BAG
            l_max_bytes: 1024,
            max_jitter_us: 500.0,
            redundancy_enabled: true,
            last_tx_timestamp_ms: -100.0,
            compliant_frames_tx: 0,
            dropped_by_policing: 0,
            next_tx_seq_num: 1,
            last_rx_seq_num: None,
            accepted_net_a: 0,
            accepted_net_b: 0,
            duplicates_rejected: 0,
            integrity_errors: 0,
        }
    }
}

impl AfdxVirtualLink {
    /// Create a new AFDX Virtual Link with specified ID, BAG, and maximum frame size.
    pub fn new(vl_id: u16, bag_ms: f64, l_max_bytes: usize) -> Self {
        Self {
            vl_id,
            bag_ms: bag_ms.clamp(1.0, 128.0),
            l_max_bytes: l_max_bytes.clamp(64, 1518),
            ..Default::default()
        }
    }

    /// Calculate allocated bandwidth for this Virtual Link in Megabits per second (Mbps).
    ///
    /// Bandwidth = (L_max * 8) / (BAG_ms * 1000) = (L_max * 8) / (BAG_s * 1e6)
    pub fn allocated_bandwidth_mbps(&self) -> f64 {
        let l_bits = (self.l_max_bytes as f64) * 8.0;
        let bag_s = self.bag_ms * 1.0e-3;
        (l_bits / bag_s) / 1.0e6
    }

    /// Submit a frame for transmission at timestamp_ms.
    /// Checks BAG regulator policing: frame is compliant if
    /// Delta t >= BAG - Jitter_allowance.
    pub fn transmit_frame(&mut self, timestamp_ms: f64, frame_size_bytes: usize) -> Result<u8, &'static str> {
        let size = frame_size_bytes.min(self.l_max_bytes);
        if size == 0 {
            return Err("Frame size must be non-zero");
        }

        let jitter_ms = self.max_jitter_us * 1.0e-3;
        let min_interval_ms = self.bag_ms - jitter_ms;

        let delta_t = timestamp_ms - self.last_tx_timestamp_ms;
        if self.last_tx_timestamp_ms >= 0.0 && delta_t < min_interval_ms {
            self.dropped_by_policing += 1;
            return Err("BAG policing violation: packet interval below (BAG - Jitter)");
        }

        self.last_tx_timestamp_ms = timestamp_ms;
        self.compliant_frames_tx += 1;

        let seq = self.next_tx_seq_num;
        self.next_tx_seq_num = self.next_tx_seq_num.wrapping_add(1);

        Ok(seq)
    }

    /// Receive frame at end system from Network A or Network B.
    /// Implements First-Valid acceptance and duplicate sequence number rejection.
    /// Returns true if frame was accepted as valid; false if discarded as duplicate or invalid.
    pub fn receive_frame(&mut self, seq_num: u8, from_network_a: bool) -> bool {
        match self.last_rx_seq_num {
            None => {
                // First frame ever received
                self.last_rx_seq_num = Some(seq_num);
                if from_network_a {
                    self.accepted_net_a += 1;
                } else {
                    self.accepted_net_b += 1;
                }
                true
            }
            Some(last_seq) => {
                if seq_num == last_seq {
                    // Exact duplicate arrival from the redundant network path
                    self.duplicates_rejected += 1;
                    false
                } else {
                    // Valid new sequence number (modulo 256 circular sequence)
                    let diff = seq_num.wrapping_sub(last_seq);
                    if diff > 0 && diff <= 128 {
                        if diff > 1 {
                            self.integrity_errors += 1; // Out-of-order or skipped frame
                        }
                        self.last_rx_seq_num = Some(seq_num);
                        if from_network_a {
                            self.accepted_net_a += 1;
                        } else {
                            self.accepted_net_b += 1;
                        }
                        true
                    } else {
                        // Stale / out-of-window packet
                        self.duplicates_rejected += 1;
                        false
                    }
                }
            }
        }
    }
}

/// AFDX Switch model simulating technical latency and queue contention.
#[derive(Debug, Clone)]
pub struct AfdxSwitch {
    /// Fixed technical switching latency T_tech in microseconds (typically 16 us for 100BASE-TX AFDX).
    pub technical_latency_us: f64,
    /// Physical link transmission rate in Mbps (100.0 or 1000.0 Mbps).
    pub port_speed_mbps: f64,
    /// Number of switched Virtual Links routing through this switch.
    pub active_vl_count: usize,
    /// Peak queue contention buffer depth in bytes.
    pub peak_queue_bytes: usize,
}

impl Default for AfdxSwitch {
    fn default() -> Self {
        Self {
            technical_latency_us: 16.0,
            port_speed_mbps: 100.0,
            active_vl_count: 8,
            peak_queue_bytes: 4096,
        }
    }
}

impl AfdxSwitch {
    /// Calculate packet transmission serialization delay in microseconds for frame of size_bytes.
    ///
    /// T_tx = (size_bytes * 8) / (port_speed_mbps * 1e6) * 1e6 = (size_bytes * 8) / port_speed_mbps
    pub fn frame_transmission_delay_us(&self, size_bytes: usize) -> f64 {
        ((size_bytes as f64) * 8.0) / self.port_speed_mbps.max(1.0)
    }

    /// Calculate worst-case queuing latency in microseconds when contending with other active VLs.
    pub fn worst_case_queuing_delay_us(&self, max_contending_frame_bytes: usize) -> f64 {
        let contending_bytes = (self.active_vl_count.saturating_sub(1) * max_contending_frame_bytes).min(self.peak_queue_bytes);
        ((contending_bytes as f64) * 8.0) / self.port_speed_mbps.max(1.0)
    }

    /// Calculate total worst-case switch transit latency T_switch = T_tech + T_tx + T_queue (us).
    pub fn worst_case_switch_transit_us(&self, frame_size_bytes: usize) -> f64 {
        let t_tx = self.frame_transmission_delay_us(frame_size_bytes);
        let t_queue = self.worst_case_queuing_delay_us(frame_size_bytes);
        self.technical_latency_us + t_tx + t_queue
    }

    /// Calculate end-to-end worst case latency bound across N cascading AFDX switches.
    pub fn end_to_end_latency_bound_us(&self, hops: usize, frame_size_bytes: usize) -> f64 {
        let single_hop = self.worst_case_switch_transit_us(frame_size_bytes);
        single_hop * (hops.max(1) as f64)
    }
}
