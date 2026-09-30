#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Quantum Metamaterial Frequency-Comb Beamformer & Hyperspectral Lidar Engine.

use phonon_models::quantum_metamaterial_beamformer::QuantumMetamaterialBeamformerParams;
use phonon_solver::quantum_metamaterial_beamformer::QuantumMetamaterialBeamformerSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = QuantumMetamaterialBeamformerParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.beamformer_coupling_mev, 1.0);
    assert_eq!(underflow.topological_comb_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.beam_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.optical_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_comb_elements_factor, 1.0);
    assert_eq!(underflow.beamformer_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = QuantumMetamaterialBeamformerParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.beamformer_coupling_mev, 35.0);
    assert_eq!(overflow.topological_comb_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.beam_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.optical_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_comb_elements_factor, 8.0);
    assert_eq!(overflow.beamformer_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = QuantumMetamaterialBeamformerParams::default();
    assert_eq!(params.beamformer_coupling_mev, 35.0);
    assert_eq!(params.topological_comb_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.beam_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.optical_probe_power_uw, 20.8);
    assert_eq!(params.synthetic_comb_elements_factor, 4.0);
    assert_eq!(params.beamformer_pitch_um, 19.8);

    let solver = QuantumMetamaterialBeamformerSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.beamformer_fidelity >= 0.9980,
        "Beamformer fidelity must be >= 0.9980, got {:.6}",
        metrics.beamformer_fidelity
    );
    assert!(
        metrics.frequency_comb_state_retention_fraction >= 0.9970,
        "Frequency comb state retention fraction must be >= 0.9970, got {:.6}",
        metrics.frequency_comb_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_element_crosstalk_isolation_db >= 55.0,
        "Inter-element crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_element_crosstalk_isolation_db
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
fn test_beamformer_coupling_monotonicity() {
    let p_low = QuantumMetamaterialBeamformerParams {
        beamformer_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = QuantumMetamaterialBeamformerParams {
        beamformer_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = QuantumMetamaterialBeamformerSolver::new(p_low);
    let s_high = QuantumMetamaterialBeamformerSolver::new(p_high);

    assert!(
        s_high.compute_beamformer_fidelity()
            > s_low.compute_beamformer_fidelity()
    );
    assert!(
        s_high.compute_frequency_comb_state_retention_fraction()
            > s_low.compute_frequency_comb_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_element_crosstalk_isolation_db()
            > s_low.compute_inter_element_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_topological_comb_gap_monotonicity() {
    let p_low = QuantumMetamaterialBeamformerParams {
        topological_comb_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = QuantumMetamaterialBeamformerParams {
        topological_comb_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = QuantumMetamaterialBeamformerSolver::new(p_low);
    let s_high = QuantumMetamaterialBeamformerSolver::new(p_high);

    assert!(
        s_high.compute_beamformer_fidelity()
            > s_low.compute_beamformer_fidelity()
    );
    assert!(
        s_high.compute_frequency_comb_state_retention_fraction()
            > s_low.compute_frequency_comb_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_element_crosstalk_isolation_db()
            > s_low.compute_inter_element_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_acoustic_drive_frequency_monotonicity() {
    let p_low = QuantumMetamaterialBeamformerParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = QuantumMetamaterialBeamformerParams {
        acoustic_drive_frequency_ghz: 11.0,
        ..Default::default()
    };

    let s_low = QuantumMetamaterialBeamformerSolver::new(p_low);
    let s_high = QuantumMetamaterialBeamformerSolver::new(p_high);

    assert!(
        s_high.compute_beamformer_fidelity()
            > s_low.compute_beamformer_fidelity()
    );
    assert!(
        s_high.compute_frequency_comb_state_retention_fraction()
            > s_low.compute_frequency_comb_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_element_crosstalk_isolation_db()
            > s_low.compute_inter_element_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_beam_dispatch_speed_monotonicity() {
    let p_low = QuantumMetamaterialBeamformerParams {
        beam_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = QuantumMetamaterialBeamformerParams {
        beam_dispatch_speed_m_per_s: 2500.0,
        ..Default::default()
    };

    let s_low = QuantumMetamaterialBeamformerSolver::new(p_low);
    let s_high = QuantumMetamaterialBeamformerSolver::new(p_high);

    assert!(
        s_high.compute_beamformer_fidelity()
            > s_low.compute_beamformer_fidelity()
    );
    assert!(
        s_high.compute_frequency_comb_state_retention_fraction()
            > s_low.compute_frequency_comb_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_element_crosstalk_isolation_db()
            > s_low.compute_inter_element_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_cryogenic_temperature_sensitivity() {
    let p_cold = QuantumMetamaterialBeamformerParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = QuantumMetamaterialBeamformerParams {
        cryogenic_temperature_mk: 45.0,
        ..Default::default()
    };

    let s_cold = QuantumMetamaterialBeamformerSolver::new(p_cold);
    let s_warm = QuantumMetamaterialBeamformerSolver::new(p_warm);

    // Warm dilution temperature degrades metrics but increases dephasing
    assert!(
        s_cold.compute_beamformer_fidelity()
            > s_warm.compute_beamformer_fidelity()
    );
    assert!(
        s_cold.compute_frequency_comb_state_retention_fraction()
            > s_warm.compute_frequency_comb_state_retention_fraction()
    );
    assert!(
        s_cold.compute_topological_protection_gap_mhz()
            > s_warm.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_cold.compute_inter_element_crosstalk_isolation_db()
            > s_warm.compute_inter_element_crosstalk_isolation_db()
    );
    assert!(
        s_cold.compute_topological_mode_dephasing_rate_hz()
            < s_warm.compute_topological_mode_dephasing_rate_hz()
    );

    // Even at warm limits, parameters still maintain physical compliance
    assert!(s_warm.evaluate_metrics().is_physically_compliant);
}

#[test]
fn test_optical_probe_power_scaling() {
    let p_low = QuantumMetamaterialBeamformerParams {
        optical_probe_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = QuantumMetamaterialBeamformerParams {
        optical_probe_power_uw: 28.0,
        ..Default::default()
    };

    let s_low = QuantumMetamaterialBeamformerSolver::new(p_low);
    let s_high = QuantumMetamaterialBeamformerSolver::new(p_high);

    assert!(
        s_high.compute_beamformer_fidelity()
            > s_low.compute_beamformer_fidelity()
    );
    assert!(
        s_high.compute_frequency_comb_state_retention_fraction()
            > s_low.compute_frequency_comb_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_element_crosstalk_isolation_db()
            > s_low.compute_inter_element_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_synthetic_comb_elements_scaling() {
    let p_few = QuantumMetamaterialBeamformerParams {
        synthetic_comb_elements_factor: 2.0,
        ..Default::default()
    };
    let p_many = QuantumMetamaterialBeamformerParams {
        synthetic_comb_elements_factor: 7.0,
        ..Default::default()
    };

    let s_few = QuantumMetamaterialBeamformerSolver::new(p_few);
    let s_many = QuantumMetamaterialBeamformerSolver::new(p_many);

    assert!(
        s_many.compute_beamformer_fidelity()
            > s_few.compute_beamformer_fidelity()
    );
    assert!(
        s_many.compute_frequency_comb_state_retention_fraction()
            > s_few.compute_frequency_comb_state_retention_fraction()
    );
    assert!(
        s_many.compute_topological_protection_gap_mhz()
            > s_few.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_many.compute_inter_element_crosstalk_isolation_db()
            > s_few.compute_inter_element_crosstalk_isolation_db()
    );
    assert!(
        s_many.compute_topological_mode_dephasing_rate_hz()
            < s_few.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_beamformer_pitch_scaling() {
    let p_tight = QuantumMetamaterialBeamformerParams {
        beamformer_pitch_um: 2.0,
        ..Default::default()
    };
    let p_wide = QuantumMetamaterialBeamformerParams {
        beamformer_pitch_um: 18.0,
        ..Default::default()
    };

    let s_tight = QuantumMetamaterialBeamformerSolver::new(p_tight);
    let s_wide = QuantumMetamaterialBeamformerSolver::new(p_wide);

    assert!(
        s_wide.compute_beamformer_fidelity()
            > s_tight.compute_beamformer_fidelity()
    );
    assert!(
        s_wide.compute_frequency_comb_state_retention_fraction()
            > s_tight.compute_frequency_comb_state_retention_fraction()
    );
    assert!(
        s_wide.compute_topological_protection_gap_mhz()
            > s_tight.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_wide.compute_inter_element_crosstalk_isolation_db()
            > s_tight.compute_inter_element_crosstalk_isolation_db()
    );
    assert!(
        s_wide.compute_topological_mode_dephasing_rate_hz()
            < s_tight.compute_topological_mode_dephasing_rate_hz()
    );
}
