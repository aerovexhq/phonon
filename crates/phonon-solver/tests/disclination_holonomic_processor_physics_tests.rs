#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! higher-order disclination bound states and chiral holonomic anyon processors.

use phonon_models::disclination_holonomic_processor::DisclinationHolonomicProcessorParams;
use phonon_solver::disclination_holonomic_processor::DisclinationHolonomicProcessorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = DisclinationHolonomicProcessorParams::new(
        0.20,  // below 0.40 rad
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        5.0,   // below 10.0 nm
        0.005, // below 0.01
    );
    assert_eq!(underflow.disclination_frank_angle_rad, 0.40);
    assert_eq!(underflow.higher_order_topological_mass_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.holonomic_shuttling_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_control_power_uw, 0.5);
    assert_eq!(underflow.disclination_core_radius_nm, 10.0);
    assert_eq!(underflow.lattice_hexagonal_strain, 0.01);

    // Test values strictly above physical maximum bounds
    let overflow = DisclinationHolonomicProcessorParams::new(
        3.00,   // above 2.20 rad
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        250.0,  // above 180.0 nm
        0.35,   // above 0.25
    );
    assert_eq!(overflow.disclination_frank_angle_rad, 2.20);
    assert_eq!(overflow.higher_order_topological_mass_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.holonomic_shuttling_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_control_power_uw, 30.0);
    assert_eq!(overflow.disclination_core_radius_nm, 180.0);
    assert_eq!(overflow.lattice_hexagonal_strain, 0.25);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = DisclinationHolonomicProcessorParams::default();
    assert_eq!(params.disclination_frank_angle_rad, 1.0472);
    assert_eq!(params.higher_order_topological_mass_mev, 21.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 6.0);
    assert_eq!(params.holonomic_shuttling_speed_m_per_s, 1400.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_control_power_uw, 5.2);
    assert_eq!(params.disclination_core_radius_nm, 45.0);
    assert_eq!(params.lattice_hexagonal_strain, 0.08);

    let solver = DisclinationHolonomicProcessorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.holonomic_gate_fidelity >= 0.9980,
        "Holonomic gate fidelity must be >= 0.9980, got {:.6}",
        metrics.holonomic_gate_fidelity
    );
    assert!(
        metrics.disclination_state_retention_fraction >= 0.9970,
        "Disclination state retention fraction must be >= 0.9970, got {:.6}",
        metrics.disclination_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_disclination_crosstalk_isolation_db >= 54.0,
        "Inter-disclination crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
        metrics.inter_disclination_crosstalk_isolation_db
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
fn test_disclination_frank_angle_scaling() {
    let mut low_frank = DisclinationHolonomicProcessorParams::default();
    low_frank.disclination_frank_angle_rad = 0.50;
    let mut high_frank = DisclinationHolonomicProcessorParams::default();
    high_frank.disclination_frank_angle_rad = 2.10;

    let low_solver = DisclinationHolonomicProcessorSolver::new(low_frank);
    let high_solver = DisclinationHolonomicProcessorSolver::new(high_frank);

    assert!(
        high_solver.compute_holonomic_gate_fidelity() > low_solver.compute_holonomic_gate_fidelity(),
        "Higher Frank angle defect curvature must enhance holonomic gate fidelity"
    );
    assert!(
        high_solver.compute_disclination_state_retention_fraction()
            > low_solver.compute_disclination_state_retention_fraction(),
        "Higher Frank angle must boost disclination state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher Frank angle must widen topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher Frank angle must suppress dephasing rate"
    );
}

#[test]
fn test_higher_order_topological_mass_scaling() {
    let mut low_mass = DisclinationHolonomicProcessorParams::default();
    low_mass.higher_order_topological_mass_mev = 3.0;
    let mut high_mass = DisclinationHolonomicProcessorParams::default();
    high_mass.higher_order_topological_mass_mev = 42.0;

    let low_solver = DisclinationHolonomicProcessorSolver::new(low_mass);
    let high_solver = DisclinationHolonomicProcessorSolver::new(high_mass);

    assert!(
        high_solver.compute_holonomic_gate_fidelity() > low_solver.compute_holonomic_gate_fidelity(),
        "Higher topological mass must enhance holonomic gate fidelity"
    );
    assert!(
        high_solver.compute_disclination_state_retention_fraction()
            > low_solver.compute_disclination_state_retention_fraction(),
        "Higher topological mass must boost bound state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher topological mass must widen the topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher topological mass must suppress dephasing rate"
    );
}

#[test]
fn test_lattice_hexagonal_strain_scaling() {
    let mut low_strain = DisclinationHolonomicProcessorParams::default();
    low_strain.lattice_hexagonal_strain = 0.02;
    let mut high_strain = DisclinationHolonomicProcessorParams::default();
    high_strain.lattice_hexagonal_strain = 0.23;

    let low_solver = DisclinationHolonomicProcessorSolver::new(low_strain);
    let high_solver = DisclinationHolonomicProcessorSolver::new(high_strain);

    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher hexagonal strain must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_disclination_crosstalk_isolation_db()
            > low_solver.compute_inter_disclination_crosstalk_isolation_db(),
        "Higher hexagonal strain must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_holonomic_gate_fidelity() > low_solver.compute_holonomic_gate_fidelity(),
        "Higher hexagonal strain must increase gate fidelity"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher hexagonal strain must decrease dephasing rate"
    );
}

#[test]
fn test_disclination_core_radius_scaling() {
    let mut low_core = DisclinationHolonomicProcessorParams::default();
    low_core.disclination_core_radius_nm = 15.0;
    let mut high_core = DisclinationHolonomicProcessorParams::default();
    high_core.disclination_core_radius_nm = 165.0;

    let low_solver = DisclinationHolonomicProcessorSolver::new(low_core);
    let high_solver = DisclinationHolonomicProcessorSolver::new(high_core);

    assert!(
        high_solver.compute_inter_disclination_crosstalk_isolation_db()
            > low_solver.compute_inter_disclination_crosstalk_isolation_db(),
        "Larger core radius must boost crosstalk isolation"
    );
    assert!(
        high_solver.compute_disclination_state_retention_fraction()
            > low_solver.compute_disclination_state_retention_fraction(),
        "Larger core radius must improve disclination state retention"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger core radius must reduce topological mode dephasing rate"
    );
}

#[test]
fn test_holonomic_shuttling_speed_scaling() {
    let mut low_speed = DisclinationHolonomicProcessorParams::default();
    low_speed.holonomic_shuttling_speed_m_per_s = 300.0;
    let mut high_speed = DisclinationHolonomicProcessorParams::default();
    high_speed.holonomic_shuttling_speed_m_per_s = 2800.0;

    let low_solver = DisclinationHolonomicProcessorSolver::new(low_speed);
    let high_solver = DisclinationHolonomicProcessorSolver::new(high_speed);

    assert!(
        high_solver.compute_holonomic_gate_fidelity() > low_solver.compute_holonomic_gate_fidelity(),
        "Higher shuttling speed must increase holonomic gate fidelity"
    );
    assert!(
        high_solver.compute_disclination_state_retention_fraction()
            > low_solver.compute_disclination_state_retention_fraction(),
        "Higher shuttling speed must improve bound state retention"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher shuttling speed must suppress dephasing by shortening transit exposure"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_f = DisclinationHolonomicProcessorParams::default();
    low_f.acoustic_drive_frequency_ghz = 1.5;
    let mut high_f = DisclinationHolonomicProcessorParams::default();
    high_f.acoustic_drive_frequency_ghz = 11.0;

    let low_solver = DisclinationHolonomicProcessorSolver::new(low_f);
    let high_solver = DisclinationHolonomicProcessorSolver::new(high_f);

    assert!(
        high_solver.compute_holonomic_gate_fidelity() > low_solver.compute_holonomic_gate_fidelity(),
        "Higher acoustic drive frequency must increase holonomic gate fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher acoustic drive frequency must enhance topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic drive frequency must reduce dephasing"
    );
}

#[test]
fn test_microwave_control_power_scaling() {
    let mut low_p = DisclinationHolonomicProcessorParams::default();
    low_p.microwave_control_power_uw = 1.0;
    let mut high_p = DisclinationHolonomicProcessorParams::default();
    high_p.microwave_control_power_uw = 28.0;

    let low_solver = DisclinationHolonomicProcessorSolver::new(low_p);
    let high_solver = DisclinationHolonomicProcessorSolver::new(high_p);

    assert!(
        high_solver.compute_holonomic_gate_fidelity() > low_solver.compute_holonomic_gate_fidelity(),
        "Higher microwave control power must increase holonomic gate fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher microwave control power must improve topological gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher microwave control power must reduce dephasing"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_t = DisclinationHolonomicProcessorParams::default();
    low_t.cryogenic_temperature_mk = 2.0;
    let mut high_t = DisclinationHolonomicProcessorParams::default();
    high_t.cryogenic_temperature_mk = 48.0;

    let low_solver = DisclinationHolonomicProcessorSolver::new(low_t);
    let high_solver = DisclinationHolonomicProcessorSolver::new(high_t);

    assert!(
        low_solver.compute_holonomic_gate_fidelity() > high_solver.compute_holonomic_gate_fidelity(),
        "Lower temperature must enhance gate fidelity due to reduced thermal noise"
    );
    assert!(
        low_solver.compute_disclination_state_retention_fraction()
            > high_solver.compute_disclination_state_retention_fraction(),
        "Lower temperature must protect disclination state retention"
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
