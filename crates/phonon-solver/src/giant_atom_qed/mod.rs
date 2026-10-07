#![deny(unsafe_code)]

//! Giant Atom Waveguide QED & Non-Markovian Multi-Point Entanglement Processor.
//!
//! Provides solvers for multi-point interference, subradiant/superradiant states,
//! bound states in the continuum (BICs), non-Markovian memory delay dynamics,
//! Fano scattering, and braided multi-atom decoherence-free entanglement.

pub mod giant_atom;
pub mod multi_atom_entanglement;
pub mod non_markovian_dynamics;

pub use giant_atom::{GiantAtomParams, GiantAtomTopology};
pub use multi_atom_entanglement::{
    CollectiveCouplingMatrix, EntanglementTrajectoryPoint, MultiAtomEntanglementResult,
    MultiAtomSystem,
};
pub use non_markovian_dynamics::{
    NonMarkovianDynamicsResult, NonMarkovianSolver, NonMarkovianTrajectoryPoint, ScatteringPoint,
    WaveguideScatteringSpectrum,
};

/// 10-point comprehensive physics audit report for giant atom waveguide QED.
#[derive(Debug, Clone, PartialEq)]
pub struct GiantAtomAuditReport {
    pub superradiant_enhancement_pass: bool,
    pub subradiant_suppression_pass: bool,
    pub bound_state_persistence_pass: bool,
    pub non_markovian_delay_pass: bool,
    pub transmission_notch_pass: bool,
    pub transparency_window_pass: bool,
    pub braided_exchange_pass: bool,
    pub bell_fidelity_pass: bool,
    pub concurrence_pass: bool,
    pub phase_coherence_pass: bool,
    pub total_pass_count: usize,
    pub is_full_pass: bool,
}

/// Unified quantum acoustic giant atom processor engine.
#[derive(Debug, Clone)]
pub struct GiantAtomProcessor {
    pub params: GiantAtomParams,
    pub dynamics_solver: NonMarkovianSolver,
    pub entanglement_system: MultiAtomSystem,
}

impl GiantAtomProcessor {
    pub fn new(params: GiantAtomParams) -> Self {
        let dynamics_solver = NonMarkovianSolver::new(params.clone());
        let entanglement_system = MultiAtomSystem::new(params.clone());
        Self {
            params,
            dynamics_solver,
            entanglement_system,
        }
    }

    /// Conducts a rigorous 10-point physics audit of the giant atom waveguide QED engine.
    pub fn audit_giant_atom(&self) -> GiantAtomAuditReport {
        // 1. Superradiant enhancement
        let super_params = GiantAtomParams::preset_superradiant();
        let super_rate = super_params.decay_rate_at_mhz(super_params.atom_freq_ghz);
        let superradiant_enhancement_pass =
            super_rate >= 3.8 * super_params.single_point_decay_mhz;

        // 2. Subradiant suppression
        let bic_params = GiantAtomParams::preset_bound_state();
        let sub_rate = bic_params.decay_rate_at_mhz(bic_params.atom_freq_ghz);
        let subradiant_suppression_pass = sub_rate <= 0.05 * bic_params.single_point_decay_mhz;

        // 3. Bound state persistence
        let bic_solver = NonMarkovianSolver::new(bic_params.clone());
        let bic_res = bic_solver.solve_dynamics(80.0, 100);
        let bound_state_persistence_pass = bic_res.bound_state_population >= 0.05;

        // 4. Non-Markovian delay ratio
        let delay_ratio = self.params.non_markovian_ratio();
        let non_markovian_delay_pass = delay_ratio >= 0.001;

        // 5 & 6. Scattering spectrum checks (superradiant giant atom extinction notch)
        let super_solver = NonMarkovianSolver::new(super_params);
        let spec = super_solver.compute_scattering_spectrum(
            self.params.atom_freq_ghz - 0.05,
            self.params.atom_freq_ghz + 0.05,
            80,
        );
        let transmission_notch_pass = spec.min_transmission_db <= -15.0;
        let transparency_window_pass = spec.max_transmission_db >= -0.5;

        // 7, 8, 9. Multi-atom braided entanglement checks
        let braided_params = GiantAtomParams::preset_braided_entanglement();
        let braided_system = MultiAtomSystem::new(braided_params);
        let matrix = braided_system.compute_coupling_matrix();
        let ent_res = braided_system.simulate_entanglement_generation(100.0, 100);

        let braided_exchange_pass = matrix.exchange_coupling_mhz >= 1.0;
        let bell_fidelity_pass = ent_res.peak_fidelity >= 0.980;
        let concurrence_pass = ent_res.peak_concurrence >= 0.950;

        // 10. Phase coherence
        let phase = self.params.resonance_phase_shift();
        let phase_coherence_pass = phase > 0.0 && phase.is_finite();

        let passes = [
            superradiant_enhancement_pass,
            subradiant_suppression_pass,
            bound_state_persistence_pass,
            non_markovian_delay_pass,
            transmission_notch_pass,
            transparency_window_pass,
            braided_exchange_pass,
            bell_fidelity_pass,
            concurrence_pass,
            phase_coherence_pass,
        ];

        let total_pass_count = passes.iter().filter(|&&p| p).count();
        let is_full_pass = total_pass_count == 10;

        GiantAtomAuditReport {
            superradiant_enhancement_pass,
            subradiant_suppression_pass,
            bound_state_persistence_pass,
            non_markovian_delay_pass,
            transmission_notch_pass,
            transparency_window_pass,
            braided_exchange_pass,
            bell_fidelity_pass,
            concurrence_pass,
            phase_coherence_pass,
            total_pass_count,
            is_full_pass,
        }
    }
}
