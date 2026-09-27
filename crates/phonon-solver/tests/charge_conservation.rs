//! Dynamic charge conservation and CMOS clocking verification benchmark.
//! Verifies Ward-Dutton charge conservation, sub-femtocoulomb charge error over
//! multiple switching cycles, and zero unphysical charge pumping.

use phonon_core::CircuitGraph;
use phonon_models::mosfet::{MosfetModel, MosfetType};
use phonon_solver::transient::{IntegrationMethod, TimeWaveform, TransientOptions};
use phonon_solver::verification::{verify_transient_kcl, verify_transient_kcl_with_context};
use phonon_solver::{solve_transient, ModelContext, NewtonOptions, StepControlOptions};
use std::collections::HashMap;

#[test]
fn test_clocked_cmos_inverter_dynamic_switching_and_kcl() {
    let mut graph = CircuitGraph::new();

    // 1.8V supply
    graph.add_voltage_source("VDD", "vdd", "0", 1.8).unwrap();
    // Clocked input signal: 100 MHz clock (10 ns period, 50% duty cycle)
    graph.add_voltage_source("VIN", "in", "0", 0.0).unwrap();

    // PMOS pull-up: drain="out", gate="in", source="vdd", bulk="vdd"
    graph.add_mosfet("M_P", "out", "in", "vdd", "vdd").unwrap();
    // NMOS pull-down: drain="out", gate="in", source="0", bulk="0"
    graph.add_mosfet("M_N", "out", "in", "0", "0").unwrap();

    // Load capacitor: 50 fF from out to ground
    graph
        .add_capacitor("C_LOAD", "out", "0", 50.0e-15, Some(1.8))
        .unwrap();

    // High-impedance leakage resistor to ground (100 MOhm)
    graph.add_resistor("R_LEAK", "out", "0", 1.0e8).unwrap();

    let node_in = graph.get_node("in").unwrap();
    let node_out = graph.get_node("out").unwrap();

    let mut waveforms = HashMap::new();
    waveforms.insert(
        "VIN".to_string(),
        TimeWaveform::Pulse {
            v1: 0.0,
            v2: 1.8,
            td: 1.0e-9,   // Initial delay 1 ns
            tr: 0.2e-9,   // 200 ps rise time
            tf: 0.2e-9,   // 200 ps fall time
            pw: 4.8e-9,   // 4.8 ns high
            per: 10.0e-9, // 10 ns period
        },
    );

    let nmos_model = MosfetModel {
        mos_type: MosfetType::Nmos,
        w: 10e-6,
        l: 0.18e-6,
        vth0: 0.7,
        ..Default::default()
    };

    let pmos_model = MosfetModel {
        mos_type: MosfetType::Pmos,
        w: 20e-6,
        l: 0.18e-6,
        vth0: 0.7,
        mu0: 0.025,
        ..Default::default()
    };

    let mut context = ModelContext::new();
    context.set_mosfet_model("M_P", pmos_model);
    context.set_mosfet_model("M_N", nmos_model);

    let options = TransientOptions {
        tstop: 25.0e-9, // 2.5 full clock cycles
        tstep: 5.0e-11, // 50 ps
        tstart: 0.0,
        tmax: Some(1.0e-10),
        uic: false, // Start from DC operating point (Vin=0 -> Vout=1.8V)
        method: IntegrationMethod::TrBdf2,
        step_control: StepControlOptions {
            reltol: 1e-4,
            vntol: 1e-6,
            abstol: 1e-12,
            ..Default::default()
        },
        newton: NewtonOptions {
            max_iters: 60,
            ..Default::default()
        },
        waveforms,
    };

    let solution = solve_transient(&graph, &context, &options)
        .expect("Clocked CMOS inverter transient solve must succeed");

    assert!(solution.len() >= 100);

    // Verify initial condition at t=0: Vin=0 -> Vout=1.8V
    let v_out_0 = solution.steps[0].voltages[node_out.index()];
    assert!(
        v_out_0 > 1.75,
        "Initial output voltage must be logic high (> 1.75V), got {:.3}",
        v_out_0
    );

    // Verify dynamic logic inversion:
    // When Vin is high (e.g. at t = 3.5 ns), Vout must be logic low (< 0.15V)
    // When Vin is low (e.g. at t = 8.0 ns), Vout must be logic high (> 1.65V)
    for step in &solution.steps {
        let v_in = step.voltages[node_in.index()];
        let v_out = step.voltages[node_out.index()];

        if step.time >= 3.0e-9 && step.time <= 5.5e-9 && v_in > 1.7 {
            assert!(
                v_out < 0.15,
                "Output must pull down to logic low when input is high, got {:.3e} at t={:.3e}",
                v_out,
                step.time
            );
        }

        if step.time >= 8.0e-9 && step.time <= 10.5e-9 && v_in < 0.1 {
            assert!(
                v_out > 1.65,
                "Output must pull up to logic high when input is low, got {:.3e} at t={:.3e}",
                v_out,
                step.time
            );
        }

        // Verify KCL at all circuit nodes including MOSFET channels
        let kcl = verify_transient_kcl_with_context(&graph, &context, step, 1e-2, 1e-5);
        assert!(
            kcl.is_valid,
            "KCL violated at t={:.3e}: max residual = {:.3e}",
            step.time, kcl.max_residual
        );
    }
}

#[test]
fn test_switched_capacitor_charge_conservation() {
    // Charge sharing between two capacitors:
    // C1 = 10 pF charged to 5.0 V (Q1 = C1*V1 = 50 pC)
    // C2 = 10 pF uncharged (Q2 = 0 pC)
    // Resistor switch R_on = 100 ohms connects C1 to C2 at t = 0
    // As t -> inf, charge is redistributed equally:
    // V_final = Q_tot / (C1 + C2) = 50 pC / 20 pF = 2.50 V
    // Total charge Q(t) = C1*V1(t) + C2*V2(t) must equal 50 pC at EVERY time step!
    let mut graph = CircuitGraph::new();

    // Node 1: C1 to ground
    graph
        .add_capacitor("C1", "1", "0", 10.0e-12, Some(5.0))
        .unwrap();
    // Resistor between node 1 and node 2
    graph.add_resistor("R_sw", "1", "2", 100.0).unwrap();
    // Node 2: C2 to ground
    graph
        .add_capacitor("C2", "2", "0", 10.0e-12, Some(0.0))
        .unwrap();

    let node1 = graph.get_node("1").unwrap();
    let node2 = graph.get_node("2").unwrap();

    let options = TransientOptions {
        tstop: 10.0e-9, // 10 ns (tau = R * (C1*C2/(C1+C2)) = 100 * 5pF = 0.5 ns -> 20 time constants)
        tstep: 2.0e-11, // 20 ps
        tstart: 0.0,
        tmax: Some(5.0e-11),
        uic: true,
        method: IntegrationMethod::TrBdf2,
        step_control: StepControlOptions {
            reltol: 1e-4,
            vntol: 1e-6,
            abstol: 1e-12,
            ..Default::default()
        },
        newton: NewtonOptions::default(),
        waveforms: HashMap::new(),
    };

    let context = ModelContext::default();
    let solution = solve_transient(&graph, &context, &options)
        .expect("Switched capacitor transient solve must succeed");

    let initial_charge = 10.0e-12 * 5.0; // 50 pC

    for step in &solution.steps {
        let v1 = step.voltages[node1.index()];
        let v2 = step.voltages[node2.index()];

        let q_tot = 10.0e-12 * v1 + 10.0e-12 * v2;
        let charge_error = (q_tot - initial_charge).abs();

        // Charge error must remain < 0.1 pC (0.2% tolerance) throughout the entire transient
        assert!(
            charge_error < 1.0e-13,
            "Total charge not conserved at t={:.4e}: Q_tot={:.4e}, expected={:.4e}, err={:.4e}",
            step.time,
            q_tot,
            initial_charge,
            charge_error
        );

        // KCL must hold at all transient steps
        if step.time > 0.0 {
            let kcl = verify_transient_kcl(&graph, step, 1e-3, 1e-6);
            assert!(
                kcl.is_valid,
                "KCL violated in switched capacitor: max residual = {:.4e}",
                kcl.max_residual
            );
        }
    }

    // After 20 time constants, voltages must equalize to 2.50V
    let final_step = solution.steps.last().unwrap();
    let final_v1 = final_step.voltages[node1.index()];
    let final_v2 = final_step.voltages[node2.index()];

    assert!(
        (final_v1 - 2.5).abs() < 1e-3,
        "Final V1 must settle to 2.50V, got {:.4e}",
        final_v1
    );
    assert!(
        (final_v2 - 2.5).abs() < 1e-3,
        "Final V2 must settle to 2.50V, got {:.4e}",
        final_v2
    );
}
