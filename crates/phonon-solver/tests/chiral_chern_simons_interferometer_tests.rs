#![deny(unsafe_code)]

use phonon_solver::chiral_chern_simons_interferometer::{
    AnyonicMemoryParams, ChernSimonsInterferometerParams, ChernSimonsMemoryProcessor,
    ChiralChernSimonsInterferometer, FractionalAnyonKind, TopologicalAnyonicQuantumMemory,
};
use std::f64::consts::PI;

#[test]
fn test_fractional_anyon_characteristics() {
    let l3 = FractionalAnyonKind::LaughlinOneThird;
    assert!((l3.fractional_charge() - 1.0 / 3.0).abs() < 1e-6);
    assert!((l3.statistical_exchange_phase_rad() - 2.0 * PI / 3.0).abs() < 1e-6);
    assert_eq!(l3.quantum_dimension(), 1.0);

    let l5 = FractionalAnyonKind::LaughlinOneFifth;
    assert!((l5.fractional_charge() - 1.0 / 5.0).abs() < 1e-6);
    assert!((l5.statistical_exchange_phase_rad() - 2.0 * PI / 5.0).abs() < 1e-6);
    assert_eq!(l5.quantum_dimension(), 1.0);

    let mr = FractionalAnyonKind::MooreReadPfaffianFiveHalves;
    assert!((mr.fractional_charge() - 0.25).abs() < 1e-6);
    assert!((mr.statistical_exchange_phase_rad() - PI / 4.0).abs() < 1e-6);
    assert!((mr.quantum_dimension() - std::f64::consts::SQRT_2).abs() < 1e-6);

    let fib = FractionalAnyonKind::FibonacciAnyon;
    assert!((fib.fractional_charge() - 0.20).abs() < 1e-6);
    assert!(fib.quantum_dimension() > 1.6);
}

#[test]
fn test_fabry_perot_interference_spectrum_and_visibility() {
    let params = ChernSimonsInterferometerParams::default();
    let interferometer = ChiralChernSimonsInterferometer::new(params);

    let spectrum = interferometer.compute_interference_spectrum(100, 6.0);
    assert_eq!(spectrum.len(), 100);

    for pt in &spectrum {
        assert!(pt.transmission >= 0.0 && pt.transmission <= 1.0);
        assert!(pt.longitudinal_conductance >= 0.0);
    }

    let metrics = interferometer.evaluate_interference_metrics();
    assert!(
        metrics.visibility >= 0.85,
        "Visibility must be >= 85%, got {}",
        metrics.visibility
    );
    assert!(
        metrics.interference_contrast_db >= 18.0,
        "Interference contrast must be >= 18 dB, got {}",
        metrics.interference_contrast_db
    );
    assert!(
        metrics.dephasing_factor > 0.5,
        "Dephasing factor must remain coherent at 15 mK, got {}",
        metrics.dephasing_factor
    );
}

#[test]
fn test_statistical_phase_jump_extraction() {
    let mut params3 = ChernSimonsInterferometerParams::default();
    params3.anyon_kind = FractionalAnyonKind::LaughlinOneThird;
    let interferometer3 = ChiralChernSimonsInterferometer::new(params3);

    let jump3 = interferometer3.extract_statistical_phase_jump();
    let expected3 = 2.0 * PI / 3.0;
    assert!(
        (jump3 - expected3).abs() < 1e-4,
        "Extracted statistical phase jump for Laughlin 1/3 must match 2*pi/3, got {}",
        jump3
    );

    let mut params5 = ChernSimonsInterferometerParams::default();
    params5.anyon_kind = FractionalAnyonKind::LaughlinOneFifth;
    let interferometer5 = ChiralChernSimonsInterferometer::new(params5);

    let jump5 = interferometer5.extract_statistical_phase_jump();
    let expected5 = 2.0 * PI / 5.0;
    assert!(
        (jump5 - expected5).abs() < 1e-4,
        "Extracted statistical phase jump for Laughlin 1/5 must match 2*pi/5, got {}",
        jump5
    );
}

#[test]
fn test_topological_memory_coherence_and_enhancement() {
    let params = AnyonicMemoryParams::default();
    let memory = TopologicalAnyonicQuantumMemory::new(params);

    let t2_topo = memory.evaluate_topo_coherence_time_us();
    let t2_bare = memory.evaluate_bare_coherence_time_us();
    let enhancement = t2_topo / t2_bare;

    assert!(
        t2_topo >= 250.0,
        "Topological coherence time T_2,topo must be >= 250 us, got {}",
        t2_topo
    );
    assert!(
        enhancement >= 20.0,
        "Coherence lifetime enhancement must be >= 20x, got {}",
        enhancement
    );

    let report = memory.evaluate_memory_performance();
    assert!(
        report.storage_retrieval_fidelity >= 0.995,
        "Storage retrieval fidelity must be >= 0.995, got {}",
        report.storage_retrieval_fidelity
    );
    assert!(
        report.parity_readout_snr_db >= 18.0,
        "Parity readout SNR must be >= 18 dB, got {}",
        report.parity_readout_snr_db
    );
    assert!(
        report.diabatic_leakage_rate < 1e-4,
        "Diabatic leakage rate must be < 1e-4, got {}",
        report.diabatic_leakage_rate
    );
    assert!(
        report.purity >= 0.990,
        "Quantum state purity must be >= 0.990, got {}",
        report.purity
    );
}

#[test]
fn test_memory_coherence_decay_trajectory() {
    let params = AnyonicMemoryParams::default();
    let memory = TopologicalAnyonicQuantumMemory::new(params);

    let decay = memory.compute_coherence_decay_curve(50, 400.0);
    assert_eq!(decay.len(), 50);

    // Initial time t = 0
    assert!((decay[0].topological_fidelity - 1.0).abs() < 1e-6);
    assert!((decay[0].bare_acoustic_fidelity - 1.0).abs() < 1e-6);

    // At t ~ 100 us: topological protected state remains above 0.85, while bare acoustic has decayed below 0.05
    let pt_100 = decay
        .iter()
        .find(|p| p.time_us >= 95.0 && p.time_us <= 110.0)
        .expect("Should find sample point near 100 us");

    assert!(
        pt_100.topological_fidelity >= 0.85,
        "Topological fidelity at 100 us must remain >= 0.85, got {}",
        pt_100.topological_fidelity
    );
    assert!(
        pt_100.bare_acoustic_fidelity < 0.05,
        "Bare acoustic fidelity at 100 us must have decayed < 0.05, got {}",
        pt_100.bare_acoustic_fidelity
    );
}

#[test]
fn test_chiral_chern_simons_master_processor_10_point_audit() {
    let processor = ChernSimonsMemoryProcessor::default();
    let audit = processor.audit_processor();

    assert!(
        audit.all_passed,
        "All 10 physics audit criteria must pass, pass_count was {}",
        audit.pass_count
    );
    assert_eq!(audit.pass_count, 10);
    assert!(audit.pass_k_matrix_quantization);
    assert!(audit.pass_statistical_phase_accuracy);
    assert!(audit.pass_interference_visibility);
    assert!(audit.pass_interference_contrast);
    assert!(audit.pass_anyon_charge_fraction);
    assert!(audit.pass_topological_coherence_time);
    assert!(audit.pass_coherence_enhancement);
    assert!(audit.pass_storage_fidelity);
    assert!(audit.pass_readout_snr);
    assert!(audit.pass_diabatic_suppression);
}
