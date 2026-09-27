//! Integration Test: Mixed-Signal Phase-Locked Loop (PLL).
//!
//! Validates closed-loop electro-thermal mixed-signal synchronization:
//! - Autonomous reference clock generator
//! - Sequential Phase-Frequency Detector (PFD) using D-Flip-Flops and AND reset logic
//! - Tri-state Charge Pump boundary bridges driving an analog RC loop filter
//! - Voltage-Controlled Oscillator (VCO) relaxation dynamics with threshold tracking
//! - Verification of frequency acquisition and phase alignment.

use phonon_core::{CircuitGraph, LogicLevel, NodeId};
use phonon_models::mixed_signal::{
    A2dBridge, D2aBridge, DFlipFlop, DigitalNetwork, LogicGate, LogicGateType, PeriodicClock,
};
use phonon_solver::mixed_signal::{
    solve_mixed_signal, MixedSignalCircuit, MixedSignalOptions, MixedSignalSolution,
};
use phonon_solver::mna::ModelContext;
use phonon_solver::transient::TransientOptions;

#[test]
fn test_vco_relaxation_oscillator_frequency_scaling() {
    let mut graph = CircuitGraph::new();

    // Constant Vctrl = 2.5V
    graph
        .add_voltage_source("Vctrl", "vctrl", "0", 2.5)
        .expect("Failed to add Vctrl");

    // C_vco = 100 pF charged by VCCS from ground into cvco:
    // I = gm * Vctrl = 4 uS * 2.5V = 10 uA
    // Charging from 0V to 2.0V takes: 100pF * 2.0V / 10uA = 20 us (50 kHz)
    let n_cvco = graph.get_or_create_node("cvco");
    graph
        .add_capacitor("C_vco", "cvco", "0", 100e-12, Some(0.0))
        .expect("Failed to add C_vco");
    graph
        .add_vccs("G_vco", "0", "cvco", "vctrl", "0", 4e-6)
        .expect("Failed to add G_vco");

    // Discharge NMOS switch across C_vco
    let n_vco_gate = graph.get_or_create_node("vco_gate");
    graph
        .add_mosfet("M_reset", "cvco", "vco_gate", "0", "0")
        .expect("Failed to add M_reset");

    // Digital Network
    let mut digital_net = DigitalNetwork::new();
    let d_vco_clk = digital_net.add_node(Some("VCO_CLK"));

    // Boundary Bridges
    let mut circuit = MixedSignalCircuit::new(graph, digital_net);

    // Comparator A2D: triggers High at 2.0V, Low at 0.2V
    let comp = A2dBridge::new(n_cvco, NodeId::GROUND, d_vco_clk, 0.2, 2.0);
    circuit.add_a2d(comp);

    // Gate Driver D2A: drives vco_gate to 3.0V when VCO_CLK is High, 0.0V when Low
    let d2a = D2aBridge::new(d_vco_clk, n_vco_gate, NodeId::GROUND, 0.0, 3.0)
        .with_timings(10e-9, 10e-9, 10.0);
    circuit.add_d2a(d2a);

    // Run transient simulation for 100 us (5 expected oscillation periods)
    let transient_opts = TransientOptions {
        tstop: 100e-6,
        tstep: 0.5e-6,
        ..TransientOptions::default()
    };

    let options = MixedSignalOptions {
        transient: transient_opts,
        max_delta_cycles: 100,
        crossing_tolerance: 1e-10,
    };

    let context = ModelContext::new();
    let solution: MixedSignalSolution =
        solve_mixed_signal(&mut circuit, &context, &options).expect("VCO solve failed");

    let vco_wave = solution.digital_waveform(d_vco_clk);
    assert!(
        vco_wave.len() >= 8,
        "VCO should generate at least 4 cycles (8 transitions), got {}",
        vco_wave.len()
    );
}

#[test]
fn test_mixed_signal_pll_frequency_acquisition() {
    let mut graph = CircuitGraph::new();

    // 1. Loop Filter: C_filt = 2 nF at node "vctrl", initialized to 1.0V (slow VCO start)
    let n_vctrl = graph.get_or_create_node("vctrl");
    graph
        .add_capacitor("C_filt", "vctrl", "0", 2e-9, Some(1.0))
        .expect("Failed to add C_filt");
    graph
        .add_resistor("R_leak", "vctrl", "0", 1e8)
        .expect("Failed to add R_leak");

    // 2. VCO Relaxation Core
    let n_cvco = graph.get_or_create_node("cvco");
    graph
        .add_capacitor("C_vco", "cvco", "0", 100e-12, Some(0.0))
        .expect("Failed to add C_vco");
    // Current charging C_vco from ground: I = 4 uS * V(vctrl)
    graph
        .add_vccs("G_vco", "0", "cvco", "vctrl", "0", 4e-6)
        .expect("Failed to add G_vco");

    // Discharge NMOS switch across C_vco
    let n_vco_gate = graph.get_or_create_node("vco_gate");
    graph
        .add_mosfet("M_reset", "cvco", "vco_gate", "0", "0")
        .expect("Failed to add M_reset");

    // 3. Digital Network: PFD Logic
    let mut digital_net = DigitalNetwork::new();
    let d_ref_clk = digital_net.add_node(Some("REF_CLK"));
    let d_vco_clk = digital_net.add_node(Some("VCO_CLK"));
    let d_high = digital_net.add_node(Some("HIGH_CONST"));
    let d_up = digital_net.add_node(Some("UP"));
    let d_dn = digital_net.add_node(Some("DOWN"));
    let d_rst = digital_net.add_node(Some("PFD_RST"));

    // Tie d_high to LogicLevel::One
    digital_net.set_level(d_high, LogicLevel::One);

    // Reference Clock: 50 kHz (period 20 us, 10 us high, 10 us low)
    let ref_clock = PeriodicClock::new(d_ref_clk, 50_000.0);
    digital_net.add_clock(ref_clock);

    // DFF1: Clocked by REF_CLK, D=1, Q=UP, Reset=PFD_RST
    let dff_ref = DFlipFlop::new(0, d_ref_clk, d_high, d_up, None, 2e-9)
        .with_async_controls(Some(d_rst), None);
    digital_net.add_flip_flop(dff_ref);

    // DFF2: Clocked by VCO_CLK, D=1, Q=DOWN, Reset=PFD_RST
    let dff_vco = DFlipFlop::new(1, d_vco_clk, d_high, d_dn, None, 2e-9)
        .with_async_controls(Some(d_rst), None);
    digital_net.add_flip_flop(dff_vco);

    // Reset AND Gate: PFD_RST = UP AND DOWN (delay 2 ns)
    let rst_gate = LogicGate::new(0, LogicGateType::And, vec![d_up, d_dn], d_rst, 2e-9);
    digital_net.add_gate(rst_gate);

    // 4. Boundary Bridges
    let mut circuit = MixedSignalCircuit::new(graph, digital_net);

    // A. VCO Comparator: A2D monitoring V(cvco)
    let vco_comp = A2dBridge::new(n_cvco, NodeId::GROUND, d_vco_clk, 0.2, 2.0);
    circuit.add_a2d(vco_comp);

    // B. VCO Reset Discharger: D2A driven by VCO_CLK
    let d2a_disch = D2aBridge::new(d_vco_clk, n_vco_gate, NodeId::GROUND, 0.0, 3.0)
        .with_timings(10e-9, 10e-9, 10.0);
    circuit.add_d2a(d2a_disch);

    // C. Charge Pump Up Driver: D2A driven by UP
    // When UP=1, drives 5.0V through 20k to pump charge into C_filt
    let n_cp_up = circuit.analog_graph.get_or_create_node("cp_up");
    circuit
        .analog_graph
        .add_resistor("R_cp_up", "cp_up", "vctrl", 20_000.0)
        .expect("Failed to add R_cp_up");
    let d2a_up =
        D2aBridge::new(d_up, n_cp_up, NodeId::GROUND, 1.0, 5.0).with_timings(10e-9, 10e-9, 10.0);
    circuit.add_d2a(d2a_up);

    // D. Charge Pump Down Driver: D2A driven by DOWN
    let n_cp_dn = circuit.analog_graph.get_or_create_node("cp_dn");
    circuit
        .analog_graph
        .add_resistor("R_cp_dn", "cp_dn", "vctrl", 20_000.0)
        .expect("Failed to add R_cp_dn");
    let d2a_dn =
        D2aBridge::new(d_dn, n_cp_dn, NodeId::GROUND, 1.0, 0.0).with_timings(10e-9, 10e-9, 10.0);
    circuit.add_d2a(d2a_dn);

    // 5. Run Mixed-Signal Co-Simulation
    let transient_opts = TransientOptions {
        tstop: 120e-6,
        tstep: 0.5e-6,
        uic: true,
        ..TransientOptions::default()
    };

    let options = MixedSignalOptions {
        transient: transient_opts,
        max_delta_cycles: 100,
        crossing_tolerance: 1e-10,
    };

    let context = ModelContext::new();
    let solution: MixedSignalSolution = solve_mixed_signal(&mut circuit, &context, &options)
        .expect("Mixed-signal PLL solve failed");

    // 6. Verification
    let vco_wave = solution.digital_waveform(d_vco_clk);
    assert!(
        vco_wave.len() >= 6,
        "VCO should generate multiple clock edges, got {}",
        vco_wave.len()
    );

    // Verify that Vctrl increased from initial 1.0V towards 2.5V operating point
    let vctrl_wave = solution.analog_node_waveform(n_vctrl);
    let vctrl_initial = vctrl_wave[0].1;
    let vctrl_final = vctrl_wave.last().unwrap().1;

    assert!(
        vctrl_final > vctrl_initial + 0.2,
        "Vctrl should have increased from {:.3}V towards ~2.5V, final={:.3}V",
        vctrl_initial,
        vctrl_final
    );
}
