#![deny(unsafe_code)]

//! Test suite for Phase 427: Topological Higher-Order Corner Kerr Microcomb & Dissipative Soliton Generator.

use phonon_solver::corner_kerr_microcomb::{
    CornerKerrMicrocombProcessor, CornerSolitonParams, CornerSolitonSolver,
    MicrocombGenerator,
};

#[test]
fn test_corner_topological_mode_confinement() {
    let params = CornerSolitonParams::default();
    let solver = CornerSolitonSolver::new(params);
    let topo = solver.evaluate_topology_metrics();

    assert!(
        topo.corner_confinement_ratio >= 90.0,
        "Corner confinement ratio must be >= 90%, got {:.2}%",
        topo.corner_confinement_ratio
    );
    assert!(
        topo.bulk_bandgap_mhz > 5.0,
        "Bulk topological bandgap must be > 5.0 MHz, got {:.2} MHz",
        topo.bulk_bandgap_mhz
    );
    assert!(
        (topo.topological_index - 0.5).abs() < 1e-4,
        "Quantized corner topological index must be 0.5"
    );
    assert!(
        topo.quality_factor > 1e4,
        "Acoustic quality factor must exceed 10^4, got {:.2}",
        topo.quality_factor
    );
}

#[test]
fn test_dissipative_kerr_soliton_formation() {
    let params = CornerSolitonParams::default();
    let solver = CornerSolitonSolver::new(params);
    let (metrics, points) = solver.solve_dissipative_soliton();

    assert!(
        metrics.soliton_regime_valid,
        "Soliton existence criteria must be met"
    );
    assert!(
        metrics.peak_phonon_number > metrics.background_intensity,
        "Peak phonon number must exceed background"
    );
    assert!(
        metrics.contrast_ratio_db >= 15.0,
        "Soliton contrast ratio must be >= 15.0 dB, got {:.2} dB",
        metrics.contrast_ratio_db
    );
    assert!(
        metrics.fwhm_rad > 0.0 && metrics.fwhm_rad < 1.0,
        "Soliton FWHM must be tightly localized (< 1.0 rad), got {:.3} rad",
        metrics.fwhm_rad
    );
    assert_eq!(
        points.len(),
        64,
        "Expected exactly 64 discrete profile points"
    );

    // Verify center intensity is at the peak
    let center_idx = points.len() / 2;
    assert!(
        points[center_idx].intensity >= points[0].intensity,
        "Center intensity must exceed edge background"
    );
}

#[test]
fn test_microcomb_spectrum_generation() {
    let params = CornerSolitonParams::default();
    let generator = MicrocombGenerator::new(params);
    let (metrics, modes) = generator.generate_comb_spectrum();

    assert!(
        metrics.total_comb_lines >= 25,
        "Expected at least 25 active comb modes, got {}",
        metrics.total_comb_lines
    );
    assert!(
        metrics.phase_coherence >= 0.99,
        "Soliton phase coherence must be >= 0.99, got {:.4}",
        metrics.phase_coherence
    );
    assert!(
        metrics.beat_note_linewidth_hz < 10.0,
        "Beat note linewidth must be < 10.0 Hz, got {:.2} Hz",
        metrics.beat_note_linewidth_hz
    );
    assert!(
        metrics.bandwidth_3db_mhz > 10.0,
        "3-dB comb bandwidth must exceed 10.0 MHz"
    );
    assert!(
        !modes.is_empty(),
        "Mode list must not be empty"
    );
}

#[test]
fn test_quantum_transduction_efficiency() {
    let params = CornerSolitonParams::default();
    let generator = MicrocombGenerator::new(params);
    let trans = generator.evaluate_quantum_transduction();

    assert!(
        trans.transduction_efficiency_percent >= 40.0,
        "Transduction efficiency must be >= 40.0%, got {:.2}%",
        trans.transduction_efficiency_percent
    );
    assert!(
        trans.added_noise_quanta <= 0.55,
        "Quantum added noise must be <= 0.55 quanta, got {:.3}",
        trans.added_noise_quanta
    );
    assert!(
        trans.signal_to_noise_ratio_db >= 20.0,
        "Transduction SNR must be >= 20.0 dB, got {:.2} dB",
        trans.signal_to_noise_ratio_db
    );
}

#[test]
fn test_10_point_physics_audit_checklist_pass() {
    let processor = CornerKerrMicrocombProcessor::default();
    let audit = processor.audit_processor();

    assert!(audit.corner_confinement_pass, "Criterion 1 failed: corner modal confinement");
    assert!(audit.anomalous_dispersion_pass, "Criterion 2 failed: anomalous dispersion");
    assert!(audit.soliton_existence_pass, "Criterion 3 failed: soliton existence threshold");
    assert!(audit.contrast_ratio_pass, "Criterion 4 failed: contrast ratio");
    assert!(audit.comb_lines_count_pass, "Criterion 5 failed: comb line count");
    assert!(audit.phase_coherence_pass, "Criterion 6 failed: phase coherence");
    assert!(audit.beat_note_narrow_pass, "Criterion 7 failed: beat note linewidth");
    assert!(audit.transduction_efficiency_pass, "Criterion 8 failed: quantum transduction efficiency");
    assert!(audit.quantum_added_noise_pass, "Criterion 9 failed: quantum added noise");
    assert!(audit.cold_boot_throughput_pass, "Criterion 10 failed: cold boot throughput");

    assert_eq!(
        audit.total_score, 10,
        "Audit score must be 10/10, got {}/10",
        audit.total_score
    );
    assert!(audit.all_passed, "All 10 audit criteria must pass");
}
