#![deny(unsafe_code)]

//! Moiré Exciton-Polariton Valley Hall Chiral Lasing Metasurface & Opto-Acoustic Synthesizer (Phase 448).
//!
//! Provides multi-physics simulation of transition metal dichalcogenide moiré superlattice
//! exciton-polaritons, topological valley Hall chiral edge waveguides, non-reciprocal polariton
//! lasing and BEC condensation, and microwave-to-optical opto-acoustic frequency synthesis.

pub mod chiral_lasing_engine;
pub mod moire_polariton_lattice;
pub mod opto_acoustic_synthesizer;
pub mod valley_hall_edge;

pub use chiral_lasing_engine::{
    ChiralLasingMetrics, ChiralLasingParams, ChiralLasingSolver, InputOutputCurvePoint,
    TemporalCoherencePoint,
};
pub use moire_polariton_lattice::{
    MoireDispersionPoint, MoireLatticeMetrics, MoirePolaritonLatticeSolver, MoirePolaritonParams,
};
pub use opto_acoustic_synthesizer::{
    FrequencyCombLine, OptoAcousticMetrics, OptoAcousticPhaseNoisePoint,
    OptoAcousticSynthesizerParams, OptoAcousticSynthesizerSolver,
};
pub use valley_hall_edge::{
    CircularPolarization, MoireEdgeDispersionPoint, ValleyHallEdgeMetrics,
    ValleyHallEdgeParams, ValleyHallEdgeSolver, WaveguideTransmissionPoint,
};

/// 10-Point rigorous physics audit report for Phase 448.
#[derive(Debug, Clone)]
pub struct MoirePolaritonAuditReport {
    /// 1. Moiré Exciton-Polariton Rabi Splitting (hbar * Omega_R >= 15.0 meV).
    pub moire_rabi_splitting_pass: bool,
    /// 2. Valley Hall Berry Curvature Quantization (|C_v| = 1).
    pub valley_berry_quantization_pass: bool,
    /// 3. Bulk Topological Valley Gap (Delta_gap >= 8.0 meV).
    pub bulk_valley_gap_pass: bool,
    /// 4. Valley-Locked Edge Dispersion Linearity (v_g >= 1.0e5 m/s).
    pub valley_edge_velocity_pass: bool,
    /// 5. Sharp-Bend Defect Immunity (T_bend >= 94.0% around 60/120 deg).
    pub sharp_bend_immunity_pass: bool,
    /// 6. Chiral Valley Isolation (>= 25.0 dB).
    pub chiral_valley_isolation_pass: bool,
    /// 7. Polariton Lasing Threshold (P_th <= 5.0 uW/um^2).
    pub polariton_lasing_threshold_pass: bool,
    /// 8. Unidirectional Lasing Directionality (>= 25.0 dB).
    pub chiral_lasing_directionality_pass: bool,
    /// 9. Opto-Acoustic Modulation Depth (beta >= 0.80 rad).
    pub opto_acoustic_modulation_pass: bool,
    /// 10. Microwave-to-Optical Transduction Efficiency (eta_trans >= 15.0%).
    pub microwave_to_optical_transduction_pass: bool,
    /// Number of verified passing checks (out of 10).
    pub total_score: usize,
    /// Whether all 10 physics invariant checks passed.
    pub all_passed: bool,
}

/// Master coordinator for moiré exciton-polariton chiral lasing and opto-acoustic synthesis.
#[derive(Debug, Clone)]
pub struct MoirePolaritonLaserProcessor {
    pub lattice_params: MoirePolaritonParams,
    pub edge_params: ValleyHallEdgeParams,
    pub lasing_params: ChiralLasingParams,
    pub synth_params: OptoAcousticSynthesizerParams,
}

impl Default for MoirePolaritonLaserProcessor {
    fn default() -> Self {
        Self {
            lattice_params: MoirePolaritonParams::default(),
            edge_params: ValleyHallEdgeParams::default(),
            lasing_params: ChiralLasingParams::default(),
            synth_params: OptoAcousticSynthesizerParams::default(),
        }
    }
}

impl MoirePolaritonLaserProcessor {
    /// Constructs a new master processor with specified parameters.
    pub fn new(
        lattice_params: MoirePolaritonParams,
        edge_params: ValleyHallEdgeParams,
        lasing_params: ChiralLasingParams,
        synth_params: OptoAcousticSynthesizerParams,
    ) -> Self {
        Self {
            lattice_params,
            edge_params,
            lasing_params,
            synth_params,
        }
    }

    /// Evaluates the rigorous 10-point physics audit checklist.
    pub fn audit_processor(&self) -> MoirePolaritonAuditReport {
        let lattice_solver = MoirePolaritonLatticeSolver::new(self.lattice_params.clone());
        let lattice_metrics = lattice_solver.evaluate_metrics();

        let edge_solver = ValleyHallEdgeSolver::new(self.edge_params.clone());
        let edge_metrics = edge_solver.evaluate_metrics();

        let lasing_solver = ChiralLasingSolver::new(self.lasing_params.clone());
        let lasing_metrics = lasing_solver.evaluate_metrics();

        let synth_solver = OptoAcousticSynthesizerSolver::new(self.synth_params.clone());
        let synth_metrics = synth_solver.evaluate_metrics();

        let check1 = lattice_metrics.rabi_splitting_mev >= 15.0;
        let check2 = lattice_metrics.valley_chern_number.abs() == 1;
        let check3 = lattice_metrics.bulk_valley_polariton_gap_mev >= 8.0;
        let check4 = edge_metrics.chiral_group_velocity_ms >= 1.0e5;
        let check5 = edge_metrics.sharp_bend_transmission_ratio >= 0.94;
        let check6 = edge_metrics.chiral_valley_isolation_db >= 25.0;
        let check7 = self.lasing_params.threshold_power_uw_um2 <= 5.0;
        let check8 = lasing_metrics.chiral_front_to_back_ratio_db >= 25.0;
        let check9 = synth_metrics.modulation_index_beta >= 0.80;
        let check10 = synth_metrics.microwave_to_optical_efficiency >= 0.15;

        let checks = [
            check1, check2, check3, check4, check5,
            check6, check7, check8, check9, check10,
        ];

        let score = checks.iter().filter(|&&c| c).count();
        let all_pass = score == 10;

        MoirePolaritonAuditReport {
            moire_rabi_splitting_pass: check1,
            valley_berry_quantization_pass: check2,
            bulk_valley_gap_pass: check3,
            valley_edge_velocity_pass: check4,
            sharp_bend_immunity_pass: check5,
            chiral_valley_isolation_pass: check6,
            polariton_lasing_threshold_pass: check7,
            chiral_lasing_directionality_pass: check8,
            opto_acoustic_modulation_pass: check9,
            microwave_to_optical_transduction_pass: check10,
            total_score: score,
            all_passed: all_pass,
        }
    }
}
