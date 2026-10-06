#![deny(unsafe_code)]

//! ECSS-E-ST-50-12C (SpaceWire) and ECSS-E-ST-50-52C (SpaceFibre) Deterministic Fabric Engine.
//!
//! Models full-duplex Data-Strobe (DS) SpaceWire links (2 to 400 Mbps) with FCT credit-token flow
//! control and credit-starvation backpressure, and multi-gigabit multi-lane SpaceFibre serial links
//! (1.0 to 10.0 Gbps per lane) with Virtual Channel (VC) QoS scheduling (Strict Priority, Weighted
//! Round-Robin, Best-Effort) and buffer overflow bounding under high-rate payload burst floods.

/// ECSS-E-ST-50-12C SpaceWire link state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpWLinkState {
    /// Initial error reset state.
    ErrorReset,
    /// Waiting for disconnect timer.
    ErrorWait,
    /// Ready to initiate connection.
    Ready,
    /// Started handshaking with null tokens.
    Started,
    /// Exchanging initial FCT tokens.
    Connecting,
    /// Active high-throughput packet transmission state.
    Run,
}

/// SpaceWire point-to-point link model.
#[derive(Debug, Clone)]
pub struct SpaceWireLink {
    /// Link state machine current state.
    pub state: SpWLinkState,
    /// Signaling data rate in Megabits per second (Mbps) (e.g. 10.0 to 400.0 Mbps).
    pub bit_rate_mbps: f64,
    /// Available transmitter credit in bytes (each FCT adds 8 bytes, max 56 bytes per spec).
    pub tx_credit_bytes: u32,
    /// Maximum receiver buffer capacity in bytes.
    pub rx_buffer_capacity_bytes: usize,
    /// Current fill level of receiver buffer in bytes.
    pub rx_buffer_fill_bytes: usize,
    /// Cumulative transmitted payload bytes.
    pub total_tx_bytes: u64,
    /// Cumulative dropped bytes due to buffer overflow.
    pub total_dropped_bytes: u64,
    /// Cumulative credit starvation stalls encountered.
    pub credit_starvation_events: u64,
}

impl Default for SpaceWireLink {
    fn default() -> Self {
        Self {
            state: SpWLinkState::Run,
            bit_rate_mbps: 200.0,
            tx_credit_bytes: 56, // Initial 7 FCT tokens * 8 bytes
            rx_buffer_capacity_bytes: 1024,
            rx_buffer_fill_bytes: 0,
            total_tx_bytes: 0,
            total_dropped_bytes: 0,
            credit_starvation_events: 0,
        }
    }
}

impl SpaceWireLink {
    /// Create a new SpaceWire link with specified bit rate and buffer capacity.
    pub fn new(bit_rate_mbps: f64, rx_buffer_capacity_bytes: usize) -> Self {
        Self {
            bit_rate_mbps: bit_rate_mbps.clamp(2.0, 400.0),
            rx_buffer_capacity_bytes: rx_buffer_capacity_bytes.max(64),
            ..Default::default()
        }
    }

    /// Check if the transmitter is currently starved of Flow Control Tokens (FCTs).
    pub fn is_credit_starved(&self) -> bool {
        self.tx_credit_bytes == 0
    }

    /// Receive Flow Control Tokens from downstream receiver (each token yields 8 bytes credit).
    pub fn receive_fct(&mut self, token_count: u32) {
        let added = token_count.saturating_mul(8);
        self.tx_credit_bytes = (self.tx_credit_bytes + added).min(56);
    }

    /// Drain payload bytes from the receiver buffer (simulating host processing),
    /// which automatically generates credit FCTs sent back upstream.
    pub fn drain_rx(&mut self, count: usize) -> u32 {
        let drained = count.min(self.rx_buffer_fill_bytes);
        self.rx_buffer_fill_bytes -= drained;
        // Generate 1 FCT per 8 bytes freed
        (drained / 8) as u32
    }

    /// Transmit payload bytes over the SpaceWire link respecting FCT flow control.
    /// Returns the number of bytes successfully accepted by the link.
    pub fn send_bytes(&mut self, requested_bytes: usize) -> usize {
        if self.state != SpWLinkState::Run {
            return 0;
        }

        if self.tx_credit_bytes == 0 {
            self.credit_starvation_events += 1;
            return 0;
        }

        // Bounded by available credit and receiver free buffer space
        let max_by_credit = self.tx_credit_bytes as usize;
        let free_rx_space = self.rx_buffer_capacity_bytes.saturating_sub(self.rx_buffer_fill_bytes);

        let transmit_bytes = requested_bytes.min(max_by_credit).min(free_rx_space);
        if transmit_bytes == 0 && requested_bytes > 0 {
            if free_rx_space == 0 {
                self.total_dropped_bytes += requested_bytes as u64;
            } else {
                self.credit_starvation_events += 1;
            }
            return 0;
        }

        self.tx_credit_bytes -= transmit_bytes as u32;
        self.rx_buffer_fill_bytes += transmit_bytes;
        self.total_tx_bytes += transmit_bytes as u64;

        transmit_bytes
    }
}

/// ECSS-E-ST-50-52C SpaceFibre Quality of Service (QoS) scheduling policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpFiQoSScheduling {
    /// Strict Priority: Highest priority active Virtual Channel transmits first.
    StrictPriority,
    /// Weighted Round-Robin (WRR): Virtual Channels receive bandwidth proportional to their weight.
    WeightedRoundRobin,
    /// Bandwidth Reservation: Dedicated bandwidth slices guaranteed per VC.
    BandwidthReservation,
}

/// SpaceFibre Virtual Channel (VC) representation.
#[derive(Debug, Clone)]
pub struct SpFiVirtualChannel {
    /// Virtual Channel Identifier (0 to 31).
    pub vc_id: u8,
    /// Priority level (0 to 7, higher is higher priority).
    pub priority: u8,
    /// Weighted Round-Robin weight in [1, 255].
    pub weight: u32,
    /// Reserved bandwidth guarantee in Megabits per second (Mbps).
    pub reserved_bandwidth_mbps: f64,
    /// Internal VC queue buffer capacity in bytes.
    pub buffer_capacity_bytes: usize,
    /// Current queue fill level in bytes.
    pub buffer_fill_bytes: usize,
    /// Cumulative transmitted bytes on this VC.
    pub total_tx_bytes: u64,
    /// Cumulative dropped bytes on this VC due to buffer exhaustion.
    pub dropped_bytes: u64,
    /// Peak buffer fill level reached during simulation.
    pub peak_buffer_fill_bytes: usize,
}

impl SpFiVirtualChannel {
    /// Create a new SpaceFibre Virtual Channel.
    pub fn new(vc_id: u8, priority: u8, weight: u32, reserved_mbps: f64, capacity: usize) -> Self {
        Self {
            vc_id,
            priority,
            weight: weight.clamp(1, 255),
            reserved_bandwidth_mbps: reserved_mbps.max(0.0),
            buffer_capacity_bytes: capacity.max(256),
            buffer_fill_bytes: 0,
            total_tx_bytes: 0,
            dropped_bytes: 0,
            peak_buffer_fill_bytes: 0,
        }
    }

    /// Enqueue incoming packet payload bytes into this Virtual Channel.
    pub fn enqueue(&mut self, bytes: usize) -> usize {
        let free_space = self.buffer_capacity_bytes.saturating_sub(self.buffer_fill_bytes);
        let accepted = bytes.min(free_space);
        let dropped = bytes.saturating_sub(accepted);

        self.buffer_fill_bytes += accepted;
        self.peak_buffer_fill_bytes = self.peak_buffer_fill_bytes.max(self.buffer_fill_bytes);
        self.dropped_bytes += dropped as u64;

        accepted
    }

    /// Buffer fill ratio in [0.0, 1.0].
    pub fn buffer_fill_ratio(&self) -> f64 {
        self.buffer_fill_bytes as f64 / self.buffer_capacity_bytes as f64
    }
}

/// SpaceFibre multi-lane serial link fabric.
#[derive(Debug, Clone)]
pub struct SpaceFibreMultiLaneLink {
    /// Number of aggregated physical serial lanes (1 to 4).
    pub lane_count: usize,
    /// Per-lane raw serial bit rate in Gigabits per second (Gbps) (e.g. 2.5 Gbps or 6.25 Gbps).
    pub lane_bit_rate_gbps: f64,
    /// Quality of Service (QoS) scheduling mechanism.
    pub scheduling: SpFiQoSScheduling,
    /// Virtual Channels configured on this link.
    pub virtual_channels: Vec<SpFiVirtualChannel>,
    /// Link line encoding efficiency (e.g. 8b/10b = 0.80, 64b/66b = 0.97).
    pub encoding_efficiency: f64,
}

impl Default for SpaceFibreMultiLaneLink {
    fn default() -> Self {
        let mut vcs = Vec::with_capacity(4);
        // VC 0: Telemetry & Critical Command (Strict Priority)
        vcs.push(SpFiVirtualChannel::new(0, 7, 100, 200.0, 16384));
        // VC 1: Science Instrument High-Rate Payload (WRR)
        vcs.push(SpFiVirtualChannel::new(1, 4, 80, 2000.0, 65536));
        // VC 2: High-Resolution Imaging / SAR Sensor (WRR)
        vcs.push(SpFiVirtualChannel::new(2, 3, 50, 1500.0, 65536));
        // VC 3: Auxiliary Housekeeping (Best Effort)
        vcs.push(SpFiVirtualChannel::new(3, 1, 10, 100.0, 8192));

        Self {
            lane_count: 2,
            lane_bit_rate_gbps: 3.125, // 2 * 3.125 = 6.25 Gbps aggregate
            scheduling: SpFiQoSScheduling::WeightedRoundRobin,
            virtual_channels: vcs,
            encoding_efficiency: 0.80, // 8b/10b
        }
    }
}

impl SpaceFibreMultiLaneLink {
    /// Calculate aggregate net payload capacity in Megabits per second (Mbps).
    pub fn aggregate_payload_rate_mbps(&self) -> f64 {
        self.lane_count as f64 * (self.lane_bit_rate_gbps * 1000.0) * self.encoding_efficiency
    }

    /// Step simulation forward by dt_us microseconds with incoming bursts.
    /// Returns transmitted bytes per VC during this step.
    pub fn step(&mut self, dt_us: f64, incoming_bytes: &[(u8, usize)]) -> Vec<(u8, usize)> {
        // Enqueue incoming traffic
        for &(vc_id, bytes) in incoming_bytes {
            if let Some(vc) = self.virtual_channels.iter_mut().find(|v| v.vc_id == vc_id) {
                vc.enqueue(bytes);
            }
        }

        // Total link transmission capacity in bytes over dt_us
        let total_link_mbps = self.aggregate_payload_rate_mbps();
        let step_capacity_bytes = ((total_link_mbps * 1.0e6 / 8.0) * (dt_us * 1.0e-6)).round() as usize;

        let mut remaining_capacity = step_capacity_bytes;
        let mut results = Vec::with_capacity(self.virtual_channels.len());

        match self.scheduling {
            SpFiQoSScheduling::StrictPriority => {
                // Sort by priority descending
                let mut sorted_indices: Vec<usize> = (0..self.virtual_channels.len()).collect();
                sorted_indices.sort_by(|&a, &b| {
                    self.virtual_channels[b].priority.cmp(&self.virtual_channels[a].priority)
                });

                for idx in sorted_indices {
                    let vc = &mut self.virtual_channels[idx];
                    let tx_bytes = vc.buffer_fill_bytes.min(remaining_capacity);
                    vc.buffer_fill_bytes -= tx_bytes;
                    vc.total_tx_bytes += tx_bytes as u64;
                    remaining_capacity -= tx_bytes;
                    results.push((vc.vc_id, tx_bytes));
                }
            }
            SpFiQoSScheduling::WeightedRoundRobin => {
                let total_weight: u32 = self
                    .virtual_channels
                    .iter()
                    .filter(|v| v.buffer_fill_bytes > 0)
                    .map(|v| v.weight)
                    .sum();

                if total_weight > 0 {
                    for vc in &mut self.virtual_channels {
                        if vc.buffer_fill_bytes > 0 {
                            let quota = ((remaining_capacity as f64 * vc.weight as f64) / total_weight as f64).round() as usize;
                            let tx_bytes = vc.buffer_fill_bytes.min(quota);
                            vc.buffer_fill_bytes -= tx_bytes;
                            vc.total_tx_bytes += tx_bytes as u64;
                            results.push((vc.vc_id, tx_bytes));
                        }
                    }
                }
            }
            SpFiQoSScheduling::BandwidthReservation => {
                for vc in &mut self.virtual_channels {
                    let reserved_bytes = ((vc.reserved_bandwidth_mbps * 1.0e6 / 8.0) * (dt_us * 1.0e-6)).round() as usize;
                    let tx_bytes = vc.buffer_fill_bytes.min(reserved_bytes).min(remaining_capacity);
                    vc.buffer_fill_bytes -= tx_bytes;
                    vc.total_tx_bytes += tx_bytes as u64;
                    remaining_capacity = remaining_capacity.saturating_sub(tx_bytes);
                    results.push((vc.vc_id, tx_bytes));
                }
            }
        }

        results
    }

    /// Simulate payload burst flood on a specified VC over duration_us.
    /// Returns vector of (time_us, VC buffer fill ratio, link utilization in [0.0, 1.0]).
    pub fn simulate_burst_flood(
        &mut self,
        duration_us: f64,
        burst_vc: u8,
        burst_rate_gbps: f64,
        points: usize,
    ) -> Vec<(f64, f64, f64)> {
        let pts = points.clamp(40, 500);
        let dt_us = duration_us / (pts - 1) as f64;
        let mut trajectory = Vec::with_capacity(pts);

        let incoming_burst_bytes = ((burst_rate_gbps * 1.0e9 / 8.0) * (dt_us * 1.0e-6)).round() as usize;

        for i in 0..pts {
            let t_us = i as f64 * dt_us;

            // Step link
            let incoming = vec![(burst_vc, incoming_burst_bytes), (0, 128)]; // Critical telemetry traffic concurrent
            let tx_results = self.step(dt_us, &incoming);

            let total_tx: usize = tx_results.iter().map(|&(_, b)| b).sum();
            let max_capacity_bytes = ((self.aggregate_payload_rate_mbps() * 1.0e6 / 8.0) * (dt_us * 1.0e-6)).max(1.0);
            let link_util = (total_tx as f64 / max_capacity_bytes).clamp(0.0, 1.0);

            let target_fill = self
                .virtual_channels
                .iter()
                .find(|v| v.vc_id == burst_vc)
                .map(|v| v.buffer_fill_ratio())
                .unwrap_or(0.0);

            trajectory.push((t_us, target_fill, link_util));
        }

        trajectory
    }
}
