#![deny(unsafe_code)]

//! Topological Acoustic Floquet Higher-Order Corner Magneto-Phonon Isolator & Circulator Module.
//!
//! Master orchestrator integrating 2D acoustic Floquet higher-order corner metamaterial lattices,
//! synthetic Coriolis rotation-induced magnetic pseudo-fields (B_synth >= 10.0 T),
//! acoustomagnonic magneto-phonon non-reciprocal dispersion, 4-port corner circulator matrices
//! with cyclic permutation symmetry, and cryogenic piezoelectric microwave-to-acoustic transducers.

pub mod floquet_corner_modes;
pub mod four_port_circulator;
pub mod magneto_phonon_coupling;
pub mod microwave_acoustic_transducer;

pub use floquet_corner_modes::{
    CornerSpatialDensityPoint, FloquetBandDispersionPoint, FloquetCornerMetrics,
    FloquetCornerParams, FloquetCornerSolver,
};
pub use four_port_circulator::{
    CirculatorSParameterPoint, FourPortCirculatorMetrics, FourPortCirculatorParams,
    FourPortCirculatorSolver,
};
pub use magneto_phonon_coupling::{
    MagnetoPhononDispersionPoint, MagnetoPhononMetrics, MagnetoPhononParams,
    MagnetoPhononSolver,
};
pub use microwave_acoustic_transducer::{
    FloquetTransducerMetrics, FloquetTransducerParams, MicrowaveAcousticTransducerSolver,
    TransducerMetrics, TransducerParams, TransducerPowerLinePoint,
};

/// 10-point rigorous physics audit report for the Floquet corner isolator & circulator.
#[derive(Debug, Clone)]
pub struct FloquetCornerIsolatorAuditReport {
    /// 1. Synthetic Coriolis magnetic pseudo-field B_synth >= 10.0 T.
    pub synthetic_magnetic_field_pass: bool,
    /// 2. 0D topological corner mode spatial confinement ratio >= 85.0%.
    pub corner_mode_confinement_pass: bool,
    /// 3. Acoustomagnonic avoided crossing polariton gap Delta_pol >= 40.0 MHz.
    pub magneto_phonon_polariton_gap_pass: bool,
    /// 4. Forward non-reciprocal insertion loss IL <= 0.40 dB (|S_21| >= 0.955).
    pub forward_insertion_loss_pass: bool,
    /// 5. Backward non-reciprocal isolation ISO >= 36.0 dB (|S_12| <= 0.0158).
    pub backward_isolation_pass: bool,
    /// 6. 4-port circulator return loss RL >= 22.0 dB (|S_ii| <= 0.079).
    pub return_loss_pass: bool,
    /// 7. Cyclic 4-fold permutation symmetry deviation <= 0.05 dB.
    pub cyclic_symmetry_pass: bool,
    /// 8. 3-dB non-reciprocal circulation bandwidth BW_3dB >= 120.0 MHz.
    pub circulation_bandwidth_pass: bool,
    /// 9. Sharp 90-degree corner defect immunity ratio T_defect / T_clean >= 0.95.
    pub corner_defect_immunity_pass: bool,
    /// 10. Cryogenic transduction efficiency >= 28.0% and added noise n_add <= 0.08 quanta at 20 mK.
    pub cryogenic_transduction_noise_pass: bool,
}

impl FloquetCornerIsolatorAuditReport {
    /// Returns true if all 10 physics audit criteria evaluated to PASS.
    pub fn all_passed(&self) -> bool {
        self.synthetic_magnetic_field_pass
            && self.corner_mode_confinement_pass
            && self.magneto_phonon_polariton_gap_pass
            && self.forward_insertion_loss_pass
            && self.backward_isolation_pass
            && self.return_loss_pass
            && self.cyclic_symmetry_pass
            && self.circulation_bandwidth_pass
            && self.corner_defect_immunity_pass
            && self.cryogenic_transduction_noise_pass
    }

    /// Returns the audit score as (passed_count, total_count).
    pub fn score(&self) -> (usize, usize) {
        let items = [
            self.synthetic_magnetic_field_pass,
            self.corner_mode_confinement_pass,
            self.magneto_phonon_polariton_gap_pass,
            self.forward_insertion_loss_pass,
            self.backward_isolation_pass,
            self.return_loss_pass,
            self.cyclic_symmetry_pass,
            self.circulation_bandwidth_pass,
            self.corner_defect_immunity_pass,
            self.cryogenic_transduction_noise_pass,
        ];
        let passed = items.iter().filter(|&&p| p).count();
        (passed, items.len())
    }

    /// Formats a human-readable text summary of the audit checklist.
    pub fn summary(&self) -> String {
        let (passed, total) = self.score();
        format!(
            "Floquet Corner Isolator & Circulator Audit: {}/{} PASS\n\
             1. Synthetic Magnetic Pseudo-Field (B_synth >= 10.0 T): {}\n\
             2. 0D Corner Mode Spatial Confinement (>= 85.0%): {}\n\
             3. Magneto-Phonon Polariton Gap (Delta >= 40.0 MHz): {}\n\
             4. Forward Insertion Loss (IL <= 0.40 dB): {}\n\
             5. Backward Non-Reciprocal Isolation (ISO >= 36.0 dB): {}\n\
             6. 4-Port Circulator Return Loss (RL >= 22.0 dB): {}\n\
             7. Cyclic Permutation Symmetry Deviation (<= 0.05 dB): {}\n\
             8. 3-dB Circulation Bandwidth (BW >= 120.0 MHz): {}\n\
             9. Sharp 90-Deg Corner Defect Immunity (Ratio >= 0.95): {}\n\
             10. Cryogenic Transduction & Noise (eta >= 28%, n_add <= 0.08): {}",
            passed,
            total,
            self.synthetic_magnetic_field_pass,
            self.corner_mode_confinement_pass,
            self.magneto_phonon_polariton_gap_pass,
            self.forward_insertion_loss_pass,
            self.backward_isolation_pass,
            self.return_loss_pass,
            self.cyclic_symmetry_pass,
            self.circulation_bandwidth_pass,
            self.corner_defect_immunity_pass,
            self.cryogenic_transduction_noise_pass,
        )
    }
}

/// Master coordinator for the Floquet corner isolator & circulator processor.
#[derive(Debug, Clone)]
pub struct FloquetCornerIsolatorProcessor {
    corner_solver: FloquetCornerSolver,
    magneto_solver: MagnetoPhononSolver,
    circulator_solver: FourPortCirculatorSolver,
    transducer_solver: MicrowaveAcousticTransducerSolver,
}

impl Default for FloquetCornerIsolatorProcessor {
    fn default() -> Self {
        Self {
            corner_solver: FloquetCornerSolver::new(FloquetCornerParams::default()),
            magneto_solver: MagnetoPhononSolver::new(MagnetoPhononParams::default()),
            circulator_solver: FourPortCirculatorSolver::new(FourPortCirculatorParams::default()),
            transducer_solver: MicrowaveAcousticTransducerSolver::new(TransducerParams::default()),
        }
    }
}

impl FloquetCornerIsolatorProcessor {
    /// Constructs a processor with explicit sub-solvers.
    pub fn new(
        corner_params: FloquetCornerParams,
        magneto_params: MagnetoPhononParams,
        circulator_params: FourPortCirculatorParams,
        transducer_params: TransducerParams,
    ) -> Self {
        Self {
            corner_solver: FloquetCornerSolver::new(corner_params),
            magneto_solver: MagnetoPhononSolver::new(magneto_params),
            circulator_solver: FourPortCirculatorSolver::new(circulator_params),
            transducer_solver: MicrowaveAcousticTransducerSolver::new(transducer_params),
        }
    }

    /// Access the Floquet corner modes solver.
    pub fn corner(&self) -> &FloquetCornerSolver {
        &self.corner_solver
    }

    /// Access the magneto-phonon solver.
    pub fn magneto(&self) -> &MagnetoPhononSolver {
        &self.magneto_solver
    }

    /// Access the 4-port circulator solver.
    pub fn circulator(&self) -> &FourPortCirculatorSolver {
        &self.circulator_solver
    }

    /// Access the microwave transducer solver.
    pub fn transducer(&self) -> &MicrowaveAcousticTransducerSolver {
        &self.transducer_solver
    }

    /// Evaluates the comprehensive 10-point physics audit checklist.
    pub fn evaluate_audit(&self) -> FloquetCornerIsolatorAuditReport {
        let corner_metrics = self.corner_solver.evaluate_metrics();
        let magneto_metrics = self.magneto_solver.evaluate_metrics();
        let circulator_metrics = self.circulator_solver.evaluate_metrics();
        let transducer_metrics = self.transducer_solver.evaluate_metrics();

        // 1. Synthetic magnetic field >= 10.0 T
        let pass1 = corner_metrics.synthetic_magnetic_field_tesla >= 10.0;

        // 2. 0D corner mode confinement >= 85.0%
        let pass2 = corner_metrics.corner_confinement_ratio >= 0.85;

        // 3. Avoided crossing polariton gap >= 40.0 MHz
        let pass3 = magneto_metrics.polariton_gap_mhz >= 40.0;

        // 4. Forward insertion loss <= 0.40 dB
        let pass4 = magneto_metrics.insertion_loss_db <= 0.40
            && circulator_metrics.insertion_loss_db <= 0.40;

        // 5. Backward isolation >= 36.0 dB
        let pass5 = magneto_metrics.isolation_db >= 36.0
            && circulator_metrics.backward_isolation_db >= 36.0;

        // 6. Return loss >= 22.0 dB
        let pass6 = circulator_metrics.return_loss_db >= 22.0;

        // 7. Cyclic permutation symmetry deviation <= 0.05 dB
        let pass7 = circulator_metrics.cyclic_symmetry_deviation_db <= 0.05;

        // 8. 3-dB circulation bandwidth >= 120.0 MHz
        let pass8 = circulator_metrics.circulation_bandwidth_3db_mhz >= 120.0;

        // 9. Sharp 90-degree corner defect immunity ratio >= 0.95
        let pass9 = circulator_metrics.corner_defect_transmission_ratio >= 0.95;

        // 10. Cryogenic transduction efficiency >= 28.0% and added noise <= 0.08 quanta
        let pass10 = transducer_metrics.transduction_efficiency_pct >= 28.0
            && transducer_metrics.added_noise_quanta <= 0.08;

        FloquetCornerIsolatorAuditReport {
            synthetic_magnetic_field_pass: pass1,
            corner_mode_confinement_pass: pass2,
            magneto_phonon_polariton_gap_pass: pass3,
            forward_insertion_loss_pass: pass4,
            backward_isolation_pass: pass5,
            return_loss_pass: pass6,
            cyclic_symmetry_pass: pass7,
            circulation_bandwidth_pass: pass8,
            corner_defect_immunity_pass: pass9,
            cryogenic_transduction_noise_pass: pass10,
        }
    }
}
