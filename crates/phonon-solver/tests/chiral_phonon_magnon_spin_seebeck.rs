//! Integration tests for chiral phonon-magnon spin Seebeck cascades,
//! angular momentum transfer, and cryogenic phononic thermocells.

use phonon_models::chiral_spin_seebeck::ChiralSpinSeebeckParams;
use phonon_solver::chiral_spin_seebeck::ChiralSpinSeebeckSolver;

#[test]
fn test_spin_seebeck_voltage_and_angular_momentum_transfer() {
    let params = ChiralSpinSeebeckParams::default();
    let solver = ChiralSpinSeebeckSolver::new(params);
    let metrics = solver.solve();

    let delta_t = solver.compute_temperature_difference_k();
    assert!(
        (delta_t - 1.8).abs() < 1.0e-5,
        "Expected temperature difference 1.8 K, got {:.4} K",
        delta_t
    );

    // Seebeck voltage must satisfy >= 5.0 uV
    assert!(
        metrics.spin_seebeck_voltage_uv >= 5.0,
        "ISHE voltage must be >= 5.0 uV, got {:.2} uV",
        metrics.spin_seebeck_voltage_uv
    );

    // Injected spin current density must be positive
    assert!(
        metrics.spin_current_density_a_m2 > 1.0e2,
        "Spin current density must exceed 100 A/m^2, got {:.2e} A/m^2",
        metrics.spin_current_density_a_m2
    );

    // Carnot limit
    let carnot = solver.compute_carnot_efficiency();
    assert!(
        carnot > 0.70 && carnot < 0.95,
        "Expected Carnot limit ~ 81.8%, got {:.2}%",
        carnot * 100.0
    );

    // Power output
    assert!(
        metrics.thermocell_power_output_pw > 1.0,
        "Power output must be > 1.0 pW, got {:.2} pW",
        metrics.thermocell_power_output_pw
    );
}

#[test]
fn test_cryogenic_thermocell_power_and_efficiency() {
    let params = ChiralSpinSeebeckParams {
        temp_hot_k: 3.5,
        temp_cold_k: 0.5,
        thermocell_internal_resistance_ohm: 100.0,
        ..Default::default()
    };

    let solver = ChiralSpinSeebeckSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.spin_seebeck_voltage_uv > 30.0,
        "Higher delta T should produce voltage > 30 uV, got {:.2} uV",
        metrics.spin_seebeck_voltage_uv
    );
    assert!(
        metrics.thermocell_efficiency_percent > 0.0,
        "Thermocell efficiency must be positive, got {:.6}%",
        metrics.thermocell_efficiency_percent
    );
    assert!(
        metrics.forward_heat_current_uw > metrics.backward_heat_current_uw,
        "Forward heat current must exceed backward current"
    );
}
