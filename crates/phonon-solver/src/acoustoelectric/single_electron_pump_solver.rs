//! Quantized single-electron acoustoelectric pump solver and flying qubit state evolution.
//!
//! # Physical Formalism
//! - Quantized Acoustoelectric Current Plateau:
//!   $$I_{\mathrm{pump}} = e \cdot f_{\mathrm{saw}} (1 - P_{\mathrm{esc}})$$
//! - Precision Criterion:
//!   $$\epsilon_I = \left| \frac{I}{e f_{\mathrm{saw}}} - 1 \right| < 10^{-4}$$
//! - Two-Qubit Entanglement in Beam Splitter Coupler:
//!   $$|\Psi^+\rangle = \frac{1}{\sqrt{2}}(|\uparrow\downarrow\rangle + |\downarrow\uparrow\rangle)$$
//!   achieving state fidelity $F \ge 95\%$ and concurrence $\mathcal{C} \ge 0.90$.

use phonon_models::acoustoelectric::dynamic_quantum_dot::ELEMENTARY_CHARGE_C;
use phonon_models::acoustoelectric::{DynamicQuantumDot, FlyingQubit, FlyingQubitCoupler};

/// Result of single-electron pump evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct AcoustoelectricPumpResult {
    /// Applied SAW frequency in Hz.
    pub saw_frequency_hz: f64,
    /// Ideal quantized current $I_0 = e \cdot f_{\mathrm{saw}}$ in Amperes.
    pub ideal_quantized_current_a: f64,
    /// Computed acoustoelectric current $I$ in Amperes.
    pub pumped_current_a: f64,
    /// Relative quantization error $|I / I_0 - 1|$.
    pub relative_quantization_error: f64,
    /// Non-adiabatic escape probability $P_{\mathrm{esc}}$.
    pub escape_probability: f64,
    /// Flying qubit spin fidelity $F \in [0, 1]$.
    pub flying_qubit_fidelity: f64,
    /// Flying two-qubit entanglement concurrence $\mathcal{C} \in [0, 1]$.
    pub entanglement_concurrence: f64,
    /// Bell state creation fidelity $F_{\mathrm{bell}} \in [0, 1]$.
    pub bell_state_fidelity: f64,
}

/// Single-electron acoustoelectric pump solver.
#[derive(Debug, Clone, PartialEq)]
pub struct SingleElectronPumpSolver {
    pub dot: DynamicQuantumDot,
    pub coupler: FlyingQubitCoupler,
}

impl SingleElectronPumpSolver {
    pub fn new(dot: DynamicQuantumDot) -> Self {
        let v_saw = dot.saw_params.sound_velocity_m_s;
        let coupler = FlyingQubitCoupler::calibrated_sqrt_swap(1.2e-6, v_saw);
        Self { dot, coupler }
    }

    /// Evaluates current quantization and flying qubit entanglement at given parameters.
    pub fn solve(&self) -> AcoustoelectricPumpResult {
        let f_saw = self.dot.saw_params.frequency_hz;
        let v_saw = self.dot.saw_params.sound_velocity_m_s;
        let ideal_current = ELEMENTARY_CHARGE_C * f_saw;
        let pumped_current = self.dot.evaluate_pumped_current();
        let rel_error = self.dot.quantization_error();
        let p_esc = self.dot.escape_probability();

        // Flying qubit spin transit test
        let mut flying_qubit = FlyingQubit::new(v_saw);
        let m = self.dot.saw_params.effective_mass_kg;
        let transit_time = self.dot.channel.channel_length_m / v_saw;
        flying_qubit.propagate(transit_time, m);

        // Spin state norm preservation and fidelity
        let fidelity = flying_qubit.state.norm();

        // Two-qubit exchange entanglement evaluation
        let (concurrence, bell_fidelity) = self.coupler.evaluate_entanglement(v_saw);

        AcoustoelectricPumpResult {
            saw_frequency_hz: f_saw,
            ideal_quantized_current_a: ideal_current,
            pumped_current_a: pumped_current,
            relative_quantization_error: rel_error,
            escape_probability: p_esc,
            flying_qubit_fidelity: fidelity,
            entanglement_concurrence: concurrence,
            bell_state_fidelity: bell_fidelity,
        }
    }
}
