//! Physical energy conservation tests: lossless LC tank oscillation and lossy RLC thermodynamic balance.

use phonon_core::CircuitGraph;
use phonon_solver::transient::{IntegrationMethod, TimeWaveform, TransientOptions};
use phonon_solver::verification::verify_energy_balance;
use phonon_solver::{solve_transient, ModelContext, NewtonOptions, StepControlOptions};
use std::collections::HashMap;

#[test]
fn test_lossless_lc_tank_energy_conservation() {
    let mut graph = CircuitGraph::new();

    // Node 1 to 0: C1 = 1 uF with initial voltage 5.0 V
    // Initial energy: E_0 = 0.5 * C * V^2 = 0.5 * 1e-6 * 25.0 = 12.5 uJ
    graph
        .add_capacitor("C1", "1", "0", 1.0e-6, Some(5.0))
        .unwrap();

    // Node 1 to 0: L1 = 10 mH with initial current 0.0 A
    // Resonant angular frequency: omega_0 = 1 / sqrt(LC) = 1 / sqrt(1e-8) = 10,000 rad/s
    // Period T = 2 * pi / 10000 ~= 0.6283 ms
    let _l1 = graph
        .add_inductor("L1", "1", "0", 10.0e-3, Some(0.0))
        .unwrap();

    let node1 = graph.get_node("1").unwrap();
    let l_branch = match graph.get_component("L1").unwrap() {
        phonon_core::ComponentRecord::Inductor { branch, .. } => *branch,
        _ => panic!("Expected inductor"),
    };

    let options = TransientOptions {
        tstop: 3.0e-3, // ~4.8 full periods
        tstep: 2.0e-6, // Fine time step for high-fidelity oscillation
        tstart: 0.0,
        tmax: Some(5.0e-6),
        uic: true,
        method: IntegrationMethod::Trapezoidal, // Trapezoidal preserves Hamiltonian energy for linear LC
        step_control: StepControlOptions {
            reltol: 1e-5,
            vntol: 1e-7,
            abstol: 1e-12,
            ..Default::default()
        },
        newton: NewtonOptions::default(),
        waveforms: HashMap::new(),
    };

    let context = ModelContext::default();
    let solution =
        solve_transient(&graph, &context, &options).expect("LC transient solve must succeed");

    assert!(solution.len() >= 200);

    let initial_energy = 0.5 * 1.0e-6 * 5.0 * 5.0; // 12.5 uJ

    // Check instantaneous total energy E(t) = 0.5*C*v^2 + 0.5*L*i^2 at every step
    for step in &solution.steps {
        let v_c = step.voltages[node1.index()];
        let i_l = step.branch_currents[l_branch.index()];

        let e_c = 0.5 * 1.0e-6 * v_c * v_c;
        let e_l = 0.5 * 10.0e-3 * i_l * i_l;
        let e_total = e_c + e_l;

        let relative_error = (e_total - initial_energy).abs() / initial_energy;
        assert!(
            relative_error < 0.005, // Within 0.5% throughout entire trajectory
            "LC tank energy conservation drift exceeded at t={:.4e}: E_tot={:.4e}, initial={:.4e}, rel_err={:.4e}",
            step.time,
            e_total,
            initial_energy,
            relative_error
        );
    }
}

#[test]
fn test_driven_rlc_thermodynamic_energy_balance() {
    let mut graph = CircuitGraph::new();

    // V1 from node 1 to 0 (5.0V step pulse)
    graph.add_voltage_source("V1", "1", "0", 5.0).unwrap();
    // Series R1 = 50 ohms from node 1 to 2
    graph.add_resistor("R1", "1", "2", 50.0).unwrap();
    // Series L1 = 5 mH from node 2 to 3
    graph
        .add_inductor("L1", "2", "3", 5.0e-3, Some(0.0))
        .unwrap();
    // Shunt C1 = 2 uF from node 3 to 0
    graph
        .add_capacitor("C1", "3", "0", 2.0e-6, Some(0.0))
        .unwrap();

    let mut waveforms = HashMap::new();
    waveforms.insert(
        "V1".to_string(),
        TimeWaveform::Pulse {
            v1: 0.0,
            v2: 5.0,
            td: 0.0,
            tr: 1e-9,
            tf: 1e-9,
            pw: 10.0,
            per: 20.0,
        },
    );

    let options = TransientOptions {
        tstop: 2.0e-3,
        tstep: 2.0e-6,
        tstart: 0.0,
        tmax: Some(5.0e-6),
        uic: true,
        method: IntegrationMethod::TrBdf2,
        step_control: StepControlOptions {
            reltol: 1e-4,
            vntol: 1e-6,
            abstol: 1e-12,
            ..Default::default()
        },
        newton: NewtonOptions::default(),
        waveforms,
    };

    let context = ModelContext::default();
    let solution =
        solve_transient(&graph, &context, &options).expect("RLC transient solve must succeed");

    assert!(solution.len() >= 100);

    // Verify First Law of Thermodynamics: Integral(P_supply) = Integral(P_joule) + Delta(E_EM)
    let report = verify_energy_balance(&graph, &solution, 0.005);

    assert!(
        report.is_valid,
        "Thermodynamic energy balance must be valid within tolerance, max relative error = {:.4e}",
        report.max_relative_error
    );

    // Final state of capacitor: 0.5 * C * V^2 = 0.5 * 2e-6 * 25 = 25 uJ
    assert!(
        report.stored_energy_change > 2.0e-5,
        "Stored energy must increase by ~25 uJ"
    );
    assert!(
        report.total_energy_supplied > 0.0,
        "Total supplied energy must be positive"
    );
    assert!(
        report.total_energy_dissipated > 0.0,
        "Total dissipated Joule heat must be positive"
    );
}
