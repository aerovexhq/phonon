//! Integration Test: Complete 4-bit Successive Approximation Register (SAR) ADC.
//!
//! Validates closed-loop mixed-signal interaction:
//! - Continuous R-2R ladder DAC driven by 4 D2A bridges
//! - Continuous analog comparator with microvolt threshold detection
//! - Discrete digital SAR sequencer and autonomous conversion clock
//! - Verification of binary search convergence against analytical quantization.

use phonon_core::{CircuitGraph, NodeId};
use phonon_models::mixed_signal::{
    A2dBridge, D2aBridge, DigitalNetwork, PeriodicClock, SarController,
};
use phonon_solver::mixed_signal::{
    solve_mixed_signal, MixedSignalCircuit, MixedSignalOptions, MixedSignalSolution,
};
use phonon_solver::mna::ModelContext;
use phonon_solver::transient::TransientOptions;

fn run_sar_adc(v_in_volts: f64) -> (u32, MixedSignalSolution) {
    let mut graph = CircuitGraph::new();

    // 1. Analog Input Source
    graph
        .add_voltage_source("Vin", "vin", "0", v_in_volts)
        .expect("Failed to add Vin");

    // 2. Continuous 4-bit R-2R Ladder DAC
    // R = 10k, 2R = 20k
    let r = 10_000.0;
    let two_r = 20_000.0;

    // Bit 0 (LSB) leg
    graph
        .add_resistor("R_b0", "b0", "n0", two_r)
        .expect("Failed to add R_b0");
    graph
        .add_resistor("R_term", "n0", "0", two_r)
        .expect("Failed to add R_term");

    // Bit 1 leg
    graph
        .add_resistor("R_s0", "n0", "n1", r)
        .expect("Failed to add R_s0");
    graph
        .add_resistor("R_b1", "b1", "n1", two_r)
        .expect("Failed to add R_b1");

    // Bit 2 leg
    graph
        .add_resistor("R_s1", "n1", "n2", r)
        .expect("Failed to add R_s1");
    graph
        .add_resistor("R_b2", "b2", "n2", two_r)
        .expect("Failed to add R_b2");

    // Bit 3 (MSB) leg
    graph
        .add_resistor("R_s2", "n2", "vdac", r)
        .expect("Failed to add R_s2");
    graph
        .add_resistor("R_b3", "b3", "vdac", two_r)
        .expect("Failed to add R_b3");

    // Small capacitor at vdac to represent comparator input capacitance
    graph
        .add_capacitor("C_comp", "vdac", "0", 1e-12, Some(0.0))
        .expect("Failed to add C_comp");

    // 3. Digital Network and Signals
    let mut digital_net = DigitalNetwork::new();
    let d_clk = digital_net.add_node(Some("SAR_CLK"));
    let d_comp = digital_net.add_node(Some("COMP_OUT"));
    let d_eoc = digital_net.add_node(Some("EOC"));

    let d_b0 = digital_net.add_node(Some("D0"));
    let d_b1 = digital_net.add_node(Some("D1"));
    let d_b2 = digital_net.add_node(Some("D2"));
    let d_b3 = digital_net.add_node(Some("D3"));

    // Clock: 20 kHz (period 50 us, high 25 us, low 25 us)
    let clk = PeriodicClock::new(d_clk, 20_000.0);
    digital_net.add_clock(clk);

    // SAR Controller: 4 bits [LSB=d_b0, d_b1, d_b2, MSB=d_b3]
    let sar = SarController::new(
        0,
        d_clk,
        d_comp,
        vec![d_b0, d_b1, d_b2, d_b3],
        Some(d_eoc),
        10e-9, // 10 ns propagation delay
    );
    digital_net.add_sar_controller(sar);

    // 4. Boundary Bridges
    let mut circuit = MixedSignalCircuit::new(graph, digital_net);

    // Analog Comparator A2D Bridge: Vin (+) vs Vdac (-)
    let n_vin = circuit.analog_graph.get_or_create_node("vin");
    let n_vdac = circuit.analog_graph.get_or_create_node("vdac");
    // Threshold with small 1 mV hysteresis around 0V
    let comp_a2d = A2dBridge::new(n_vin, n_vdac, d_comp, -1e-3, 1e-3).with_impedance(1e9, 0.0);
    circuit.add_a2d(comp_a2d);

    // D2A Bridges for each bit driving R-2R ladder (Vref = 4.0 V)
    let bits = [(d_b0, "b0"), (d_b1, "b1"), (d_b2, "b2"), (d_b3, "b3")];
    for (d_node, ana_name) in bits {
        let n_ana = circuit.analog_graph.get_or_create_node(ana_name);
        let d2a = D2aBridge::new(d_node, n_ana, NodeId::GROUND, 0.0, 4.0)
            .with_timings(100e-9, 100e-9, 1.0); // 1 Ohm output resistance, 100 ns ramp
        circuit.add_d2a(d2a);
    }

    // 5. Run Mixed-Signal Co-Simulation
    // 6 clock cycles @ 50 us = 300 us
    let transient_opts = TransientOptions {
        tstop: 300e-6,
        tstep: 1e-6,
        ..TransientOptions::default()
    };

    let options = MixedSignalOptions {
        transient: transient_opts,
        max_delta_cycles: 100,
        crossing_tolerance: 1e-10,
    };

    let context = ModelContext::new();
    let solution =
        solve_mixed_signal(&mut circuit, &context, &options).expect("Mixed-signal solve failed");

    // Read final latched digital bits at conversion completion (t = 280 us)
    let t_eval = 280e-6;
    let b0_val = solution.digital_state_at(d_b0, t_eval);
    let b1_val = solution.digital_state_at(d_b1, t_eval);
    let b2_val = solution.digital_state_at(d_b2, t_eval);
    let b3_val = solution.digital_state_at(d_b3, t_eval);

    let mut final_code = 0u32;
    if b0_val.is_high() {
        final_code |= 1 << 0;
    }
    if b1_val.is_high() {
        final_code |= 1 << 1;
    }
    if b2_val.is_high() {
        final_code |= 1 << 2;
    }
    if b3_val.is_high() {
        final_code |= 1 << 3;
    }

    (final_code, solution)
}

#[test]
fn test_sar_adc_midscale_conversion() {
    // Vin = 2.7V with Vref = 4.0V
    // Expected code: floor(2.7 / 4.0 * 16) = floor(10.8) = 10 (binary 1010)
    let (code, sol) = run_sar_adc(2.7);
    assert_eq!(
        code, 10,
        "SAR ADC conversion for Vin=2.7V should yield code 10 (1010), got {}",
        code
    );
    assert!(sol.total_digital_events > 10);
}

#[test]
fn test_sar_adc_low_input_conversion() {
    // Vin = 1.1V with Vref = 4.0V
    // Expected code: floor(1.1 / 4.0 * 16) = floor(4.4) = 4 (binary 0100)
    let (code, sol) = run_sar_adc(1.1);
    assert_eq!(
        code, 4,
        "SAR ADC conversion for Vin=1.1V should yield code 4 (0100), got {}",
        code
    );
    assert!(sol.total_digital_events > 10);
}

#[test]
fn test_sar_adc_high_input_conversion() {
    // Vin = 3.6V with Vref = 4.0V
    // Expected code: floor(3.6 / 4.0 * 16) = floor(14.4) = 14 (binary 1110)
    let (code, sol) = run_sar_adc(3.6);
    assert_eq!(
        code, 14,
        "SAR ADC conversion for Vin=3.6V should yield code 14 (1110), got {}",
        code
    );
    assert!(sol.total_digital_events > 10);
}
