#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for non-Abelian quantum acoustic
//! Kitaev spin-liquid anyon braiding and Majorana nanoresonator transceivers.

use phonon_models::kitaev_spin_liquid_braiding::KitaevSpinLiquidBraidingParams;
use phonon_solver::kitaev_spin_liquid_braiding::KitaevSpinLiquidBraidingSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = KitaevSpinLiquidBraidingParams::new(
        0.1,    // below 0.5 meV
        0.05,   // below 0.10
        0.2,    // below 0.5 T
        5.0,    // below 10.0 ns
        0.5,    // below 1.0 GHz
        0.5,    // below 1.0 mK
        0.2,    // below 0.5 um
        0.005,  // below 0.01 um^-2
    );
    assert_eq!(underflow.kitaev_exchange_coupling_j_mev, 0.5);
    assert_eq!(underflow.strain_gauge_coupling_lambda, 0.10);
    assert_eq!(underflow.external_magnetic_field_tesla, 0.5);
    assert_eq!(underflow.braiding_operation_time_ns, 10.0);
    assert_eq!(underflow.nanoresonator_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.inter_qubit_separation_um, 0.5);
    assert_eq!(underflow.non_abelian_quasiparticle_density_per_um2, 0.01);

    // Test values strictly above physical maximum bounds
    let overflow = KitaevSpinLiquidBraidingParams::new(
        35.0,   // above 25.0 meV
        1.25,   // above 0.95
        18.0,   // above 12.0 T
        800.0,  // above 500.0 ns
        22.0,   // above 15.0 GHz
        120.0,  // above 50.0 mK
        20.0,   // above 10.0 um
        2.5,    // above 1.0 um^-2
    );
    assert_eq!(overflow.kitaev_exchange_coupling_j_mev, 25.0);
    assert_eq!(overflow.strain_gauge_coupling_lambda, 0.95);
    assert_eq!(overflow.external_magnetic_field_tesla, 12.0);
    assert_eq!(overflow.braiding_operation_time_ns, 500.0);
    assert_eq!(overflow.nanoresonator_frequency_ghz, 15.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.inter_qubit_separation_um, 10.0);
    assert_eq!(overflow.non_abelian_quasiparticle_density_per_um2, 1.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = KitaevSpinLiquidBraidingParams::default();
    let solver = KitaevSpinLiquidBraidingSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.majorana_anyon_braiding_fidelity >= 0.9980,
        "Braiding fidelity must be >= 0.9980, got {:.6}",
        metrics.majorana_anyon_braiding_fidelity
    );
    assert!(
        metrics.topological_gap_protection_mhz >= 35.0,
        "Topological gap protection must be >= 35.0 MHz, got {:.2} MHz",
        metrics.topological_gap_protection_mhz
    );
    assert!(
        metrics.non_abelian_state_leakage <= 1.0e-5,
        "Non-Abelian state leakage must be <= 1.0e-5, got {:.4e}",
        metrics.non_abelian_state_leakage
    );
    assert!(
        metrics.inter_qubit_crosstalk_isolation_db >= 48.0,
        "Inter-qubit crosstalk isolation must be >= 48.0 dB, got {:.2} dB",
        metrics.inter_qubit_crosstalk_isolation_db
    );
    assert!(
        metrics.chiral_edge_energy_flux_uw_per_m2 >= 120.0,
        "Chiral edge energy flux must be >= 120.0 uW/m^2, got {:.2} uW/m^2",
        metrics.chiral_edge_energy_flux_uw_per_m2
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_kitaev_exchange_coupling_scaling() {
    let low_j = KitaevSpinLiquidBraidingParams {
        kitaev_exchange_coupling_j_mev: 1.0,
        ..KitaevSpinLiquidBraidingParams::default()
    };
    let high_j = KitaevSpinLiquidBraidingParams {
        kitaev_exchange_coupling_j_mev: 22.0,
        ..KitaevSpinLiquidBraidingParams::default()
    };

    let solver_low = KitaevSpinLiquidBraidingSolver::new(low_j);
    let solver_high = KitaevSpinLiquidBraidingSolver::new(high_j);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.topological_gap_protection_mhz > m_low.topological_gap_protection_mhz,
        "Higher Kitaev exchange coupling must enhance topological gap protection"
    );
    assert!(
        m_high.majorana_anyon_braiding_fidelity > m_low.majorana_anyon_braiding_fidelity,
        "Higher Kitaev exchange coupling must enhance braiding fidelity"
    );
    assert!(
        m_high.chiral_edge_energy_flux_uw_per_m2 > m_low.chiral_edge_energy_flux_uw_per_m2,
        "Higher Kitaev exchange coupling must enhance chiral edge energy flux"
    );
    assert!(
        m_high.non_abelian_state_leakage < m_low.non_abelian_state_leakage,
        "Higher Kitaev exchange coupling must suppress non-Abelian state leakage"
    );
    assert!(
        m_high.inter_qubit_crosstalk_isolation_db > m_low.inter_qubit_crosstalk_isolation_db,
        "Higher Kitaev exchange coupling must improve crosstalk isolation"
    );
}

#[test]
fn test_strain_gauge_coupling_scaling() {
    let low_lambda = KitaevSpinLiquidBraidingParams {
        strain_gauge_coupling_lambda: 0.15,
        ..KitaevSpinLiquidBraidingParams::default()
    };
    let high_lambda = KitaevSpinLiquidBraidingParams {
        strain_gauge_coupling_lambda: 0.90,
        ..KitaevSpinLiquidBraidingParams::default()
    };

    let solver_low = KitaevSpinLiquidBraidingSolver::new(low_lambda);
    let solver_high = KitaevSpinLiquidBraidingSolver::new(high_lambda);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.topological_gap_protection_mhz > m_low.topological_gap_protection_mhz,
        "Higher strain-gauge coupling must enhance topological gap protection"
    );
    assert!(
        m_high.majorana_anyon_braiding_fidelity > m_low.majorana_anyon_braiding_fidelity,
        "Higher strain-gauge coupling must enhance braiding fidelity"
    );
    assert!(
        m_high.chiral_edge_energy_flux_uw_per_m2 > m_low.chiral_edge_energy_flux_uw_per_m2,
        "Higher strain-gauge coupling must enhance chiral edge energy flux"
    );
    assert!(
        m_high.inter_qubit_crosstalk_isolation_db > m_low.inter_qubit_crosstalk_isolation_db,
        "Higher strain-gauge coupling must improve crosstalk isolation"
    );
    assert!(
        m_high.non_abelian_state_leakage < m_low.non_abelian_state_leakage,
        "Higher strain-gauge coupling must suppress state leakage"
    );
}

#[test]
fn test_magnetic_field_scaling() {
    let low_b = KitaevSpinLiquidBraidingParams {
        external_magnetic_field_tesla: 1.0,
        ..KitaevSpinLiquidBraidingParams::default()
    };
    let high_b = KitaevSpinLiquidBraidingParams {
        external_magnetic_field_tesla: 10.0,
        ..KitaevSpinLiquidBraidingParams::default()
    };

    let solver_low = KitaevSpinLiquidBraidingSolver::new(low_b);
    let solver_high = KitaevSpinLiquidBraidingSolver::new(high_b);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.topological_gap_protection_mhz > m_low.topological_gap_protection_mhz,
        "Higher magnetic field must widen non-Abelian gap protection"
    );
    assert!(
        m_high.majorana_anyon_braiding_fidelity > m_low.majorana_anyon_braiding_fidelity,
        "Higher magnetic field must improve braiding fidelity"
    );
    assert!(
        m_high.chiral_edge_energy_flux_uw_per_m2 > m_low.chiral_edge_energy_flux_uw_per_m2,
        "Higher magnetic field must increase chiral edge energy flux"
    );
    assert!(
        m_high.inter_qubit_crosstalk_isolation_db > m_low.inter_qubit_crosstalk_isolation_db,
        "Higher magnetic field must enhance crosstalk isolation"
    );
    assert!(
        m_high.non_abelian_state_leakage < m_low.non_abelian_state_leakage,
        "Higher magnetic field must reduce state leakage"
    );
}

#[test]
fn test_braiding_operation_time_detuning() {
    let optimal_time = KitaevSpinLiquidBraidingParams {
        braiding_operation_time_ns: 80.0,
        ..KitaevSpinLiquidBraidingParams::default()
    };
    let detuned_time = KitaevSpinLiquidBraidingParams {
        braiding_operation_time_ns: 400.0,
        ..KitaevSpinLiquidBraidingParams::default()
    };

    let solver_optimal = KitaevSpinLiquidBraidingSolver::new(optimal_time);
    let solver_detuned = KitaevSpinLiquidBraidingSolver::new(detuned_time);

    let m_optimal = solver_optimal.evaluate_metrics();
    let m_detuned = solver_detuned.evaluate_metrics();

    assert!(
        m_optimal.majorana_anyon_braiding_fidelity > m_detuned.majorana_anyon_braiding_fidelity,
        "Optimal braiding time must yield higher fidelity than detuned duration"
    );
    assert!(
        m_optimal.non_abelian_state_leakage < m_detuned.non_abelian_state_leakage,
        "Optimal braiding time must produce lower leakage than detuned duration"
    );
}

#[test]
fn test_nanoresonator_frequency_scaling() {
    let low_freq = KitaevSpinLiquidBraidingParams {
        nanoresonator_frequency_ghz: 2.0,
        ..KitaevSpinLiquidBraidingParams::default()
    };
    let high_freq = KitaevSpinLiquidBraidingParams {
        nanoresonator_frequency_ghz: 12.0,
        ..KitaevSpinLiquidBraidingParams::default()
    };

    let solver_low = KitaevSpinLiquidBraidingSolver::new(low_freq);
    let solver_high = KitaevSpinLiquidBraidingSolver::new(high_freq);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.chiral_edge_energy_flux_uw_per_m2 > m_low.chiral_edge_energy_flux_uw_per_m2,
        "Higher nanoresonator frequency must increase chiral edge energy flux"
    );
    assert!(
        m_high.topological_gap_protection_mhz > m_low.topological_gap_protection_mhz,
        "Higher nanoresonator frequency must enhance topological gap protection"
    );
    assert!(
        m_high.inter_qubit_crosstalk_isolation_db > m_low.inter_qubit_crosstalk_isolation_db,
        "Higher nanoresonator frequency must improve crosstalk isolation"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let low_temp = KitaevSpinLiquidBraidingParams {
        cryogenic_temperature_mk: 2.0,
        ..KitaevSpinLiquidBraidingParams::default()
    };
    let high_temp = KitaevSpinLiquidBraidingParams {
        cryogenic_temperature_mk: 45.0,
        ..KitaevSpinLiquidBraidingParams::default()
    };

    let solver_low = KitaevSpinLiquidBraidingSolver::new(low_temp);
    let solver_high = KitaevSpinLiquidBraidingSolver::new(high_temp);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_low.majorana_anyon_braiding_fidelity > m_high.majorana_anyon_braiding_fidelity,
        "Lower temperature must improve braiding fidelity"
    );
    assert!(
        m_low.topological_gap_protection_mhz > m_high.topological_gap_protection_mhz,
        "Lower temperature must preserve higher topological gap protection"
    );
    assert!(
        m_low.non_abelian_state_leakage < m_high.non_abelian_state_leakage,
        "Lower temperature must reduce non-Abelian state leakage"
    );
    assert!(
        m_low.inter_qubit_crosstalk_isolation_db > m_high.inter_qubit_crosstalk_isolation_db,
        "Lower temperature must improve crosstalk isolation"
    );
}

#[test]
fn test_inter_qubit_separation_scaling() {
    let close_sep = KitaevSpinLiquidBraidingParams {
        inter_qubit_separation_um: 1.0,
        ..KitaevSpinLiquidBraidingParams::default()
    };
    let far_sep = KitaevSpinLiquidBraidingParams {
        inter_qubit_separation_um: 8.0,
        ..KitaevSpinLiquidBraidingParams::default()
    };

    let solver_close = KitaevSpinLiquidBraidingSolver::new(close_sep);
    let solver_far = KitaevSpinLiquidBraidingSolver::new(far_sep);

    let m_close = solver_close.evaluate_metrics();
    let m_far = solver_far.evaluate_metrics();

    assert!(
        m_far.inter_qubit_crosstalk_isolation_db > m_close.inter_qubit_crosstalk_isolation_db,
        "Greater separation must significantly improve crosstalk isolation"
    );
    assert!(
        m_far.majorana_anyon_braiding_fidelity > m_close.majorana_anyon_braiding_fidelity,
        "Greater separation must improve braiding fidelity"
    );
    assert!(
        m_far.non_abelian_state_leakage < m_close.non_abelian_state_leakage,
        "Greater separation must suppress non-Abelian state leakage"
    );
}

#[test]
fn test_quasiparticle_density_scaling() {
    let low_qp = KitaevSpinLiquidBraidingParams {
        non_abelian_quasiparticle_density_per_um2: 0.02,
        ..KitaevSpinLiquidBraidingParams::default()
    };
    let high_qp = KitaevSpinLiquidBraidingParams {
        non_abelian_quasiparticle_density_per_um2: 0.85,
        ..KitaevSpinLiquidBraidingParams::default()
    };

    let solver_low = KitaevSpinLiquidBraidingSolver::new(low_qp);
    let solver_high = KitaevSpinLiquidBraidingSolver::new(high_qp);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_low.majorana_anyon_braiding_fidelity > m_high.majorana_anyon_braiding_fidelity,
        "Lower quasiparticle density must improve braiding fidelity"
    );
    assert!(
        m_low.topological_gap_protection_mhz > m_high.topological_gap_protection_mhz,
        "Lower quasiparticle density must preserve higher topological gap protection"
    );
    assert!(
        m_low.non_abelian_state_leakage < m_high.non_abelian_state_leakage,
        "Lower quasiparticle density must reduce non-Abelian state leakage"
    );
    assert!(
        m_low.inter_qubit_crosstalk_isolation_db > m_high.inter_qubit_crosstalk_isolation_db,
        "Lower quasiparticle density must improve crosstalk isolation"
    );
}
