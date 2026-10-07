#![deny(unsafe_code)]

//! Phase 417: Phonon Studio Topological Acoustic Valley-Hall Chiral Edge Filter & Non-Reciprocal Microwave-Phonon Isolator.
//!
//! Orchestrates the three core subsystems:
//! 1. Valley-Hall phononic crystal lattice and zero-backscattering edge waveguide.
//! 2. Non-reciprocal spatio-temporal synthetic gauge field acoustic isolator.
//! 3. Cryogenic microwave-to-phonon quantum piezoelectric transducer array.

pub mod valley_hall_edge;
pub mod chiral_isolator;
pub mod microwave_transducer;

pub use valley_hall_edge::{ValleyEdgeMode, ValleyEdgeParams, ValleyHallLattice, ValleyPolarity};
pub use chiral_isolator::{ChiralIsolatorParams, ChiralIsolatorSolver, ChiralSParameterPoint};
pub use microwave_transducer::{MicrowavePhononTransducer, TransducerParams, TransducerResponsePoint};

/// Master configuration bundle for the entire Valley-Hall and non-reciprocal isolator system.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ValleyChiralParams {
    pub valley_hall: ValleyEdgeParams,
    pub isolator: ChiralIsolatorParams,
    pub transducer: TransducerParams,
}

/// 10-point physics audit report certifying readiness across all topological and RF domains.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyChiralAuditReport {
    pub valley_bandgap_opened: bool,
    pub valley_chern_difference_quantized: bool,
    pub topological_edge_state_verified: bool,
    pub sharp_corner_zero_backscattering: bool,
    pub return_loss_suppressed: bool,
    pub nonreciprocal_fwd_transmission: bool,
    pub reverse_acoustic_isolation: bool,
    pub isolation_contrast_verified: bool,
    pub microwave_phonon_efficiency_verified: bool,
    pub cryogenic_quantum_noise_limit: bool,
    pub total_pass_score: usize,
    pub total_checks: usize,
    pub all_passed: bool,
    pub summary_message: String,
}

/// Master orchestrator for the Topological Acoustic Valley-Hall Chiral Edge Filter & Microwave-Phonon Isolator.
#[derive(Debug, Clone)]
pub struct ValleyChiralIsolator {
    pub params: ValleyChiralParams,
    pub valley_hall: ValleyHallLattice,
    pub isolator: ChiralIsolatorSolver,
    pub transducer: MicrowavePhononTransducer,
}

impl Default for ValleyChiralIsolator {
    fn default() -> Self {
        let params = ValleyChiralParams::default();
        let valley_hall = ValleyHallLattice::new(params.valley_hall.clone());
        let isolator = ChiralIsolatorSolver::new(params.isolator.clone());
        let transducer = MicrowavePhononTransducer::new(params.transducer.clone());

        Self {
            params,
            valley_hall,
            isolator,
            transducer,
        }
    }
}

impl ValleyChiralIsolator {
    /// Creates a new ValleyChiralIsolator with custom parameters.
    pub fn new(params: ValleyChiralParams) -> Self {
        let valley_hall = ValleyHallLattice::new(params.valley_hall.clone());
        let isolator = ChiralIsolatorSolver::new(params.isolator.clone());
        let transducer = MicrowavePhononTransducer::new(params.transducer.clone());

        Self {
            params,
            valley_hall,
            isolator,
            transducer,
        }
    }

    /// Recomputes internal solvers when parameters are updated.
    pub fn update_params(&mut self, params: ValleyChiralParams) {
        self.params = params;
        self.valley_hall = ValleyHallLattice::new(self.params.valley_hall.clone());
        self.isolator = ChiralIsolatorSolver::new(self.params.isolator.clone());
        self.transducer = MicrowavePhononTransducer::new(self.params.transducer.clone());
    }

    /// Executes the 10-point physics audit checklist.
    pub fn audit_valley_chiral_isolator(&self) -> ValleyChiralAuditReport {
        // 1. Valley bandgap opening: Delta_gap >= 0.20 GHz
        let gap = self.valley_hall.bulk_bandgap_ghz();
        let c1 = gap >= 0.20;

        // 2. Quantized valley Chern difference: |Delta C_v| == 1.0
        let c_diff = self.valley_hall.valley_chern_difference();
        let c2 = (c_diff - 1.0).abs() < 1e-6;

        // 3. Topological edge state inside bulk gap
        let dispersion = self.valley_hall.compute_edge_dispersion(31);
        let c3 = dispersion.iter().any(|m| m.localization_ratio >= 0.85);

        // 4. Sharp corner zero backscattering: T_corner >= 95.0%
        let (t_corner, _il, rl) = self.valley_hall.evaluate_corner_transmission();
        let c4 = t_corner >= 0.95;

        // 5. Return loss suppression: S11 <= -25.0 dB
        let c5 = rl <= -25.0;

        // 6. Non-reciprocal forward transmission: S21 >= -0.80 dB
        let s21_fwd = self.isolator.evaluate_s21_fwd_db(self.params.isolator.center_freq_ghz);
        let c6 = s21_fwd >= -0.80;

        // 7. Reverse acoustic isolation: S12 <= -30.0 dB
        let s12_rev = self.isolator.evaluate_s12_rev_db(self.params.isolator.center_freq_ghz);
        let c7 = s12_rev <= -30.0;

        // 8. Isolation contrast: S21 - S12 >= 28.0 dB
        let contrast = self.isolator.peak_isolation_contrast_db();
        let c8 = contrast >= 28.0;

        // 9. Microwave-to-phonon quantum conversion efficiency: eta >= 40.0%
        let eta = self.transducer.peak_efficiency();
        let c9 = eta >= 0.40;

        // 10. Cryogenic added noise: n_add <= 0.55 quanta
        let n_add = self.transducer.added_noise_quanta();
        let c10 = n_add <= 0.55;

        let checks = [c1, c2, c3, c4, c5, c6, c7, c8, c9, c10];
        let pass_score = checks.iter().filter(|&&b| b).count();
        let all_pass = pass_score == checks.len();

        let summary = format!(
            "Valley-Hall & Isolator Audit: {}/10 Criteria Passed. Gap: {:.2} GHz, Isolation: {:.1} dB, Transduction: {:.1}%, Noise: {:.2} quanta",
            pass_score, gap, contrast, eta * 100.0, n_add
        );

        ValleyChiralAuditReport {
            valley_bandgap_opened: c1,
            valley_chern_difference_quantized: c2,
            topological_edge_state_verified: c3,
            sharp_corner_zero_backscattering: c4,
            return_loss_suppressed: c5,
            nonreciprocal_fwd_transmission: c6,
            reverse_acoustic_isolation: c7,
            isolation_contrast_verified: c8,
            microwave_phonon_efficiency_verified: c9,
            cryogenic_quantum_noise_limit: c10,
            total_pass_score: pass_score,
            total_checks: checks.len(),
            all_passed: all_pass,
            summary_message: summary,
        }
    }
}
