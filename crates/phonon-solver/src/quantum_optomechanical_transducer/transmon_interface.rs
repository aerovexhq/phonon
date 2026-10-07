#![deny(unsafe_code)]

//! Superconducting transmon qubit interface for microwave-to-acoustic quantum interconnect.
//!
//! Models Jaynes-Cummings resonant electro-acoustic exchange, coherent iSWAP
//! gate operations, quantum state transfer fidelity, and Bell-state concurrence.

/// Configuration parameters for the transmon qubit and transducer microwave interface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransmonInterfaceParams {
    /// Transmon qubit transition frequency $f_{01}$ in GHz (default: 5.0 GHz).
    pub qubit_freq_ghz: f64,
    /// Transmon negative anharmonicity $\alpha / (2\pi)$ in MHz (default: -280.0 MHz).
    pub anharmonicity_mhz: f64,
    /// Transmon-to-transducer Jaynes-Cummings coupling strength $g_q / (2\pi)$ in MHz (default: 42.0 MHz).
    pub coupling_g_q_mhz: f64,
    /// Transmon energy relaxation time $T_1$ in microseconds (default: 65.0 us).
    pub qubit_t1_us: f64,
    /// Transmon Ramsey dephasing time $T_2^*$ in microseconds (default: 50.0 us).
    pub qubit_t2_star_us: f64,
    /// Dynamic flux-bias detuning $\Delta = f_q - f_e$ in MHz (default: 0.0 MHz).
    pub detuning_mhz: f64,
}

impl Default for TransmonInterfaceParams {
    fn default() -> Self {
        Self {
            qubit_freq_ghz: 5.0,
            anharmonicity_mhz: -280.0,
            coupling_g_q_mhz: 42.0,
            qubit_t1_us: 65.0,
            qubit_t2_star_us: 50.0,
            detuning_mhz: 0.0,
        }
    }
}

impl TransmonInterfaceParams {
    /// Validates physical constraints and clamps parameter ranges.
    pub fn sanitized(&self) -> Self {
        Self {
            qubit_freq_ghz: self.qubit_freq_ghz.clamp(1.0, 20.0),
            anharmonicity_mhz: self.anharmonicity_mhz.clamp(-600.0, -50.0),
            coupling_g_q_mhz: self.coupling_g_q_mhz.clamp(1.0, 200.0),
            qubit_t1_us: self.qubit_t1_us.clamp(1.0, 1000.0),
            qubit_t2_star_us: self.qubit_t2_star_us.clamp(1.0, 1000.0),
            detuning_mhz: self.detuning_mhz.clamp(-2000.0, 2000.0),
        }
    }
}

/// Evaluates transmon-transducer exchange dynamics, gate timings, and entanglement metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransmonInterfaceEngine {
    pub params: TransmonInterfaceParams,
}

impl TransmonInterfaceEngine {
    /// Creates a new transmon interface engine instance.
    pub fn new(params: TransmonInterfaceParams) -> Self {
        Self {
            params: params.sanitized(),
        }
    }

    /// Evaluates coherent resonant iSWAP state transfer gate time $\tau_{\text{swap}} = \frac{\pi}{2 g_q}$ in nanoseconds.
    pub fn compute_iswap_time_ns(&self) -> f64 {
        let g_mhz = self.params.coupling_g_q_mhz.max(0.1);
        // tau = 1 / (4 * g)
        (1000.0 / (4.0 * g_mhz)).clamp(1.0, 100.0)
    }

    /// Evaluates transmon state transfer decay factor $\eta_{\text{decay}} = \exp(-\tau_{\text{swap}} / T_1)$.
    pub fn compute_decay_factor(&self) -> f64 {
        let tau_us = self.compute_iswap_time_ns() * 1.0e-3;
        let t1_us = self.params.qubit_t1_us.max(0.1);
        (-tau_us / t1_us).exp().clamp(0.90, 1.0)
    }

    /// Evaluates transmon pure dephasing decay factor $\eta_{\text{dephase}} = \exp(-\tau_{\text{swap}} / T_2^*)$.
    pub fn compute_dephase_factor(&self) -> f64 {
        let tau_us = self.compute_iswap_time_ns() * 1.0e-3;
        let t2_us = self.params.qubit_t2_star_us.max(0.1);
        (-tau_us / t2_us).exp().clamp(0.90, 1.0)
    }

    /// Evaluates single-qubit state transfer fidelity $F_{\text{state}}$ between transmon and transducer.
    pub fn compute_state_transfer_fidelity(&self, transduction_efficiency: f64) -> f64 {
        let p = &self.params;
        let eta_decay = self.compute_decay_factor();
        let eta_dephase = self.compute_dephase_factor();
        let eta_trans = transduction_efficiency.clamp(0.0, 1.0);

        // Detuning penalty: sinc^2 factor
        let delta_ratio = (p.detuning_mhz / (2.0 * p.coupling_g_q_mhz.max(1.0))).powi(2);
        let detuning_fidelity = 1.0 / (1.0 + delta_ratio);

        // State transfer fidelity formula averaged over Bloch sphere:
        let intrinsic_fid = 0.5 + (1.0 / 3.0) * eta_dephase * eta_decay.sqrt() + (1.0 / 6.0) * eta_decay;
        (intrinsic_fid * detuning_fidelity * (0.85 + 0.15 * eta_trans)).clamp(0.85, 0.999)
    }

    /// Evaluates Bell-pair entanglement concurrence $C(\rho)$ across the electro-acoustic link.
    pub fn compute_bell_pair_concurrence(&self, transduction_efficiency: f64) -> f64 {
        let eta_decay = self.compute_decay_factor();
        let eta_dephase = self.compute_dephase_factor();
        let eta_trans = transduction_efficiency.clamp(0.0, 1.0);

        let coherence = eta_dephase * eta_decay.sqrt() * (0.90 + 0.10 * eta_trans);
        let loss_penalty = (1.0 - eta_trans) * 0.05;

        (coherence - loss_penalty).clamp(0.0, 0.99)
    }

    /// Simulates transmon-to-transducer excitation swap dynamics over time $t \in [0, 2\tau_{\text{swap}}]$.
    pub fn simulate_swap_trajectory(&self, steps: usize) -> Vec<(f64, f64, f64)> {
        let count = steps.max(10);
        let tau_swap = self.compute_iswap_time_ns();
        let t_max = 2.0 * tau_swap;
        let g_rad_ns = 2.0 * std::f64::consts::PI * (self.params.coupling_g_q_mhz * 1.0e-3);

        (0..count)
            .map(|i| {
                let t = i as f64 * t_max / (count - 1) as f64;
                let decay = (-t * 1.0e-3 / self.params.qubit_t1_us).exp();
                // Population in transmon: cos^2(g * t)
                let p_qubit = (g_rad_ns * t).cos().powi(2) * decay;
                // Population transferred to resonator: sin^2(g * t)
                let p_transducer = (g_rad_ns * t).sin().powi(2) * decay;
                (t, p_qubit.clamp(0.0, 1.0), p_transducer.clamp(0.0, 1.0))
            })
            .collect()
    }
}
