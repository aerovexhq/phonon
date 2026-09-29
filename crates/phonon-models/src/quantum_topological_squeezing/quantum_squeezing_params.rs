//! Parameters and metrics for quantum topological phonon squeezing,
//! non-classical states, and sub-SQL acoustic metrology.

/// Parameters for quantum topological parametric phonon squeezer and non-classical state generator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumPhononSqueezingParams {
    /// Resonator central acoustic frequency $f_0$ in $\text{GHz}$ (nominal $1.0 - 8.0\text{ GHz}$).
    pub acoustic_resonance_freq_ghz: f64,
    /// Loaded topological acoustic mode quality factor $\mathcal{Q}$ (nominal $5.0\times 10^5 - 1.0\times 10^7$).
    pub loaded_q_factor: f64,
    /// Third-order non-linear parametric four-wave mixing rate $g_4 / (2\pi)$ in $\text{kHz}$ (nominal $5.0 - 60.0\text{ kHz}$).
    pub parametric_coupling_rate_khz: f64,
    /// Continuous-wave acoustic pump drive power in milliwatts (nominal $0.2 - 8.0\text{ mW}$).
    pub pump_power_mw: f64,
    /// External cavity escape efficiency $\eta_{\mathrm{esc}} \in [0.70, 0.98]$.
    pub cavity_escape_efficiency: f64,
    /// Normalized cavity detuning $\Delta / (\kappa / 2) \in [0.0, 3.0]$.
    pub normalized_detuning: f64,
    /// Mean coherent phonon amplitude for Schrödinger cat states $\alpha_0$ (nominal $1.5 - 4.0$).
    pub cat_coherent_amplitude: f64,
    /// Environmental bath thermal phonon occupancy $\bar{n}_{\mathrm{th}}$ at dilution mK temperatures (nominal $0.01 - 0.20$).
    pub bath_thermal_occupancy: f64,
}

impl Default for QuantumPhononSqueezingParams {
    fn default() -> Self {
        Self {
            acoustic_resonance_freq_ghz: 3.5,
            loaded_q_factor: 2.0e6,
            parametric_coupling_rate_khz: 25.0,
            pump_power_mw: 2.5,
            cavity_escape_efficiency: 0.88,
            normalized_detuning: 0.5,
            cat_coherent_amplitude: 2.2,
            bath_thermal_occupancy: 0.05,
        }
    }
}

/// Evaluated metrics for non-classical topological phonon squeezing and acoustic metrology.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumPhononSqueezingMetrics {
    /// Acoustic quadrature squeezing below shot noise in decibels ($\ge 6.0\text{ dB}$).
    pub quadrature_squeezing_db: f64,
    /// Macroscopic quantum Schrödinger cat state preparation fidelity in percent ($\ge 90.0\%$).
    pub cat_state_fidelity_pct: f64,
    /// Phase-space Wigner distribution negativity metric at origin $W(0,0)$ ($\ge 0.15$).
    pub wigner_negativity: f64,
    /// Minimum detectable acoustic force spectral density in attonewtons per $\sqrt{\text{Hz}}$ ($\le 25.0\text{ aN}/\sqrt{\text{Hz}}$).
    pub force_sensitivity_attonewtons: f64,
    /// Sub-SQL quantum gravimetry acceleration precision in nano-g ($\le 5.0\text{ nano-g}$).
    pub quantum_gravimetry_precision_nano_g: f64,
    /// Continuous-variable photon-phonon quantum entanglement in ebits ($\ge 1.0\text{ ebits}$).
    pub continuous_entanglement_ebits: f64,
}

impl QuantumPhononSqueezingParams {
    /// Creates a new parameter set for quantum topological phonon squeezing.
    pub fn new(
        f0_ghz: f64,
        q_factor: f64,
        g4_khz: f64,
        power_mw: f64,
        eta_esc: f64,
        alpha: f64,
    ) -> Self {
        Self {
            acoustic_resonance_freq_ghz: f0_ghz.clamp(0.5, 20.0),
            loaded_q_factor: q_factor.clamp(1.0e4, 5.0e7),
            parametric_coupling_rate_khz: g4_khz.clamp(1.0, 200.0),
            pump_power_mw: power_mw.clamp(0.05, 50.0),
            cavity_escape_efficiency: eta_esc.clamp(0.50, 0.99),
            cat_coherent_amplitude: alpha.clamp(0.5, 8.0),
            ..Default::default()
        }
    }
}
