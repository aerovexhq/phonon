#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! anyonic quantum memory and chiral Fibonacci braiding gate fabrics.

use phonon_models::fibonacci_anyon_quantum_memory::FibonacciAnyonQuantumMemoryParams;
use phonon_solver::fibonacci_anyon_quantum_memory::FibonacciAnyonQuantumMemorySolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FibonacciAnyonQuantumMemoryParams::new(
        1.20,   // below 1.50
        20.0,   // below 30.0 MHz
        5.0,    // below 10.0
        0.5,    // below 1.0 GHz
        0.5,    // below 1.0 mK
        0.2,    // below 0.5 um
        5.0,    // below 10.0 us
        100.0,  // below 200.0 mps
    );
    assert_eq!(underflow.golden_ratio_tau, 1.50);
    assert_eq!(underflow.topological_gap_energy_mhz, 30.0);
    assert_eq!(underflow.braid_word_length, 10.0);
    assert_eq!(underflow.acoustic_clock_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.inter_anyon_separation_um, 0.5);
    assert_eq!(underflow.memory_retention_time_us, 10.0);
    assert_eq!(underflow.strain_shuttling_velocity_mps, 200.0);

    // Test values strictly above physical maximum bounds
    let overflow = FibonacciAnyonQuantumMemoryParams::new(
        2.00,    // above 1.70
        120.0,   // above 90.0 MHz
        150.0,   // above 100.0
        20.0,    // above 15.0 GHz
        80.0,    // above 50.0 mK
        12.0,    // above 8.0 um
        700.0,   // above 500.0 us
        4000.0,  // above 3000.0 mps
    );
    assert_eq!(overflow.golden_ratio_tau, 1.70);
    assert_eq!(overflow.topological_gap_energy_mhz, 90.0);
    assert_eq!(overflow.braid_word_length, 100.0);
    assert_eq!(overflow.acoustic_clock_frequency_ghz, 15.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.inter_anyon_separation_um, 8.0);
    assert_eq!(overflow.memory_retention_time_us, 500.0);
    assert_eq!(overflow.strain_shuttling_velocity_mps, 3000.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FibonacciAnyonQuantumMemoryParams::default();
    assert_eq!(params.golden_ratio_tau, 1.618033988749895);
    assert_eq!(params.topological_gap_energy_mhz, 58.0);
    assert_eq!(params.braid_word_length, 32.0);
    assert_eq!(params.acoustic_clock_frequency_ghz, 5.2);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.inter_anyon_separation_um, 2.6);
    assert_eq!(params.memory_retention_time_us, 120.0);
    assert_eq!(params.strain_shuttling_velocity_mps, 1250.0);

    let solver = FibonacciAnyonQuantumMemorySolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.braiding_gate_fidelity >= 0.9980,
        "Braiding gate fidelity must be >= 0.9980, got {:.6}",
        metrics.braiding_gate_fidelity
    );
    assert!(
        metrics.anyon_memory_retention_fraction >= 0.9970,
        "Anyon memory retention fraction must be >= 0.9970, got {:.6}",
        metrics.anyon_memory_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 44.0,
        "Topological protection gap must be >= 44.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_qubit_crosstalk_isolation_db >= 54.0,
        "Inter-qubit crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
        metrics.inter_qubit_crosstalk_isolation_db
    );
    assert!(
        metrics.topological_mode_dephasing_rate_hz <= 14.0,
        "Topological mode dephasing rate must be <= 14.0 Hz, got {:.4} Hz",
        metrics.topological_mode_dephasing_rate_hz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_golden_ratio_tau_scaling() {
    let mut tuned_params = FibonacciAnyonQuantumMemoryParams::default();
    tuned_params.golden_ratio_tau = 1.618033988749895;
    let mut detuned_params = FibonacciAnyonQuantumMemoryParams::default();
    detuned_params.golden_ratio_tau = 1.52;

    let tuned_solver = FibonacciAnyonQuantumMemorySolver::new(tuned_params);
    let detuned_solver = FibonacciAnyonQuantumMemorySolver::new(detuned_params);

    assert!(
        tuned_solver.compute_braiding_gate_fidelity()
            > detuned_solver.compute_braiding_gate_fidelity(),
        "Exact golden ratio tuning must maximize braiding gate fidelity"
    );
    assert!(
        tuned_solver.compute_topological_protection_gap_mhz()
            > detuned_solver.compute_topological_protection_gap_mhz(),
        "Exact golden ratio tuning must maximize topological protection gap"
    );
    assert!(
        tuned_solver.compute_anyon_memory_retention_fraction()
            > detuned_solver.compute_anyon_memory_retention_fraction(),
        "Exact golden ratio tuning must maximize anyon memory retention"
    );
    assert!(
        tuned_solver.compute_topological_mode_dephasing_rate_hz()
            < detuned_solver.compute_topological_mode_dephasing_rate_hz(),
        "Exact golden ratio tuning must suppress topological dephasing rate"
    );
}

#[test]
fn test_topological_gap_energy_scaling() {
    let mut low_params = FibonacciAnyonQuantumMemoryParams::default();
    low_params.topological_gap_energy_mhz = 35.0;
    let mut high_params = FibonacciAnyonQuantumMemoryParams::default();
    high_params.topological_gap_energy_mhz = 85.0;

    let low_solver = FibonacciAnyonQuantumMemorySolver::new(low_params);
    let high_solver = FibonacciAnyonQuantumMemorySolver::new(high_params);

    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Elevated topological gap energy must widen the spectral protection gap"
    );
    assert!(
        high_solver.compute_braiding_gate_fidelity()
            > low_solver.compute_braiding_gate_fidelity(),
        "Elevated topological gap energy must enhance braiding gate fidelity"
    );
    assert!(
        high_solver.compute_anyon_memory_retention_fraction()
            > low_solver.compute_anyon_memory_retention_fraction(),
        "Elevated topological gap energy must enhance memory retention fraction"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Elevated topological gap energy must suppress topological dephasing rate"
    );
}

#[test]
fn test_braid_word_length_scaling() {
    let mut short_params = FibonacciAnyonQuantumMemoryParams::default();
    short_params.braid_word_length = 12.0;
    let mut long_params = FibonacciAnyonQuantumMemoryParams::default();
    long_params.braid_word_length = 85.0;

    let short_solver = FibonacciAnyonQuantumMemorySolver::new(short_params);
    let long_solver = FibonacciAnyonQuantumMemorySolver::new(long_params);

    assert!(
        long_solver.compute_braiding_gate_fidelity()
            > short_solver.compute_braiding_gate_fidelity(),
        "Longer braid word decomposition must improve unitary gate fidelity"
    );
    assert!(
        long_solver.compute_topological_protection_gap_mhz()
            > short_solver.compute_topological_protection_gap_mhz(),
        "Longer braid word decomposition must increase topological protection gap"
    );
}

#[test]
fn test_acoustic_clock_frequency_scaling() {
    let mut low_f = FibonacciAnyonQuantumMemoryParams::default();
    low_f.acoustic_clock_frequency_ghz = 1.5;
    let mut high_f = FibonacciAnyonQuantumMemoryParams::default();
    high_f.acoustic_clock_frequency_ghz = 12.0;

    let low_solver = FibonacciAnyonQuantumMemorySolver::new(low_f);
    let high_solver = FibonacciAnyonQuantumMemorySolver::new(high_f);

    assert!(
        high_solver.compute_braiding_gate_fidelity()
            > low_solver.compute_braiding_gate_fidelity(),
        "Higher acoustic clock frequency must improve braiding gate fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher acoustic clock frequency must widen topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic clock frequency must suppress dephasing rate"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_temp = FibonacciAnyonQuantumMemoryParams::default();
    low_temp.cryogenic_temperature_mk = 2.0;
    let mut high_temp = FibonacciAnyonQuantumMemoryParams::default();
    high_temp.cryogenic_temperature_mk = 45.0;

    let low_solver = FibonacciAnyonQuantumMemorySolver::new(low_temp);
    let high_solver = FibonacciAnyonQuantumMemorySolver::new(high_temp);

    assert!(
        low_solver.compute_topological_mode_dephasing_rate_hz()
            < high_solver.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature must suppress topological dephasing rate"
    );
    assert!(
        low_solver.compute_anyon_memory_retention_fraction()
            > high_solver.compute_anyon_memory_retention_fraction(),
        "Lower cryogenic temperature must improve memory retention fraction"
    );
    assert!(
        low_solver.compute_braiding_gate_fidelity()
            > high_solver.compute_braiding_gate_fidelity(),
        "Lower cryogenic temperature must enhance braiding gate fidelity"
    );
}

#[test]
fn test_inter_anyon_separation_scaling() {
    let mut close_anyons = FibonacciAnyonQuantumMemoryParams::default();
    close_anyons.inter_anyon_separation_um = 0.8;
    let mut distant_anyons = FibonacciAnyonQuantumMemoryParams::default();
    distant_anyons.inter_anyon_separation_um = 7.5;

    let close_solver = FibonacciAnyonQuantumMemorySolver::new(close_anyons);
    let distant_solver = FibonacciAnyonQuantumMemorySolver::new(distant_anyons);

    assert!(
        distant_solver.compute_inter_qubit_crosstalk_isolation_db()
            > close_solver.compute_inter_qubit_crosstalk_isolation_db(),
        "Larger inter-anyon separation must enhance crosstalk isolation"
    );
    assert!(
        distant_solver.compute_topological_protection_gap_mhz()
            > close_solver.compute_topological_protection_gap_mhz(),
        "Larger inter-anyon separation must widen topological protection gap"
    );
}

#[test]
fn test_memory_retention_time_scaling() {
    let mut short_retention = FibonacciAnyonQuantumMemoryParams::default();
    short_retention.memory_retention_time_us = 20.0;
    let mut long_retention = FibonacciAnyonQuantumMemoryParams::default();
    long_retention.memory_retention_time_us = 450.0;

    let short_solver = FibonacciAnyonQuantumMemorySolver::new(short_retention);
    let long_solver = FibonacciAnyonQuantumMemorySolver::new(long_retention);

    assert!(
        short_solver.compute_anyon_memory_retention_fraction()
            > long_solver.compute_anyon_memory_retention_fraction(),
        "Shorter memory storage time must exhibit higher retention fraction"
    );
    assert!(
        short_solver.compute_topological_mode_dephasing_rate_hz()
            < long_solver.compute_topological_mode_dephasing_rate_hz(),
        "Shorter memory holding duration must accumulate lower mode dephasing rate"
    );
}

#[test]
fn test_strain_shuttling_velocity_scaling() {
    let mut slow_params = FibonacciAnyonQuantumMemoryParams::default();
    slow_params.strain_shuttling_velocity_mps = 300.0;
    let mut fast_params = FibonacciAnyonQuantumMemoryParams::default();
    fast_params.strain_shuttling_velocity_mps = 2800.0;

    let slow_solver = FibonacciAnyonQuantumMemorySolver::new(slow_params);
    let fast_solver = FibonacciAnyonQuantumMemorySolver::new(fast_params);

    assert!(
        fast_solver.compute_braiding_gate_fidelity()
            > slow_solver.compute_braiding_gate_fidelity(),
        "Faster strain shuttling velocity must improve braiding gate fidelity"
    );
    assert!(
        fast_solver.compute_anyon_memory_retention_fraction()
            > slow_solver.compute_anyon_memory_retention_fraction(),
        "Faster strain shuttling velocity must enhance memory retention fraction"
    );
}
