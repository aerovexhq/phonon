#![deny(unsafe_code)]

//! Phase 457: Phonon Studio Topological Acoustic Floquet Corner Spin-Orbit Polariton Laser & Non-Hermitian Quantum Sensor.
//!
//! Master coordinator and physics invariant audit engine integrating:
//! 1. Higher-order topological acoustic 0D corner state Floquet spin-orbit polariton lasing
//!    with ultra-low threshold (P_th <= 15.0 mW), high confinement (>= 85.0%), and DOCP >= 90.0%.
//! 2. Non-Hermitian modal selection suppressing bulk and edge modes with SMSR >= 35.0 dB
//!    and corner-to-edge gain contrast >= 12.0 dB.
//! 3. Synthetic gauge field rotation sensing via acoustic Sagnac effect with micro-radian
//!    sensitivity (Omega_min <= 1.0e-5 rad/s / sqrt(Hz)) and scale factor stability <= 10.0 ppm.
//! 4. Sub-picotesla acoustic magnetometry with ultra-low noise floor (B_min <= 0.80 pT / sqrt(Hz))
//!    and high dynamic range >= 75.0 dB.

pub mod floquet_corner_polariton_laser;
pub mod non_hermitian_mode_selector;
pub mod sub_picotesla_magnetometer;
pub mod synthetic_gauge_rotation_sensor;

pub use floquet_corner_polariton_laser::{
    CornerLasingLICurvePoint, CornerPolaritonLaserMetrics, CornerPolaritonLaserParams,
    CornerPolaritonLaserSolver, CornerSpatialIntensityPoint,
};
pub use non_hermitian_mode_selector::{
    NonHermitianEigenvaluePoint, NonHermitianModeSelectorMetrics,
    NonHermitianModeSelectorParams, NonHermitianModeSelectorSolver,
};
pub use sub_picotesla_magnetometer::{
    MagneticFieldSweepPoint, SubPicoteslaMagnetometerMetrics,
    SubPicoteslaMagnetometerParams, SubPicoteslaMagnetometerSolver,
};
pub use synthetic_gauge_rotation_sensor::{
    RotationSweepPoint, SyntheticGaugeRotationMetrics, SyntheticGaugeRotationParams,
    SyntheticGaugeRotationSolver,
};

/// 10-point rigorous physics audit report for Phase 457.
#[derive(Debug, Clone)]
pub struct FloquetCornerSensorAuditReport {
    /// 1. Topological corner state energy confinement >= 85.0%.
    pub topological_corner_confinement: bool,
    /// 2. Floquet corner polariton lasing threshold P_th <= 15.0 mW.
    pub lasing_threshold_power: bool,
    /// 3. Degree of circular polarization DOCP >= 90.0%.
    pub degree_circular_polarization: bool,
    /// 4. Laser emission spectral linewidth Delta_nu <= 50.0 kHz.
    pub emission_linewidth: bool,
    /// 5. Non-Hermitian side-mode suppression ratio SMSR >= 35.0 dB.
    pub side_mode_suppression_ratio: bool,
    /// 6. Corner-to-edge modal gain contrast >= 12.0 dB.
    pub corner_to_edge_gain_contrast: bool,
    /// 7. Non-Hermitian single-mode selection invariant (only corner modes have Im(E) > 0).
    pub single_mode_selection: bool,
    /// 8. Synthetic gauge minimum detectable rotation rate Omega_min <= 1.0e-5 rad/s / sqrt(Hz).
    pub minimum_detectable_rotation: bool,
    /// 9. Rotation sensor scale factor stability <= 10.0 ppm.
    pub rotation_scale_factor_stability: bool,
    /// 10. Minimum detectable magnetic flux density B_min <= 0.80 pT / sqrt(Hz).
    pub minimum_detectable_magnetic_field: bool,
}

impl FloquetCornerSensorAuditReport {
    /// Returns the (passed_count, total_count) score.
    pub fn score(&self) -> (usize, usize) {
        let items = [
            self.topological_corner_confinement,
            self.lasing_threshold_power,
            self.degree_circular_polarization,
            self.emission_linewidth,
            self.side_mode_suppression_ratio,
            self.corner_to_edge_gain_contrast,
            self.single_mode_selection,
            self.minimum_detectable_rotation,
            self.rotation_scale_factor_stability,
            self.minimum_detectable_magnetic_field,
        ];
        let passed = items.iter().filter(|&&v| v).count();
        (passed, items.len())
    }

    /// Returns true if all 10 physics audit criteria scored PASS.
    pub fn is_pass(&self) -> bool {
        let (passed, total) = self.score();
        passed == total
    }
}

/// Master coordinator processor for Floquet Corner Sensor (Phase 457).
#[derive(Debug, Clone)]
pub struct FloquetCornerSensorProcessor {
    pub laser_params: CornerPolaritonLaserParams,
    pub mode_selector_params: NonHermitianModeSelectorParams,
    pub rotation_params: SyntheticGaugeRotationParams,
    pub magnetometer_params: SubPicoteslaMagnetometerParams,
}

impl FloquetCornerSensorProcessor {
    /// Creates a new master processor instance.
    pub fn new(
        laser_params: CornerPolaritonLaserParams,
        mode_selector_params: NonHermitianModeSelectorParams,
        rotation_params: SyntheticGaugeRotationParams,
        magnetometer_params: SubPicoteslaMagnetometerParams,
    ) -> Self {
        Self {
            laser_params,
            mode_selector_params,
            rotation_params,
            magnetometer_params,
        }
    }

    /// Evaluates the 10-point physics audit across all sub-solvers.
    pub fn evaluate_audit(&self) -> FloquetCornerSensorAuditReport {
        let laser_solver = CornerPolaritonLaserSolver::new(self.laser_params.clone());
        let mode_solver = NonHermitianModeSelectorSolver::new(self.mode_selector_params.clone());
        let rot_solver = SyntheticGaugeRotationSolver::new(self.rotation_params.clone());
        let mag_solver = SubPicoteslaMagnetometerSolver::new(self.magnetometer_params.clone());

        let laser_m = laser_solver.evaluate_metrics();
        let mode_m = mode_solver.evaluate_metrics();
        let rot_m = rot_solver.evaluate_metrics();
        let mag_m = mag_solver.evaluate_metrics();

        FloquetCornerSensorAuditReport {
            topological_corner_confinement: laser_m.corner_confinement_pct >= 85.0,
            lasing_threshold_power: laser_m.lasing_threshold_mw <= 15.0,
            degree_circular_polarization: laser_m.circular_polarization_pct >= 90.0,
            emission_linewidth: laser_m.emission_linewidth_khz <= 50.0,
            side_mode_suppression_ratio: mode_m.side_mode_suppression_ratio_db >= 35.0,
            corner_to_edge_gain_contrast: mode_m.corner_to_edge_gain_contrast_db >= 12.0,
            single_mode_selection: mode_m.single_mode_selection_invariant,
            minimum_detectable_rotation: rot_m.minimum_detectable_rotation_rad_s_sqrt_hz <= 1.0e-5,
            rotation_scale_factor_stability: rot_m.scale_factor_stability_ppm <= 10.0,
            minimum_detectable_magnetic_field: mag_m.minimum_detectable_field_pt_sqrt_hz <= 0.80,
        }
    }
}
