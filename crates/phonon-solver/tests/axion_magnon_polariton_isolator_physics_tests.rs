#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Topological Axion-Magnon Polariton Isolator & Quantum Memory Routing Engine.

use phonon_models::axion_magnon_polariton_isolator::AxionMagnonPolaritonIsolatorParams;
use phonon_solver::axion_magnon_polariton_isolator::AxionMagnonPolaritonIsolatorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = AxionMagnonPolaritonIsolatorParams::new(
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
    assert_eq!(underflow.topological_axion_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.routing_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_memory_ports_factor, 1.0);
    assert_eq!(underflow.heterostructure_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = AxionMagnonPolaritonIsolatorParams::new(
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
    assert_eq!(overflow.topological_axion_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.routing_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_memory_ports_factor, 8.0);
    assert_eq!(overflow.heterostructure_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AxionMagnonPolaritonIsolatorParams::default();
    assert_eq!(params.isolator_coupling_mev, 35.0);
    assert_eq!(params.topological_axion_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.routing_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 21.8);
    assert_eq!(params.synthetic_memory_ports_factor, 4.0);
    assert_eq!(params.heterostructure_pitch_um, 20.8);

    let solver = AxionMagnonPolaritonIsolatorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.isolator_fidelity >= 0.9980,
        "Isolator fidelity must be >= 0.9980, got {:.6}",
        metrics.isolator_fidelity
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
        metrics.isolation_contrast_db >= 55.0,
        "Isolation contrast must be >= 55.0 dB, got {:.4} dB",
        metrics.isolation_contrast_db
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
    let p_low = AxionMagnonPolaritonIsolatorParams {
        isolator_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = AxionMagnonPolaritonIsolatorParams {
        isolator_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = AxionMagnonPolaritonIsolatorSolver::new(p_low);
    let s_high = AxionMagnonPolaritonIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_isolator_fidelity()
            > s_low.compute_isolator_fidelity()
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
        s_high.compute_isolation_contrast_db()
            > s_low.compute_isolation_contrast_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_topological_axion_gap_monotonicity() {
    let p_low = AxionMagnonPolaritonIsolatorParams {
        topological_axion_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = AxionMagnonPolaritonIsolatorParams {
        topological_axion_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = AxionMagnonPolaritonIsolatorSolver::new(p_low);
    let s_high = AxionMagnonPolaritonIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_isolator_fidelity()
            > s_low.compute_isolator_fidelity()
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
        s_high.compute_isolation_contrast_db()
            > s_low.compute_isolation_contrast_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_acoustic_drive_frequency_monotonicity() {
    let p_low = AxionMagnonPolaritonIsolatorParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = AxionMagnonPolaritonIsolatorParams {
        acoustic_drive_frequency_ghz: 11.0,
        ..Default::default()
    };

    let s_low = AxionMagnonPolaritonIsolatorSolver::new(p_low);
    let s_high = AxionMagnonPolaritonIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_isolator_fidelity()
            > s_low.compute_isolator_fidelity()
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
        s_high.compute_isolation_contrast_db()
            > s_low.compute_isolation_contrast_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_routing_dispatch_speed_monotonicity() {
    let p_low = AxionMagnonPolaritonIsolatorParams {
        routing_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = AxionMagnonPolaritonIsolatorParams {
        routing_dispatch_speed_m_per_s: 2800.0,
        ..Default::default()
    };

    let s_low = AxionMagnonPolaritonIsolatorSolver::new(p_low);
    let s_high = AxionMagnonPolaritonIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_isolator_fidelity()
            > s_low.compute_isolator_fidelity()
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
        s_high.compute_isolation_contrast_db()
            > s_low.compute_isolation_contrast_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_cryogenic_temperature_sensitivity() {
    let p_cold = AxionMagnonPolaritonIsolatorParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = AxionMagnonPolaritonIsolatorParams {
        cryogenic_temperature_mk: 45.0,
        ..Default::default()
    };

    let s_cold = AxionMagnonPolaritonIsolatorSolver::new(p_cold);
    let s_warm = AxionMagnonPolaritonIsolatorSolver::new(p_warm);

    assert!(
        s_cold.compute_isolator_fidelity()
            > s_warm.compute_isolator_fidelity()
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
        s_cold.compute_isolation_contrast_db()
            > s_warm.compute_isolation_contrast_db()
    );
    assert!(
        s_cold.compute_topological_mode_dephasing_rate_hz()
            < s_warm.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_microwave_probe_power_monotonicity() {
    let p_low = AxionMagnonPolaritonIsolatorParams {
        microwave_probe_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = AxionMagnonPolaritonIsolatorParams {
        microwave_probe_power_uw: 28.0,
        ..Default::default()
    };

    let s_low = AxionMagnonPolaritonIsolatorSolver::new(p_low);
    let s_high = AxionMagnonPolaritonIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_isolator_fidelity()
            > s_low.compute_isolator_fidelity()
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
        s_high.compute_isolation_contrast_db()
            > s_low.compute_isolation_contrast_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_heterostructure_pitch_monotonicity() {
    let p_low = AxionMagnonPolaritonIsolatorParams {
        heterostructure_pitch_um: 2.0,
        ..Default::default()
    };
    let p_high = AxionMagnonPolaritonIsolatorParams {
        heterostructure_pitch_um: 18.0,
        ..Default::default()
    };

    let s_low = AxionMagnonPolaritonIsolatorSolver::new(p_low);
    let s_high = AxionMagnonPolaritonIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_isolator_fidelity()
            > s_low.compute_isolator_fidelity()
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
        s_high.compute_isolation_contrast_db()
            > s_low.compute_isolation_contrast_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_extreme_boundary_compliance() {
    // Minimum physical bounds under maximum thermal penalty (worst-case scenario)
    let p_worst = AxionMagnonPolaritonIsolatorParams::new(
        1.0,   // isolator_coupling_mev min
        2.0,   // topological_axion_gap_mev min
        1.0,   // acoustic_drive_frequency_ghz min
        200.0, // routing_dispatch_speed_m_per_s min
        50.0,  // cryogenic_temperature_mk max
        0.5,   // microwave_probe_power_uw min
        1.0,   // synthetic_memory_ports_factor min
        0.5,   // heterostructure_pitch_um min
    );
    let s_worst = AxionMagnonPolaritonIsolatorSolver::new(p_worst);
    let m_worst = s_worst.evaluate_metrics();

    assert!(
        m_worst.isolator_fidelity >= 0.9980,
        "Worst-case isolator fidelity must be >= 0.9980, got {:.6}",
        m_worst.isolator_fidelity
    );
    assert!(
        m_worst.polariton_state_retention_fraction >= 0.9970,
        "Worst-case polariton state retention fraction must be >= 0.9970, got {:.6}",
        m_worst.polariton_state_retention_fraction
    );
    assert!(
        m_worst.topological_protection_gap_mhz >= 45.0,
        "Worst-case topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        m_worst.topological_protection_gap_mhz
    );
    assert!(
        m_worst.isolation_contrast_db >= 55.0,
        "Worst-case isolation contrast must be >= 55.0 dB, got {:.4} dB",
        m_worst.isolation_contrast_db
    );
    assert!(
        m_worst.topological_mode_dephasing_rate_hz <= 12.0,
        "Worst-case topological mode dephasing rate must be <= 12.0 Hz, got {:.4} Hz",
        m_worst.topological_mode_dephasing_rate_hz
    );
    assert!(
        m_worst.is_physically_compliant,
        "Worst-case parameters must be physically compliant"
    );

    // Maximum physical bounds under minimum thermal penalty (best-case scenario)
    let p_best = AxionMagnonPolaritonIsolatorParams::new(
        35.0,   // isolator_coupling_mev max
        45.0,   // topological_axion_gap_mev max
        12.0,   // acoustic_drive_frequency_ghz max
        3000.0, // routing_dispatch_speed_m_per_s max
        1.0,    // cryogenic_temperature_mk min
        30.0,   // microwave_probe_power_uw max
        8.0,    // synthetic_memory_ports_factor max
        20.0,   // heterostructure_pitch_um max
    );
    let s_best = AxionMagnonPolaritonIsolatorSolver::new(p_best);
    let m_best = s_best.evaluate_metrics();

    assert!(m_best.isolator_fidelity >= 0.9980);
    assert!(m_best.polariton_state_retention_fraction >= 0.9970);
    assert!(m_best.topological_protection_gap_mhz >= 45.0);
    assert!(m_best.isolation_contrast_db >= 55.0);
    assert!(m_best.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(m_best.is_physically_compliant);
}
