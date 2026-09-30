#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Topological Axion-Polariton Waveguide & Quantum Hall Beam Splitter Engine.

use phonon_models::axion_polariton_beam_splitter::AxionPolaritonBeamSplitterParams;
use phonon_solver::axion_polariton_beam_splitter::AxionPolaritonBeamSplitterSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = AxionPolaritonBeamSplitterParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.splitter_coupling_mev, 1.0);
    assert_eq!(underflow.topological_axion_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.beam_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_hall_ports_factor, 1.0);
    assert_eq!(underflow.waveguide_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = AxionPolaritonBeamSplitterParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.splitter_coupling_mev, 35.0);
    assert_eq!(overflow.topological_axion_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.beam_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_hall_ports_factor, 8.0);
    assert_eq!(overflow.waveguide_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AxionPolaritonBeamSplitterParams::default();
    assert_eq!(params.splitter_coupling_mev, 35.0);
    assert_eq!(params.topological_axion_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.beam_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 21.0);
    assert_eq!(params.synthetic_hall_ports_factor, 4.0);
    assert_eq!(params.waveguide_pitch_um, 20.0);

    let solver = AxionPolaritonBeamSplitterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.beam_splitter_fidelity >= 0.9980,
        "Beam splitter fidelity must be >= 0.9980, got {:.6}",
        metrics.beam_splitter_fidelity
    );
    assert!(
        metrics.quantum_hall_state_retention_fraction >= 0.9970,
        "Quantum Hall state retention fraction must be >= 0.9970, got {:.6}",
        metrics.quantum_hall_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_port_crosstalk_isolation_db >= 55.0,
        "Inter-port crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_port_crosstalk_isolation_db
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
fn test_splitter_coupling_monotonicity() {
    let p_low = AxionPolaritonBeamSplitterParams {
        splitter_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonBeamSplitterParams {
        splitter_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonBeamSplitterSolver::new(p_low);
    let s_high = AxionPolaritonBeamSplitterSolver::new(p_high);

    assert!(
        s_high.compute_beam_splitter_fidelity()
            > s_low.compute_beam_splitter_fidelity()
    );
    assert!(
        s_high.compute_quantum_hall_state_retention_fraction()
            > s_low.compute_quantum_hall_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_port_crosstalk_isolation_db()
            > s_low.compute_inter_port_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_topological_axion_gap_monotonicity() {
    let p_low = AxionPolaritonBeamSplitterParams {
        topological_axion_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonBeamSplitterParams {
        topological_axion_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonBeamSplitterSolver::new(p_low);
    let s_high = AxionPolaritonBeamSplitterSolver::new(p_high);

    assert!(
        s_high.compute_beam_splitter_fidelity()
            > s_low.compute_beam_splitter_fidelity()
    );
    assert!(
        s_high.compute_quantum_hall_state_retention_fraction()
            > s_low.compute_quantum_hall_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_port_crosstalk_isolation_db()
            > s_low.compute_inter_port_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_acoustic_drive_frequency_monotonicity() {
    let p_low = AxionPolaritonBeamSplitterParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonBeamSplitterParams {
        acoustic_drive_frequency_ghz: 11.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonBeamSplitterSolver::new(p_low);
    let s_high = AxionPolaritonBeamSplitterSolver::new(p_high);

    assert!(
        s_high.compute_beam_splitter_fidelity()
            > s_low.compute_beam_splitter_fidelity()
    );
    assert!(
        s_high.compute_quantum_hall_state_retention_fraction()
            > s_low.compute_quantum_hall_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_port_crosstalk_isolation_db()
            > s_low.compute_inter_port_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_beam_dispatch_speed_monotonicity() {
    let p_low = AxionPolaritonBeamSplitterParams {
        beam_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonBeamSplitterParams {
        beam_dispatch_speed_m_per_s: 2800.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonBeamSplitterSolver::new(p_low);
    let s_high = AxionPolaritonBeamSplitterSolver::new(p_high);

    assert!(
        s_high.compute_beam_splitter_fidelity()
            > s_low.compute_beam_splitter_fidelity()
    );
    assert!(
        s_high.compute_quantum_hall_state_retention_fraction()
            > s_low.compute_quantum_hall_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_port_crosstalk_isolation_db()
            > s_low.compute_inter_port_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_cryogenic_temperature_sensitivity() {
    let p_cold = AxionPolaritonBeamSplitterParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = AxionPolaritonBeamSplitterParams {
        cryogenic_temperature_mk: 45.0,
        ..Default::default()
    };

    let s_cold = AxionPolaritonBeamSplitterSolver::new(p_cold);
    let s_warm = AxionPolaritonBeamSplitterSolver::new(p_warm);

    // Warm temperature increases thermal dephasing and degrades fidelity/retention/gap/isolation
    assert!(
        s_cold.compute_beam_splitter_fidelity()
            > s_warm.compute_beam_splitter_fidelity()
    );
    assert!(
        s_cold.compute_quantum_hall_state_retention_fraction()
            > s_warm.compute_quantum_hall_state_retention_fraction()
    );
    assert!(
        s_cold.compute_topological_protection_gap_mhz()
            > s_warm.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_cold.compute_inter_port_crosstalk_isolation_db()
            > s_warm.compute_inter_port_crosstalk_isolation_db()
    );
    assert!(
        s_warm.compute_topological_mode_dephasing_rate_hz()
            > s_cold.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let p_low = AxionPolaritonBeamSplitterParams {
        microwave_probe_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonBeamSplitterParams {
        microwave_probe_power_uw: 28.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonBeamSplitterSolver::new(p_low);
    let s_high = AxionPolaritonBeamSplitterSolver::new(p_high);

    assert!(
        s_high.compute_beam_splitter_fidelity()
            > s_low.compute_beam_splitter_fidelity()
    );
    assert!(
        s_high.compute_quantum_hall_state_retention_fraction()
            > s_low.compute_quantum_hall_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_port_crosstalk_isolation_db()
            > s_low.compute_inter_port_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_synthetic_hall_ports_scaling() {
    let p_few = AxionPolaritonBeamSplitterParams {
        synthetic_hall_ports_factor: 2.0,
        ..Default::default()
    };
    let p_many = AxionPolaritonBeamSplitterParams {
        synthetic_hall_ports_factor: 7.0,
        ..Default::default()
    };

    let s_few = AxionPolaritonBeamSplitterSolver::new(p_few);
    let s_many = AxionPolaritonBeamSplitterSolver::new(p_many);

    assert!(
        s_many.compute_beam_splitter_fidelity()
            > s_few.compute_beam_splitter_fidelity()
    );
    assert!(
        s_many.compute_quantum_hall_state_retention_fraction()
            > s_few.compute_quantum_hall_state_retention_fraction()
    );
    assert!(
        s_many.compute_topological_protection_gap_mhz()
            > s_few.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_many.compute_inter_port_crosstalk_isolation_db()
            > s_few.compute_inter_port_crosstalk_isolation_db()
    );
    assert!(
        s_many.compute_topological_mode_dephasing_rate_hz()
            < s_few.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_waveguide_pitch_scaling() {
    let p_tight = AxionPolaritonBeamSplitterParams {
        waveguide_pitch_um: 2.0,
        ..Default::default()
    };
    let p_wide = AxionPolaritonBeamSplitterParams {
        waveguide_pitch_um: 18.0,
        ..Default::default()
    };

    let s_tight = AxionPolaritonBeamSplitterSolver::new(p_tight);
    let s_wide = AxionPolaritonBeamSplitterSolver::new(p_wide);

    assert!(
        s_wide.compute_beam_splitter_fidelity()
            > s_tight.compute_beam_splitter_fidelity()
    );
    assert!(
        s_wide.compute_quantum_hall_state_retention_fraction()
            > s_tight.compute_quantum_hall_state_retention_fraction()
    );
    assert!(
        s_wide.compute_topological_protection_gap_mhz()
            > s_tight.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_wide.compute_inter_port_crosstalk_isolation_db()
            > s_tight.compute_inter_port_crosstalk_isolation_db()
    );
    assert!(
        s_wide.compute_topological_mode_dephasing_rate_hz()
            < s_tight.compute_topological_mode_dephasing_rate_hz()
    );
}
