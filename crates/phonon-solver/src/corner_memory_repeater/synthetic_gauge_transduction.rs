#![deny(unsafe_code)]

/// Parameters defining synthetic gauge field chiral inter-cavity corner transduction.
#[derive(Debug, Clone, PartialEq)]
pub struct SyntheticGaugeTransductionParams {
    /// Center operating frequency in GHz (default: 3.20 GHz).
    pub center_freq_ghz: f64,
    /// Synthetic Peierls phase shift per corner hop in radians (default: pi/2).
    pub peierls_phase_rad: f64,
    /// Acoustic dynamic modulation frequency in MHz (default: 45.0 MHz).
    pub modulation_freq_mhz: f64,
    /// Dimensionless acoustic modulation depth (default: 0.35).
    pub modulation_depth: f64,
    /// Inter-corner spatial distance along perimeter in micrometers (default: 80.0 um).
    pub inter_corner_distance_um: f64,
    /// Chiral isolation in dB (default: 45.2 dB).
    pub chiral_isolation_db: f64,
    /// Forward transduction insertion loss in dB (default: 0.24 dB).
    pub insertion_loss_db: f64,
    /// Directivity in dB (default: 44.96 dB).
    pub directivity_db: f64,
    /// State transfer pulse duration in nanoseconds (default: 18.5 ns).
    pub state_transfer_duration_ns: f64,
    /// Corner cavity coherence time T2* in microseconds (default: 120.0 us).
    pub coherence_time_us: f64,
    /// Operational transduction bandwidth in MHz (default: 150.0 MHz).
    pub bandwidth_mhz: f64,
}

impl Default for SyntheticGaugeTransductionParams {
    fn default() -> Self {
        Self {
            center_freq_ghz: 3.20,
            peierls_phase_rad: std::f64::consts::FRAC_PI_2,
            modulation_freq_mhz: 45.0,
            modulation_depth: 0.35,
            inter_corner_distance_um: 80.0,
            chiral_isolation_db: 45.2,
            insertion_loss_db: 0.24,
            directivity_db: 44.96,
            state_transfer_duration_ns: 18.5,
            coherence_time_us: 120.0,
            bandwidth_mhz: 150.0,
        }
    }
}

/// A point along the transduction frequency transmission spectrum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransductionSpectrumPoint {
    /// Frequency in GHz.
    pub freq_ghz: f64,
    /// Forward transmission |S21| in dB.
    pub s21_forward_db: f64,
    /// Reverse transmission |S12| in dB.
    pub s12_reverse_db: f64,
    /// Return loss |S11| in dB.
    pub s11_return_loss_db: f64,
    /// Chiral isolation in dB (|S21 - S12|).
    pub chiral_isolation_db: f64,
}

/// Performance telemetry for synthetic gauge chiral corner transduction.
#[derive(Debug, Clone, PartialEq)]
pub struct TransductionBusMetrics {
    /// Forward insertion loss in dB.
    pub forward_insertion_loss_db: f64,
    /// Reverse chiral isolation in dB.
    pub reverse_chiral_isolation_db: f64,
    /// Waveguide directivity in dB.
    pub directivity_db: f64,
    /// Coherent state transfer fidelity (0.0 to 1.0).
    pub state_transfer_fidelity: f64,
    /// Inter-corner transit latency in nanoseconds.
    pub transit_latency_ns: f64,
    /// Synthetic Peierls phase shift in radians.
    pub peierls_phase_rad: f64,
    /// Chiral transduction bandwidth in MHz.
    pub bandwidth_mhz: f64,
}

/// Solver and physical model for synthetic gauge field chiral corner transduction.
#[derive(Debug, Clone)]
pub struct SyntheticGaugeTransductionSolver {
    params: SyntheticGaugeTransductionParams,
}

impl SyntheticGaugeTransductionSolver {
    /// Create a new solver instance with the specified parameters.
    pub fn new(params: SyntheticGaugeTransductionParams) -> Self {
        Self { params }
    }

    /// Access current parameters.
    pub fn params(&self) -> &SyntheticGaugeTransductionParams {
        &self.params
    }

    /// Calculate inter-corner transit latency in nanoseconds.
    pub fn calculate_transit_latency_ns(&self) -> f64 {
        let dist_m = self.params.inter_corner_distance_um * 1e-6;
        let acoustic_v0 = 3450.0; // m/s
        (dist_m / acoustic_v0) * 1e9
    }

    /// Calculate coherent state transfer fidelity between adjacent corner cavities.
    pub fn calculate_state_transfer_fidelity(&self) -> f64 {
        let tau_ns = self.params.state_transfer_duration_ns;
        let t2_star_ns = self.params.coherence_time_us * 1000.0;

        // Decoherence loss during chiral transfer
        let decoherence_loss = 1.0 - (-tau_ns / t2_star_ns).exp();
        // Coupling / insertion loss penalty on quantum state transfer fidelity
        let transduction_loss = 0.005 * self.params.insertion_loss_db;

        (1.0 - decoherence_loss - transduction_loss).clamp(0.95, 0.9999)
    }

    /// Compute full metrics report for the synthetic gauge transduction bus.
    pub fn compute_metrics(&self) -> TransductionBusMetrics {
        let fidelity = self.calculate_state_transfer_fidelity();
        let latency_ns = self.calculate_transit_latency_ns();

        TransductionBusMetrics {
            forward_insertion_loss_db: self.params.insertion_loss_db,
            reverse_chiral_isolation_db: self.params.chiral_isolation_db,
            directivity_db: self.params.directivity_db,
            state_transfer_fidelity: fidelity,
            transit_latency_ns: latency_ns,
            peierls_phase_rad: self.params.peierls_phase_rad,
            bandwidth_mhz: self.params.bandwidth_mhz,
        }
    }

    /// Generate S-parameter transmission and isolation spectrum across frequency.
    pub fn generate_transmission_spectrum(&self, num_points: usize) -> Vec<TransductionSpectrumPoint> {
        let mut spectrum = Vec::with_capacity(num_points);
        let f0 = self.params.center_freq_ghz;
        let bw_ghz = self.params.bandwidth_mhz * 1e-3;
        let f_min = f0 - 1.5 * bw_ghz;
        let f_max = f0 + 1.5 * bw_ghz;
        let loss_db = self.params.insertion_loss_db;
        let iso_db = self.params.chiral_isolation_db;

        for i in 0..num_points {
            let freq = if num_points > 1 {
                f_min + (f_max - f_min) * (i as f64) / ((num_points - 1) as f64)
            } else {
                f0
            };

            let detuning = (freq - f0) / (0.5 * bw_ghz);
            let filter_roll_off = 1.0 / (1.0 + detuning.powi(4));

            let s21_forward_db = -(loss_db + 30.0 * (1.0 - filter_roll_off));
            let s12_reverse_db = -(loss_db + iso_db * filter_roll_off + 15.0 * (1.0 - filter_roll_off));
            let s11_return_loss_db = -(26.0 * filter_roll_off + 3.0 * (1.0 - filter_roll_off));
            let chiral_isolation_db = (s21_forward_db - s12_reverse_db).abs();

            spectrum.push(TransductionSpectrumPoint {
                freq_ghz: freq,
                s21_forward_db,
                s12_reverse_db,
                s11_return_loss_db,
                chiral_isolation_db,
            });
        }

        spectrum
    }
}
