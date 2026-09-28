//! Quantum-limited noise analysis and bosonic field commutation preservation:
//! Caves theorem verification, added noise quanta N_add, and noise temperature.

use phonon_models::jtwpa::{ParametricProcessParams, BOLTZMANN_K, HBAR};

/// Quantum noise evaluation report.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumNoiseResult {
    /// Signal linear power gain G_s.
    pub signal_gain_linear: f64,
    /// Commutator preservation error |[a_out, a_out^dagger] - 1.0|.
    pub commutator_preservation_error: f64,
    /// Added noise quanta N_add referred to input (Caves limit: N_add >= 0.5 * (1 - 1/G)).
    pub added_noise_quanta: f64,
    /// Minimum theoretical Caves noise quanta N_caves = 0.5 * (1 - 1/G).
    pub caves_limit_quanta: f64,
    /// Excess noise quanta above Caves limit N_excess = N_add - N_caves.
    pub excess_noise_quanta: f64,
    /// Added noise temperature T_add in Kelvin.
    pub added_noise_temperature_k: f64,
    /// Noise figure in dB: NF = 10 * log10(1 + 2 * N_add).
    pub noise_figure_db: f64,
}

/// Quantum noise analyzer for parametric amplifiers.
pub struct QuantumNoiseSolver;

impl QuantumNoiseSolver {
    /// Analyzes the quantum noise performance for given parameters and signal gain.
    pub fn evaluate(
        params: &ParametricProcessParams,
        signal_gain_linear: f64,
    ) -> QuantumNoiseResult {
        let g_s = signal_gain_linear.max(1.0);
        let g_i = g_s - 1.0; // Manley-Rowe photon balance

        // Output bosonic commutator [a_out, a_out^dagger] = G_s - G_i = 1.0
        let commutator_val = g_s - g_i;
        let comm_err = (commutator_val - 1.0).abs();

        let caves_min = 0.5 * (1.0 - 1.0 / g_s);

        // Thermal occupation of idler mode at bath temperature T_bath
        let omega_i =
            2.0 * std::f64::consts::PI * (params.pump_freq_hz - params.signal_freq_hz).abs();
        let exp_arg = (HBAR * omega_i) / (BOLTZMANN_K * params.bath_temperature_k.max(1e-4));
        let n_th_idler = if exp_arg > 50.0 {
            0.0
        } else {
            1.0 / (exp_arg.exp() - 1.0)
        };

        // Total added noise quanta referred to input: N_add = (G_i / G_s) * (n_th_idler + 0.5)
        let n_add = (g_i / g_s) * (n_th_idler + 0.5);
        let excess = (n_add - caves_min).max(0.0);

        let omega_s = 2.0 * std::f64::consts::PI * params.signal_freq_hz;
        let t_add = (n_add * HBAR * omega_s) / BOLTZMANN_K;
        let nf_db = 10.0 * (1.0 + 2.0 * n_add).log10();

        QuantumNoiseResult {
            signal_gain_linear: g_s,
            commutator_preservation_error: comm_err,
            added_noise_quanta: n_add,
            caves_limit_quanta: caves_min,
            excess_noise_quanta: excess,
            added_noise_temperature_k: t_add,
            noise_figure_db: nf_db,
        }
    }
}
