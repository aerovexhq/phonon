#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for topological quantum
//! acoustic memory and Majorana surface code decoders.

use phonon_models::majorana_surface_memory::MajoranaSurfaceMemoryParams;
use phonon_solver::majorana_surface_memory::MajoranaSurfaceMemorySolver;

#[test]
fn test_quantum_coherence_t2_bounds() {
    let params = MajoranaSurfaceMemoryParams::default();
    let solver = MajoranaSurfaceMemorySolver::new(params);
    let t2 = solver.compute_quantum_coherence_t2_ms();

    // Target quantum coherence time T2 >= 10.0 ms
    assert!(
        t2 >= 10.0,
        "Quantum memory coherence time T2 must be >= 10.0 ms, got {:.2} ms",
        t2
    );
    assert!(
        t2 <= 100.0,
        "Quantum coherence time T2 must not exceed physical bound of 100.0 ms, got {:.2} ms",
        t2
    );

    // Test scaling with high Q-factor
    let high_q_params = MajoranaSurfaceMemoryParams::new(5, 4.8, 1.0e8, 0.0015, 250.0, 35.0, 12.0, 8.5);
    let high_q_solver = MajoranaSurfaceMemorySolver::new(high_q_params);
    assert!(
        high_q_solver.compute_quantum_coherence_t2_ms() >= t2,
        "Higher acoustic Q-factor should extend or preserve coherence time"
    );
}

#[test]
fn test_fault_tolerant_threshold_bounds() {
    let params = MajoranaSurfaceMemoryParams::default();
    let solver = MajoranaSurfaceMemorySolver::new(params);
    let threshold = solver.compute_fault_tolerant_threshold();

    // Target fault-tolerant threshold >= 1.0% (0.010)
    assert!(
        threshold >= 0.010,
        "Fault-tolerant threshold must be >= 0.010 (1.0%), got {:.4}",
        threshold
    );
    assert!(
        threshold <= 0.050,
        "Threshold must stay within realistic topological surface code limits, got {:.4}",
        threshold
    );
}

#[test]
fn test_syndrome_decoding_latency_bounds() {
    let distances = [3, 5, 7, 9, 11, 13, 15];
    for &d in &distances {
        let params = MajoranaSurfaceMemoryParams::new(d, 4.8, 2.5e7, 0.0015, 250.0, 35.0, 12.0, 8.5);
        let solver = MajoranaSurfaceMemorySolver::new(params);
        let lat = solver.compute_syndrome_decoding_latency_us();

        // Target decoding latency <= 2.50 us and >= 0.50 us
        assert!(
            lat <= 2.50,
            "Syndrome decoding latency for d={} must be <= 2.50 us, got {:.3} us",
            d,
            lat
        );
        assert!(
            lat >= 0.50,
            "Syndrome decoding latency for d={} must be >= 0.50 us, got {:.3} us",
            d,
            lat
        );
    }
}

#[test]
fn test_logical_error_rate_scaling() {
    let params = MajoranaSurfaceMemoryParams::default();
    let solver = MajoranaSurfaceMemorySolver::new(params);
    let p_l = solver.compute_logical_error_rate();

    // Target logical error rate <= 1.0e-5
    assert!(
        p_l <= 1.0e-5,
        "Logical error rate must be <= 1.0e-5, got {:.4e}",
        p_l
    );
    assert!(
        p_l >= 1.0e-9,
        "Logical error rate must be >= 1.0e-9, got {:.4e}",
        p_l
    );

    // Verify distance scaling: increasing distance d suppresses or preserves logical error rate
    let p_d3 = MajoranaSurfaceMemorySolver::new(MajoranaSurfaceMemoryParams::new(3, 4.8, 2.5e7, 0.0015, 250.0, 35.0, 12.0, 8.5)).compute_logical_error_rate();
    let p_d7 = MajoranaSurfaceMemorySolver::new(MajoranaSurfaceMemoryParams::new(7, 4.8, 2.5e7, 0.0015, 250.0, 35.0, 12.0, 8.5)).compute_logical_error_rate();
    assert!(
        p_d7 <= p_d3,
        "Logical error rate at d=7 ({:.4e}) must be <= d=3 ({:.4e})",
        p_d7,
        p_d3
    );
}

#[test]
fn test_acoustic_qubit_storage_fidelity_bounds() {
    let params = MajoranaSurfaceMemoryParams::default();
    let solver = MajoranaSurfaceMemorySolver::new(params);
    let fid = solver.compute_acoustic_qubit_storage_fidelity();

    // Target acoustic qubit storage fidelity >= 99.5% (0.995)
    assert!(
        fid >= 0.995,
        "Acoustic qubit storage fidelity must be >= 0.995, got {:.5}",
        fid
    );
    assert!(
        fid <= 1.0,
        "Storage fidelity cannot exceed unity, got {:.5}",
        fid
    );
}

#[test]
fn test_parameter_clamping_and_defaults() {
    let def = MajoranaSurfaceMemoryParams::default();
    assert_eq!(def.code_distance, 5);
    assert_eq!(def.cavity_resonance_ghz, 4.8);
    assert_eq!(def.acoustic_quality_factor, 2.5e7);
    assert_eq!(def.physical_error_rate, 0.0015);
    assert_eq!(def.syndrome_extraction_time_ns, 250.0);
    assert_eq!(def.majorana_coupling_mhz, 35.0);
    assert_eq!(def.operating_temp_m_k, 12.0);
    assert_eq!(def.readout_dispersive_shift_mhz, 8.5);

    // Test out of range clamping
    let clamped = MajoranaSurfaceMemoryParams::new(1, 0.1, 1.0e5, 1.0e-5, 10.0, 1.0, 0.2, 0.5);
    assert_eq!(clamped.code_distance, 3);
    assert_eq!(clamped.cavity_resonance_ghz, 1.0);
    assert_eq!(clamped.acoustic_quality_factor, 1.0e6);
    assert_eq!(clamped.physical_error_rate, 1.0e-4);
    assert_eq!(clamped.syndrome_extraction_time_ns, 50.0);
    assert_eq!(clamped.majorana_coupling_mhz, 5.0);
    assert_eq!(clamped.operating_temp_m_k, 1.0);
    assert_eq!(clamped.readout_dispersive_shift_mhz, 1.0);

    let clamped_high = MajoranaSurfaceMemoryParams::new(99, 99.0, 1.0e12, 0.99, 5000.0, 500.0, 200.0, 100.0);
    assert_eq!(clamped_high.code_distance, 15);
    assert_eq!(clamped_high.cavity_resonance_ghz, 12.0);
    assert_eq!(clamped_high.acoustic_quality_factor, 1.0e9);
    assert_eq!(clamped_high.physical_error_rate, 0.05);
    assert_eq!(clamped_high.syndrome_extraction_time_ns, 1000.0);
    assert_eq!(clamped_high.majorana_coupling_mhz, 100.0);
    assert_eq!(clamped_high.operating_temp_m_k, 50.0);
    assert_eq!(clamped_high.readout_dispersive_shift_mhz, 30.0);
}

#[test]
fn test_full_metrics_physical_compliance() {
    let params = MajoranaSurfaceMemoryParams::default();
    let solver = MajoranaSurfaceMemorySolver::new(params);
    let m = solver.evaluate_metrics();

    assert!(m.is_physically_compliant, "Default parameters must be physically compliant");
    assert!(m.quantum_coherence_t2_ms >= 10.0);
    assert!(m.fault_tolerant_threshold >= 0.010);
    assert!(m.syndrome_decoding_latency_us <= 2.50);
    assert!(m.logical_error_rate <= 1.0e-5);
    assert!(m.acoustic_qubit_storage_fidelity >= 0.995);
}
