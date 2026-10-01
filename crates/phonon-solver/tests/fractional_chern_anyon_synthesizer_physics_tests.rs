#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Topological Fractional Chern Insulator Anyon Braiding & Non-Abelian State Synthesizer Engine.

use phonon_models::fractional_chern_anyon_synthesizer::FractionalChernAnyonSynthesizerParams;
use phonon_solver::fractional_chern_anyon_synthesizer::FractionalChernAnyonSynthesizerSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FractionalChernAnyonSynthesizerParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.anyon_coupling_mev, 1.0);
    assert_eq!(underflow.topological_fci_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.braiding_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_braiding_nodes_factor, 1.0);
    assert_eq!(underflow.anyon_lattice_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = FractionalChernAnyonSynthesizerParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        35.0,   // above 25.0 um
    );
    assert_eq!(overflow.anyon_coupling_mev, 35.0);
    assert_eq!(overflow.topological_fci_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.braiding_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_braiding_nodes_factor, 8.0);
    assert_eq!(overflow.anyon_lattice_pitch_um, 25.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FractionalChernAnyonSynthesizerParams::default();
    assert_eq!(params.anyon_coupling_mev, 35.0);
    assert_eq!(params.topological_fci_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.braiding_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 24.5);
    assert_eq!(params.synthetic_braiding_nodes_factor, 4.0);
    assert_eq!(params.anyon_lattice_pitch_um, 23.5);

    let solver = FractionalChernAnyonSynthesizerSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.anyon_braiding_fidelity >= 0.9980,
        "Anyon braiding fidelity must be >= 0.9980, got {:.6}",
        metrics.anyon_braiding_fidelity
    );
    assert!(
        metrics.non_abelian_state_retention_fraction >= 0.9970,
        "Non-Abelian state retention fraction must be >= 0.9970, got {:.6}",
        metrics.non_abelian_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_node_crosstalk_isolation_db >= 55.0,
        "Inter-node crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_node_crosstalk_isolation_db
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
fn test_anyon_coupling_monotonicity() {
    let p_low = FractionalChernAnyonSynthesizerParams {
        anyon_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = FractionalChernAnyonSynthesizerParams {
        anyon_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = FractionalChernAnyonSynthesizerSolver::new(p_low);
    let s_high = FractionalChernAnyonSynthesizerSolver::new(p_high);

    assert!(
        s_high.compute_anyon_braiding_fidelity()
            > s_low.compute_anyon_braiding_fidelity()
    );
    assert!(
        s_high.compute_non_abelian_state_retention_fraction()
            > s_low.compute_non_abelian_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_node_crosstalk_isolation_db()
            > s_low.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_topological_fci_gap_monotonicity() {
    let p_low = FractionalChernAnyonSynthesizerParams {
        topological_fci_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = FractionalChernAnyonSynthesizerParams {
        topological_fci_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = FractionalChernAnyonSynthesizerSolver::new(p_low);
    let s_high = FractionalChernAnyonSynthesizerSolver::new(p_high);

    assert!(
        s_high.compute_anyon_braiding_fidelity()
            > s_low.compute_anyon_braiding_fidelity()
    );
    assert!(
        s_high.compute_non_abelian_state_retention_fraction()
            > s_low.compute_non_abelian_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_node_crosstalk_isolation_db()
            > s_low.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_acoustic_drive_frequency_monotonicity() {
    let p_low = FractionalChernAnyonSynthesizerParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = FractionalChernAnyonSynthesizerParams {
        acoustic_drive_frequency_ghz: 10.0,
        ..Default::default()
    };

    let s_low = FractionalChernAnyonSynthesizerSolver::new(p_low);
    let s_high = FractionalChernAnyonSynthesizerSolver::new(p_high);

    assert!(
        s_high.compute_anyon_braiding_fidelity()
            > s_low.compute_anyon_braiding_fidelity()
    );
    assert!(
        s_high.compute_non_abelian_state_retention_fraction()
            > s_low.compute_non_abelian_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_node_crosstalk_isolation_db()
            > s_low.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_braiding_dispatch_speed_monotonicity() {
    let p_low = FractionalChernAnyonSynthesizerParams {
        braiding_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = FractionalChernAnyonSynthesizerParams {
        braiding_dispatch_speed_m_per_s: 2500.0,
        ..Default::default()
    };

    let s_low = FractionalChernAnyonSynthesizerSolver::new(p_low);
    let s_high = FractionalChernAnyonSynthesizerSolver::new(p_high);

    assert!(
        s_high.compute_anyon_braiding_fidelity()
            > s_low.compute_anyon_braiding_fidelity()
    );
    assert!(
        s_high.compute_non_abelian_state_retention_fraction()
            > s_low.compute_non_abelian_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_node_crosstalk_isolation_db()
            > s_low.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_microwave_probe_power_monotonicity() {
    let p_low = FractionalChernAnyonSynthesizerParams {
        microwave_probe_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = FractionalChernAnyonSynthesizerParams {
        microwave_probe_power_uw: 25.0,
        ..Default::default()
    };

    let s_low = FractionalChernAnyonSynthesizerSolver::new(p_low);
    let s_high = FractionalChernAnyonSynthesizerSolver::new(p_high);

    assert!(
        s_high.compute_anyon_braiding_fidelity()
            > s_low.compute_anyon_braiding_fidelity()
    );
    assert!(
        s_high.compute_non_abelian_state_retention_fraction()
            > s_low.compute_non_abelian_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_node_crosstalk_isolation_db()
            > s_low.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_synthetic_braiding_nodes_monotonicity() {
    let p_low = FractionalChernAnyonSynthesizerParams {
        synthetic_braiding_nodes_factor: 2.0,
        ..Default::default()
    };
    let p_high = FractionalChernAnyonSynthesizerParams {
        synthetic_braiding_nodes_factor: 6.0,
        ..Default::default()
    };

    let s_low = FractionalChernAnyonSynthesizerSolver::new(p_low);
    let s_high = FractionalChernAnyonSynthesizerSolver::new(p_high);

    assert!(
        s_high.compute_anyon_braiding_fidelity()
            > s_low.compute_anyon_braiding_fidelity()
    );
    assert!(
        s_high.compute_non_abelian_state_retention_fraction()
            > s_low.compute_non_abelian_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_node_crosstalk_isolation_db()
            > s_low.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_cryogenic_temperature_sensitivity() {
    let p_cold = FractionalChernAnyonSynthesizerParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = FractionalChernAnyonSynthesizerParams {
        cryogenic_temperature_mk: 40.0,
        ..Default::default()
    };

    let s_cold = FractionalChernAnyonSynthesizerSolver::new(p_cold);
    let s_warm = FractionalChernAnyonSynthesizerSolver::new(p_warm);

    assert!(
        s_cold.compute_anyon_braiding_fidelity()
            > s_warm.compute_anyon_braiding_fidelity()
    );
    assert!(
        s_cold.compute_non_abelian_state_retention_fraction()
            > s_warm.compute_non_abelian_state_retention_fraction()
    );
    assert!(
        s_cold.compute_topological_protection_gap_mhz()
            > s_warm.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_cold.compute_inter_node_crosstalk_isolation_db()
            > s_warm.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_cold.compute_topological_mode_dephasing_rate_hz()
            < s_warm.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_anyon_lattice_pitch_scaling() {
    let p_narrow = FractionalChernAnyonSynthesizerParams {
        anyon_lattice_pitch_um: 2.0,
        ..Default::default()
    };
    let p_wide = FractionalChernAnyonSynthesizerParams {
        anyon_lattice_pitch_um: 22.0,
        ..Default::default()
    };

    let s_narrow = FractionalChernAnyonSynthesizerSolver::new(p_narrow);
    let s_wide = FractionalChernAnyonSynthesizerSolver::new(p_wide);

    assert!(
        s_wide.compute_anyon_braiding_fidelity()
            > s_narrow.compute_anyon_braiding_fidelity()
    );
    assert!(
        s_wide.compute_non_abelian_state_retention_fraction()
            > s_narrow.compute_non_abelian_state_retention_fraction()
    );
    assert!(
        s_wide.compute_topological_protection_gap_mhz()
            > s_narrow.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_wide.compute_inter_node_crosstalk_isolation_db()
            > s_narrow.compute_inter_node_crosstalk_isolation_db()
    );
    assert!(
        s_wide.compute_topological_mode_dephasing_rate_hz()
            < s_narrow.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_extreme_physical_limits_compliance() {
    let min_params = FractionalChernAnyonSynthesizerParams::new(
        1.0, 2.0, 1.0, 200.0, 50.0, 0.5, 1.0, 0.5,
    );
    let solver_min = FractionalChernAnyonSynthesizerSolver::new(min_params);
    let metrics_min = solver_min.evaluate_metrics();

    assert!(metrics_min.anyon_braiding_fidelity >= 0.9980);
    assert!(metrics_min.non_abelian_state_retention_fraction >= 0.9970);
    assert!(metrics_min.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_min.inter_node_crosstalk_isolation_db >= 55.0);
    assert!(metrics_min.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_min.is_physically_compliant);

    let max_params = FractionalChernAnyonSynthesizerParams::new(
        35.0, 45.0, 12.0, 3000.0, 1.0, 30.0, 8.0, 25.0,
    );
    let solver_max = FractionalChernAnyonSynthesizerSolver::new(max_params);
    let metrics_max = solver_max.evaluate_metrics();

    assert!(metrics_max.anyon_braiding_fidelity >= 0.9980);
    assert!(metrics_max.non_abelian_state_retention_fraction >= 0.9970);
    assert!(metrics_max.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_max.inter_node_crosstalk_isolation_db >= 55.0);
    assert!(metrics_max.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_max.is_physically_compliant);
}
