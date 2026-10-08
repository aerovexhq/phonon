#![deny(unsafe_code)]

//! Test suite for Phase 428: Topological Acoustic Superconducting Nanowire
//! Single-Phonon Detector (SNSPD) & Quantum Transceiver.

use phonon_solver::acoustic_snspd::{
    AcousticSnspdProcessor, NanowireHotspotSolver, NanowireParams,
    QuantumTransceiverEngine, QuantumTransceiverParams,
};

#[test]
fn test_nanowire_hotspot_electrothermal_telemetry() {
    let params = NanowireParams::default();
    let solver = NanowireHotspotSolver::new(params);
    let tele = solver.evaluate_telemetry();

    assert!(
        tele.bias_ratio >= 0.85 && tele.bias_ratio <= 0.95,
        "Bias ratio must be within [0.85, 0.95], got {:.3}",
        tele.bias_ratio
    );
    assert!(
        tele.peak_voltage_mv > 0.2,
        "Peak voltage pulse must be > 0.2 mV, got {:.3} mV",
        tele.peak_voltage_mv
    );
    assert!(
        tele.rise_time_ps < 60.0 && tele.rise_time_ps > 10.0,
        "Rise time must be within (10, 60) ps, got {:.2} ps",
        tele.rise_time_ps
    );
    assert!(
        tele.reset_time_ps <= 1500.0 && tele.reset_time_ps > 200.0,
        "Reset time must be within (200, 1500] ps, got {:.2} ps",
        tele.reset_time_ps
    );
    assert!(
        tele.internal_efficiency_percent >= 85.0,
        "Internal efficiency must be >= 85%, got {:.2}%",
        tele.internal_efficiency_percent
    );
    assert!(
        tele.dark_count_rate_hz <= 20.0,
        "Cryogenic dark count rate must be <= 20 Hz, got {:.2} Hz",
        tele.dark_count_rate_hz
    );
    assert!(
        tele.timing_jitter_fwhm_ps < 6.0,
        "Timing jitter FWHM must be < 6.0 ps, got {:.2} ps",
        tele.timing_jitter_fwhm_ps
    );
}

#[test]
fn test_pulse_waveform_simulation() {
    let params = NanowireParams::default();
    let solver = NanowireHotspotSolver::new(params);
    let waveform = solver.simulate_pulse_waveform();

    assert!(waveform.len() >= 100);
    let max_v = waveform.iter().map(|p| p.voltage_mv).fold(0.0_f64, f64::max);
    assert!(
        max_v > 0.2,
        "Maximum voltage pulse height must exceed 0.2 mV, got {:.3} mV",
        max_v
    );

    // Initial time (< 50 ps) must have zero voltage
    assert_eq!(waveform[0].voltage_mv, 0.0);

    // Hotspot resistance should be non-zero during the pulse peak
    let max_r = waveform.iter().map(|p| p.resistance_ohm).fold(0.0_f64, f64::max);
    assert!(
        max_r > 5.0,
        "Peak hotspot resistance must be > 5.0 Ohm, got {:.2} Ohm",
        max_r
    );
}

#[test]
fn test_timing_jitter_irf_generation() {
    let trans_params = QuantumTransceiverParams::default();
    let engine = QuantumTransceiverEngine::new(trans_params);
    let irf = engine.generate_jitter_histogram();

    assert!(irf.len() >= 50);

    // Find peak of Gaussian IRF
    let peak_pt = irf.iter().max_by(|a, b| a.count_density.partial_cmp(&b.count_density).unwrap()).unwrap();
    assert!(
        peak_pt.time_offset_ps.abs() < 1.0,
        "IRF Gaussian peak should be centered near t = 0 ps, got {:.2} ps",
        peak_pt.time_offset_ps
    );
    assert!(
        peak_pt.count_density > 0.9,
        "Normalized peak density should be near 1.0, got {:.3}",
        peak_pt.count_density
    );
}

#[test]
fn test_quantum_transceiver_metrics_and_fock_discrimination() {
    let trans_params = QuantumTransceiverParams::default();
    let engine = QuantumTransceiverEngine::new(trans_params);
    let metrics = engine.evaluate_transceiver_metrics();

    assert!(
        metrics.max_count_rate_mcps >= 800.0,
        "Max count rate must exceed 800 Mcps, got {:.1} Mcps",
        metrics.max_count_rate_mcps
    );
    assert!(
        metrics.qber_percent <= 2.0,
        "Quantum Bit Error Rate must be <= 2.0%, got {:.2}%",
        metrics.qber_percent
    );
    assert!(
        metrics.secret_key_rate_mbps >= 10.0,
        "Secret key generation rate must be >= 10 Mbps, got {:.2} Mbps",
        metrics.secret_key_rate_mbps
    );
    assert!(
        metrics.single_phonon_fidelity >= 0.98,
        "Single-phonon fidelity must be >= 0.98, got {:.4}",
        metrics.single_phonon_fidelity
    );

    let fock_pts = engine.generate_fock_discrimination();
    assert_eq!(fock_pts.len(), 4);
    assert!(fock_pts[0].discrimination_fidelity >= 0.99);
    assert!(fock_pts[1].discrimination_fidelity >= 0.98);
}

#[test]
fn test_10_point_physics_audit_checklist_pass() {
    let processor = AcousticSnspdProcessor::new_default();
    let audit = processor.audit_processor();

    assert!(audit.bias_ratio_pass, "Bias ratio check failed");
    assert!(audit.hotspot_resistance_pass, "Hotspot resistance check failed");
    assert!(audit.rise_time_pass, "Rise time check failed");
    assert!(audit.reset_time_pass, "Reset time check failed");
    assert!(audit.timing_jitter_pass, "Timing jitter check failed");
    assert!(audit.internal_efficiency_pass, "Internal efficiency check failed");
    assert!(audit.dark_count_rate_pass, "Dark count rate check failed");
    assert!(audit.max_count_rate_pass, "Max count rate check failed");
    assert!(audit.qber_pass, "QBER check failed");
    assert!(audit.cold_boot_throughput_pass, "Cold boot throughput check failed");

    assert_eq!(
        audit.total_score, 10,
        "Audit score must be 10/10, got {}/10",
        audit.total_score
    );
    assert!(audit.all_passed, "All audit criteria must pass");
}
