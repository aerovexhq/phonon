#![deny(unsafe_code)]

use phonon_models::superconducting_spintronics::{
    CryoDac, CryoPll, CryoReadoutTia, CryoThermalBackaction,
};
use phonon_solver::superconducting_spintronics::{
    CryoSpintronicBenchmarkRunner, LindbladTrajectorySolver, TopologicalQubitDensityMatrix,
    QUANTUM_CONDUCTANCE_G0,
};

#[test]
fn test_cryo_cmos_components() {
    let pll = CryoPll::standard_4k();
    assert_eq!(pll.frequency_hz, 2.5e9);
    assert!(pll.phase_noise_floor_dbc_per_hz() < -100.0);

    let dac = CryoDac::standard_12bit();
    assert_eq!(dac.resolution_bits, 12);
    let lsb = dac.lsb_step_volts();
    assert!(lsb > 0.0 && lsb < 1.0e-3);

    // Midscale code (2048) produces 0.0 V
    let v_mid = dac.code_to_voltage(2048);
    assert!(v_mid.abs() < 1.0e-3);

    let tia = CryoReadoutTia::standard_4k();
    let i_noise = tia.input_current_noise(1000.0);
    assert!(i_noise > 0.0);

    let v_out = tia.amplify_conductance_signal(QUANTUM_CONDUCTANCE_G0, 10.0e-6);
    assert!(v_out > 0.0);

    let thermal = CryoThermalBackaction::stage_4k(5.0e-3);
    assert!(thermal.local_temperature_k() >= 4.0);
    assert!(thermal.dephasing_enhancement_factor() >= 1.0);
}

#[test]
fn test_lindblad_trajectory_solver() {
    let mut state = TopologicalQubitDensityMatrix::new_zero();
    let solver = LindbladTrajectorySolver::new(100.0, 50.0);

    // Verify initial purity is 1.0
    assert!((state.purity() - 1.0).abs() < 1e-9);
    assert!((state.fidelity_to_pure(0.0, 0.0, 1.0) - 1.0).abs() < 1e-9);

    // Execute braid 12
    solver.execute_braid_12(&mut state, 1.0e-8);
    assert!(state.purity() > 0.999);

    // Execute braid 23
    solver.execute_braid_23(&mut state, 1.0e-8);
    assert!(state.purity() > 0.999);

    // Apply decoherence over 1 us
    solver.step_dissipation(&mut state, 1.0e-6);
    assert!(state.purity() > 0.99);
}

#[test]
fn test_cryo_spintronic_benchmark_10000_braids() {
    let runner = CryoSpintronicBenchmarkRunner::new();
    let report = runner.run_benchmark(10_000);

    assert_eq!(report.braid_operations_executed, 10_000);
    assert!(
        report.average_gate_fidelity >= 0.9999,
        "Average fidelity must be >= 99.99%, got {}",
        report.average_gate_fidelity
    );
    assert!(
        report.minimum_gate_fidelity >= 0.999,
        "Minimum fidelity must be >= 99.9%, got {}",
        report.minimum_gate_fidelity
    );
    assert!(report.mixing_chamber_temp_mk < 50.0);
    assert!(report.is_sub_kelvin_stable);
    assert!(
        report.steps_per_second > 10_000.0,
        "Throughput was {} ops/sec",
        report.steps_per_second
    );
    assert!(report.zero_bias_conductance_normalized > 0.99);
}
