#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Topological Axion-Polariton Circulator & Quantum Interconnect Hub Engine.

use phonon_models::axion_polariton_circulator::AxionPolaritonCirculatorParams;
use phonon_solver::axion_polariton_circulator::AxionPolaritonCirculatorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = AxionPolaritonCirculatorParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.circulator_coupling_mev, 1.0);
    assert_eq!(underflow.topological_axion_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.circulation_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_circulator_ports_factor, 1.0);
    assert_eq!(underflow.circulator_junction_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = AxionPolaritonCirculatorParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        35.0,   // above 25.0 um
    );
    assert_eq!(overflow.circulator_coupling_mev, 35.0);
    assert_eq!(overflow.topological_axion_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.circulation_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_circulator_ports_factor, 8.0);
    assert_eq!(overflow.circulator_junction_pitch_um, 25.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AxionPolaritonCirculatorParams::default();
    assert_eq!(params.circulator_coupling_mev, 35.0);
    assert_eq!(params.topological_axion_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.circulation_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 23.0);
    assert_eq!(params.synthetic_circulator_ports_factor, 4.0);
    assert_eq!(params.circulator_junction_pitch_um, 22.0);

    let solver = AxionPolaritonCirculatorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.circulator_fidelity >= 0.9980,
        "Circulator fidelity must be >= 0.9980, got {:.6}",
        metrics.circulator_fidelity
    );
    assert!(
        metrics.polariton_state_retention_fraction >= 0.9970,
        "Polariton state retention fraction must be >= 0.9970, got {:.6}",
        metrics.polariton_state_retention_fraction
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
fn test_circulator_coupling_monotonicity() {
    let p_low = AxionPolaritonCirculatorParams {
        circulator_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonCirculatorParams {
        circulator_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonCirculatorSolver::new(p_low);
    let s_high = AxionPolaritonCirculatorSolver::new(p_high);

    assert!(
        s_high.compute_circulator_fidelity()
            > s_low.compute_circulator_fidelity()
    );
    assert!(
        s_high.compute_polariton_state_retention_fraction()
            > s_low.compute_polariton_state_retention_fraction()
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
fn test_topological_axion_gap_monotonicity() {
    let p_low = AxionPolaritonCirculatorParams {
        topological_axion_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonCirculatorParams {
        topological_axion_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonCirculatorSolver::new(p_low);
    let s_high = AxionPolaritonCirculatorSolver::new(p_high);

    assert!(
        s_high.compute_circulator_fidelity()
            > s_low.compute_circulator_fidelity()
    );
    assert!(
        s_high.compute_polariton_state_retention_fraction()
            > s_low.compute_polariton_state_retention_fraction()
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
    let p_low = AxionPolaritonCirculatorParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonCirculatorParams {
        acoustic_drive_frequency_ghz: 10.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonCirculatorSolver::new(p_low);
    let s_high = AxionPolaritonCirculatorSolver::new(p_high);

    assert!(
        s_high.compute_circulator_fidelity()
            > s_low.compute_circulator_fidelity()
    );
    assert!(
        s_high.compute_polariton_state_retention_fraction()
            > s_low.compute_polariton_state_retention_fraction()
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
fn test_circulation_dispatch_speed_monotonicity() {
    let p_low = AxionPolaritonCirculatorParams {
        circulation_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonCirculatorParams {
        circulation_dispatch_speed_m_per_s: 2500.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonCirculatorSolver::new(p_low);
    let s_high = AxionPolaritonCirculatorSolver::new(p_high);

    assert!(
        s_high.compute_circulator_fidelity()
            > s_low.compute_circulator_fidelity()
    );
    assert!(
        s_high.compute_polariton_state_retention_fraction()
            > s_low.compute_polariton_state_retention_fraction()
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
fn test_microwave_probe_power_monotonicity() {
    let p_low = AxionPolaritonCirculatorParams {
        microwave_probe_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonCirculatorParams {
        microwave_probe_power_uw: 25.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonCirculatorSolver::new(p_low);
    let s_high = AxionPolaritonCirculatorSolver::new(p_high);

    assert!(
        s_high.compute_circulator_fidelity()
            > s_low.compute_circulator_fidelity()
    );
    assert!(
        s_high.compute_polariton_state_retention_fraction()
            > s_low.compute_polariton_state_retention_fraction()
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
fn test_synthetic_circulator_ports_monotonicity() {
    let p_low = AxionPolaritonCirculatorParams {
        synthetic_circulator_ports_factor: 2.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonCirculatorParams {
        synthetic_circulator_ports_factor: 6.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonCirculatorSolver::new(p_low);
    let s_high = AxionPolaritonCirculatorSolver::new(p_high);

    assert!(
        s_high.compute_circulator_fidelity()
            > s_low.compute_circulator_fidelity()
    );
    assert!(
        s_high.compute_polariton_state_retention_fraction()
            > s_low.compute_polariton_state_retention_fraction()
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
    let p_cold = AxionPolaritonCirculatorParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = AxionPolaritonCirculatorParams {
        cryogenic_temperature_mk: 40.0,
        ..Default::default()
    };

    let s_cold = AxionPolaritonCirculatorSolver::new(p_cold);
    let s_warm = AxionPolaritonCirculatorSolver::new(p_warm);

    assert!(
        s_cold.compute_circulator_fidelity()
            > s_warm.compute_circulator_fidelity()
    );
    assert!(
        s_cold.compute_polariton_state_retention_fraction()
            > s_warm.compute_polariton_state_retention_fraction()
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
        s_cold.compute_topological_mode_dephasing_rate_hz()
            < s_warm.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_circulator_junction_pitch_scaling() {
    let p_narrow = AxionPolaritonCirculatorParams {
        circulator_junction_pitch_um: 2.0,
        ..Default::default()
    };
    let p_wide = AxionPolaritonCirculatorParams {
        circulator_junction_pitch_um: 22.0,
        ..Default::default()
    };

    let s_narrow = AxionPolaritonCirculatorSolver::new(p_narrow);
    let s_wide = AxionPolaritonCirculatorSolver::new(p_wide);

    assert!(
        s_wide.compute_circulator_fidelity()
            > s_narrow.compute_circulator_fidelity()
    );
    assert!(
        s_wide.compute_polariton_state_retention_fraction()
            > s_narrow.compute_polariton_state_retention_fraction()
    );
    assert!(
        s_wide.compute_topological_protection_gap_mhz()
            > s_narrow.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_wide.compute_isolation_directivity_db()
            > s_narrow.compute_isolation_directivity_db()
    );
    assert!(
        s_wide.compute_topological_mode_dephasing_rate_hz()
            < s_narrow.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_extreme_physical_limits_compliance() {
    let min_params = AxionPolaritonCirculatorParams::new(
        1.0, 2.0, 1.0, 200.0, 50.0, 0.5, 1.0, 0.5,
    );
    let solver_min = AxionPolaritonCirculatorSolver::new(min_params);
    let metrics_min = solver_min.evaluate_metrics();

    assert!(metrics_min.circulator_fidelity >= 0.9980);
    assert!(metrics_min.polariton_state_retention_fraction >= 0.9970);
    assert!(metrics_min.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_min.isolation_directivity_db >= 55.0);
    assert!(metrics_min.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_min.is_physically_compliant);

    let max_params = AxionPolaritonCirculatorParams::new(
        35.0, 45.0, 12.0, 3000.0, 1.0, 30.0, 8.0, 25.0,
    );
    let solver_max = AxionPolaritonCirculatorSolver::new(max_params);
    let metrics_max = solver_max.evaluate_metrics();

    assert!(metrics_max.circulator_fidelity >= 0.9980);
    assert!(metrics_max.polariton_state_retention_fraction >= 0.9970);
    assert!(metrics_max.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_max.isolation_directivity_db >= 55.0);
    assert!(metrics_max.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_max.is_physically_compliant);
}
