#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for topological acoustic
//! higher-rank tensor gauge fields and chiral monopole-plaquette phononic sensors.

use phonon_models::tensor_gauge_monopole_sensor::TensorGaugeMonopoleSensorParams;
use phonon_solver::tensor_gauge_monopole_sensor::TensorGaugeMonopoleSensorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = TensorGaugeMonopoleSensorParams::new(
        0.10,   // below 0.20
        0.5,    // below 1.0 meV
        0.5,    // below 1.0 GHz
        0.5,    // below 1.0 mK
        30.0,   // below 40.0 nm
        0.40,   // below 0.50
        0.2,    // below 0.5 T
        5.0e3,  // below 1.0e4
    );
    assert_eq!(underflow.tensor_gauge_coupling_constant, 0.20);
    assert_eq!(underflow.chiral_plaquette_coupling_energy_mev, 1.0);
    assert_eq!(underflow.acoustic_sensor_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.lattice_cell_dimension_nm, 40.0);
    assert_eq!(underflow.dipole_conservation_constraint_weight, 0.50);
    assert_eq!(underflow.monopole_pinning_field_tesla, 0.5);
    assert_eq!(underflow.sensing_cavity_quality_factor, 1.0e4);

    // Test values strictly above physical maximum bounds
    let overflow = TensorGaugeMonopoleSensorParams::new(
        6.0,    // above 5.0
        35.0,   // above 30.0 meV
        18.0,   // above 15.0 GHz
        60.0,   // above 50.0 mK
        500.0,  // above 400.0 nm
        1.05,   // above 0.99
        12.0,   // above 10.0 T
        8.0e5,  // above 5.0e5
    );
    assert_eq!(overflow.tensor_gauge_coupling_constant, 5.0);
    assert_eq!(overflow.chiral_plaquette_coupling_energy_mev, 30.0);
    assert_eq!(overflow.acoustic_sensor_frequency_ghz, 15.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.lattice_cell_dimension_nm, 400.0);
    assert_eq!(overflow.dipole_conservation_constraint_weight, 0.99);
    assert_eq!(overflow.monopole_pinning_field_tesla, 10.0);
    assert_eq!(overflow.sensing_cavity_quality_factor, 5.0e5);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = TensorGaugeMonopoleSensorParams::default();
    assert_eq!(params.tensor_gauge_coupling_constant, 1.65);
    assert_eq!(params.chiral_plaquette_coupling_energy_mev, 12.5);
    assert_eq!(params.acoustic_sensor_frequency_ghz, 5.4);
    assert_eq!(params.cryogenic_temperature_mk, 14.0);
    assert_eq!(params.lattice_cell_dimension_nm, 150.0);
    assert_eq!(params.dipole_conservation_constraint_weight, 0.94);
    assert_eq!(params.monopole_pinning_field_tesla, 3.8);
    assert_eq!(params.sensing_cavity_quality_factor, 1.2e5);

    let solver = TensorGaugeMonopoleSensorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.tensor_charge_sensitivity_enhancement >= 75.0,
        "Tensor charge sensitivity enhancement must be >= 75.0, got {:.2}",
        metrics.tensor_charge_sensitivity_enhancement
    );
    assert!(
        metrics.plaquette_phase_stability_error_rad <= 0.0015,
        "Plaquette phase stability error must be <= 0.0015 rad, got {:.6} rad",
        metrics.plaquette_phase_stability_error_rad
    );
    assert!(
        metrics.sub_dimensional_leakage <= 1.0e-5,
        "Sub-dimensional leakage must be <= 1.0e-5, got {:.3e}",
        metrics.sub_dimensional_leakage
    );
    assert!(
        metrics.topological_monopole_lifetime_ms >= 25.0,
        "Topological monopole lifetime must be >= 25.0 ms, got {:.2} ms",
        metrics.topological_monopole_lifetime_ms
    );
    assert!(
        metrics.tensor_gauge_flux_quantization_fidelity >= 0.9970,
        "Tensor gauge flux quantization fidelity must be >= 0.9970, got {:.6}",
        metrics.tensor_gauge_flux_quantization_fidelity
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_tensor_gauge_coupling_scaling() {
    let low_g = TensorGaugeMonopoleSensorParams {
        tensor_gauge_coupling_constant: 0.50,
        ..TensorGaugeMonopoleSensorParams::default()
    };
    let high_g = TensorGaugeMonopoleSensorParams {
        tensor_gauge_coupling_constant: 4.50,
        ..TensorGaugeMonopoleSensorParams::default()
    };

    let solver_low = TensorGaugeMonopoleSensorSolver::new(low_g);
    let solver_high = TensorGaugeMonopoleSensorSolver::new(high_g);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.tensor_charge_sensitivity_enhancement > m_low.tensor_charge_sensitivity_enhancement,
        "Higher tensor gauge coupling must enhance charge sensitivity"
    );
    assert!(
        m_high.plaquette_phase_stability_error_rad < m_low.plaquette_phase_stability_error_rad,
        "Higher tensor gauge coupling must reduce plaquette phase error"
    );
    assert!(
        m_high.sub_dimensional_leakage < m_low.sub_dimensional_leakage,
        "Higher tensor gauge coupling must suppress sub-dimensional leakage"
    );
    assert!(
        m_high.topological_monopole_lifetime_ms > m_low.topological_monopole_lifetime_ms,
        "Higher tensor gauge coupling must extend monopole lifetime"
    );
    assert!(
        m_high.tensor_gauge_flux_quantization_fidelity > m_low.tensor_gauge_flux_quantization_fidelity,
        "Higher tensor gauge coupling must improve flux quantization fidelity"
    );
}

#[test]
fn test_chiral_plaquette_coupling_scaling() {
    let low_j = TensorGaugeMonopoleSensorParams {
        chiral_plaquette_coupling_energy_mev: 3.0,
        ..TensorGaugeMonopoleSensorParams::default()
    };
    let high_j = TensorGaugeMonopoleSensorParams {
        chiral_plaquette_coupling_energy_mev: 25.0,
        ..TensorGaugeMonopoleSensorParams::default()
    };

    let solver_low = TensorGaugeMonopoleSensorSolver::new(low_j);
    let solver_high = TensorGaugeMonopoleSensorSolver::new(high_j);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.tensor_charge_sensitivity_enhancement > m_low.tensor_charge_sensitivity_enhancement,
        "Stronger plaquette ring exchange must enhance sensor sensitivity"
    );
    assert!(
        m_high.plaquette_phase_stability_error_rad < m_low.plaquette_phase_stability_error_rad,
        "Stronger plaquette ring exchange must stabilize plaquette phase"
    );
    assert!(
        m_high.sub_dimensional_leakage < m_low.sub_dimensional_leakage,
        "Stronger plaquette ring exchange must suppress leakage"
    );
    assert!(
        m_high.topological_monopole_lifetime_ms > m_low.topological_monopole_lifetime_ms,
        "Stronger plaquette ring exchange must lengthen monopole lifetime"
    );
    assert!(
        m_high.tensor_gauge_flux_quantization_fidelity > m_low.tensor_gauge_flux_quantization_fidelity,
        "Stronger plaquette ring exchange must improve flux fidelity"
    );
}

#[test]
fn test_acoustic_sensor_frequency_scaling() {
    let low_f = TensorGaugeMonopoleSensorParams {
        acoustic_sensor_frequency_ghz: 2.0,
        ..TensorGaugeMonopoleSensorParams::default()
    };
    let high_f = TensorGaugeMonopoleSensorParams {
        acoustic_sensor_frequency_ghz: 12.0,
        ..TensorGaugeMonopoleSensorParams::default()
    };

    let solver_low = TensorGaugeMonopoleSensorSolver::new(low_f);
    let solver_high = TensorGaugeMonopoleSensorSolver::new(high_f);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.tensor_charge_sensitivity_enhancement > m_low.tensor_charge_sensitivity_enhancement,
        "Higher sensor frequency must enhance dynamic strain sensitivity"
    );
    assert!(
        m_high.plaquette_phase_stability_error_rad > m_low.plaquette_phase_stability_error_rad,
        "Higher frequency drive introduces slight acoustic phase jitter"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let cold = TensorGaugeMonopoleSensorParams {
        cryogenic_temperature_mk: 2.0,
        ..TensorGaugeMonopoleSensorParams::default()
    };
    let warm = TensorGaugeMonopoleSensorParams {
        cryogenic_temperature_mk: 45.0,
        ..TensorGaugeMonopoleSensorParams::default()
    };

    let solver_cold = TensorGaugeMonopoleSensorSolver::new(cold);
    let solver_warm = TensorGaugeMonopoleSensorSolver::new(warm);

    let m_cold = solver_cold.evaluate_metrics();
    let m_warm = solver_warm.evaluate_metrics();

    assert!(
        m_cold.tensor_charge_sensitivity_enhancement > m_warm.tensor_charge_sensitivity_enhancement,
        "Colder cryogenic operation must enhance charge sensitivity"
    );
    assert!(
        m_cold.plaquette_phase_stability_error_rad < m_warm.plaquette_phase_stability_error_rad,
        "Colder cryogenic operation must minimize phase fluctuation error"
    );
    assert!(
        m_cold.sub_dimensional_leakage < m_warm.sub_dimensional_leakage,
        "Colder cryogenic operation must suppress thermal leakage"
    );
    assert!(
        m_cold.topological_monopole_lifetime_ms > m_warm.topological_monopole_lifetime_ms,
        "Colder cryogenic operation must preserve monopole lifetime"
    );
    assert!(
        m_cold.tensor_gauge_flux_quantization_fidelity > m_warm.tensor_gauge_flux_quantization_fidelity,
        "Colder cryogenic operation must maximize flux quantization fidelity"
    );
}

#[test]
fn test_lattice_cell_dimension_scaling() {
    let small_a = TensorGaugeMonopoleSensorParams {
        lattice_cell_dimension_nm: 60.0,
        ..TensorGaugeMonopoleSensorParams::default()
    };
    let large_a = TensorGaugeMonopoleSensorParams {
        lattice_cell_dimension_nm: 350.0,
        ..TensorGaugeMonopoleSensorParams::default()
    };

    let solver_small = TensorGaugeMonopoleSensorSolver::new(small_a);
    let solver_large = TensorGaugeMonopoleSensorSolver::new(large_a);

    let m_small = solver_small.evaluate_metrics();
    let m_large = solver_large.evaluate_metrics();

    assert!(
        m_large.tensor_charge_sensitivity_enhancement > m_small.tensor_charge_sensitivity_enhancement,
        "Larger unit cell provides larger acoustic integration area"
    );
    assert!(
        m_small.sub_dimensional_leakage < m_large.sub_dimensional_leakage,
        "Smaller unit cell reduces sub-dimensional acoustic leakage"
    );
    assert!(
        m_large.topological_monopole_lifetime_ms > m_small.topological_monopole_lifetime_ms,
        "Larger unit cell provides greater topological spatial protection"
    );
}

#[test]
fn test_dipole_conservation_scaling() {
    let low_w = TensorGaugeMonopoleSensorParams {
        dipole_conservation_constraint_weight: 0.55,
        ..TensorGaugeMonopoleSensorParams::default()
    };
    let high_w = TensorGaugeMonopoleSensorParams {
        dipole_conservation_constraint_weight: 0.98,
        ..TensorGaugeMonopoleSensorParams::default()
    };

    let solver_low = TensorGaugeMonopoleSensorSolver::new(low_w);
    let solver_high = TensorGaugeMonopoleSensorSolver::new(high_w);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.tensor_charge_sensitivity_enhancement > m_low.tensor_charge_sensitivity_enhancement,
        "Strict dipole conservation enforces concentrated charge sensitivity"
    );
    assert!(
        m_high.plaquette_phase_stability_error_rad < m_low.plaquette_phase_stability_error_rad,
        "Strict dipole conservation minimizes phase error"
    );
    assert!(
        m_high.sub_dimensional_leakage < m_low.sub_dimensional_leakage,
        "Strict dipole conservation strongly suppresses sub-dimensional leakage"
    );
    assert!(
        m_high.topological_monopole_lifetime_ms > m_low.topological_monopole_lifetime_ms,
        "Strict dipole conservation protects monopole lifetime"
    );
    assert!(
        m_high.tensor_gauge_flux_quantization_fidelity > m_low.tensor_gauge_flux_quantization_fidelity,
        "Strict dipole conservation enforces higher flux fidelity"
    );
}

#[test]
fn test_monopole_pinning_field_scaling() {
    let low_b = TensorGaugeMonopoleSensorParams {
        monopole_pinning_field_tesla: 1.0,
        ..TensorGaugeMonopoleSensorParams::default()
    };
    let high_b = TensorGaugeMonopoleSensorParams {
        monopole_pinning_field_tesla: 8.5,
        ..TensorGaugeMonopoleSensorParams::default()
    };

    let solver_low = TensorGaugeMonopoleSensorSolver::new(low_b);
    let solver_high = TensorGaugeMonopoleSensorSolver::new(high_b);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.topological_monopole_lifetime_ms > m_low.topological_monopole_lifetime_ms,
        "Higher magnetic pinning field substantially extends monopole lifetime"
    );
    assert!(
        m_high.tensor_charge_sensitivity_enhancement > m_low.tensor_charge_sensitivity_enhancement,
        "Higher magnetic pinning field enhances charge sensitivity"
    );
    assert!(
        m_high.sub_dimensional_leakage < m_low.sub_dimensional_leakage,
        "Higher magnetic pinning field prevents unwanted fracton diffusion"
    );
    assert!(
        m_high.plaquette_phase_stability_error_rad < m_low.plaquette_phase_stability_error_rad,
        "Higher magnetic pinning field locks plaquette phase"
    );
}

#[test]
fn test_sensing_cavity_quality_factor_scaling() {
    let low_q = TensorGaugeMonopoleSensorParams {
        sensing_cavity_quality_factor: 2.0e4,
        ..TensorGaugeMonopoleSensorParams::default()
    };
    let high_q = TensorGaugeMonopoleSensorParams {
        sensing_cavity_quality_factor: 4.0e5,
        ..TensorGaugeMonopoleSensorParams::default()
    };

    let solver_low = TensorGaugeMonopoleSensorSolver::new(low_q);
    let solver_high = TensorGaugeMonopoleSensorSolver::new(high_q);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.tensor_charge_sensitivity_enhancement > m_low.tensor_charge_sensitivity_enhancement,
        "Higher cavity Q-factor sharpens resonant sensitivity enhancement"
    );
    assert!(
        m_high.plaquette_phase_stability_error_rad < m_low.plaquette_phase_stability_error_rad,
        "Higher cavity Q-factor suppresses phase noise error"
    );
    assert!(
        m_high.sub_dimensional_leakage < m_low.sub_dimensional_leakage,
        "Higher cavity Q-factor suppresses radiative leakage"
    );
    assert!(
        m_high.topological_monopole_lifetime_ms > m_low.topological_monopole_lifetime_ms,
        "Higher cavity Q-factor lengthens monopole lifetime"
    );
    assert!(
        m_high.tensor_gauge_flux_quantization_fidelity > m_low.tensor_gauge_flux_quantization_fidelity,
        "Higher cavity Q-factor improves flux quantization fidelity"
    );
}
