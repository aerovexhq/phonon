//! Integration tests for Atomistic Reliability physics:
//! Electromigration (EM), Time-Dependent Dielectric Breakdown (TDDB),
//! and Contact Resistance (TLM) coupled circuit co-simulation.

use phonon_core::CircuitGraph;
use phonon_models::{ContactResistanceModel, ElectromigrationModel, TddbPercolationModel};
use phonon_solver::mna::{solve_dc_linear, SolverOptions};

#[test]
fn test_electromigration_black_law_and_flux_divergence() {
    let wire_w = 40e-9;
    let wire_h = 40e-9;
    let em = ElectromigrationModel::copper(25.0, wire_w, wire_h);

    // Current density: 1 mA in 40nm x 40nm = 6.25e11 A/m^2
    let current = 1.0e-3;
    let temp = 373.15; // 100 C

    let j_atom = em.atomic_mass_flux(current, temp);
    assert!(
        j_atom > 0.0,
        "Electromigration wind force must induce positive mass flux"
    );

    let mttf_100c = em.mean_time_to_failure_hours(current, temp);
    let mttf_125c = em.mean_time_to_failure_hours(current, 398.15); // 125 C

    // Higher temperature accelerates diffusion, lowering MTTF exponentially
    assert!(
        mttf_100c > mttf_125c * 3.0,
        "MTTF at 100C must be significantly higher than at 125C: {} vs {}",
        mttf_100c,
        mttf_125c
    );

    // Resistance degradation
    let r_init = em.degraded_resistance(current, temp, 0.0);
    assert_eq!(r_init, 25.0);

    let r_aged = em.degraded_resistance(current, temp, mttf_100c);
    assert!(
        (r_aged - 50.0).abs() < 1e-3,
        "At t = MTTF, resistance should have degraded by 2x: got {}",
        r_aged
    );
}

#[test]
fn test_tddb_percolation_breakdown_conductance_surge() {
    let tox = 1.8e-9; // 1.8 nm HfO2
    let mut tddb = TddbPercolationModel::hfo2(tox);

    let g_pre = tddb.effective_conductance();
    assert!(
        g_pre < 1e-10,
        "Pre-breakdown leakage must be tiny, got {}",
        g_pre
    );

    // Apply high electric field stress: 2.2 V across 1.8 nm = 12.2 MV/cm at 400 K
    let mut step = 0;
    while !tddb.has_broken_down && step < 200 {
        tddb.stress_step(2.2, 400.0, 1.0);
        step += 1;
    }

    assert!(
        tddb.has_broken_down,
        "Oxide must break down under accelerated stress"
    );
    let g_post = tddb.effective_conductance();
    assert!(
        g_post >= 1e-3,
        "Post-breakdown filament conductance must surge to ~mS, got {}",
        g_post
    );
    assert!(
        g_post > g_pre * 1e7,
        "Conductance jump should exceed 7 orders of magnitude"
    );
}

#[test]
fn test_contact_resistance_tlm_scaling() {
    let width = 2.0e-6; // 2 um contact width
    let length_long = 100e-9;
    let length_short = 10e-9;

    let contact_long = ContactResistanceModel::tmd_metal_contact(width, length_long);
    let contact_short = ContactResistanceModel::tmd_metal_contact(width, length_short);

    let rc_long = contact_long.evaluate_contact_resistance();
    let rc_short = contact_short.evaluate_contact_resistance();

    assert!(rc_long > 0.0);
    assert!(rc_short > 0.0);
    // As contact length drops below transfer length L_T, coth(Lc/Lt) increases Rc
    assert!(
        rc_short >= rc_long,
        "Short contacts have higher total contact resistance due to current crowding: {} vs {}",
        rc_short,
        rc_long
    );
}

#[test]
fn test_coupled_circuit_electromigration_and_tddb_modulation() {
    // Circuit:
    // Node 0: GND
    // Node 1: Supply VDD (+1.2 V)
    // Node 2: Interconnect wire node, fed by V1 through Cu wire (initially 20 Ohms)
    // Node 2 connected to GND through gate oxide (TDDB)
    // As Cu wire degrades (EM) and oxide breaks down (TDDB), node 2 voltage drops dramatically!
    let em = ElectromigrationModel::copper(20.0, 30e-9, 30e-9);
    let mut tddb = TddbPercolationModel::hfo2(1.5e-9);

    let opts = SolverOptions::default();

    // 1. Initial State: wire is pristine (20 Ohms), oxide is intact (~10^-11 S -> 100 GOhm)
    let mut graph_init = CircuitGraph::new();
    graph_init.add_voltage_source("Vdd", "1", "0", 1.2).unwrap();
    graph_init
        .add_resistor("Rwire", "1", "2", em.initial_resistance_ohms)
        .unwrap();
    let r_ox_init = 1.0 / tddb.effective_conductance();
    graph_init.add_resistor("Rox", "2", "0", r_ox_init).unwrap();

    let sol_init = solve_dc_linear(&graph_init, &opts).expect("Linear solve init");
    let v_node2_init = sol_init.node_voltages[2];
    assert!(
        (v_node2_init - 1.2).abs() < 1e-4,
        "Initially node 2 must sit at ~1.2 V: got {}",
        v_node2_init
    );

    // 2. Accelerated aging: EM doubles wire resistance, TDDB triggers oxide percolation
    let r_wire_aged = em.degraded_resistance(2e-3, 380.0, 5000.0); // e.g. 45 Ohms
                                                                   // Trigger breakdown
    for _ in 0..100 {
        tddb.stress_step(2.0, 400.0, 1.0);
    }
    assert!(tddb.has_broken_down);
    let r_ox_broken = 1.0 / tddb.effective_conductance(); // ~ 1 kOhm

    let mut graph_aged = CircuitGraph::new();
    graph_aged.add_voltage_source("Vdd", "1", "0", 1.2).unwrap();
    graph_aged
        .add_resistor("Rwire", "1", "2", r_wire_aged)
        .unwrap();
    graph_aged
        .add_resistor("Rox", "2", "0", r_ox_broken)
        .unwrap();

    let sol_aged = solve_dc_linear(&graph_aged, &opts).expect("Linear solve aged");
    let v_node2_aged = sol_aged.node_voltages[2];

    // Voltage divider: 1.2 V * Rox / (Rwire + Rox)
    let expected_v = 1.2 * r_ox_broken / (r_wire_aged + r_ox_broken);
    assert!(
        (v_node2_aged - expected_v).abs() < 1e-3,
        "Degraded circuit voltage must follow analytical voltage divider: {} vs {}",
        v_node2_aged,
        expected_v
    );
    assert!(v_node2_aged < v_node2_init);
}
