#![deny(unsafe_code)]

//! Phase 441: Phonon Studio Topological Non-Hermitian Floquet Acoustic Chiral
//! Lasing Metasurface & Vortex Waveguide.
//!
//! Master orchestrator and 10-point physics audit checklist.

pub mod acoustic_vortex_waveguide;
pub mod floquet_chiral_lattice;
pub mod mode_competition_rates;

pub use acoustic_vortex_waveguide::{
    AcousticVortexMetrics, AcousticVortexParams, AcousticVortexWaveguideSolver,
    FarFieldRadiationPoint, VortexRadialPoint,
};
pub use floquet_chiral_lattice::{
    ComplexQuasiEnergyPoint, FloquetChiralLatticeMetrics, FloquetChiralLatticeParams,
    FloquetChiralLatticeSolver, MetasurfaceSpatialNode,
};
pub use mode_competition_rates::{
    LasingSpectrumPoint, LasingTransientPoint, ModeCompetitionMetrics, ModeCompetitionParams,
    ModeCompetitionRateSolver,
};

/// 10-point physics audit report for Phase 441.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralLasingMetasurfaceAuditReport {
    /// 1. Floquet time-reversal symmetry breaking (Delta_F >= 5.0 MHz).
    pub trs_breaking_pass: bool,
    /// 2. Non-Hermitian chiral edge modal gain (Im(epsilon_chiral) > 0).
    pub chiral_modal_gain_pass: bool,
    /// 3. Bulk and counter-propagating mode suppression (Im(epsilon_bulk) < 0 and Im(epsilon_counter) < 0).
    pub bulk_counter_suppression_pass: bool,
    /// 4. Non-reciprocal chiral lasing isolation ratio (I_chiral >= 25.0 dB).
    pub chiral_isolation_pass: bool,
    /// 5. Quantized OAM vortex topological charge (|ell| >= 1).
    pub quantized_oam_charge_pass: bool,
    /// 6. OAM vortex beam modal purity (M_purity >= 92.0%).
    pub oam_modal_purity_pass: bool,
    /// 7. Lasing threshold pump power (P_th <= 15.0 mW).
    pub threshold_power_pass: bool,
    /// 8. Single-mode stability & side-mode suppression ratio (SMSR >= 30.0 dB).
    pub smsr_stability_pass: bool,
    /// 9. Schawlow-Townes narrowed emission linewidth (Delta f <= 5.0 kHz).
    pub linewidth_narrowing_pass: bool,
    /// 10. Sub-12ns turn-on transient latency (tau_turn_on <= 12.0 ns).
    pub turn_on_latency_pass: bool,
    /// Total score out of 10.
    pub total_score: usize,
    /// Whether all 10 criteria passed.
    pub all_passed: bool,
}

/// Master processor orchestrating Floquet chiral lattices, vortex waveguides, and mode competition.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralLasingMetasurfaceProcessor {
    pub lattice_solver: FloquetChiralLatticeSolver,
    pub vortex_solver: AcousticVortexWaveguideSolver,
    pub rate_solver: ModeCompetitionRateSolver,
}

impl Default for ChiralLasingMetasurfaceProcessor {
    fn default() -> Self {
        Self {
            lattice_solver: FloquetChiralLatticeSolver::default(),
            vortex_solver: AcousticVortexWaveguideSolver::default(),
            rate_solver: ModeCompetitionRateSolver::default(),
        }
    }
}

impl ChiralLasingMetasurfaceProcessor {
    pub fn new(
        lattice_params: FloquetChiralLatticeParams,
        vortex_params: AcousticVortexParams,
        rate_params: ModeCompetitionParams,
    ) -> Self {
        Self {
            lattice_solver: FloquetChiralLatticeSolver::new(lattice_params),
            vortex_solver: AcousticVortexWaveguideSolver::new(vortex_params),
            rate_solver: ModeCompetitionRateSolver::new(rate_params),
        }
    }

    /// Executes the 10-point physics audit checklist.
    pub fn audit_lasing_metasurface(&self) -> ChiralLasingMetasurfaceAuditReport {
        let lm = self.lattice_solver.evaluate_metrics();
        let vm = self.vortex_solver.evaluate_metrics();
        let rm = self.rate_solver.evaluate_metrics();

        let trs_breaking_pass = lm.floquet_bandgap_mhz >= 5.0;
        let chiral_modal_gain_pass = lm.chiral_mode_gain_mhz > 0.0;
        let bulk_counter_suppression_pass = lm.bulk_mode_loss_mhz < 0.0 && lm.counter_mode_loss_mhz < 0.0;
        let chiral_isolation_pass = lm.chiral_isolation_db >= 25.0;
        let quantized_oam_charge_pass = vm.measured_topological_charge.abs() >= 1;
        let oam_modal_purity_pass = vm.oam_modal_purity >= 0.920;
        let threshold_power_pass = lm.threshold_power_mw <= 15.0;
        let smsr_stability_pass = rm.smsr_db >= 30.0;
        let linewidth_narrowing_pass = rm.emission_linewidth_khz <= 5.0;
        let turn_on_latency_pass = rm.turn_on_delay_ns <= 12.0;

        let passes = [
            trs_breaking_pass,
            chiral_modal_gain_pass,
            bulk_counter_suppression_pass,
            chiral_isolation_pass,
            quantized_oam_charge_pass,
            oam_modal_purity_pass,
            threshold_power_pass,
            smsr_stability_pass,
            linewidth_narrowing_pass,
            turn_on_latency_pass,
        ];

        let total_score = passes.iter().filter(|&&p| p).count();
        let all_passed = total_score == 10;

        ChiralLasingMetasurfaceAuditReport {
            trs_breaking_pass,
            chiral_modal_gain_pass,
            bulk_counter_suppression_pass,
            chiral_isolation_pass,
            quantized_oam_charge_pass,
            oam_modal_purity_pass,
            threshold_power_pass,
            smsr_stability_pass,
            linewidth_narrowing_pass,
            turn_on_latency_pass,
            total_score,
            all_passed,
        }
    }
}
