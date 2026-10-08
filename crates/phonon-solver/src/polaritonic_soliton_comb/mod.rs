#![deny(unsafe_code)]

//! Phase 434: Phonon Studio Quantum Metamaterial Polaritonic Soliton Frequency Comb &
//! Dissipative Kerr Squeezed State Generator.
//!
//! Master orchestrator and 10-point physics audit checklist.

pub mod kagome_polariton;
pub mod kerr_squeezed_comb;
pub mod quantum_noise_quadratures;

pub use kagome_polariton::{
    KagomeBandPoint, KagomeBandStructure, KagomePolaritonParams, KagomePolaritonSolver,
};
pub use kerr_squeezed_comb::{
    CombModePoint, DissipativeSolitonMetrics, LugiatoLefeverSoliton,
    LugiatoLefeverSolitonSolver, SolitonCombParams, SolitonProfilePoint,
};
pub use quantum_noise_quadratures::{
    QuadratureScanPoint, QuantumNoiseQuadratures, QuantumNoiseQuadraturesSolver,
    SqueezingMetrics, SqueezingParams, WignerGrid,
};

pub type PolaritonicCombModePoint = CombModePoint;
pub type PolaritonicSolitonMetrics = DissipativeSolitonMetrics;
pub type PolaritonicSolitonProfilePoint = SolitonProfilePoint;
pub type PolaritonicSqueezingMetrics = SqueezingMetrics;
pub type PolaritonicSqueezingParams = SqueezingParams;

/// 10-point physics audit report for Phase 434.
#[derive(Debug, Clone, PartialEq)]
pub struct SolitonCombAuditReport {
    /// 1. Kagome flat-band isolation & flatness (Delta_E_flat < 1e-4 * t).
    pub flat_band_flatness_pass: bool,
    /// 2. Anomalous group velocity dispersion (D_2 > 0).
    pub anomalous_gvd_pass: bool,
    /// 3. Dissipative Kerr soliton stability (sech profile convergence < 1e-4).
    pub soliton_stability_pass: bool,
    /// 4. Multi-octave comb line count (>= 30 active lines above -40 dBc).
    pub comb_line_count_pass: bool,
    /// 5. Repetition frequency equidistance (f_rep consistent across comb, jitter < 10 fs).
    pub repetition_equidistance_pass: bool,
    /// 6. Quadrature noise squeezing below SQL (S_dB >= 6.0 dB).
    pub quadrature_squeezing_pass: bool,
    /// 7. Sub-Poissonian phonon statistics (g^(2)(0) < 1.0).
    pub sub_poissonian_stats_pass: bool,
    /// 8. Negative Mandel Q parameter (Q_M < 0).
    pub negative_mandel_q_pass: bool,
    /// 9. Wigner function phase-space ellipticity (ratio >= 2.0).
    pub wigner_ellipticity_pass: bool,
    /// 10. High-fidelity soliton contrast ratio (>= 20.0 dB).
    pub soliton_contrast_pass: bool,
    /// Total score out of 10.
    pub total_score: usize,
    /// Whether all 10 criteria passed.
    pub all_passed: bool,
}

/// Master orchestrator for Phase 434: Polaritonic Soliton Frequency Comb & Kerr Squeezing Generator.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonicSolitonCombProcessor {
    pub kagome_solver: KagomePolaritonSolver,
    pub soliton_solver: LugiatoLefeverSoliton,
    pub noise_solver: QuantumNoiseQuadratures,
}

impl Default for PolaritonicSolitonCombProcessor {
    fn default() -> Self {
        Self {
            kagome_solver: KagomePolaritonSolver::default(),
            soliton_solver: LugiatoLefeverSoliton::default(),
            noise_solver: QuantumNoiseQuadratures::default(),
        }
    }
}

impl PolaritonicSolitonCombProcessor {
    /// Creates a new processor with custom parameters for all sub-engines.
    pub fn new(
        kagome_params: KagomePolaritonParams,
        comb_params: SolitonCombParams,
        squeezing_params: SqueezingParams,
    ) -> Self {
        Self {
            kagome_solver: KagomePolaritonSolver::new(kagome_params),
            soliton_solver: LugiatoLefeverSoliton::new(comb_params),
            noise_solver: QuantumNoiseQuadratures::new(squeezing_params),
        }
    }

    /// Evaluates the 10-point physics audit checklist.
    pub fn audit_processor(&self) -> SolitonCombAuditReport {
        let band_struct = self.kagome_solver.evaluate_band_structure();
        let (soliton_metrics, _, _) = self.soliton_solver.solve_soliton();
        let (squeezing_metrics, _, _) = self.noise_solver.evaluate_noise_and_wigner();

        // 1. Kagome Flat-Band Isolation & Flatness (Delta_E_flat < 1e-4 * t)
        let flat_band_flatness_pass = band_struct.relative_flatness < 1e-4;

        // 2. Anomalous Group Velocity Dispersion (D_2 > 0)
        let anomalous_gvd_pass = band_struct.gvd_d2_khz > 0.0;

        // 3. Dissipative Kerr Soliton Stability (sech profile convergence < 1e-4)
        let soliton_stability_pass = soliton_metrics.stability_residual < 1e-4 && soliton_metrics.soliton_regime_valid;

        // 4. Multi-Octave Comb Line Count (>= 30 active lines above -40 dBc)
        let comb_line_count_pass = soliton_metrics.active_comb_lines >= 30;

        // 5. Repetition Frequency Equidistance (f_rep consistent across comb, jitter < 10 fs)
        let repetition_equidistance_pass = soliton_metrics.repetition_jitter_fs < 10.0;

        // 6. Quadrature Noise Squeezing Below SQL (S_dB >= 6.0 dB)
        let quadrature_squeezing_pass = squeezing_metrics.squeezing_db >= 6.0;

        // 7. Sub-Poissonian Phonon Statistics (g^(2)(0) < 1.0)
        let sub_poissonian_stats_pass = squeezing_metrics.g2_zero < 1.0;

        // 8. Negative Mandel Q Parameter (Q_M < 0)
        let negative_mandel_q_pass = squeezing_metrics.mandel_q < 0.0;

        // 9. Wigner Function Phase-Space Ellipticity (ratio >= 2.0)
        let wigner_ellipticity_pass = squeezing_metrics.wigner_ellipticity >= 2.0;

        // 10. High-Fidelity Soliton Contrast Ratio (>= 20.0 dB)
        let soliton_contrast_pass = soliton_metrics.contrast_ratio_db >= 20.0;

        let checks = [
            flat_band_flatness_pass,
            anomalous_gvd_pass,
            soliton_stability_pass,
            comb_line_count_pass,
            repetition_equidistance_pass,
            quadrature_squeezing_pass,
            sub_poissonian_stats_pass,
            negative_mandel_q_pass,
            wigner_ellipticity_pass,
            soliton_contrast_pass,
        ];

        let total_score = checks.iter().filter(|&&c| c).count();
        let all_passed = total_score == 10;

        SolitonCombAuditReport {
            flat_band_flatness_pass,
            anomalous_gvd_pass,
            soliton_stability_pass,
            comb_line_count_pass,
            repetition_equidistance_pass,
            quadrature_squeezing_pass,
            sub_poissonian_stats_pass,
            negative_mandel_q_pass,
            wigner_ellipticity_pass,
            soliton_contrast_pass,
            total_score,
            all_passed,
        }
    }
}
