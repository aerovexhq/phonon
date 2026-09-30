#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological Pfaffian superconducting qubit resonators and parity-protected anyonic gate engines.

use phonon_models::pfaffian_quantum_resonator::PfaffianQuantumResonatorParams;
use phonon_solver::pfaffian_quantum_resonator::PfaffianQuantumResonatorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = PfaffianQuantumResonatorParams::new(
        1.0,  // below 2.0 meV
        0.05, // below 0.1 GHz
        0.5,  // below 1.0 GHz
        0.2,  // below 0.5 %
        0.01, // below 0.05 Phi_0
        0.5,  // below 1.0 mK
        0.2,  // below 0.5 uW
        5.0,  // below 10.0 k
    );
    assert_eq!(underflow.pfaffian_pairing_gap_mev, 2.0);
    assert_eq!(underflow.superconducting_charging_energy_ghz, 0.1);
    assert_eq!(underflow.acoustic_resonator_frequency_ghz, 1.0);
    assert_eq!(underflow.piezoelectric_coupling_strength_percent, 0.5);
    assert_eq!(underflow.magnetic_flux_bias_phi0, 0.05);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_drive_power_uw, 0.5);
    assert_eq!(underflow.resonator_quality_factor_k, 10.0);

    // Test values strictly above physical maximum bounds
    let overflow = PfaffianQuantumResonatorParams::new(
        60.0,  // above 45.0 meV
        3.5,   // above 2.5 GHz
        15.0,  // above 12.0 GHz
        20.0,  // above 15.0 %
        1.25,  // above 0.95 Phi_0
        75.0,  // above 50.0 mK
        45.0,  // above 30.0 uW
        750.0, // above 500.0 k
    );
    assert_eq!(overflow.pfaffian_pairing_gap_mev, 45.0);
    assert_eq!(overflow.superconducting_charging_energy_ghz, 2.5);
    assert_eq!(overflow.acoustic_resonator_frequency_ghz, 12.0);
    assert_eq!(overflow.piezoelectric_coupling_strength_percent, 15.0);
    assert_eq!(overflow.magnetic_flux_bias_phi0, 0.95);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_drive_power_uw, 30.0);
    assert_eq!(overflow.resonator_quality_factor_k, 500.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = PfaffianQuantumResonatorParams::default();
    assert_eq!(params.pfaffian_pairing_gap_mev, 21.5);
    assert_eq!(params.superconducting_charging_energy_ghz, 0.85);
    assert_eq!(params.acoustic_resonator_frequency_ghz, 5.5);
    assert_eq!(params.piezoelectric_coupling_strength_percent, 6.2);
    assert_eq!(params.magnetic_flux_bias_phi0, 0.45);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_drive_power_uw, 5.8);
    assert_eq!(params.resonator_quality_factor_k, 180.0);

    let solver = PfaffianQuantumResonatorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.gate_fidelity >= 0.9980,
        "Gate fidelity must be >= 0.9980, got {:.6}",
        metrics.gate_fidelity
    );
    assert!(
        metrics.pfaffian_state_retention_fraction >= 0.9970,
        "Pfaffian state retention fraction must be >= 0.9970, got {:.6}",
        metrics.pfaffian_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_resonator_crosstalk_isolation_db >= 54.0,
        "Inter-resonator crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
        metrics.inter_resonator_crosstalk_isolation_db
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
fn test_pfaffian_pairing_gap_scaling() {
    let mut low_gap = PfaffianQuantumResonatorParams::default();
    low_gap.pfaffian_pairing_gap_mev = 3.0;
    let mut high_gap = PfaffianQuantumResonatorParams::default();
    high_gap.pfaffian_pairing_gap_mev = 42.0;

    let low_solver = PfaffianQuantumResonatorSolver::new(low_gap);
    let high_solver = PfaffianQuantumResonatorSolver::new(high_gap);

    assert!(
        high_solver.compute_gate_fidelity() > low_solver.compute_gate_fidelity(),
        "Larger Pfaffian pairing gap must enhance gate fidelity"
    );
    assert!(
        high_solver.compute_pfaffian_state_retention_fraction()
            > low_solver.compute_pfaffian_state_retention_fraction(),
        "Larger Pfaffian pairing gap must boost Pfaffian state retention fraction"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger Pfaffian pairing gap must expand topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger Pfaffian pairing gap must suppress dephasing rate"
    );
}

#[test]
fn test_superconducting_charging_energy_scaling() {
    let mut low_ec = PfaffianQuantumResonatorParams::default();
    low_ec.superconducting_charging_energy_ghz = 0.2;
    let mut high_ec = PfaffianQuantumResonatorParams::default();
    high_ec.superconducting_charging_energy_ghz = 2.2;

    let low_solver = PfaffianQuantumResonatorSolver::new(low_ec);
    let high_solver = PfaffianQuantumResonatorSolver::new(high_ec);

    assert!(
        high_solver.compute_gate_fidelity() > low_solver.compute_gate_fidelity(),
        "Higher charging energy must enhance gate fidelity"
    );
    assert!(
        high_solver.compute_pfaffian_state_retention_fraction()
            > low_solver.compute_pfaffian_state_retention_fraction(),
        "Higher charging energy must boost state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher charging energy must widen protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher charging energy must suppress dephasing rate"
    );
}

#[test]
fn test_acoustic_resonator_frequency_scaling() {
    let mut low_f = PfaffianQuantumResonatorParams::default();
    low_f.acoustic_resonator_frequency_ghz = 1.5;
    let mut high_f = PfaffianQuantumResonatorParams::default();
    high_f.acoustic_resonator_frequency_ghz = 11.0;

    let low_solver = PfaffianQuantumResonatorSolver::new(low_f);
    let high_solver = PfaffianQuantumResonatorSolver::new(high_f);

    assert!(
        high_solver.compute_gate_fidelity() > low_solver.compute_gate_fidelity(),
        "Higher acoustic resonator frequency must increase gate fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher acoustic resonator frequency must widen topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic resonator frequency must reduce dephasing"
    );
}

#[test]
fn test_piezoelectric_coupling_strength_scaling() {
    let mut low_piezo = PfaffianQuantumResonatorParams::default();
    low_piezo.piezoelectric_coupling_strength_percent = 1.0;
    let mut high_piezo = PfaffianQuantumResonatorParams::default();
    high_piezo.piezoelectric_coupling_strength_percent = 14.0;

    let low_solver = PfaffianQuantumResonatorSolver::new(low_piezo);
    let high_solver = PfaffianQuantumResonatorSolver::new(high_piezo);

    assert!(
        high_solver.compute_gate_fidelity() > low_solver.compute_gate_fidelity(),
        "Stronger piezoelectric coupling must enhance gate fidelity"
    );
    assert!(
        high_solver.compute_pfaffian_state_retention_fraction()
            > low_solver.compute_pfaffian_state_retention_fraction(),
        "Stronger piezoelectric coupling must boost retention fraction"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Stronger piezoelectric coupling must widen topological gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Stronger piezoelectric coupling must suppress dephasing"
    );
}

#[test]
fn test_magnetic_flux_bias_scaling() {
    let mut low_flux = PfaffianQuantumResonatorParams::default();
    low_flux.magnetic_flux_bias_phi0 = 0.10;
    let mut high_flux = PfaffianQuantumResonatorParams::default();
    high_flux.magnetic_flux_bias_phi0 = 0.90;

    let low_solver = PfaffianQuantumResonatorSolver::new(low_flux);
    let high_solver = PfaffianQuantumResonatorSolver::new(high_flux);

    assert!(
        high_solver.compute_gate_fidelity() > low_solver.compute_gate_fidelity(),
        "Optimal magnetic flux bias must enhance gate fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Optimal magnetic flux bias must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_resonator_crosstalk_isolation_db()
            > low_solver.compute_inter_resonator_crosstalk_isolation_db(),
        "Optimal magnetic flux bias must increase crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Optimal magnetic flux bias must suppress dephasing rate"
    );
}

#[test]
fn test_microwave_drive_power_scaling() {
    let mut low_p = PfaffianQuantumResonatorParams::default();
    low_p.microwave_drive_power_uw = 1.0;
    let mut high_p = PfaffianQuantumResonatorParams::default();
    high_p.microwave_drive_power_uw = 28.0;

    let low_solver = PfaffianQuantumResonatorSolver::new(low_p);
    let high_solver = PfaffianQuantumResonatorSolver::new(high_p);

    assert!(
        high_solver.compute_gate_fidelity() > low_solver.compute_gate_fidelity(),
        "Higher microwave drive power must increase gate fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher microwave drive power must improve topological gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher microwave drive power must reduce dephasing"
    );
}

#[test]
fn test_resonator_quality_factor_scaling() {
    let mut low_q = PfaffianQuantumResonatorParams::default();
    low_q.resonator_quality_factor_k = 20.0;
    let mut high_q = PfaffianQuantumResonatorParams::default();
    high_q.resonator_quality_factor_k = 450.0;

    let low_solver = PfaffianQuantumResonatorSolver::new(low_q);
    let high_solver = PfaffianQuantumResonatorSolver::new(high_q);

    assert!(
        high_solver.compute_pfaffian_state_retention_fraction()
            > low_solver.compute_pfaffian_state_retention_fraction(),
        "Higher quality factor must improve state retention fraction"
    );
    assert!(
        high_solver.compute_inter_resonator_crosstalk_isolation_db()
            > low_solver.compute_inter_resonator_crosstalk_isolation_db(),
        "Higher quality factor must significantly increase crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher quality factor must suppress dephasing rate"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_t = PfaffianQuantumResonatorParams::default();
    low_t.cryogenic_temperature_mk = 2.0;
    let mut high_t = PfaffianQuantumResonatorParams::default();
    high_t.cryogenic_temperature_mk = 48.0;

    let low_solver = PfaffianQuantumResonatorSolver::new(low_t);
    let high_solver = PfaffianQuantumResonatorSolver::new(high_t);

    assert!(
        low_solver.compute_gate_fidelity() > high_solver.compute_gate_fidelity(),
        "Lower temperature must enhance gate fidelity due to reduced thermal quasiparticles"
    );
    assert!(
        low_solver.compute_pfaffian_state_retention_fraction()
            > high_solver.compute_pfaffian_state_retention_fraction(),
        "Lower temperature must protect Pfaffian state retention fraction"
    );
    assert!(
        low_solver.compute_topological_protection_gap_mhz()
            > high_solver.compute_topological_protection_gap_mhz(),
        "Lower temperature must preserve a wider topological protection gap"
    );
    assert!(
        low_solver.compute_topological_mode_dephasing_rate_hz()
            < high_solver.compute_topological_mode_dephasing_rate_hz(),
        "Lower temperature must suppress thermal dephasing rate"
    );
}
