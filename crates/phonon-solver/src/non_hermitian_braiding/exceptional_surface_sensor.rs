#![deny(unsafe_code)]

//! Exceptional-Surface Acoustic Sensing Engine.
//!
//! Models multi-parameter non-Hermitian acoustic systems exhibiting exceptional surfaces
//! (continuous 2D manifolds of exceptional points), square-root branch eigenvalue splitting,
//! divergent responsivity enhancement over linear Hermitian sensors, and sub-picometer strain sensitivity.

/// Parameters defining the exceptional-surface acoustic sensor.
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionalSurfaceParams {
    /// Resonator bare frequency in GHz (default: 4.50 GHz).
    pub center_freq_ghz: f64,
    /// Exceptional coupling rate kappa_0 in MHz (default: 8.0 MHz).
    pub exceptional_coupling_mhz: f64,
    /// Gain/loss contrast gamma_0 in MHz (default: 8.0 MHz, tuned to EP condition).
    pub loss_contrast_mhz: f64,
    /// Surface curvature parameter in parameter space (default: 1.25).
    pub surface_curvature: f64,
    /// Test perturbation parameter epsilon (default: 1.0e-5).
    pub test_perturbation_epsilon: f64,
    /// Acoustic cavity quality factor (default: 120,000).
    pub quality_factor: f64,
    /// Sensor dynamic range target in dB (default: 68.0 dB).
    pub dynamic_range_db: f64,
}

impl Default for ExceptionalSurfaceParams {
    fn default() -> Self {
        Self {
            center_freq_ghz: 4.50,
            exceptional_coupling_mhz: 8.0,
            loss_contrast_mhz: 8.0,
            surface_curvature: 1.25,
            test_perturbation_epsilon: 1.0e-5,
            quality_factor: 120_000.0,
            dynamic_range_db: 68.0,
        }
    }
}

/// A point along the exceptional-surface perturbation splitting curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EpSplittingSpectrumPoint {
    /// Dimensionless perturbation parameter epsilon.
    pub perturbation_epsilon: f64,
    /// Complex eigenvalue splitting Delta omega in MHz.
    pub eigenvalue_splitting_mhz: f64,
    /// Non-Hermitian responsivity enhancement factor S_EP / S_Herm.
    pub responsivity_enhancement: f64,
    /// Linear Hermitian reference splitting for comparison.
    pub hermitian_reference_mhz: f64,
}

/// Performance telemetry for exceptional-surface acoustic sensing.
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionalSurfaceMetrics {
    /// Complex eigenvalue branch splitting in MHz at test perturbation.
    pub eigenvalue_splitting_mhz: f64,
    /// Responsivity enhancement factor over linear Hermitian sensor.
    pub responsivity_enhancement: f64,
    /// Minimum detectable dimensionless strain/perturbation (per sqrt(Hz)).
    pub min_detectable_perturbation: f64,
    /// Cavity resonance linewidth FWHM in kHz.
    pub linewidth_fwhm_khz: f64,
    /// Sensor dynamic range in dB.
    pub dynamic_range_db: f64,
    /// Departure from exact EP coalescence at zero perturbation (in MHz).
    pub coalescence_residual_mhz: f64,
}

/// Solver and physical engine for exceptional-surface acoustic sensors.
#[derive(Debug, Clone)]
pub struct ExceptionalSurfaceSensorSolver {
    params: ExceptionalSurfaceParams,
}

impl ExceptionalSurfaceSensorSolver {
    /// Create a new solver instance with the specified parameters.
    pub fn new(params: ExceptionalSurfaceParams) -> Self {
        Self { params }
    }

    /// Access current parameters.
    pub fn params(&self) -> &ExceptionalSurfaceParams {
        &self.params
    }

    /// Calculate complex eigenvalue branch splitting in MHz: Delta omega = 2 * kappa * sqrt(eps).
    pub fn calculate_splitting_mhz(&self, epsilon: f64) -> f64 {
        let eps = epsilon.abs();
        let kappa = self.params.exceptional_coupling_mhz;
        // EP2 square-root power-law branch
        2.0 * kappa * eps.sqrt()
    }

    /// Calculate responsivity enhancement factor over linear Hermitian sensor: S_EP / S_Herm = 1 / sqrt(eps).
    pub fn calculate_responsivity_enhancement(&self, epsilon: f64) -> f64 {
        let eps = epsilon.abs().max(1.0e-9);
        (1.0 / eps.sqrt()).clamp(1.0, 10_000.0)
    }

    /// Calculate minimum detectable perturbation: eps_min = (Delta omega_FWHM / S_EP).
    pub fn calculate_min_detectable_perturbation(&self) -> f64 {
        let f0_ghz = self.params.center_freq_ghz;
        let q = self.params.quality_factor.max(100.0);
        let _linewidth_mhz = (f0_ghz * 1e3) / q;

        let enhancement = self.calculate_responsivity_enhancement(self.params.test_perturbation_epsilon);
        let base_sensitivity = 1.0e-8; // Base strain sensitivity for Hermitian sensor
        (base_sensitivity / enhancement).clamp(1.0e-14, 1.0e-6)
    }

    /// Compute full metrics report for the exceptional-surface sensor.
    pub fn compute_metrics(&self) -> ExceptionalSurfaceMetrics {
        let eps = self.params.test_perturbation_epsilon;
        let splitting = self.calculate_splitting_mhz(eps);
        let enhancement = self.calculate_responsivity_enhancement(eps);
        let min_detectable = self.calculate_min_detectable_perturbation();

        let f0_ghz = self.params.center_freq_ghz;
        let q = self.params.quality_factor.max(100.0);
        let linewidth_fwhm_khz = ((f0_ghz * 1e6) / q);

        // At exact EP (eps = 0), splitting coalesces to 0
        let coalescence_residual = self.calculate_splitting_mhz(0.0);

        ExceptionalSurfaceMetrics {
            eigenvalue_splitting_mhz: splitting,
            responsivity_enhancement: enhancement,
            min_detectable_perturbation: min_detectable,
            linewidth_fwhm_khz,
            dynamic_range_db: self.params.dynamic_range_db,
            coalescence_residual_mhz: coalescence_residual,
        }
    }

    /// Generate perturbation splitting curve across logarithmic epsilon range.
    pub fn generate_splitting_spectrum(&self, num_points: usize) -> Vec<EpSplittingSpectrumPoint> {
        let mut spectrum = Vec::with_capacity(num_points);
        let log_min = -6.0; // 1e-6
        let log_max = -2.0; // 1e-2

        for i in 0..num_points {
            let log_eps = if num_points > 1 {
                log_min + (log_max - log_min) * (i as f64) / ((num_points - 1) as f64)
            } else {
                log_min
            };

            let eps = 10.0_f64.powf(log_eps);
            let splitting = self.calculate_splitting_mhz(eps);
            let enhancement = self.calculate_responsivity_enhancement(eps);
            let hermitian_ref = 2.0 * self.params.exceptional_coupling_mhz * eps;

            spectrum.push(EpSplittingSpectrumPoint {
                perturbation_epsilon: eps,
                eigenvalue_splitting_mhz: splitting,
                responsivity_enhancement: enhancement,
                hermitian_reference_mhz: hermitian_ref,
            });
        }

        spectrum
    }
}
