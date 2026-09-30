#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic topological
//! non-Abelian fracton gauge-matter ensembles and chiral quadrupole entanglement routers.

use phonon_models::fracton_quadrupole_router::FractonQuadrupoleRouterParams;
use phonon_solver::fracton_quadrupole_router::FractonQuadrupoleRouterSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FractonQuadrupoleRouterParams::new(
        1.0,  // below 2.0 meV
        0.05, // below 0.10
        0.5,  // below 1.0 GHz
        0.01, // below 0.05
        0.5,  // below 1.0 mK
        0.1,  // below 0.5 uW
        0.2,  // below 0.5 um
        5.0,  // below 10.0 GPa
    );
    assert_eq!(underflow.higher_rank_tensor_coupling_mev, 2.0);
    assert_eq!(underflow.quadrupole_polarization_intensity, 0.10);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.sub_dimensional_mobility_fraction, 0.05);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_routing_power_uw, 0.5);
    assert_eq!(underflow.router_separation_um, 0.5);
    assert_eq!(underflow.acoustic_shear_modulus_gpa, 10.0);

    // Test values strictly above physical maximum bounds
    let overflow = FractonQuadrupoleRouterParams::new(
        55.0,  // above 45.0 meV
        1.05,  // above 0.95
        15.0,  // above 12.0 GHz
        0.95,  // above 0.85
        60.0,  // above 50.0 mK
        40.0,  // above 30.0 uW
        20.0,  // above 15.0 um
        150.0, // above 120.0 GPa
    );
    assert_eq!(overflow.higher_rank_tensor_coupling_mev, 45.0);
    assert_eq!(overflow.quadrupole_polarization_intensity, 0.95);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.sub_dimensional_mobility_fraction, 0.85);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_routing_power_uw, 30.0);
    assert_eq!(overflow.router_separation_um, 15.0);
    assert_eq!(overflow.acoustic_shear_modulus_gpa, 120.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FractonQuadrupoleRouterParams::default();
    assert_eq!(params.higher_rank_tensor_coupling_mev, 22.0);
    assert_eq!(params.quadrupole_polarization_intensity, 0.65);
    assert_eq!(params.acoustic_drive_frequency_ghz, 5.8);
    assert_eq!(params.sub_dimensional_mobility_fraction, 0.35);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_routing_power_uw, 5.0);
    assert_eq!(params.router_separation_um, 3.8);
    assert_eq!(params.acoustic_shear_modulus_gpa, 45.0);

    let solver = FractonQuadrupoleRouterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.routing_fidelity >= 0.9980,
        "Routing fidelity must be >= 0.9980, got {:.6}",
        metrics.routing_fidelity
    );
    assert!(
        metrics.fracton_state_retention_fraction >= 0.9970,
        "Fracton state retention fraction must be >= 0.9970, got {:.6}",
        metrics.fracton_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_router_crosstalk_isolation_db >= 55.0,
        "Inter-router crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_router_crosstalk_isolation_db
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
fn test_higher_rank_tensor_coupling_scaling() {
    let mut low_tensor = FractonQuadrupoleRouterParams::default();
    low_tensor.higher_rank_tensor_coupling_mev = 3.0;
    let mut high_tensor = FractonQuadrupoleRouterParams::default();
    high_tensor.higher_rank_tensor_coupling_mev = 42.0;

    let low_solver = FractonQuadrupoleRouterSolver::new(low_tensor);
    let high_solver = FractonQuadrupoleRouterSolver::new(high_tensor);

    assert!(
        high_solver.compute_routing_fidelity() > low_solver.compute_routing_fidelity(),
        "Higher tensor coupling must enhance routing fidelity"
    );
    assert!(
        high_solver.compute_fracton_state_retention_fraction()
            > low_solver.compute_fracton_state_retention_fraction(),
        "Higher tensor coupling must boost fracton state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher tensor coupling must widen the topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher tensor coupling must suppress dephasing rate"
    );
}

#[test]
fn test_quadrupole_polarization_scaling() {
    let mut low_quad = FractonQuadrupoleRouterParams::default();
    low_quad.quadrupole_polarization_intensity = 0.15;
    let mut high_quad = FractonQuadrupoleRouterParams::default();
    high_quad.quadrupole_polarization_intensity = 0.90;

    let low_solver = FractonQuadrupoleRouterSolver::new(low_quad);
    let high_solver = FractonQuadrupoleRouterSolver::new(high_quad);

    assert!(
        high_solver.compute_routing_fidelity() > low_solver.compute_routing_fidelity(),
        "Higher quadrupole polarization must enhance routing fidelity"
    );
    assert!(
        high_solver.compute_fracton_state_retention_fraction()
            > low_solver.compute_fracton_state_retention_fraction(),
        "Higher quadrupole polarization must boost fracton retention fraction"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher quadrupole polarization must expand the topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher quadrupole polarization must suppress dephasing rate"
    );
}

#[test]
fn test_acoustic_shear_modulus_scaling() {
    let mut low_shear = FractonQuadrupoleRouterParams::default();
    low_shear.acoustic_shear_modulus_gpa = 15.0;
    let mut high_shear = FractonQuadrupoleRouterParams::default();
    high_shear.acoustic_shear_modulus_gpa = 110.0;

    let low_solver = FractonQuadrupoleRouterSolver::new(low_shear);
    let high_solver = FractonQuadrupoleRouterSolver::new(high_shear);

    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher acoustic shear modulus must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_router_crosstalk_isolation_db()
            > low_solver.compute_inter_router_crosstalk_isolation_db(),
        "Higher acoustic shear modulus must improve inter-router crosstalk isolation"
    );
    assert!(
        high_solver.compute_routing_fidelity() > low_solver.compute_routing_fidelity(),
        "Higher acoustic shear modulus must increase routing fidelity"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic shear modulus must decrease dephasing rate"
    );
}

#[test]
fn test_router_separation_scaling() {
    let mut low_sep = FractonQuadrupoleRouterParams::default();
    low_sep.router_separation_um = 1.0;
    let mut high_sep = FractonQuadrupoleRouterParams::default();
    high_sep.router_separation_um = 14.0;

    let low_solver = FractonQuadrupoleRouterSolver::new(low_sep);
    let high_solver = FractonQuadrupoleRouterSolver::new(high_sep);

    assert!(
        high_solver.compute_inter_router_crosstalk_isolation_db()
            > low_solver.compute_inter_router_crosstalk_isolation_db(),
        "Larger router separation must significantly boost crosstalk isolation"
    );
    assert!(
        high_solver.compute_fracton_state_retention_fraction()
            > low_solver.compute_fracton_state_retention_fraction(),
        "Larger router separation must improve fracton retention"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger router separation must reduce topological dephasing rate"
    );
}

#[test]
fn test_sub_dimensional_mobility_scaling() {
    let mut low_mob = FractonQuadrupoleRouterParams::default();
    low_mob.sub_dimensional_mobility_fraction = 0.10;
    let mut high_mob = FractonQuadrupoleRouterParams::default();
    high_mob.sub_dimensional_mobility_fraction = 0.80;

    let low_solver = FractonQuadrupoleRouterSolver::new(low_mob);
    let high_solver = FractonQuadrupoleRouterSolver::new(high_mob);

    assert!(
        high_solver.compute_routing_fidelity() > low_solver.compute_routing_fidelity(),
        "Higher sub-dimensional mobility fraction must improve routing fidelity"
    );
    assert!(
        high_solver.compute_fracton_state_retention_fraction()
            > low_solver.compute_fracton_state_retention_fraction(),
        "Higher sub-dimensional mobility fraction must enhance state retention"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher sub-dimensional mobility fraction must suppress dephasing"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_f = FractonQuadrupoleRouterParams::default();
    low_f.acoustic_drive_frequency_ghz = 1.5;
    let mut high_f = FractonQuadrupoleRouterParams::default();
    high_f.acoustic_drive_frequency_ghz = 11.0;

    let low_solver = FractonQuadrupoleRouterSolver::new(low_f);
    let high_solver = FractonQuadrupoleRouterSolver::new(high_f);

    assert!(
        high_solver.compute_routing_fidelity() > low_solver.compute_routing_fidelity(),
        "Higher acoustic drive frequency must increase routing fidelity"
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
fn test_microwave_routing_power_scaling() {
    let mut low_p = FractonQuadrupoleRouterParams::default();
    low_p.microwave_routing_power_uw = 1.0;
    let mut high_p = FractonQuadrupoleRouterParams::default();
    high_p.microwave_routing_power_uw = 28.0;

    let low_solver = FractonQuadrupoleRouterSolver::new(low_p);
    let high_solver = FractonQuadrupoleRouterSolver::new(high_p);

    assert!(
        high_solver.compute_routing_fidelity() > low_solver.compute_routing_fidelity(),
        "Higher microwave routing power must increase routing fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher microwave routing power must improve topological gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher microwave routing power must reduce dephasing"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_t = FractonQuadrupoleRouterParams::default();
    low_t.cryogenic_temperature_mk = 2.0;
    let mut high_t = FractonQuadrupoleRouterParams::default();
    high_t.cryogenic_temperature_mk = 48.0;

    let low_solver = FractonQuadrupoleRouterSolver::new(low_t);
    let high_solver = FractonQuadrupoleRouterSolver::new(high_t);

    assert!(
        low_solver.compute_routing_fidelity() > high_solver.compute_routing_fidelity(),
        "Lower temperature must enhance routing fidelity due to reduced thermal noise"
    );
    assert!(
        low_solver.compute_fracton_state_retention_fraction()
            > high_solver.compute_fracton_state_retention_fraction(),
        "Lower temperature must protect fracton state retention"
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
