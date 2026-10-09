#![deny(unsafe_code)]

/// Physical parameters for the multi-node cryogenic valley bus.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyBusParams {
    /// Number of connected memory/qubit nodes along the bus (default: 8).
    pub node_count: usize,
    /// Total waveguide bus length in millimeters (default: 1.2 mm).
    pub bus_length_mm: f64,
    /// Spatial spacing between adjacent nodes in micrometers (default: 150.0 um).
    pub inter_node_spacing_um: f64,
    /// Intrinsic chiral valley isolation in dB (default: 44.5 dB).
    pub chiral_isolation_db: f64,
    /// Forward insertion loss per node traversal in dB (default: 0.035 dB).
    pub insertion_loss_per_node_db: f64,
    /// Waveguide directivity in dB (default: 44.2 dB).
    pub directivity_db: f64,
    /// Inter-node crosstalk isolation in dB (default: 46.5 dB).
    pub crosstalk_isolation_db: f64,
    /// Cryo-CMOS interface power dissipation per node in mW (default: 0.095 mW).
    pub cryo_power_per_node_mw: f64,
    /// Operating center frequency in GHz (default: 3.50 GHz).
    pub center_freq_ghz: f64,
    /// Chiral valley transmission bandwidth in MHz (default: 180.0 MHz).
    pub bandwidth_mhz: f64,
}

impl Default for ValleyBusParams {
    fn default() -> Self {
        Self {
            node_count: 8,
            bus_length_mm: 1.2,
            inter_node_spacing_um: 150.0,
            chiral_isolation_db: 44.5,
            insertion_loss_per_node_db: 0.035,
            directivity_db: 44.2,
            crosstalk_isolation_db: 46.5,
            cryo_power_per_node_mw: 0.095,
            center_freq_ghz: 3.50,
            bandwidth_mhz: 180.0,
        }
    }
}

/// S-parameter transmission spectrum point across frequency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BusSpectrumPoint {
    /// Frequency in GHz.
    pub freq_ghz: f64,
    /// Forward transmission |S21| in dB (for valley K).
    pub s21_forward_db: f64,
    /// Reverse transmission |S12| in dB (for valley K).
    pub s12_reverse_db: f64,
    /// Return loss |S11| in dB.
    pub s11_return_loss_db: f64,
    /// Valley isolation in dB (S21 - S12).
    pub valley_isolation_db: f64,
}

/// Two-qubit entanglement metrics generated via the valley bus.
#[derive(Debug, Clone, PartialEq)]
pub struct BusEntanglementMetrics {
    /// Wootters concurrence C in [0.0, 1.0].
    pub concurrence: f64,
    /// Bell state generation fidelity (0.0 to 1.0).
    pub bell_fidelity: f64,
    /// Bell-CHSH inequality parameter S (classical bound <= 2.0, maximal = 2*sqrt(2) ~ 2.828).
    pub chsh_parameter: f64,
    /// Entanglement formation rate in MHz.
    pub entanglement_rate_mhz: f64,
}

/// Performance telemetry for the cryogenic valley bus.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyBusRoutingMetrics {
    /// Total end-to-end forward insertion loss in dB.
    pub end_to_end_insertion_loss_db: f64,
    /// Worst-case reverse isolation across all nodes in dB.
    pub reverse_isolation_db: f64,
    /// Inter-channel crosstalk suppression in dB.
    pub crosstalk_suppression_db: f64,
    /// Waveguide directivity in dB.
    pub directivity_db: f64,
    /// Single-hop inter-node transit latency in nanoseconds.
    pub single_hop_latency_ns: f64,
    /// Total Cryo-CMOS routing power dissipation across all nodes in mW.
    pub total_cryo_power_mw: f64,
    /// Two-qubit entanglement metrics.
    pub entanglement: BusEntanglementMetrics,
}

/// Multi-node cryogenic valley bus solver and router.
#[derive(Debug, Clone)]
pub struct MultiNodeValleyBus {
    params: ValleyBusParams,
}

impl MultiNodeValleyBus {
    /// Create a new multi-node valley bus solver.
    pub fn new(params: ValleyBusParams) -> Self {
        Self { params }
    }

    /// Access current parameters.
    pub fn params(&self) -> &ValleyBusParams {
        &self.params
    }

    /// Calculate total end-to-end forward insertion loss across all N nodes.
    pub fn calculate_end_to_end_loss_db(&self) -> f64 {
        let base_waveguide_loss_db = 0.05;
        let hops = (self.params.node_count.max(1) - 1) as f64;
        base_waveguide_loss_db + hops * self.params.insertion_loss_per_node_db
    }

    /// Calculate single-hop transit latency in nanoseconds.
    pub fn calculate_single_hop_latency_ns(&self) -> f64 {
        let spacing_m = self.params.inter_node_spacing_um * 1e-6;
        let acoustic_v0 = 3450.0; // m/s
        (spacing_m / acoustic_v0) * 1e9
    }

    /// Calculate total cryo-CMOS interface power dissipation in mW.
    pub fn calculate_total_power_mw(&self) -> f64 {
        (self.params.node_count as f64) * self.params.cryo_power_per_node_mw
    }

    /// Evaluate two-qubit entanglement metrics mediated by the valley bus.
    pub fn evaluate_entanglement(&self) -> BusEntanglementMetrics {
        let end_to_end_loss = self.calculate_end_to_end_loss_db();
        let transmission_linear = 10.0_f64.powf(-end_to_end_loss / 10.0);

        // Concurrence scales with valley transmission and bus coherence
        let concurrence = (0.955 * transmission_linear.sqrt()).clamp(0.85, 0.99);
        let bell_fidelity = (0.5 * (1.0 + concurrence) + 0.035).clamp(0.90, 0.9995);
        let chsh_parameter = 2.0 * std::f64::consts::SQRT_2 * concurrence;
        let entanglement_rate_mhz = 12.5 * transmission_linear;

        BusEntanglementMetrics {
            concurrence,
            bell_fidelity,
            chsh_parameter,
            entanglement_rate_mhz,
        }
    }

    /// Compute full routing performance metrics for the valley bus.
    pub fn compute_metrics(&self) -> ValleyBusRoutingMetrics {
        let end_to_end_loss = self.calculate_end_to_end_loss_db();
        let latency_ns = self.calculate_single_hop_latency_ns();
        let total_power = self.calculate_total_power_mw();
        let entanglement = self.evaluate_entanglement();

        ValleyBusRoutingMetrics {
            end_to_end_insertion_loss_db: end_to_end_loss,
            reverse_isolation_db: self.params.chiral_isolation_db,
            crosstalk_suppression_db: self.params.crosstalk_isolation_db,
            directivity_db: self.params.directivity_db,
            single_hop_latency_ns: latency_ns,
            total_cryo_power_mw: total_power,
            entanglement,
        }
    }

    /// Generate S-parameter transmission and isolation spectrum across frequency.
    pub fn generate_transmission_spectrum(&self, num_points: usize) -> Vec<BusSpectrumPoint> {
        let mut spectrum = Vec::with_capacity(num_points);
        let f0 = self.params.center_freq_ghz;
        let bw_ghz = self.params.bandwidth_mhz * 1e-3;
        let f_min = f0 - 1.5 * bw_ghz;
        let f_max = f0 + 1.5 * bw_ghz;
        let min_loss_db = self.calculate_end_to_end_loss_db();
        let max_isolation_db = self.params.chiral_isolation_db;

        for i in 0..num_points {
            let freq = if num_points > 1 {
                f_min + (f_max - f_min) * (i as f64) / ((num_points - 1) as f64)
            } else {
                f0
            };

            let detuning = (freq - f0) / (0.5 * bw_ghz);
            let filter_roll_off = 1.0 / (1.0 + detuning.powi(4));

            let s21_forward_db = -(min_loss_db + 25.0 * (1.0 - filter_roll_off));
            let s12_reverse_db = -(min_loss_db + max_isolation_db * filter_roll_off + 15.0 * (1.0 - filter_roll_off));
            let s11_return_loss_db = -(24.0 * filter_roll_off + 3.0 * (1.0 - filter_roll_off));
            let valley_isolation_db = (s21_forward_db - s12_reverse_db).abs();

            spectrum.push(BusSpectrumPoint {
                freq_ghz: freq,
                s21_forward_db,
                s12_reverse_db,
                s11_return_loss_db,
                valley_isolation_db,
            });
        }

        spectrum
    }
}
