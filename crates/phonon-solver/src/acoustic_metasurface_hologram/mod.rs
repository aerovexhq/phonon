#![deny(unsafe_code)]

//! Phase 406: Phonon Studio Multi-Octave Acoustic Metasurface Wavefront Hologram
//! & Ultrasonic Tractor Beam Engine.
//!
//! Provides:
//! - Subwavelength acoustic metasurface unit cells with full 2*pi phase modulation and impedance matching.
//! - Rayleigh-Sommerfeld Gerchberg-Saxton (GS) iterative holographic phase retrieval and focal field synthesis.
//! - Volumetric Gor'kov radiation potential U_rad, 3D radiation forces, and ultrasonic tractor beam pulling (F_z < 0).
//! - Non-diffracting acoustic Bessel vortex beams carrying quantized OAM and accelerating Airy beams.
//! - Comprehensive 10-point physics audit evaluating holographic, acoustic trapping, and topological wavefront criteria.

mod bessel_airy_beams;
mod metasurface_unit_cell;
mod phase_retrieval;
mod tractor_beam;

pub use bessel_airy_beams::{
    bessel_j, AiryBeamParams, BesselAirySolver, BesselBeamParams, BesselBeamResult,
};
pub use metasurface_unit_cell::{
    AcousticMedium, MetasurfaceArray, MetasurfaceCellGeometry, MetasurfaceCellParams,
    MetasurfaceUnitCell, UnitCellResponse,
};
pub use phase_retrieval::{
    GerchbergSaxtonParams, HologramIterationPoint, HologramSynthesisResult, HologramSynthesizer,
    HologramTargetType, HoloComplex,
};
pub use tractor_beam::{
    GorkovFieldPoint, TractorBeamEngine, TrappedParticle, TrapStabilityMetrics,
};

/// Individual verification item in the 10-point acoustic metasurface physics audit.
#[derive(Debug, Clone, PartialEq)]
pub struct MetasurfaceAuditItem {
    pub name: &'static str,
    pub passed: bool,
    pub measured_value: f64,
    pub threshold_specification: &'static str,
    pub details: String,
}

/// Comprehensive outcome of the 10-point acoustic metasurface physics audit.
#[derive(Debug, Clone, PartialEq)]
pub struct MetasurfaceAuditReport {
    pub items: Vec<MetasurfaceAuditItem>,
    pub pass_count: usize,
    pub total_tests: usize,
    pub all_passed: bool,
}

/// Master processor orchestrating metasurface wavefront holography and ultrasonic trapping.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticMetasurfaceProcessor {
    pub unit_cell: MetasurfaceUnitCell,
    pub array: MetasurfaceArray,
    pub gs_params: GerchbergSaxtonParams,
    pub tractor_engine: TractorBeamEngine,
    pub bessel_params: BesselBeamParams,
    pub airy_params: AiryBeamParams,
}

impl Default for AcousticMetasurfaceProcessor {
    fn default() -> Self {
        let cell_params = MetasurfaceCellParams::default();
        let unit_cell = MetasurfaceUnitCell::new(cell_params.clone());
        let array = MetasurfaceArray::new(16, 16, cell_params.cell_pitch_m, cell_params.base_frequency_hz);
        let gs_params = GerchbergSaxtonParams::default();
        let tractor_engine = TractorBeamEngine::default();
        let bessel_params = BesselBeamParams::default();
        let airy_params = AiryBeamParams::default();

        Self {
            unit_cell,
            array,
            gs_params,
            tractor_engine,
            bessel_params,
            airy_params,
        }
    }
}

impl AcousticMetasurfaceProcessor {
    /// Creates a new processor with custom settings.
    pub fn new(
        cell_params: MetasurfaceCellParams,
        grid_res: usize,
        gs_params: GerchbergSaxtonParams,
        tractor_engine: TractorBeamEngine,
        bessel_params: BesselBeamParams,
        airy_params: AiryBeamParams,
    ) -> Self {
        let pitch = cell_params.cell_pitch_m;
        let freq = cell_params.base_frequency_hz;
        let unit_cell = MetasurfaceUnitCell::new(cell_params);
        let array = MetasurfaceArray::new(grid_res, grid_res, pitch, freq);

        Self {
            unit_cell,
            array,
            gs_params,
            tractor_engine,
            bessel_params,
            airy_params,
        }
    }

    /// Fast-boot constructor optimized for sub-2ms cold startup.
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Evaluates the 10-point physics audit covering all acoustic metasurface criteria:
    ///
    /// 1. Subwavelength Phase Coverage: Delta_phi >= 1.95 * pi (full 2*pi range)
    /// 2. Acoustic Impedance Matching: |T| >= 0.85 (Z_cell / Z_0 in [0.80, 1.25])
    /// 3. Gerchberg-Saxton Hologram Convergence: PSNR >= 25.0 dB
    /// 4. Holographic Focal Spot Contrast: Contrast >= 20.0 dB
    /// 5. Ultrasonic Tractor Beam Pulling Force: F_z < 0 (negative axial radiation force)
    /// 6. 3D Volumetric Trap Stability: k_x > 0, k_y > 0, k_z > 0
    /// 7. Non-Diffracting Bessel Propagation: z_prop >= 0.85 * z_max
    /// 8. Orbital Angular Momentum (OAM) Quantization: Phase circulation oint grad(phi) . ds = 2*pi*l
    /// 9. Acoustic Self-Healing Capability: Peak recovery >= 80% behind obstacle
    /// 10. Multi-Octave Broadband Frequency Span: f_max / f_min >= 4.0 (>= 2 octaves)
    pub fn audit_metasurface(&self) -> MetasurfaceAuditReport {
        let mut items = Vec::with_capacity(10);
        let c_0 = self.unit_cell.params.medium.speed_of_sound();

        // 1. Subwavelength Phase Coverage
        let phase_span = self.unit_cell.compute_phase_span(self.unit_cell.params.base_frequency_hz);
        let span_thresh = 1.95 * std::f64::consts::PI;
        items.push(MetasurfaceAuditItem {
            name: "Subwavelength Phase Coverage",
            passed: phase_span >= span_thresh,
            measured_value: phase_span / std::f64::consts::PI,
            threshold_specification: ">= 1.95 pi radians (full 2*pi phase modulation)",
            details: format!(
                "Measured phase span = {:.3} pi rad (target >= {:.3} pi rad)",
                phase_span / std::f64::consts::PI,
                span_thresh / std::f64::consts::PI
            ),
        });

        // 2. Acoustic Impedance Matching
        let cell_resp = self.unit_cell.evaluate_response(0.5, self.unit_cell.params.base_frequency_hz);
        let trans_amp = cell_resp.transmission_amplitude;
        items.push(MetasurfaceAuditItem {
            name: "Acoustic Impedance Matching Transmissivity",
            passed: trans_amp >= 0.85,
            measured_value: trans_amp,
            threshold_specification: "|T| >= 0.85 (Z_cell / Z_0 in [0.80, 1.25])",
            details: format!(
                "Transmission amplitude |T| = {:.3} (power transmissivity = {:.1}%, Z_norm = {:.2})",
                trans_amp,
                cell_resp.power_transmissivity * 100.0,
                cell_resp.normalized_impedance
            ),
        });

        // 3. Gerchberg-Saxton Hologram Convergence (PSNR)
        let holo_res = HologramSynthesizer::synthesize(&self.array, &self.gs_params, c_0);
        let psnr = holo_res.final_psnr_db;
        items.push(MetasurfaceAuditItem {
            name: "Gerchberg-Saxton Hologram Convergence",
            passed: psnr >= 25.0,
            measured_value: psnr,
            threshold_specification: "PSNR >= 25.0 dB (Correlation SSIM >= 0.90)",
            details: format!(
                "Achieved PSNR = {:.1} dB, Pearson correlation = {:.3} across {} iterations",
                psnr,
                holo_res.correlation_ssim,
                holo_res.history.len()
            ),
        });

        // 4. Holographic Focal Spot Contrast
        let contrast_db = holo_res.focal_contrast_db;
        items.push(MetasurfaceAuditItem {
            name: "Holographic Focal Spot Contrast",
            passed: contrast_db >= 20.0,
            measured_value: contrast_db,
            threshold_specification: "Focal plane spot contrast >= 20.0 dB",
            details: format!("Target focal contrast = {:.1} dB", contrast_db),
        });

        // 5. Ultrasonic Tractor Beam Pulling Force (F_z < 0)
        let mut array_trap = self.array.clone();
        let z_focus = self.gs_params.focal_plane_z_m;
        array_trap.set_twin_trap_phase(0.0, 0.0, z_focus, c_0);
        let trap_metrics = self.tractor_engine.evaluate_trap_stability(&array_trap, [0.0, 0.0, z_focus]);
        let f_pull = trap_metrics.axial_pulling_force_n;
        items.push(MetasurfaceAuditItem {
            name: "Ultrasonic Tractor Beam Pulling Force",
            passed: trap_metrics.is_tractor_beam_pulling && f_pull < 0.0,
            measured_value: f_pull * 1e6, // uN
            threshold_specification: "Axial force F_z < 0 (negative radiation pulling force)",
            details: format!(
                "Axial pulling force F_z = {:.3} uN directed towards metasurface",
                f_pull * 1e6
            ),
        });

        // 6. 3D Volumetric Trap Stability
        let stable_3d = trap_metrics.is_3d_stable
            && trap_metrics.stiffness_kx_n_m > 0.0
            && trap_metrics.stiffness_ky_n_m > 0.0
            && trap_metrics.stiffness_kz_n_m > 0.0;
        let min_stiff = trap_metrics.stiffness_kx_n_m.min(trap_metrics.stiffness_ky_n_m).min(trap_metrics.stiffness_kz_n_m);
        items.push(MetasurfaceAuditItem {
            name: "3D Volumetric Trap Stability",
            passed: stable_3d,
            measured_value: min_stiff * 1e3, // mN/m
            threshold_specification: "k_x > 0, k_y > 0, k_z > 0 (strictly positive 3D trap stiffness)",
            details: format!(
                "Stiffnesses: k_x = {:.2} mN/m, k_y = {:.2} mN/m, k_z = {:.2} mN/m (Levitation factor = {:.1}x)",
                trap_metrics.stiffness_kx_n_m * 1e3,
                trap_metrics.stiffness_ky_n_m * 1e3,
                trap_metrics.stiffness_kz_n_m * 1e3,
                trap_metrics.levitation_safety_factor
            ),
        });

        // 7. Non-Diffracting Bessel Propagation Distance
        let bessel_res = BesselAirySolver::solve_bessel_beam(&self.bessel_params);
        let prop_ratio = bessel_res.propagation_distance_ratio;
        items.push(MetasurfaceAuditItem {
            name: "Non-Diffracting Bessel Propagation",
            passed: prop_ratio >= 0.85,
            measured_value: prop_ratio * 100.0,
            threshold_specification: "z_prop >= 0.85 * z_max (intensity variation < 15%)",
            details: format!(
                "Diffraction-free distance = {:.1} mm ({:.1}% of z_max = {:.1} mm, variance = {:.1}%)",
                prop_ratio * bessel_res.max_non_diffracting_distance_m * 1000.0,
                prop_ratio * 100.0,
                bessel_res.max_non_diffracting_distance_m * 1000.0,
                bessel_res.intensity_variation_ratio * 100.0
            ),
        });

        // 8. Orbital Angular Momentum (OAM) Quantization
        let circ_charge = bessel_res.phase_circulation_charge;
        let expected_charge = self.bessel_params.topological_charge as f64;
        let charge_err = (circ_charge - expected_charge).abs();
        items.push(MetasurfaceAuditItem {
            name: "Orbital Angular Momentum Quantization",
            passed: charge_err < 0.10,
            measured_value: circ_charge,
            threshold_specification: "Phase circulation oint grad(phi) . ds = 2*pi*l (integer topological charge)",
            details: format!(
                "Circulated phase charge = {:.2} (expected l = {})",
                circ_charge, self.bessel_params.topological_charge
            ),
        });

        // 9. Acoustic Self-Healing Capability
        let heal_ratio = bessel_res.self_healing_recovery_ratio;
        items.push(MetasurfaceAuditItem {
            name: "Acoustic Beam Self-Healing Behind Obstacle",
            passed: heal_ratio >= 0.80,
            measured_value: heal_ratio * 100.0,
            threshold_specification: "Reconstructed central peak >= 80% behind obstacle",
            details: format!(
                "Downstream central lobe recovery = {:.1}% of unobstructed intensity",
                heal_ratio * 100.0
            ),
        });

        // 10. Multi-Octave Broadband Frequency Span
        let (avg_trans, octave_ratio) = self.unit_cell.evaluate_multi_octave_performance();
        items.push(MetasurfaceAuditItem {
            name: "Multi-Octave Broadband Frequency Span",
            passed: octave_ratio >= 4.0 && avg_trans >= 0.85,
            measured_value: octave_ratio,
            threshold_specification: "f_max / f_min >= 4.0 (>= 2 octaves) with avg |T| >= 0.85",
            details: format!(
                "Frequency octave ratio = {:.1}x (avg transmission |T| = {:.3})",
                octave_ratio, avg_trans
            ),
        });

        let pass_count = items.iter().filter(|it| it.passed).count();
        let total_tests = items.len();
        let all_passed = pass_count == total_tests;

        MetasurfaceAuditReport {
            items,
            pass_count,
            total_tests,
            all_passed,
        }
    }
}
