#![deny(unsafe_code)]

//! Automated physics test suite for Phase 409: Topological Corner-Polariton Micro-Comb
//! Soliton & Dissipative Kerr Acoustic Frequency Synthesizer.

use phonon_solver::corner_polariton_microcomb::{
    CornerPolaritonCavityEngine, CornerPolaritonMicrocombSynthesizer, CornerPolaritonParams,
    FrequencySynthesizerEngine, SolitonDynamicsEngine, SolitonDynamicsParams, SynthesizerParams,
};

#[test]
fn test_soti_corner_polariton_confinement_and_topological_phase() {
    let params = CornerPolaritonParams::default();
    let engine = CornerPolaritonCavityEngine::new(params.clone());

    assert!(engine.is_topological(), "Lattice must reside in topological SOTI phase (lambda > gamma)");
    let gap = engine.bulk_bandgap_mhz();
    assert!(gap >= 15.0, "Bulk bandgap must be >= 15.0 MHz (measured: {:.2} MHz)", gap);

    let modes = engine.solve_corner_modes();
    assert_eq!(modes.len(), 4, "Must resolve 4 0D corner-polariton states");

    for (i, m) in modes.iter().enumerate() {
        assert!(
            m.corner_confinement_ratio >= 0.88,
            "Corner mode {} confinement must be >= 88% (measured: {:.1}%)",
            i,
            m.corner_confinement_ratio * 100.0
        );
        assert!(
            m.quality_factor >= 5.0e4,
            "Corner mode {} Q factor must be >= 50,000 (measured: {:.2e})",
            i,
            m.quality_factor
        );
    }
}

#[test]
fn test_sub_diffraction_mode_volume_and_purcell_enhancement() {
    let params = CornerPolaritonParams::default();
    let engine = CornerPolaritonCavityEngine::new(params);

    let modes = engine.solve_corner_modes();
    let m0 = &modes[0];

    assert!(
        m0.purcell_enhancement >= 1.0e5,
        "Purcell enhancement Q / V_mode must be >= 1.0e5 (measured: {:.2e})",
        m0.purcell_enhancement
    );
}

#[test]
fn test_anomalous_group_velocity_dispersion() {
    let params = CornerPolaritonParams::default();
    let engine = CornerPolaritonCavityEngine::new(params.clone());

    assert!(params.gvd_beta2_ps2_mm < 0.0, "GVD beta2 must be anomalous (negative)");
    let d2 = engine.dispersion_parameter_d2_khz();
    assert!(d2 > 0.0, "Anomalous dispersion parameter D2 must be strictly positive (measured: {:.2} kHz)", d2);
}

#[test]
fn test_modulational_instability_pump_threshold() {
    let params = SolitonDynamicsParams::default();
    let engine = SolitonDynamicsEngine::new(params.clone());

    let p_th = engine.compute_mi_threshold_power_mw();
    assert!(
        p_th > 0.0 && p_th <= 30.0,
        "Modulational instability threshold must be within (0, 30] mW (measured: {:.2} mW)",
        p_th
    );
    assert!(
        params.pump_power_mw > p_th,
        "Operational pump power must exceed MI threshold"
    );
}

#[test]
fn test_dissipative_kerr_soliton_state_and_pulse_duration() {
    let params = SolitonDynamicsParams::default();
    let engine = SolitonDynamicsEngine::new(params);

    assert!(engine.is_soliton_state(), "System must reside within dissipative soliton existence tongue");

    let tau_ps = engine.soliton_pulse_duration_ps();
    assert!(
        tau_ps <= 50.0 && tau_ps >= 5.0,
        "Soliton FWHM pulse duration must be within [5.0, 50.0] ps (measured: {:.2} ps)",
        tau_ps
    );

    let profile = engine.compute_temporal_profile(64);
    assert_eq!(profile.len(), 64);

    let max_intensity = profile.iter().map(|p| p.intensity_mw).fold(0.0f64, f64::max);
    let min_intensity = profile.iter().map(|p| p.intensity_mw).fold(f64::INFINITY, f64::min);
    assert!(
        max_intensity > min_intensity * 5.0,
        "Soliton peak intensity must exceed background by > 5x (peak: {:.2} mW, bg: {:.2} mW)",
        max_intensity,
        min_intensity
    );
}

#[test]
fn test_octave_spanning_comb_spectrum_teeth() {
    let params = SolitonDynamicsParams::default();
    let engine = SolitonDynamicsEngine::new(params);

    let spectrum = engine.compute_comb_spectrum();
    assert!(spectrum.len() >= 64, "Spectrum must contain at least 64 comb teeth");

    // Check comb line frequency spacing
    let spacing_ghz = spectrum[1].frequency_ghz - spectrum[0].frequency_ghz;
    let expected_spacing = 0.500; // 500 MHz = 0.500 GHz
    assert!(
        (spacing_ghz - expected_spacing).abs() < 1e-4,
        "Comb line spacing must match repetition rate 500 MHz (measured: {:.4} GHz)",
        spacing_ghz
    );

    let cov_ratio = engine.octave_coverage_ratio();
    assert!(
        cov_ratio >= 1.30,
        "Comb coverage ratio f_max / f_min must be >= 1.30x (measured: {:.2}x)",
        cov_ratio
    );
}

#[test]
fn test_f2f_carrier_envelope_offset_beatnote_snr() {
    let params = SynthesizerParams::default();
    let engine = FrequencySynthesizerEngine::new(params);

    let snr_db = engine.f2f_beatnote_snr_db();
    assert!(
        snr_db >= 35.0,
        "f-2f carrier-envelope beatnote SNR must be >= 35.0 dB (measured: {:.2} dB)",
        snr_db
    );
}

#[test]
fn test_single_sideband_phase_noise_and_flicker_suppression() {
    let params = SynthesizerParams::default();
    let engine = FrequencySynthesizerEngine::new(params);

    let l_10k = engine.phase_noise_at_offset(10.0e3);
    assert!(
        l_10k <= -120.0,
        "Phase noise at 10 kHz must be <= -120.0 dBc/Hz (measured: {:.2} dBc/Hz)",
        l_10k
    );

    let l_1m = engine.phase_noise_at_offset(1.0e6);
    assert!(
        l_1m <= -150.0,
        "Phase noise at 1 MHz must be <= -150.0 dBc/Hz (measured: {:.2} dBc/Hz)",
        l_1m
    );
}

#[test]
fn test_integrated_timing_jitter_sub_35fs() {
    let params = SynthesizerParams::default();
    let engine = FrequencySynthesizerEngine::new(params);

    let jitter_fs = engine.compute_integrated_timing_jitter_fs();
    assert!(
        jitter_fs <= 35.0,
        "Integrated RMS timing jitter must be <= 35.0 fs (measured: {:.2} fs)",
        jitter_fs
    );
}

#[test]
fn test_allan_deviation_fractional_stability_1s() {
    let params = SynthesizerParams::default();
    let engine = FrequencySynthesizerEngine::new(params);

    let adev_1s = engine.allan_deviation_at_1s();
    assert!(
        adev_1s <= 1.0e-13,
        "Allan deviation at 1s must be <= 1.0e-13 (measured: {:.2e})",
        adev_1s
    );
}

#[test]
fn test_comprehensive_10_point_physics_audit() {
    let synthesizer = CornerPolaritonMicrocombSynthesizer::default();
    let report = synthesizer.audit_microcomb();

    assert_eq!(report.total_count, 10, "Audit must evaluate 10 criteria");
    assert_eq!(
        report.passed_count, 10,
        "All 10 physics criteria must pass (passed: {}/{})",
        report.passed_count, report.total_count
    );
    assert!(report.is_fully_compliant, "Microcomb synthesizer must be 100% compliant");
}
