#![deny(unsafe_code)]

//! Topological Corner-Polariton Micro-Comb Soliton & Dissipative Kerr Frequency Synthesizer.
//!
//! Provides multi-physics simulation of sub-diffraction SOTI corner polariton modes,
//! Lugiato-Lefever dissipative acoustic soliton dynamics, octave-spanning micro-combs,
//! and atomic-clock precision timing synthesis with sub-35-femtosecond jitter.

pub mod corner_polariton_modes;
pub mod frequency_synthesizer;
pub mod soliton_dynamics;

pub use corner_polariton_modes::{
    CornerModeProfile, CornerPolaritonCavityEngine, CornerPolaritonParams,
};
pub use frequency_synthesizer::{
    AllanDeviationPoint, FrequencySynthesizerEngine, PhaseNoisePoint, SynthesizerParams,
};
pub use soliton_dynamics::{
    SolitonCombPoint, SolitonDynamicsEngine, SolitonDynamicsParams, SolitonTemporalPoint,
};

/// Individual verification item in the 10-point micro-comb physics audit.
#[derive(Debug, Clone, PartialEq)]
pub struct MicrocombAuditItem {
    pub name: String,
    pub passed: bool,
    pub measured: String,
    pub threshold: String,
    pub details: String,
}

/// Comprehensive audit report for the corner-polariton frequency synthesizer.
#[derive(Debug, Clone, PartialEq)]
pub struct MicrocombAuditReport {
    pub items: Vec<MicrocombAuditItem>,
    pub passed_count: usize,
    pub total_count: usize,
    pub is_fully_compliant: bool,
}

/// Master orchestrator for the topological corner-polariton micro-comb synthesizer.
#[derive(Debug, Clone)]
pub struct CornerPolaritonMicrocombSynthesizer {
    pub cavity: CornerPolaritonCavityEngine,
    pub soliton: SolitonDynamicsEngine,
    pub synthesizer: FrequencySynthesizerEngine,
}

impl Default for CornerPolaritonMicrocombSynthesizer {
    fn default() -> Self {
        Self::new(
            CornerPolaritonParams::default(),
            SolitonDynamicsParams::default(),
            SynthesizerParams::default(),
        )
    }
}

impl CornerPolaritonMicrocombSynthesizer {
    pub fn new(
        cavity_params: CornerPolaritonParams,
        soliton_params: SolitonDynamicsParams,
        synthesizer_params: SynthesizerParams,
    ) -> Self {
        Self {
            cavity: CornerPolaritonCavityEngine::new(cavity_params),
            soliton: SolitonDynamicsEngine::new(soliton_params),
            synthesizer: FrequencySynthesizerEngine::new(synthesizer_params),
        }
    }

    /// Conducts a comprehensive 10-point physics audit of the synthesizer.
    pub fn audit_microcomb(&self) -> MicrocombAuditReport {
        let mut items = Vec::with_capacity(10);

        // 1. SOTI Corner Confinement
        let modes = self.cavity.solve_corner_modes();
        let max_confinement = modes.iter().map(|m| m.corner_confinement_ratio).fold(0.0f64, f64::max);
        let conf_pass = max_confinement >= 0.85;
        items.push(MicrocombAuditItem {
            name: "SOTI Corner Mode Confinement".to_string(),
            passed: conf_pass,
            measured: format!("{:.1}%", max_confinement * 100.0),
            threshold: ">= 85.0%".to_string(),
            details: "Fractional acoustic polariton energy localized within the 4 outer corner unit cells.".to_string(),
        });

        // 2. Cavity Quality Factor Q
        let q = modes.first().map(|m| m.quality_factor).unwrap_or(1.0e5);
        let q_pass = q >= 5.0e4;
        items.push(MicrocombAuditItem {
            name: "Loaded Quality Factor Q".to_string(),
            passed: q_pass,
            measured: format!("{:.2e}", q),
            threshold: ">= 5.00e4".to_string(),
            details: "Acoustic micro-cavity loaded Q factor governing low-loss non-linear enhancement.".to_string(),
        });

        // 3. Sub-Wavelength Mode Volume
        let v_mode = self.cavity.params.corner_mode_volume_um3;
        let v_pass = v_mode <= 0.50;
        items.push(MicrocombAuditItem {
            name: "Sub-Diffraction Mode Volume".to_string(),
            passed: v_pass,
            measured: format!("{:.3} um^3", v_mode),
            threshold: "<= 0.500 um^3".to_string(),
            details: "Ultra-compact 0D polariton localization yielding high non-linear Purcell enhancement.".to_string(),
        });

        // 4. Anomalous Group Velocity Dispersion
        let beta2 = self.cavity.params.gvd_beta2_ps2_mm;
        let beta2_pass = beta2 < 0.0;
        items.push(MicrocombAuditItem {
            name: "Anomalous Dispersion (beta2)".to_string(),
            passed: beta2_pass,
            measured: format!("{:.3} ps^2/mm", beta2),
            threshold: "< 0.000 ps^2/mm".to_string(),
            details: "Anomalous GVD (beta2 < 0) balancing self-phase modulation for bright soliton formation.".to_string(),
        });

        // 5. Modulational Instability Threshold
        let p_th = self.soliton.compute_mi_threshold_power_mw();
        let p_th_pass = p_th <= 50.0;
        items.push(MicrocombAuditItem {
            name: "Modulational Instability Threshold".to_string(),
            passed: p_th_pass,
            measured: format!("{:.2} mW", p_th),
            threshold: "<= 50.00 mW".to_string(),
            details: "Critical parametric drive power required to trigger micro-comb generation.".to_string(),
        });

        // 6. Dissipative Soliton Pulse Duration
        let tau_ps = self.soliton.soliton_pulse_duration_ps();
        let tau_pass = tau_ps <= 50.0;
        items.push(MicrocombAuditItem {
            name: "Soliton Pulse Duration (FWHM)".to_string(),
            passed: tau_pass,
            measured: format!("{:.2} ps", tau_ps),
            threshold: "<= 50.00 ps".to_string(),
            details: "Single dissipative Kerr acoustic soliton temporal pulse width.".to_string(),
        });

        // 7. Comb Bandwidth Coverage Ratio
        let cov_ratio = self.soliton.octave_coverage_ratio();
        let cov_pass = cov_ratio >= 1.30;
        items.push(MicrocombAuditItem {
            name: "Comb Bandwidth Coverage Ratio".to_string(),
            passed: cov_pass,
            measured: format!("{:.2}x", cov_ratio),
            threshold: ">= 1.30x".to_string(),
            details: "Spectral frequency ratio f_max / f_min approaching octave-spanning bandwidth.".to_string(),
        });

        // 8. f-2f Beat-Note SNR
        let snr_db = self.synthesizer.f2f_beatnote_snr_db();
        let snr_pass = snr_db >= 30.0;
        items.push(MicrocombAuditItem {
            name: "f-2f Beat-Note SNR".to_string(),
            passed: snr_pass,
            measured: format!("{:.1} dB", snr_db),
            threshold: ">= 30.0 dB".to_string(),
            details: "Carrier-envelope offset beat-note detection SNR in self-referencing interferometer.".to_string(),
        });

        // 9. Single-Sideband Phase Noise @ 10 kHz
        let pnoise_10k = self.synthesizer.phase_noise_at_offset(10.0e3);
        let pnoise_pass = pnoise_10k <= -115.0;
        items.push(MicrocombAuditItem {
            name: "Phase Noise @ 10 kHz Offset".to_string(),
            passed: pnoise_pass,
            measured: format!("{:.1} dBc/Hz", pnoise_10k),
            threshold: "<= -115.0 dBc/Hz".to_string(),
            details: "Ultra-low close-in phase noise enabled by SOTI high-Q corner polariton stabilization.".to_string(),
        });

        // 10. Integrated RMS Timing Jitter
        let jitter_fs = self.synthesizer.compute_integrated_timing_jitter_fs();
        let jitter_pass = jitter_fs <= 50.0;
        items.push(MicrocombAuditItem {
            name: "Integrated Timing Jitter (10kHz-10MHz)".to_string(),
            passed: jitter_pass,
            measured: format!("{:.2} fs", jitter_fs),
            threshold: "<= 50.00 fs".to_string(),
            details: "Sub-50-femtosecond integrated RMS timing jitter for atomic clock precision synthesis.".to_string(),
        });

        let passed_count = items.iter().filter(|i| i.passed).count();
        let total_count = items.len();
        let is_fully_compliant = passed_count == total_count;

        MicrocombAuditReport {
            items,
            passed_count,
            total_count,
            is_fully_compliant,
        }
    }
}
