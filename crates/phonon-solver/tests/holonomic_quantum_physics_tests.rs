#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Non-Abelian Holonomic Quantum Computing Gate Synthesizer &
//! Geometric Phase Engine.

use phonon_models::holonomic_quantum::HolonomicQuantumParams;
use phonon_solver::holonomic_quantum::HolonomicQuantumSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = HolonomicQuantumParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.holonomic_coupling_mev, 1.0);
    assert_eq!(underflow.topological_holonomic_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.gate_synthesis_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_holonomic_loops_factor, 1.0);
    assert_eq!(underflow.gate_loop_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = HolonomicQuantumParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.holonomic_coupling_mev, 35.0);
    assert_eq!(overflow.topological_holonomic_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.gate_synthesis_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_holonomic_loops_factor, 8.0);
    assert_eq!(overflow.gate_loop_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = HolonomicQuantumParams::default();
    assert_eq!(params.holonomic_coupling_mev, 26.0);
    assert_eq!(params.topological_holonomic_gap_mev, 32.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 10.5);
    assert_eq!(params.gate_synthesis_dispatch_speed_m_per_s, 2300.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 10.5);
    assert_eq!(params.synthetic_holonomic_loops_factor, 4.0);
    assert_eq!(params.gate_loop_pitch_um, 9.5);

    let solver = HolonomicQuantumSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.gate_synthesis_fidelity >= 0.9980,
        "Gate synthesis fidelity must be >= 0.9980, got {:.6}",
        metrics.gate_synthesis_fidelity
    );
    assert!(
        metrics.geometric_phase_retention_fraction >= 0.9970,
        "Geometric phase retention fraction must be >= 0.9970, got {:.6}",
        metrics.geometric_phase_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_gate_crosstalk_isolation_db >= 55.0,
        "Inter-gate crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_gate_crosstalk_isolation_db
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
fn test_holonomic_coupling_scaling() {
    let mut p_low = HolonomicQuantumParams::default();
    p_low.holonomic_coupling_mev = 2.0;
    let mut p_high = HolonomicQuantumParams::default();
    p_high.holonomic_coupling_mev = 34.0;

    let solver_low = HolonomicQuantumSolver::new(p_low);
    let solver_high = HolonomicQuantumSolver::new(p_high);

    assert!(
        solver_high.compute_gate_synthesis_fidelity()
            > solver_low.compute_gate_synthesis_fidelity(),
        "Higher holonomic coupling energy should enhance gate synthesis fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher holonomic coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher holonomic coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_holonomic_gap_scaling() {
    let mut p_low = HolonomicQuantumParams::default();
    p_low.topological_holonomic_gap_mev = 3.0;
    let mut p_high = HolonomicQuantumParams::default();
    p_high.topological_holonomic_gap_mev = 44.0;

    let solver_low = HolonomicQuantumSolver::new(p_low);
    let solver_high = HolonomicQuantumSolver::new(p_high);

    assert!(
        solver_high.compute_geometric_phase_retention_fraction()
            > solver_low.compute_geometric_phase_retention_fraction(),
        "Wider topological holonomic gap should improve retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological holonomic gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut p_low = HolonomicQuantumParams::default();
    p_low.acoustic_drive_frequency_ghz = 1.5;
    let mut p_high = HolonomicQuantumParams::default();
    p_high.acoustic_drive_frequency_ghz = 11.5;

    let solver_low = HolonomicQuantumSolver::new(p_low);
    let solver_high = HolonomicQuantumSolver::new(p_high);

    assert!(
        solver_high.compute_gate_synthesis_fidelity()
            > solver_low.compute_gate_synthesis_fidelity(),
        "Higher acoustic drive frequency should enhance gate synthesis fidelity"
    );
    assert!(
        solver_high.compute_inter_gate_crosstalk_isolation_db()
            > solver_low.compute_inter_gate_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_gate_synthesis_dispatch_speed_scaling() {
    let mut p_low = HolonomicQuantumParams::default();
    p_low.gate_synthesis_dispatch_speed_m_per_s = 300.0;
    let mut p_high = HolonomicQuantumParams::default();
    p_high.gate_synthesis_dispatch_speed_m_per_s = 2900.0;

    let solver_low = HolonomicQuantumSolver::new(p_low);
    let solver_high = HolonomicQuantumSolver::new(p_high);

    assert!(
        solver_high.compute_gate_synthesis_fidelity()
            > solver_low.compute_gate_synthesis_fidelity(),
        "Higher dispatch speed should improve gate synthesis fidelity"
    );
    assert!(
        solver_high.compute_geometric_phase_retention_fraction()
            > solver_low.compute_geometric_phase_retention_fraction(),
        "Higher dispatch speed should improve retention fraction"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut p_cold = HolonomicQuantumParams::default();
    p_cold.cryogenic_temperature_mk = 2.0;
    let mut p_warm = HolonomicQuantumParams::default();
    p_warm.cryogenic_temperature_mk = 48.0;

    let solver_cold = HolonomicQuantumSolver::new(p_cold);
    let solver_warm = HolonomicQuantumSolver::new(p_warm);

    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature must strictly suppress dephasing rate"
    );
    assert!(
        solver_cold.compute_gate_synthesis_fidelity()
            > solver_warm.compute_gate_synthesis_fidelity(),
        "Lower cryogenic temperature should improve gate synthesis fidelity"
    );
    assert!(
        solver_cold.compute_geometric_phase_retention_fraction()
            > solver_warm.compute_geometric_phase_retention_fraction(),
        "Lower cryogenic temperature should improve retention fraction"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let mut p_low = HolonomicQuantumParams::default();
    p_low.microwave_probe_power_uw = 1.0;
    let mut p_high = HolonomicQuantumParams::default();
    p_high.microwave_probe_power_uw = 29.0;

    let solver_low = HolonomicQuantumSolver::new(p_low);
    let solver_high = HolonomicQuantumSolver::new(p_high);

    assert!(
        solver_high.compute_gate_synthesis_fidelity()
            > solver_low.compute_gate_synthesis_fidelity(),
        "Higher microwave probe power should improve gate synthesis fidelity"
    );
    assert!(
        solver_high.compute_inter_gate_crosstalk_isolation_db()
            > solver_low.compute_inter_gate_crosstalk_isolation_db(),
        "Higher microwave probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_holonomic_loops_scaling() {
    let mut p_low = HolonomicQuantumParams::default();
    p_low.synthetic_holonomic_loops_factor = 1.5;
    let mut p_high = HolonomicQuantumParams::default();
    p_high.synthetic_holonomic_loops_factor = 7.5;

    let solver_low = HolonomicQuantumSolver::new(p_low);
    let solver_high = HolonomicQuantumSolver::new(p_high);

    assert!(
        solver_high.compute_inter_gate_crosstalk_isolation_db()
            > solver_low.compute_inter_gate_crosstalk_isolation_db(),
        "Higher synthetic holonomic loops factor should improve crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher synthetic holonomic loops factor should increase protection gap"
    );
}

#[test]
fn test_gate_loop_pitch_scaling() {
    let mut p_narrow = HolonomicQuantumParams::default();
    p_narrow.gate_loop_pitch_um = 1.0;
    let mut p_wide = HolonomicQuantumParams::default();
    p_wide.gate_loop_pitch_um = 19.0;

    let solver_narrow = HolonomicQuantumSolver::new(p_narrow);
    let solver_wide = HolonomicQuantumSolver::new(p_wide);

    assert!(
        solver_wide.compute_inter_gate_crosstalk_isolation_db()
            > solver_narrow.compute_inter_gate_crosstalk_isolation_db(),
        "Larger gate loop pitch should enhance inter-gate crosstalk isolation"
    );
    assert!(
        solver_wide.compute_topological_mode_dephasing_rate_hz()
            < solver_narrow.compute_topological_mode_dephasing_rate_hz(),
        "Larger gate loop pitch should suppress dephasing rate"
    );
}
