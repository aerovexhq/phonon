#![deny(unsafe_code)]

//! Phase 421: Phonon Studio Topological Acoustic Floquet Higher-Order Corner-State Laser
//! & Non-Hermitian Vortex Amplifier Engine.
//!
//! Provides higher-order topological acoustic corner-state lasing models with side-mode
//! suppression ratio >= 35 dB, and spatio-temporal Floquet gain-loss modulated non-reciprocal
//! orbital angular momentum (OAM) vortex beam amplifiers with gain >= 22.0 dB.

pub mod corner_laser;
pub mod vortex_amplifier;

pub use corner_laser::{
    CornerLaserParams, CornerLaserSolver, CornerLasingMode, LaserSpectralPoint,
};
pub use vortex_amplifier::{
    VortexAmplifierParams, VortexAmplifierPoint, VortexAmplifierSolver, VortexOamCharge,
    VortexSpatialPoint,
};

/// Combined parameter configuration for Phase 421 engine.
#[derive(Debug, Clone)]
pub struct FloquetCornerLaserParams {
    pub laser: CornerLaserParams,
    pub amplifier: VortexAmplifierParams,
}

impl Default for FloquetCornerLaserParams {
    fn default() -> Self {
        Self {
            laser: CornerLaserParams::default(),
            amplifier: VortexAmplifierParams::default(),
        }
    }
}

/// 10-Point physics audit report for the laser and vortex amplifier system.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetLaserAuditReport {
    pub higher_order_topology_pass: bool,
    pub corner_confinement_pass: bool,
    pub lasing_threshold_pass: bool,
    pub coherent_output_power_pass: bool,
    pub side_mode_suppression_pass: bool,
    pub linewidth_narrowing_pass: bool,
    pub vortex_forward_gain_pass: bool,
    pub non_reciprocal_isolation_pass: bool,
    pub oam_modal_purity_pass: bool,
    pub floquet_stability_pass: bool,
    pub total_pass_score: usize,
    pub all_passed: bool,
}

/// Master orchestrator for Phase 421.
#[derive(Debug, Clone)]
pub struct FloquetCornerLaserProcessor {
    pub params: FloquetCornerLaserParams,
    pub laser: CornerLaserSolver,
    pub amplifier: VortexAmplifierSolver,
}

impl FloquetCornerLaserProcessor {
    pub fn new(params: FloquetCornerLaserParams) -> Self {
        let laser = CornerLaserSolver::new(params.laser.clone());
        let amplifier = VortexAmplifierSolver::new(params.amplifier.clone());
        Self {
            params,
            laser,
            amplifier,
        }
    }

    /// Executes the 10-point physics audit checklist.
    pub fn audit_laser_amplifier(&self) -> FloquetLaserAuditReport {
        // 1. Higher-order topology
        let topo = self.laser.is_topological() && self.laser.bulk_bandgap_mhz() >= 8.0;
        // 2. Corner confinement >= 85%
        let modes = self.laser.solve_corner_modes();
        let corner_conf = modes.iter().all(|m| m.confinement_ratio >= 0.85);
        // 3. Lasing threshold <= 20 mW
        let p_th = self.laser.threshold_pump_power_mw();
        let thresh = p_th <= 20.0;
        // 4. Output power >= 2.0 mW
        let p_out = self.laser.calculate_output_power_mw(self.params.laser.pump_power_mw);
        let power_pass = p_out >= 2.0;
        // 5. SMSR >= 35.0 dB
        let smsr = self.laser.calculate_smsr_db();
        let smsr_pass = smsr >= 35.0;
        // 6. Linewidth narrowing <= 50.0 kHz
        let lw = self.laser.calculate_laser_linewidth_khz();
        let lw_pass = lw <= 50.0;
        // 7. Vortex forward gain >= 22.0 dB
        let f_gain = self.amplifier.forward_power_gain_db();
        let f_gain_pass = f_gain >= 22.0;
        // 8. Isolation contrast >= 25.0 dB
        let iso = self.amplifier.isolation_contrast_db();
        let iso_pass = iso >= 25.0;
        // 9. OAM modal purity >= 90.0%
        let purity = self.amplifier.calculate_oam_purity_percent();
        let purity_pass = purity >= 90.0;
        // 10. Floquet stability (positive dynamic amplitude and interaction length > 5mm)
        let floquet_pass = self.params.amplifier.dynamic_modulation_amplitude_mhz > 0.0
            && self.params.amplifier.interaction_length_mm >= 5.0;

        let checks = [
            topo,
            corner_conf,
            thresh,
            power_pass,
            smsr_pass,
            lw_pass,
            f_gain_pass,
            iso_pass,
            purity_pass,
            floquet_pass,
        ];

        let total_pass_score = checks.iter().filter(|&&c| c).count();
        let all_passed = total_pass_score == 10;

        FloquetLaserAuditReport {
            higher_order_topology_pass: topo,
            corner_confinement_pass: corner_conf,
            lasing_threshold_pass: thresh,
            coherent_output_power_pass: power_pass,
            side_mode_suppression_pass: smsr_pass,
            linewidth_narrowing_pass: lw_pass,
            vortex_forward_gain_pass: f_gain_pass,
            non_reciprocal_isolation_pass: iso_pass,
            oam_modal_purity_pass: purity_pass,
            floquet_stability_pass: floquet_pass,
            total_pass_score,
            all_passed,
        }
    }
}
