//! Macroscopic continuous-variable quantum entanglement, steering,
//! quantum transducer efficiency, and non-classical state fidelity metrics.

/// Evaluated quantum entanglement and transduction metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnomechanicalEntanglementMetrics {
    /// Bipartite magnon-phonon logarithmic negativity $E_{N, mb} > 0$.
    pub magnon_phonon_log_negativity: f64,
    /// Bipartite photon-magnon logarithmic negativity $E_{N, am} > 0$.
    pub photon_magnon_log_negativity: f64,
    /// EPR steering asymmetry parameter $\\mathcal{S}_{m \\to b} > 0$.
    pub quantum_steering_parameter: f64,
    /// Non-classical macroscopic state fidelity $\\mathcal{F} \\ge 90\\%$.
    pub quantum_state_fidelity: f64,
    /// Coherent microwave-to-phonon quantum transduction efficiency $\\eta_{\\mathrm{trans}} \\ge 50\\%$.
    pub transduction_efficiency_fraction: f64,
    /// Magnomechanical displacement force sensitivity in $\\text{N}/\\sqrt{\\text{Hz}}$ ($S_{FF}^{1/2} \\le 10^{-16}\\text{ N}/\\sqrt{\\text{Hz}}$).
    pub force_sensitivity_n_per_rt_hz: f64,
}
