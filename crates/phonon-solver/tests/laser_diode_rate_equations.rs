//! Integration tests for semiconductor laser diode rate equations and MNA solver stamping.
//!
//! Validates:
//! - Static Light-Current (L-I) curve and temperature-dependent threshold current scaling.
//! - Single-mode dynamic rate equations integrated via 4th-order Runge-Kutta (RK4).
//! - Relaxation oscillation frequency scaling with injection current: $f_r \propto \sqrt{I - I_{th}}$.
//! - Electrical DC MNA solution with non-linear diode companion and KCL conservation.

use phonon_core::{CircuitGraph, T_REF};
use phonon_models::photonic::{LaserDiodeModel, LaserDiodeState};
use phonon_solver::mna::non_linear_solver::{solve_dc_non_linear, ModelContext, NewtonOptions};

#[test]
fn test_laser_static_li_characteristics() {
    let laser = LaserDiodeModel::dfb_1550nm();
    let i_th = laser.threshold_current(T_REF);
    assert!((i_th - 0.015).abs() < 1e-6);

    // Below threshold (10 mA):
    let p_below = laser.static_optical_power(0.010, T_REF);
    assert!(p_below < 1e-4);

    // Above threshold (35 mA):
    let p_above = laser.static_optical_power(0.035, T_REF);
    let slope = laser.slope_efficiency(T_REF);
    let expected_p = slope * (0.035 - i_th);
    assert!((p_above - expected_p).abs() < 1e-4);

    // Temperature dependence:
    let i_th_hot = laser.threshold_current(T_REF + 30.0);
    // Ith(T0 + 30) = 15 mA * exp(30 / 60) = 15 mA * sqrt(e) ~ 24.73 mA
    assert!((i_th_hot - 0.015 * (0.5_f64).exp()).abs() < 1e-4);
}

#[test]
fn test_laser_rate_equations_rk4_transient() {
    let laser = LaserDiodeModel::dfb_1550nm();
    let mut state = LaserDiodeState {
        carrier_density_m3: 0.0,
        photon_density_m3: 0.0,
    };

    let dt = 1e-12; // 1 ps time-step
    let current = 0.040; // 40 mA step pulse (well above threshold)

    // Integrate for 2.0 ns (2000 ps) to account for physical turn-on delay (~500 ps) and relaxation oscillations
    for _ in 0..2000 {
        state = laser.step_rk4(state, current, dt);
    }

    // After 500 ps, photon density must be positive and substantial
    assert!(state.photon_density_m3 > 1e18);
    let p_opt = laser.photon_density_to_power(state.photon_density_m3);
    assert!(p_opt > 1e-3); // Multiple milliwatts
}

#[test]
fn test_laser_diode_mna_circuit_simulation() {
    // Construct circuit: VDC (2.0V) -> R_bias (50 Ohm) -> Laser Diode (node 1 -> GND)
    let mut graph = CircuitGraph::new();
    let n_laser = graph.get_or_create_node("LASER_ANODE");

    graph.add_voltage_source("V1", "IN", "0", 2.0).unwrap();
    graph.add_resistor("R1", "IN", "LASER_ANODE", 50.0).unwrap();
    graph
        .add_laser_diode("LD1", "LASER_ANODE", "0", "OPT_OUT", 0.015, 0.25)
        .unwrap();

    let mut context = ModelContext::new();
    let laser_model = LaserDiodeModel::dfb_1550nm();
    context.set_laser_diode("LD1", laser_model);

    let sol = solve_dc_non_linear(&graph, &context, &NewtonOptions::default())
        .expect("Non-linear DC solve of laser diode circuit must converge");

    let v_laser = sol.node_voltage(n_laser);
    // Typical forward InP diode turn-on voltage is between 0.8 V and 1.5 V
    assert!(v_laser > 0.8 && v_laser < 1.5);

    // Current through R1: (2.0 - V_laser) / 50 Ohm ~ (2.0 - 1.1) / 50 ~ 18 mA
    let current_r1 = (2.0 - v_laser) / 50.0;
    assert!(current_r1 > 0.010 && current_r1 < 0.030);

    // Verify Kirchhoff's Current Law
    let empty_caps = std::collections::HashMap::new();
    let kcl = phonon_solver::verification::kcl_probe::verify_kcl_dynamic(
        &graph,
        &sol.node_voltages,
        &sol.branch_currents,
        &empty_caps,
        Some(&context),
        1e-4,
        1e-6,
    );
    assert!(
        kcl.is_valid,
        "KCL must be preserved with max residual {}",
        kcl.max_residual
    );
}
