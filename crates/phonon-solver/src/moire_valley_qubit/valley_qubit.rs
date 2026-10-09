#![deny(unsafe_code)]

/// Parameters defining a twisted-bilayer acoustic moire valley qubit.
#[derive(Debug, Clone, PartialEq)]
pub struct MoireValleyQubitParams {
    /// Twist angle in degrees (e.g. 1.08 deg magic angle).
    pub twist_angle_deg: f64,
    /// Acoustic center frequency in GHz (default: 3.50 GHz).
    pub center_freq_ghz: f64,
    /// Acoustic Rabi drive frequency in MHz (default: 25.0 MHz).
    pub rabi_freq_mhz: f64,
    /// Inversion symmetry breaking valley detuning in MHz (default: 0.0 MHz for resonance).
    pub detuning_delta_mhz: f64,
    /// Pure dephasing rate in kHz (default: 8.0 kHz).
    pub dephasing_rate_khz: f64,
    /// Longitudinal relaxation rate in kHz (default: 2.5 kHz).
    pub relaxation_rate_khz: f64,
    /// Single-qubit pulse duration in nanoseconds (default: 20.0 ns).
    pub pulse_duration_ns: f64,
    /// Pulse envelope distortion / calibration error (default: 1.5e-4).
    pub pulse_error: f64,
}

impl Default for MoireValleyQubitParams {
    fn default() -> Self {
        Self {
            twist_angle_deg: 1.08,
            center_freq_ghz: 3.50,
            rabi_freq_mhz: 25.0,
            detuning_delta_mhz: 0.0,
            dephasing_rate_khz: 8.0,
            relaxation_rate_khz: 2.5,
            pulse_duration_ns: 20.0,
            pulse_error: 1.5e-4,
        }
    }
}

/// Valley pseudospin basis states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValleyPseudospinState {
    /// Valley K state (+1 eigenvalue of tau_z).
    ValleyK,
    /// Valley K-prime state (-1 eigenvalue of tau_z).
    ValleyKPrime,
    /// Symmetric superposition (|K> + |K'>) / sqrt(2).
    SuperpositionPlus,
    /// Anti-symmetric superposition (|K> - |K'>) / sqrt(2).
    SuperpositionMinus,
}

/// 3D Bloch vector representation of the valley pseudospin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyBlochVector {
    /// In-plane x projection: <tau_x>.
    pub tau_x: f64,
    /// In-plane y projection: <tau_y>.
    pub tau_y: f64,
    /// Longitudinal z projection: <tau_z> (valley contrast).
    pub tau_z: f64,
    /// Norm of the Bloch vector (purity = (1 + |tau|^2)/2).
    pub norm: f64,
}

impl ValleyBlochVector {
    /// Construct a Bloch vector from spherical coordinates (theta, phi).
    pub fn from_angles(theta_rad: f64, phi_rad: f64) -> Self {
        let tau_x = theta_rad.sin() * phi_rad.cos();
        let tau_y = theta_rad.sin() * phi_rad.sin();
        let tau_z = theta_rad.cos();
        let norm = (tau_x * tau_x + tau_y * tau_y + tau_z * tau_z).sqrt();
        Self {
            tau_x,
            tau_y,
            tau_z,
            norm,
        }
    }

    /// Construct a Bloch vector with purity decay factor r in [0, 1].
    pub fn with_purity(mut self, purity_factor: f64) -> Self {
        let pf = purity_factor.clamp(0.0, 1.0);
        self.tau_x *= pf;
        self.tau_y *= pf;
        self.tau_z *= pf;
        self.norm = (self.tau_x * self.tau_x + self.tau_y * self.tau_y + self.tau_z * self.tau_z).sqrt();
        self
    }
}

/// A point along the dynamic Rabi oscillation trajectory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyRabiPoint {
    /// Timestamp in nanoseconds.
    pub time_ns: f64,
    /// Population in Valley K.
    pub prob_k: f64,
    /// Population in Valley K-prime.
    pub prob_k_prime: f64,
    /// Valley inversion / contrast <tau_z> = prob_k - prob_k_prime.
    pub valley_contrast: f64,
}

/// Computed performance metrics for the valley qubit.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyQubitMetrics {
    /// Effective Rabi frequency in MHz.
    pub effective_rabi_freq_mhz: f64,
    /// Inhomogeneous dephasing time T2* in microseconds.
    pub coherence_time_t2_star_us: f64,
    /// Longitudinal lifetime T1 in microseconds.
    pub lifetime_t1_us: f64,
    /// Pure dephasing time T_phi in microseconds.
    pub dephasing_time_t_phi_us: f64,
    /// Single-qubit Clifford gate fidelity (0.0 to 1.0).
    pub gate_fidelity: f64,
    /// Valley leakage probability to higher moire sub-bands.
    pub leakage_probability: f64,
    /// Valley Chern number difference between K and K'.
    pub delta_valley_chern: i32,
}

/// Simulation and analytical solver engine for moire valley qubits.
#[derive(Debug, Clone)]
pub struct ValleyQubitEngine {
    params: MoireValleyQubitParams,
}

impl ValleyQubitEngine {
    /// Create a new solver instance with the specified parameters.
    pub fn new(params: MoireValleyQubitParams) -> Self {
        Self { params }
    }

    /// Get current parameters.
    pub fn params(&self) -> &MoireValleyQubitParams {
        &self.params
    }

    /// Calculate effective Rabi frequency: Omega_eff = sqrt(Omega_R^2 + Delta^2).
    pub fn effective_rabi_freq_mhz(&self) -> f64 {
        let rabi = self.params.rabi_freq_mhz;
        let delta = self.params.detuning_delta_mhz;
        (rabi * rabi + delta * delta).sqrt()
    }

    /// Calculate coherence times: T1, T_phi, and T2*.
    pub fn calculate_coherence_times(&self) -> (f64, f64, f64) {
        let gamma_1_khz = self.params.relaxation_rate_khz.max(1e-6);
        let gamma_phi_khz = self.params.dephasing_rate_khz.max(1e-6);

        let t1_us = 1000.0 / gamma_1_khz;
        let t_phi_us = 1000.0 / gamma_phi_khz;
        // 1 / T2* = 1 / (2*T1) + 1 / T_phi
        let rate_t2_star_khz = (gamma_1_khz * 0.5) + gamma_phi_khz;
        let t2_star_us = 1000.0 / rate_t2_star_khz;

        (t1_us, t_phi_us, t2_star_us)
    }

    /// Evaluate single-qubit gate fidelity and leakage probability.
    pub fn evaluate_gate_fidelity(&self) -> (f64, f64) {
        let (_, _, t2_star_us) = self.calculate_coherence_times();
        let tau_gate_us = self.params.pulse_duration_ns * 1e-3;

        // Decoherence loss: 1 - exp(-tau_gate / (2 * T2*)) approx tau_gate / (2 * T2*)
        let decoherence_loss = tau_gate_us / (2.0 * t2_star_us);
        let pulse_error = self.params.pulse_error;

        // Higher moire sub-band leakage: suppressed by inter-subband moire gap (e.g. ~160 MHz) and DRAG pulse shaping
        let moire_subband_gap_mhz = 160.0;
        let rabi = self.params.rabi_freq_mhz;
        let drag_suppression = 0.50;
        let leakage_probability = (drag_suppression * (rabi / (2.0 * moire_subband_gap_mhz)).powi(4)).clamp(1.0e-7, 1.0e-3);

        let total_infidelity = decoherence_loss + pulse_error + leakage_probability;
        let gate_fidelity = (1.0 - total_infidelity).clamp(0.0, 1.0);

        (gate_fidelity, leakage_probability)
    }

    /// Compute full metrics report for the valley qubit.
    pub fn compute_metrics(&self) -> ValleyQubitMetrics {
        let effective_rabi = self.effective_rabi_freq_mhz();
        let (t1_us, t_phi_us, t2_star_us) = self.calculate_coherence_times();
        let (gate_fidelity, leakage_probability) = self.evaluate_gate_fidelity();

        ValleyQubitMetrics {
            effective_rabi_freq_mhz: effective_rabi,
            coherence_time_t2_star_us: t2_star_us,
            lifetime_t1_us: t1_us,
            dephasing_time_t_phi_us: t_phi_us,
            gate_fidelity,
            leakage_probability,
            delta_valley_chern: 2,
        }
    }

    /// Generate time-resolved Rabi oscillation trajectory over a given duration.
    pub fn generate_rabi_trajectory(&self, max_time_ns: f64, num_steps: usize) -> Vec<ValleyRabiPoint> {
        let mut trajectory = Vec::with_capacity(num_steps);
        let omega_eff_rad_ns = 2.0 * std::f64::consts::PI * self.effective_rabi_freq_mhz() * 1e-3;
        let rabi_sq = self.params.rabi_freq_mhz * self.params.rabi_freq_mhz;
        let eff_sq = (self.effective_rabi_freq_mhz() * self.effective_rabi_freq_mhz()).max(1e-9);
        let (_, _, t2_star_us) = self.calculate_coherence_times();
        let gamma_decay_ns = 1.0 / (t2_star_us * 1000.0);

        for step in 0..num_steps {
            let t_ns = if num_steps > 1 {
                max_time_ns * (step as f64) / ((num_steps - 1) as f64)
            } else {
                0.0
            };

            let envelope = (-gamma_decay_ns * t_ns).exp();
            let raw_prob_k_prime = (rabi_sq / eff_sq) * (0.5 * omega_eff_rad_ns * t_ns).sin().powi(2);
            let prob_k_prime = (raw_prob_k_prime * envelope).clamp(0.0, 1.0);
            let prob_k = (1.0 - prob_k_prime).clamp(0.0, 1.0);
            let valley_contrast = prob_k - prob_k_prime;

            trajectory.push(ValleyRabiPoint {
                time_ns: t_ns,
                prob_k,
                prob_k_prime,
                valley_contrast,
            });
        }

        trajectory
    }
}
