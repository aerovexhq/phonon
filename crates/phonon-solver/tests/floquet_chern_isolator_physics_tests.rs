#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Floquet-Chern Photonic Waveguide & Topologically Protected Quantum Isolator Engine.

use phonon_models::floquet_chern_isolator::FloquetChernIsolatorParams;
use phonon_solver::floquet_chern_isolator::FloquetChernIsolatorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FloquetChernIsolatorParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.isolator_coupling_mev, 1.0);
    assert_eq!(underflow.topological_floquet_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.isolation_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.optical_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_gauge_modes_factor, 1.0);
    assert_eq!(underflow.waveguide_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = FloquetChernIsolatorParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.isolator_coupling_mev, 35.0);
    assert_eq!(overflow.topological_floquet_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.isolation_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.optical_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_gauge_modes_factor, 8.0);
    assert_eq!(overflow.waveguide_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FloquetChernIsolatorParams::default();
    assert_eq!(params.isolator_coupling_mev, 35.0);
    assert_eq!(params.topological_floquet_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.isolation_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.optical_probe_power_uw, 21.2);
    assert_eq!(params.synthetic_gauge_modes_factor, 4.0);
    assert_eq!(params.waveguide_pitch_um, 20.2);

    let solver = FloquetChernIsolatorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.quantum_isolator_fidelity >= 0.9980,
        "Quantum isolator fidelity must be >= 0.9980, got {:.6}",
        metrics.quantum_isolator_fidelity
    );
    assert!(
        metrics.floquet_state_retention_fraction >= 0.9970,
        "Floquet state retention fraction must be >= 0.9970, got {:.6}",
        metrics.floquet_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.isolation_directivity_db >= 55.0,
        "Isolation directivity must be >= 55.0 dB, got {:.4} dB",
        metrics.isolation_directivity_db
    );
    assert!(
        metrics.topological_mode_dephasing_rate_hz <= 12.0,
        "Topological mode dephasing rate must be <= 12.0 Hz, got {:.4} Hz",
        metrics.topological_mode_dephasing_rate_hz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_isolator_coupling_monotonicity() {
    let p_low = FloquetChernIsolatorParams {
        isolator_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = FloquetChernIsolatorParams {
        isolator_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = FloquetChernIsolatorSolver::new(p_low);
    let s_high = FloquetChernIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_quantum_isolator_fidelity()
            > s_low.compute_quantum_isolator_fidelity()
    );
    assert!(
        s_high.compute_floquet_state_retention_fraction()
            > s_low.compute_floquet_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_isolation_directivity_db()
            > s_low.compute_isolation_directivity_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_topological_floquet_gap_monotonicity() {
    let p_low = FloquetChernIsolatorParams {
        topological_floquet_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = FloquetChernIsolatorParams {
        topological_floquet_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = FloquetChernIsolatorSolver::new(p_low);
    let s_high = FloquetChernIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_quantum_isolator_fidelity()
            > s_low.compute_quantum_isolator_fidelity()
    );
    assert!(
        s_high.compute_floquet_state_retention_fraction()
            > s_low.compute_floquet_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_isolation_directivity_db()
            > s_low.compute_isolation_directivity_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_acoustic_drive_frequency_monotonicity() {
    let p_low = FloquetChernIsolatorParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = FloquetChernIsolatorParams {
        acoustic_drive_frequency_ghz: 11.0,
        ..Default::default()
    };

    let s_low = FloquetChernIsolatorSolver::new(p_low);
    let s_high = FloquetChernIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_quantum_isolator_fidelity()
            > s_low.compute_quantum_isolator_fidelity()
    );
    assert!(
        s_high.compute_floquet_state_retention_fraction()
            > s_low.compute_floquet_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_isolation_directivity_db()
            > s_low.compute_isolation_directivity_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_isolation_dispatch_speed_monotonicity() {
    let p_low = FloquetChernIsolatorParams {
        isolation_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = FloquetChernIsolatorParams {
        isolation_dispatch_speed_m_per_s: 2800.0,
        ..Default::default()
    };

    let s_low = FloquetChernIsolatorSolver::new(p_low);
    let s_high = FloquetChernIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_quantum_isolator_fidelity()
            > s_low.compute_quantum_isolator_fidelity()
    );
    assert!(
        s_high.compute_floquet_state_retention_fraction()
            > s_low.compute_floquet_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_isolation_directivity_db()
            > s_low.compute_isolation_directivity_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_cryogenic_temperature_sensitivity() {
    let p_cold = FloquetChernIsolatorParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = FloquetChernIsolatorParams {
        cryogenic_temperature_mk: 45.0,
        ..Default::default()
    };

    let s_cold = FloquetChernIsolatorSolver::new(p_cold);
    let s_warm = FloquetChernIsolatorSolver::new(p_warm);

    // Warm temperature increases thermal dephasing and degrades fidelity/retention/gap/isolation
    assert!(
        s_cold.compute_quantum_isolator_fidelity()
            > s_warm.compute_quantum_isolator_fidelity()
    );
    assert!(
        s_cold.compute_floquet_state_retention_fraction()
            > s_warm.compute_floquet_state_retention_fraction()
    );
    assert!(
        s_cold.compute_topological_protection_gap_mhz()
            > s_warm.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_cold.compute_isolation_directivity_db()
            > s_warm.compute_isolation_directivity_db()
    );
    assert!(
        s_warm.compute_topological_mode_dephasing_rate_hz()
            > s_cold.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_optical_probe_power_scaling() {
    let p_low = FloquetChernIsolatorParams {
        optical_probe_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = FloquetChernIsolatorParams {
        optical_probe_power_uw: 28.0,
        ..Default::default()
    };

    let s_low = FloquetChernIsolatorSolver::new(p_low);
    let s_high = FloquetChernIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_quantum_isolator_fidelity()
            > s_low.compute_quantum_isolator_fidelity()
    );
    assert!(
        s_high.compute_floquet_state_retention_fraction()
            > s_low.compute_floquet_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_isolation_directivity_db()
            > s_low.compute_isolation_directivity_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_synthetic_gauge_modes_scaling() {
    let p_few = FloquetChernIsolatorParams {
        synthetic_gauge_modes_factor: 2.0,
        ..Default::default()
    };
    let p_many = FloquetChernIsolatorParams {
        synthetic_gauge_modes_factor: 7.0,
        ..Default::default()
    };

    let s_few = FloquetChernIsolatorSolver::new(p_few);
    let s_many = FloquetChernIsolatorSolver::new(p_many);

    assert!(
        s_many.compute_quantum_isolator_fidelity()
            > s_few.compute_quantum_isolator_fidelity()
    );
    assert!(
        s_many.compute_floquet_state_retention_fraction()
            > s_few.compute_floquet_state_retention_fraction()
    );
    assert!(
        s_many.compute_topological_protection_gap_mhz()
            > s_few.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_many.compute_isolation_directivity_db()
            > s_few.compute_isolation_directivity_db()
    );
    assert!(
        s_many.compute_topological_mode_dephasing_rate_hz()
            < s_few.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_waveguide_pitch_scaling() {
    let p_tight = FloquetChernIsolatorParams {
        waveguide_pitch_um: 2.0,
        ..Default::default()
    };
    let p_wide = FloquetChernIsolatorParams {
        waveguide_pitch_um: 18.0,
        ..Default::default()
    };

    let s_tight = FloquetChernIsolatorSolver::new(p_tight);
    let s_wide = FloquetChernIsolatorSolver::new(p_wide);

    assert!(
        s_wide.compute_quantum_isolator_fidelity()
            > s_tight.compute_quantum_isolator_fidelity()
    );
    assert!(
        s_wide.compute_floquet_state_retention_fraction()
            > s_tight.compute_floquet_state_retention_fraction()
    );
    assert!(
        s_wide.compute_topological_protection_gap_mhz()
            > s_tight.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_wide.compute_isolation_directivity_db()
            > s_tight.compute_isolation_directivity_db()
    );
    assert!(
        s_wide.compute_topological_mode_dephasing_rate_hz()
            < s_tight.compute_topological_mode_dephasing_rate_hz()
    );
}
