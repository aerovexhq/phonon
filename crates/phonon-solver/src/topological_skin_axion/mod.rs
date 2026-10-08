#![deny(unsafe_code)]

//! Phase 446: Topological Phononic Non-Hermitian Skin-Effect Microwave Amplification & Directional Axion Transducer.
//!
//! Master module integrating non-Hermitian skin-effect (NHSE) traveling-wave microwave
//! amplification, generalized Brillouin zone point-gap topology, coherent dark-matter
//! axion-to-phonon conversion, and cryogenic microwave readout crossbars.

pub mod skin_microwave_amplifier;
pub mod axion_phonon_transducer;
pub mod cryogenic_readout_crossbar;

pub use skin_microwave_amplifier::{
    GainBandwidthPoint, GbzPoint, SkinAmplifierMetrics, SkinAmplifierParams,
    SkinMicrowaveAmplifierSolver, SkinSpatialProfilePoint,
};
pub use axion_phonon_transducer::{
    AxionCouplingScanPoint, AxionPhononTransducerSolver, AxionResonancePoint,
    AxionTransducerMetrics, AxionTransducerParams,
};
pub use cryogenic_readout_crossbar::{
    CrossbarLinearityPoint, CrossbarSParameterPoint, CryogenicCrossbarMetrics,
    CryogenicCrossbarParams, CryogenicReadoutCrossbarSolver,
};

/// 10-Point rigorous physical audit report for the topological skin-axion processor.
#[derive(Debug, Clone)]
pub struct SkinAxionAuditReport {
    pub point_gap_winding_pass: bool,
    pub skin_localization_pass: bool,
    pub forward_gain_pass: bool,
    pub reverse_isolation_pass: bool,
    pub quantum_added_noise_pass: bool,
    pub acoustic_quality_factor_pass: bool,
    pub axion_conversion_efficiency_pass: bool,
    pub yoctowatt_sensitivity_pass: bool,
    pub cryogenic_directivity_pass: bool,
    pub dispersive_readout_snr_pass: bool,

    pub total_score: usize,
    pub all_passed: bool,
}

/// Unified master orchestrator for Phase 446.
#[derive(Debug, Clone)]
pub struct TopologicalSkinAxionProcessor {
    pub amplifier_solver: SkinMicrowaveAmplifierSolver,
    pub transducer_solver: AxionPhononTransducerSolver,
    pub crossbar_solver: CryogenicReadoutCrossbarSolver,
}

impl Default for TopologicalSkinAxionProcessor {
    fn default() -> Self {
        Self {
            amplifier_solver: SkinMicrowaveAmplifierSolver::new(SkinAmplifierParams::default()),
            transducer_solver: AxionPhononTransducerSolver::new(AxionTransducerParams::default()),
            crossbar_solver: CryogenicReadoutCrossbarSolver::new(CryogenicCrossbarParams::default()),
        }
    }
}

impl TopologicalSkinAxionProcessor {
    pub fn new(
        amp_params: SkinAmplifierParams,
        trans_params: AxionTransducerParams,
        cross_params: CryogenicCrossbarParams,
    ) -> Self {
        Self {
            amplifier_solver: SkinMicrowaveAmplifierSolver::new(amp_params),
            transducer_solver: AxionPhononTransducerSolver::new(trans_params),
            crossbar_solver: CryogenicReadoutCrossbarSolver::new(cross_params),
        }
    }

    /// Executes the 10-point physics audit checklist.
    pub fn audit_system(&self) -> SkinAxionAuditReport {
        let am = self.amplifier_solver.evaluate_metrics();
        let tm = self.transducer_solver.evaluate_metrics();
        let cm = self.crossbar_solver.evaluate_metrics();

        let point_gap_winding_pass = (am.point_gap_winding_number - 1.0).abs() < 1e-4;
        let skin_localization_pass = am.skin_localization_ratio >= 0.85;
        let forward_gain_pass = am.forward_power_gain_db >= 24.0;
        let reverse_isolation_pass = am.backward_isolation_db >= 25.0;
        let quantum_added_noise_pass = am.added_noise_quanta <= 0.55;

        let acoustic_quality_factor_pass = tm.acoustic_quality_factor >= 1.0e5;
        let axion_conversion_efficiency_pass = tm.conversion_efficiency >= 1.0e-4;
        let yoctowatt_sensitivity_pass = tm.yoctowatt_sensitivity_w_sqrt_hz <= 1.0e-21;

        let cryogenic_directivity_pass = cm.directivity_db >= 30.0;
        let dispersive_readout_snr_pass = cm.dispersive_readout_snr_db >= 18.0;

        let items = [
            point_gap_winding_pass,
            skin_localization_pass,
            forward_gain_pass,
            reverse_isolation_pass,
            quantum_added_noise_pass,
            acoustic_quality_factor_pass,
            axion_conversion_efficiency_pass,
            yoctowatt_sensitivity_pass,
            cryogenic_directivity_pass,
            dispersive_readout_snr_pass,
        ];

        let total_score = items.iter().filter(|&&p| p).count();
        let all_passed = total_score == 10;

        SkinAxionAuditReport {
            point_gap_winding_pass,
            skin_localization_pass,
            forward_gain_pass,
            reverse_isolation_pass,
            quantum_added_noise_pass,
            acoustic_quality_factor_pass,
            axion_conversion_efficiency_pass,
            yoctowatt_sensitivity_pass,
            cryogenic_directivity_pass,
            dispersive_readout_snr_pass,
            total_score,
            all_passed,
        }
    }
}
