#![deny(unsafe_code)]

//! Phase 435: Transmon Circuit QED & Quantum State Transfer Engine.
//!
//! Models superconducting transmon qubits coupled to higher-order acoustic corner modes,
//! Jaynes-Cummings vacuum Rabi oscillations, coherent state transfer, and dispersive QND readout.

/// Parameters for the superconducting transmon qubit.
#[derive(Debug, Clone, PartialEq)]
pub struct TransmonParams {
    /// Josephson energy EJ in GHz (default: 18.5 GHz).
    pub ej_ghz: f64,
    /// Charging energy EC in MHz (default: 240.0 MHz).
    pub ec_mhz: f64,
    /// Energy relaxation time T1 in microseconds (default: 80.0 us).
    pub t1_us: f64,
    /// Dephasing time T2 in microseconds (default: 65.0 us).
    pub t2_us: f64,
    /// Detuning Delta / 2pi between transmon and corner phonon mode in MHz (default: 150.0 MHz).
    pub detuning_mhz: f64,
    /// Dispersive readout cavity integration time in nanoseconds (default: 120.0 ns).
    pub readout_time_ns: f64,
}

impl Default for TransmonParams {
    fn default() -> Self {
        Self {
            ej_ghz: 18.5,
            ec_mhz: 240.0,
            t1_us: 80.0,
            t2_us: 65.0,
            detuning_mhz: 150.0,
            readout_time_ns: 120.0,
        }
    }
}

/// Point on the Rabi oscillation time trajectory.
#[derive(Debug, Clone, PartialEq)]
pub struct RabiOscillationPoint {
    /// Timestamp in nanoseconds.
    pub time_ns: f64,
    /// Excited state population of the transmon qubit Pe(t).
    pub p_qubit: f64,
    /// Single-phonon state occupancy of the acoustic corner mode Pph(t).
    pub p_phonon: f64,
    /// Instantaneous state transfer fidelity.
    pub fidelity: f64,
}

/// Metrics for the transmon-acoustic circuit QED interaction.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitQedMetrics {
    /// Transmon transition frequency omega_q / 2pi in GHz.
    pub qubit_freq_ghz: f64,
    /// Transmon negative anharmonicity alpha in MHz (approximately -EC).
    pub anharmonicity_mhz: f64,
    /// Ratio of Josephson energy to charging energy EJ / EC.
    pub ej_ec_ratio: f64,
    /// On-resonance Jaynes-Cummings vacuum Rabi splitting 2g / 2pi in MHz.
    pub vacuum_rabi_mhz: f64,
    /// Dispersive frequency shift chi / 2pi in MHz (g^2 / Delta).
    pub dispersive_shift_mhz: f64,
    /// Coherent state transfer swap time tau_swap in nanoseconds (pi / 2g).
    pub tau_swap_ns: f64,
    /// Transmon-to-phonon quantum state transfer fidelity F_transfer in [0.0, 1.0].
    pub state_transfer_fidelity: f64,
    /// Dispersive QND readout Signal-to-Noise Ratio (SNR) in dB.
    pub qnd_readout_snr_db: f64,
    /// Ratio of qubit dephasing time to state transfer swap time (T2* / tau_swap).
    pub coherence_ratio: f64,
}

/// Solver for transmon circuit QED and quantum state transfer.
#[derive(Debug, Clone, PartialEq)]
pub struct TransmonCircuitQedSolver {
    pub params: TransmonParams,
}

impl Default for TransmonCircuitQedSolver {
    fn default() -> Self {
        Self {
            params: TransmonParams::default(),
        }
    }
}

impl TransmonCircuitQedSolver {
    /// Creates a new solver with custom transmon parameters.
    pub fn new(params: TransmonParams) -> Self {
        Self { params }
    }

    /// Solves the circuit QED eigensystem, vacuum Rabi dynamics, and state transfer fidelity.
    pub fn solve_dynamics(
        &self,
        coupling_trans_mhz: f64,
    ) -> (CircuitQedMetrics, Vec<RabiOscillationPoint>) {
        let p = &self.params;

        // Convert EJ to MHz: 1 GHz = 1000 MHz
        let ej_mhz = p.ej_ghz * 1000.0;
        let ec_mhz = p.ec_mhz;

        // EJ / EC ratio
        let ej_ec_ratio = if ec_mhz > 1e-6 {
            ej_mhz / ec_mhz
        } else {
            100.0
        };

        // Transmon transition frequency omega_q / 2pi = sqrt(8 * EJ * EC) - EC
        let qubit_freq_mhz = (8.0 * ej_mhz * ec_mhz).sqrt() - ec_mhz;
        let qubit_freq_ghz = qubit_freq_mhz / 1000.0;

        // Anharmonicity alpha = -EC
        let anharmonicity_mhz = -ec_mhz;

        // Vacuum Rabi splitting on resonance: 2 * g_trans
        let vacuum_rabi_mhz = 2.0 * coupling_trans_mhz;

        // State transfer swap time: tau_swap = pi / (2 * g_trans_rad)
        // With g_trans_mhz = g / (2*pi), g_rad = 2 * pi * g_trans_mhz * 1e6 rad/s
        // tau_swap = pi / (2 * 2 * pi * g_trans_mhz * 1e6) = 1 / (4 * g_trans_mhz * 1e6) s
        // In nanoseconds: 1e9 / (4 * g_trans_mhz * 1e6) = 250.0 / g_trans_mhz
        let tau_swap_ns = if coupling_trans_mhz > 1e-6 {
            250.0 / coupling_trans_mhz
        } else {
            1000.0
        };

        // Dispersive shift: chi = g^2 / Delta
        let dispersive_shift_mhz = if p.detuning_mhz.abs() > 1e-6 {
            (coupling_trans_mhz * coupling_trans_mhz) / p.detuning_mhz.abs()
        } else {
            coupling_trans_mhz
        };

        // Coherence ratio: T2* / tau_swap
        let t2_ns = p.t2_us * 1000.0;
        let coherence_ratio = if tau_swap_ns > 1e-6 {
            t2_ns / tau_swap_ns
        } else {
            10000.0
        };

        // Coherent state transfer fidelity: F = exp(-tau_swap / T2*)
        let state_transfer_fidelity = (-tau_swap_ns / t2_ns).exp();

        // Dispersive QND Readout SNR:
        // SNR = 10 * log10(4 * chi^2 * n_photons * tau_meas / kappa_ro)
        // With steady-state readout cavity photon occupancy n_photons ~ 18 (< n_crit):
        let n_photons = 18.0;
        let chi_rad_s = dispersive_shift_mhz * 1e6 * 2.0 * std::f64::consts::PI;
        let tau_meas_s = p.readout_time_ns * 1e-9;
        let kappa_ro_rad_s = 2.0 * std::f64::consts::PI * 2.0e6; // 2 MHz cavity linewidth
        let raw_snr = (4.0 * chi_rad_s * chi_rad_s * n_photons * tau_meas_s / kappa_ro_rad_s).max(1.0);
        let qnd_readout_snr_db = 10.0 * raw_snr.log10();

        // Compute Rabi oscillation trajectory over 100 steps
        let t_max_ns = (4.0 * tau_swap_ns).max(80.0);
        let steps = 100;
        let mut trajectory = Vec::with_capacity(steps);

        let t1_ns = p.t1_us * 1000.0;
        let omega_rabi_rad_ns = 2.0 * std::f64::consts::PI * coupling_trans_mhz * 1e-3;

        for i in 0..steps {
            let t = (i as f64 / (steps - 1) as f64) * t_max_ns;
            let decay = (-t / t1_ns).exp();

            // Pe(t) = cos^2(omega_rabi * t / 2) * decay
            let cos_term = (omega_rabi_rad_ns * t).cos();
            let p_qubit = (0.5 * (1.0 + cos_term)) * decay;
            let p_phonon = (0.5 * (1.0 - cos_term)) * decay;

            let fidelity = p_phonon * (-t / t2_ns).exp();

            trajectory.push(RabiOscillationPoint {
                time_ns: t,
                p_qubit,
                p_phonon,
                fidelity,
            });
        }

        let metrics = CircuitQedMetrics {
            qubit_freq_ghz,
            anharmonicity_mhz,
            ej_ec_ratio,
            vacuum_rabi_mhz,
            dispersive_shift_mhz,
            tau_swap_ns,
            state_transfer_fidelity,
            qnd_readout_snr_db,
            coherence_ratio,
        };

        (metrics, trajectory)
    }
}
