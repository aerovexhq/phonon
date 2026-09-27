//! Integration tests for 3-terminal nanoelectromechanical atomic relay mechanics, pull-in/pull-out, and zero leakage.

use phonon_models::relay::{AtomicRelayModel, AtomicRelayParameters};
use phonon_solver::{CoupledRelaySolver, CoupledSolverConfig};

#[test]
fn test_sub_100mv_actuation_with_nanoscale_gap() {
    let params = AtomicRelayParameters {
        initial_gap_m: 2.0e-9,
        contact_gap_m: 1.0e-9,
        gate_area_m2: 100.0e-9 * 300.0e-9,
        spring_constant_n_per_m: 0.2,
        ..Default::default()
    };

    let relay = AtomicRelayModel::new(params);
    let v_pi = relay.pull_in_voltage_v();

    // Pull-in voltage must be strictly below 100 mV
    assert!(
        v_pi < 0.10,
        "V_pi was {:.3} V, expected < 0.10 V (sub-100mV actuation)",
        v_pi
    );
    assert!(v_pi > 0.02);
}

#[test]
fn test_electrostatic_hysteresis_and_contact_stiction() {
    let params = AtomicRelayParameters {
        initial_gap_m: 3.0e-9,
        contact_gap_m: 1.5e-9,
        spring_constant_n_per_m: 0.5,
        adhesion_force_n: 0.2e-9,
        ..Default::default()
    };

    let relay = AtomicRelayModel::new(params);
    let v_pi = relay.pull_in_voltage_v();
    let v_po = relay.pull_out_voltage_v();

    // Pull-out voltage must be less than pull-in voltage (hysteresis window)
    assert!(v_po < v_pi, "V_po ({}) must be < V_pi ({})", v_po, v_pi);
    assert!(
        v_po > 0.0,
        "V_po must be positive to ensure contact release"
    );
    let hysteresis_margin = v_pi - v_po;
    assert!(hysteresis_margin > 0.01);
}

#[test]
fn test_true_zero_subthreshold_leakage_and_steep_swing() {
    let relay = AtomicRelayModel::new(AtomicRelayParameters::default());

    // When contact is open, leakage across 8nm air gap is purely vacuum tunneling (< 10^-15 A)
    let v_ds = 0.8;
    let i_leak = relay.off_state_leakage_current_a(v_ds);
    assert!(
        i_leak < 1.0e-15,
        "Subthreshold leakage was {:.2e} A, expected < 1e-15 A",
        i_leak
    );

    // Subthreshold swing must be < 5 mV/dec (far below 60 mV/dec Boltzmann thermionic limit)
    let swing = relay.effective_subthreshold_swing_mv_per_dec();
    assert!(
        swing < 5.0,
        "Subthreshold swing was {:.2} mV/dec, expected < 5.0 mV/dec",
        swing
    );
}

#[test]
fn test_coupled_transient_dynamic_pull_in_and_joule_heating() {
    let config = CoupledSolverConfig::default();
    let solver = CoupledRelaySolver::new(config);

    let params = AtomicRelayParameters {
        initial_gap_m: 2.5e-9,
        contact_gap_m: 1.2e-9,
        gate_area_m2: 120.0e-9 * 250.0e-9,
        spring_constant_n_per_m: 0.25,
        ..Default::default()
    };

    // Pulse with 100 mV actuation
    let res = solver.simulate_atomic_relay_transient(params, 0.10, 80.0e-9, 0.4e-9);

    assert!(
        res.pull_in_delay_s.is_some(),
        "Relay failed to achieve pull-in"
    );
    let pull_in_time = res.pull_in_delay_s.unwrap();
    assert!(pull_in_time > 5.0e-9 && pull_in_time < 75.0e-9);

    // Peak temperature rise should be physically bounded (< 900 K)
    assert!(res.peak_temperature_k >= 300.0);
    assert!(res.peak_temperature_k < 900.0);

    // Displacement at pull-in must reach travel limit
    let max_disp = *res
        .displacement_m
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    assert!(max_disp >= 1.2e-9);
}
