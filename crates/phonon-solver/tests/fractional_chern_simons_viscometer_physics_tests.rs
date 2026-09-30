#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic chiral
//! fractional Chern-Simons hydrodynamics and anyonic holographic edge viscometers.

use phonon_models::fractional_chern_simons_viscometer::FractionalChernSimonsViscometerParams;
use phonon_solver::fractional_chern_simons_viscometer::FractionalChernSimonsViscometerSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FractionalChernSimonsViscometerParams::new(
        0.10, // below 0.20
        1.0,  // below 2.0 T
        0.05, // below 0.10
        0.5,  // below 1.0 GHz
        0.5,  // below 1.0 mK
        1.0,  // below 2.0 um
        10.0, // below 20.0 nm
        0.02, // below 0.05
    );
    assert_eq!(underflow.fractional_filling_factor_nu, 0.20);
    assert_eq!(underflow.magnetic_field_tesla, 2.0);
    assert_eq!(underflow.piezoelectric_stress_coupling_coefficient, 0.10);
    assert_eq!(underflow.acoustic_shear_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.viscometer_channel_length_um, 2.0);
    assert_eq!(underflow.edge_channel_width_nm, 20.0);
    assert_eq!(underflow.electron_effective_mass_ratio, 0.05);

    // Test values strictly above physical maximum bounds
    let overflow = FractionalChernSimonsViscometerParams::new(
        1.50,  // above 1.00
        25.0,  // above 16.0 T
        1.20,  // above 0.95
        20.0,  // above 15.0 GHz
        80.0,  // above 50.0 mK
        50.0,  // above 30.0 um
        300.0, // above 200.0 nm
        0.80,  // above 0.50
    );
    assert_eq!(overflow.fractional_filling_factor_nu, 1.00);
    assert_eq!(overflow.magnetic_field_tesla, 16.0);
    assert_eq!(overflow.piezoelectric_stress_coupling_coefficient, 0.95);
    assert_eq!(overflow.acoustic_shear_frequency_ghz, 15.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.viscometer_channel_length_um, 30.0);
    assert_eq!(overflow.edge_channel_width_nm, 200.0);
    assert_eq!(overflow.electron_effective_mass_ratio, 0.50);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FractionalChernSimonsViscometerParams::default();
    assert_eq!(params.fractional_filling_factor_nu, 0.3333333333333333);
    assert_eq!(params.magnetic_field_tesla, 9.5);
    assert_eq!(params.piezoelectric_stress_coupling_coefficient, 0.68);
    assert_eq!(params.acoustic_shear_frequency_ghz, 4.2);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.viscometer_channel_length_um, 8.5);
    assert_eq!(params.edge_channel_width_nm, 65.0);
    assert_eq!(params.electron_effective_mass_ratio, 0.067);

    let solver = FractionalChernSimonsViscometerSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.hall_viscosity_measurement_fidelity >= 0.9980,
        "Hall viscosity measurement fidelity must be >= 0.9980, got {:.6}",
        metrics.hall_viscosity_measurement_fidelity
    );
    assert!(
        metrics.edge_to_bulk_acoustic_isolation_db >= 55.0,
        "Edge-to-bulk acoustic isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.edge_to_bulk_acoustic_isolation_db
    );
    assert!(
        metrics.edge_mode_velocity_stability_fraction >= 0.9970,
        "Edge mode velocity stability fraction must be >= 0.9970, got {:.6}",
        metrics.edge_mode_velocity_stability_fraction
    );
    assert!(
        metrics.anomalous_edge_acoustic_dissipation_db_per_um <= 0.0015,
        "Anomalous edge acoustic dissipation must be <= 0.0015 dB/um, got {:.6} dB/um",
        metrics.anomalous_edge_acoustic_dissipation_db_per_um
    );
    assert!(
        metrics.hydrodynamic_entropy_generation_rate_w_per_k <= 1.0e-5,
        "Hydrodynamic entropy generation rate must be <= 1.0e-5 W/K, got {:.6e} W/K",
        metrics.hydrodynamic_entropy_generation_rate_w_per_k
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_fractional_filling_factor_scaling() {
    let mut low_nu = FractionalChernSimonsViscometerParams::default();
    low_nu.fractional_filling_factor_nu = 0.25;
    let solver_low = FractionalChernSimonsViscometerSolver::new(low_nu);
    let metrics_low = solver_low.evaluate_metrics();

    let mut high_nu = FractionalChernSimonsViscometerParams::default();
    high_nu.fractional_filling_factor_nu = 0.95;
    let solver_high = FractionalChernSimonsViscometerSolver::new(high_nu);
    let metrics_high = solver_high.evaluate_metrics();

    // Higher filling factor enhances Hall viscosity density, fidelity, and edge isolation
    assert!(
        metrics_high.hall_viscosity_measurement_fidelity
            > metrics_low.hall_viscosity_measurement_fidelity
    );
    assert!(
        metrics_high.edge_to_bulk_acoustic_isolation_db
            > metrics_low.edge_to_bulk_acoustic_isolation_db
    );
    assert!(
        metrics_high.anomalous_edge_acoustic_dissipation_db_per_um
            < metrics_low.anomalous_edge_acoustic_dissipation_db_per_um
    );
    assert!(
        metrics_high.hydrodynamic_entropy_generation_rate_w_per_k
            < metrics_low.hydrodynamic_entropy_generation_rate_w_per_k
    );
}

#[test]
fn test_magnetic_field_scaling() {
    let mut low_b = FractionalChernSimonsViscometerParams::default();
    low_b.magnetic_field_tesla = 3.0;
    let solver_low = FractionalChernSimonsViscometerSolver::new(low_b);
    let metrics_low = solver_low.evaluate_metrics();

    let mut high_b = FractionalChernSimonsViscometerParams::default();
    high_b.magnetic_field_tesla = 15.0;
    let solver_high = FractionalChernSimonsViscometerSolver::new(high_b);
    let metrics_high = solver_high.evaluate_metrics();

    // Higher quantizing magnetic field widens fractional gap and increases isolation and fidelity
    assert!(
        metrics_high.edge_to_bulk_acoustic_isolation_db
            > metrics_low.edge_to_bulk_acoustic_isolation_db
    );
    assert!(
        metrics_high.hall_viscosity_measurement_fidelity
            > metrics_low.hall_viscosity_measurement_fidelity
    );
    assert!(
        metrics_high.edge_mode_velocity_stability_fraction
            > metrics_low.edge_mode_velocity_stability_fraction
    );
    assert!(
        metrics_high.anomalous_edge_acoustic_dissipation_db_per_um
            < metrics_low.anomalous_edge_acoustic_dissipation_db_per_um
    );
}

#[test]
fn test_piezoelectric_coupling_scaling() {
    let mut low_p = FractionalChernSimonsViscometerParams::default();
    low_p.piezoelectric_stress_coupling_coefficient = 0.20;
    let solver_low = FractionalChernSimonsViscometerSolver::new(low_p);
    let metrics_low = solver_low.evaluate_metrics();

    let mut high_p = FractionalChernSimonsViscometerParams::default();
    high_p.piezoelectric_stress_coupling_coefficient = 0.90;
    let solver_high = FractionalChernSimonsViscometerSolver::new(high_p);
    let metrics_high = solver_high.evaluate_metrics();

    // Stronger piezoelectric coupling improves measurement fidelity and edge acoustic isolation
    assert!(
        metrics_high.hall_viscosity_measurement_fidelity
            > metrics_low.hall_viscosity_measurement_fidelity
    );
    assert!(
        metrics_high.edge_to_bulk_acoustic_isolation_db
            > metrics_low.edge_to_bulk_acoustic_isolation_db
    );
    assert!(
        metrics_high.hydrodynamic_entropy_generation_rate_w_per_k
            < metrics_low.hydrodynamic_entropy_generation_rate_w_per_k
    );
}

#[test]
fn test_acoustic_shear_frequency_scaling() {
    let mut low_f = FractionalChernSimonsViscometerParams::default();
    low_f.acoustic_shear_frequency_ghz = 1.5;
    let solver_low = FractionalChernSimonsViscometerSolver::new(low_f);
    let metrics_low = solver_low.evaluate_metrics();

    let mut high_f = FractionalChernSimonsViscometerParams::default();
    high_f.acoustic_shear_frequency_ghz = 13.5;
    let solver_high = FractionalChernSimonsViscometerSolver::new(high_f);
    let metrics_high = solver_high.evaluate_metrics();

    // Higher frequency improves edge mode velocity stability and isolation
    assert!(
        metrics_high.hall_viscosity_measurement_fidelity
            > metrics_low.hall_viscosity_measurement_fidelity
    );
    assert!(
        metrics_high.edge_to_bulk_acoustic_isolation_db
            > metrics_low.edge_to_bulk_acoustic_isolation_db
    );
    assert!(
        metrics_high.anomalous_edge_acoustic_dissipation_db_per_um
            < metrics_low.anomalous_edge_acoustic_dissipation_db_per_um
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut cold = FractionalChernSimonsViscometerParams::default();
    cold.cryogenic_temperature_mk = 2.0;
    let solver_cold = FractionalChernSimonsViscometerSolver::new(cold);
    let metrics_cold = solver_cold.evaluate_metrics();

    let mut warm = FractionalChernSimonsViscometerParams::default();
    warm.cryogenic_temperature_mk = 45.0;
    let solver_warm = FractionalChernSimonsViscometerSolver::new(warm);
    let metrics_warm = solver_warm.evaluate_metrics();

    // Cooler temperatures enhance fidelity and suppress dissipation and thermal entropy generation
    assert!(
        metrics_cold.hall_viscosity_measurement_fidelity
            > metrics_warm.hall_viscosity_measurement_fidelity
    );
    assert!(
        metrics_cold.edge_to_bulk_acoustic_isolation_db
            > metrics_warm.edge_to_bulk_acoustic_isolation_db
    );
    assert!(
        metrics_cold.anomalous_edge_acoustic_dissipation_db_per_um
            < metrics_warm.anomalous_edge_acoustic_dissipation_db_per_um
    );
    assert!(
        metrics_cold.hydrodynamic_entropy_generation_rate_w_per_k
            < metrics_warm.hydrodynamic_entropy_generation_rate_w_per_k
    );
}

#[test]
fn test_viscometer_channel_length_scaling() {
    let mut short_l = FractionalChernSimonsViscometerParams::default();
    short_l.viscometer_channel_length_um = 3.0;
    let solver_short = FractionalChernSimonsViscometerSolver::new(short_l);
    let metrics_short = solver_short.evaluate_metrics();

    let mut long_l = FractionalChernSimonsViscometerParams::default();
    long_l.viscometer_channel_length_um = 28.0;
    let solver_long = FractionalChernSimonsViscometerSolver::new(long_l);
    let metrics_long = solver_long.evaluate_metrics();

    // Longer channel length provides more interaction cycles, enhancing fidelity and isolation
    assert!(
        metrics_long.hall_viscosity_measurement_fidelity
            > metrics_short.hall_viscosity_measurement_fidelity
    );
    assert!(
        metrics_long.edge_to_bulk_acoustic_isolation_db
            > metrics_short.edge_to_bulk_acoustic_isolation_db
    );
    assert!(
        metrics_long.anomalous_edge_acoustic_dissipation_db_per_um
            < metrics_short.anomalous_edge_acoustic_dissipation_db_per_um
    );
}

#[test]
fn test_edge_channel_width_scaling() {
    let mut narrow = FractionalChernSimonsViscometerParams::default();
    narrow.edge_channel_width_nm = 30.0;
    let solver_narrow = FractionalChernSimonsViscometerSolver::new(narrow);
    let metrics_narrow = solver_narrow.evaluate_metrics();

    let mut wide = FractionalChernSimonsViscometerParams::default();
    wide.edge_channel_width_nm = 180.0;
    let solver_wide = FractionalChernSimonsViscometerSolver::new(wide);
    let metrics_wide = solver_wide.evaluate_metrics();

    // Narrow edge channel suppresses bulk leakage, enhancing isolation and velocity stability
    assert!(
        metrics_narrow.edge_to_bulk_acoustic_isolation_db
            > metrics_wide.edge_to_bulk_acoustic_isolation_db
    );
    assert!(
        metrics_narrow.edge_mode_velocity_stability_fraction
            > metrics_wide.edge_mode_velocity_stability_fraction
    );
    assert!(
        metrics_narrow.anomalous_edge_acoustic_dissipation_db_per_um
            < metrics_wide.anomalous_edge_acoustic_dissipation_db_per_um
    );
}

#[test]
fn test_electron_effective_mass_scaling() {
    let mut light = FractionalChernSimonsViscometerParams::default();
    light.electron_effective_mass_ratio = 0.067;
    let solver_light = FractionalChernSimonsViscometerSolver::new(light);
    let metrics_light = solver_light.evaluate_metrics();

    let mut heavy = FractionalChernSimonsViscometerParams::default();
    heavy.electron_effective_mass_ratio = 0.45;
    let solver_heavy = FractionalChernSimonsViscometerSolver::new(heavy);
    let metrics_heavy = solver_heavy.evaluate_metrics();

    // Lighter effective mass boosts cyclotron frequency and fractional gap, improving metrics
    assert!(
        metrics_light.hall_viscosity_measurement_fidelity
            > metrics_heavy.hall_viscosity_measurement_fidelity
    );
    assert!(
        metrics_light.edge_mode_velocity_stability_fraction
            > metrics_heavy.edge_mode_velocity_stability_fraction
    );
    assert!(
        metrics_light.hydrodynamic_entropy_generation_rate_w_per_k
            < metrics_heavy.hydrodynamic_entropy_generation_rate_w_per_k
    );
}
