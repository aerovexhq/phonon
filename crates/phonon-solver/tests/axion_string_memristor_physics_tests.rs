#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological axion string-vortex entanglement networks and chiral gauge-symmetric
//! quantum memristors.

use phonon_models::axion_string_memristor::AxionStringMemristorParams;
use phonon_solver::axion_string_memristor::AxionStringMemristorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = AxionStringMemristorParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        0.05,  // below 0.1 rad
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.05,  // below 0.1 um^-2
    );
    assert_eq!(underflow.axion_coupling_constant_mev, 1.0);
    assert_eq!(underflow.superconducting_vortex_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.dynamical_axion_angle_rad, 0.1);
    assert_eq!(underflow.strain_modulation_velocity_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_write_power_uw, 0.5);
    assert_eq!(underflow.string_network_density_um2, 0.1);

    // Test values strictly above physical maximum bounds
    let overflow = AxionStringMemristorParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        4.0,    // above 3.14 rad
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        15.0,   // above 10.0 um^-2
    );
    assert_eq!(overflow.axion_coupling_constant_mev, 35.0);
    assert_eq!(overflow.superconducting_vortex_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.dynamical_axion_angle_rad, 3.14);
    assert_eq!(overflow.strain_modulation_velocity_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_write_power_uw, 30.0);
    assert_eq!(overflow.string_network_density_um2, 10.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AxionStringMemristorParams::default();
    assert_eq!(params.axion_coupling_constant_mev, 16.8);
    assert_eq!(params.superconducting_vortex_gap_mev, 22.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 5.7);
    assert_eq!(params.dynamical_axion_angle_rad, 1.57);
    assert_eq!(params.strain_modulation_velocity_m_per_s, 1450.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_write_power_uw, 6.0);
    assert_eq!(params.string_network_density_um2, 3.2);

    let solver = AxionStringMemristorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.memristive_retention_fidelity >= 0.9980,
        "Memristive retention fidelity must be >= 0.9980, got {:.6}",
        metrics.memristive_retention_fidelity
    );
    assert!(
        metrics.string_vortex_state_retention_fraction >= 0.9970,
        "String-vortex state retention fraction must be >= 0.9970, got {:.6}",
        metrics.string_vortex_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_string_crosstalk_isolation_db >= 54.0,
        "Inter-string crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
        metrics.inter_string_crosstalk_isolation_db
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
fn test_axion_coupling_scaling() {
    let mut low_axion = AxionStringMemristorParams::default();
    low_axion.axion_coupling_constant_mev = 2.0;
    let mut high_axion = AxionStringMemristorParams::default();
    high_axion.axion_coupling_constant_mev = 34.0;

    let low_solver = AxionStringMemristorSolver::new(low_axion);
    let high_solver = AxionStringMemristorSolver::new(high_axion);

    assert!(
        high_solver.compute_memristive_retention_fidelity()
            > low_solver.compute_memristive_retention_fidelity(),
        "Larger axion coupling constant must enhance memristive retention fidelity"
    );
    assert!(
        high_solver.compute_string_vortex_state_retention_fraction()
            > low_solver.compute_string_vortex_state_retention_fraction(),
        "Larger axion coupling constant must enhance string-vortex state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger axion coupling constant must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_string_crosstalk_isolation_db()
            > low_solver.compute_inter_string_crosstalk_isolation_db(),
        "Larger axion coupling constant must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger axion coupling constant must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_superconducting_vortex_gap_scaling() {
    let mut low_vortex = AxionStringMemristorParams::default();
    low_vortex.superconducting_vortex_gap_mev = 3.0;
    let mut high_vortex = AxionStringMemristorParams::default();
    high_vortex.superconducting_vortex_gap_mev = 42.0;

    let low_solver = AxionStringMemristorSolver::new(low_vortex);
    let high_solver = AxionStringMemristorSolver::new(high_vortex);

    assert!(
        high_solver.compute_memristive_retention_fidelity()
            > low_solver.compute_memristive_retention_fidelity(),
        "Larger superconducting vortex gap must enhance memristive retention fidelity"
    );
    assert!(
        high_solver.compute_string_vortex_state_retention_fraction()
            > low_solver.compute_string_vortex_state_retention_fraction(),
        "Larger superconducting vortex gap must enhance string-vortex retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger superconducting vortex gap must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_string_crosstalk_isolation_db()
            > low_solver.compute_inter_string_crosstalk_isolation_db(),
        "Larger superconducting vortex gap must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger superconducting vortex gap must suppress dephasing rate"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_freq = AxionStringMemristorParams::default();
    low_freq.acoustic_drive_frequency_ghz = 1.5;
    let mut high_freq = AxionStringMemristorParams::default();
    high_freq.acoustic_drive_frequency_ghz = 11.5;

    let low_solver = AxionStringMemristorSolver::new(low_freq);
    let high_solver = AxionStringMemristorSolver::new(high_freq);

    assert!(
        high_solver.compute_memristive_retention_fidelity()
            > low_solver.compute_memristive_retention_fidelity(),
        "Higher acoustic drive frequency must enhance retention fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher acoustic drive frequency must expand topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic drive frequency must suppress dephasing rate"
    );
}

#[test]
fn test_dynamical_axion_angle_scaling() {
    let mut low_theta = AxionStringMemristorParams::default();
    low_theta.dynamical_axion_angle_rad = 0.2;
    let mut high_theta = AxionStringMemristorParams::default();
    high_theta.dynamical_axion_angle_rad = 3.0;

    let low_solver = AxionStringMemristorSolver::new(low_theta);
    let high_solver = AxionStringMemristorSolver::new(high_theta);

    assert!(
        high_solver.compute_memristive_retention_fidelity()
            > low_solver.compute_memristive_retention_fidelity(),
        "Higher dynamical axion angle must enhance retention fidelity"
    );
    assert!(
        high_solver.compute_string_vortex_state_retention_fraction()
            > low_solver.compute_string_vortex_state_retention_fraction(),
        "Higher dynamical axion angle must boost state retention fraction"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher dynamical axion angle must expand topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher dynamical axion angle must suppress dephasing rate"
    );
}

#[test]
fn test_strain_modulation_velocity_scaling() {
    let mut low_vel = AxionStringMemristorParams::default();
    low_vel.strain_modulation_velocity_m_per_s = 300.0;
    let mut high_vel = AxionStringMemristorParams::default();
    high_vel.strain_modulation_velocity_m_per_s = 2800.0;

    let low_solver = AxionStringMemristorSolver::new(low_vel);
    let high_solver = AxionStringMemristorSolver::new(high_vel);

    assert!(
        high_solver.compute_memristive_retention_fidelity()
            > low_solver.compute_memristive_retention_fidelity(),
        "Higher strain modulation velocity must enhance retention fidelity"
    );
    assert!(
        high_solver.compute_string_vortex_state_retention_fraction()
            > low_solver.compute_string_vortex_state_retention_fraction(),
        "Higher strain velocity must boost state retention"
    );
    assert!(
        high_solver.compute_inter_string_crosstalk_isolation_db()
            > low_solver.compute_inter_string_crosstalk_isolation_db(),
        "Higher strain velocity must enhance crosstalk isolation"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_temp = AxionStringMemristorParams::default();
    low_temp.cryogenic_temperature_mk = 2.0;
    let mut high_temp = AxionStringMemristorParams::default();
    high_temp.cryogenic_temperature_mk = 48.0;

    let low_solver = AxionStringMemristorSolver::new(low_temp);
    let high_solver = AxionStringMemristorSolver::new(high_temp);

    assert!(
        low_solver.compute_memristive_retention_fidelity()
            > high_solver.compute_memristive_retention_fidelity(),
        "Lower temperature must enhance memristive retention fidelity"
    );
    assert!(
        low_solver.compute_string_vortex_state_retention_fraction()
            > high_solver.compute_string_vortex_state_retention_fraction(),
        "Lower temperature must enhance string-vortex state retention fraction"
    );
    assert!(
        low_solver.compute_topological_protection_gap_mhz()
            > high_solver.compute_topological_protection_gap_mhz(),
        "Lower temperature must expand topological protection gap"
    );
    assert!(
        low_solver.compute_inter_string_crosstalk_isolation_db()
            > high_solver.compute_inter_string_crosstalk_isolation_db(),
        "Lower temperature must improve crosstalk isolation"
    );
    assert!(
        low_solver.compute_topological_mode_dephasing_rate_hz()
            < high_solver.compute_topological_mode_dephasing_rate_hz(),
        "Lower temperature must suppress dephasing rate"
    );
}

#[test]
fn test_microwave_write_power_scaling() {
    let mut low_pow = AxionStringMemristorParams::default();
    low_pow.microwave_write_power_uw = 1.0;
    let mut high_pow = AxionStringMemristorParams::default();
    high_pow.microwave_write_power_uw = 28.0;

    let low_solver = AxionStringMemristorSolver::new(low_pow);
    let high_solver = AxionStringMemristorSolver::new(high_pow);

    assert!(
        high_solver.compute_memristive_retention_fidelity()
            > low_solver.compute_memristive_retention_fidelity(),
        "Higher microwave write power must enhance retention fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher microwave write power must expand topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher microwave write power must suppress dephasing rate"
    );
}

#[test]
fn test_string_network_density_scaling() {
    let mut low_dens = AxionStringMemristorParams::default();
    low_dens.string_network_density_um2 = 0.5;
    let mut high_dens = AxionStringMemristorParams::default();
    high_dens.string_network_density_um2 = 9.5;

    let low_solver = AxionStringMemristorSolver::new(low_dens);
    let high_solver = AxionStringMemristorSolver::new(high_dens);

    assert!(
        high_solver.compute_memristive_retention_fidelity()
            > low_solver.compute_memristive_retention_fidelity(),
        "Higher string network density must enhance retention fidelity"
    );
    assert!(
        high_solver.compute_string_vortex_state_retention_fraction()
            > low_solver.compute_string_vortex_state_retention_fraction(),
        "Higher string network density must boost string-vortex state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher string network density must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_string_crosstalk_isolation_db()
            > low_solver.compute_inter_string_crosstalk_isolation_db(),
        "Higher string network density must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher string network density must suppress dephasing rate"
    );
}
