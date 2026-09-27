//! Integration Test: Mixed-Signal PWM Switch-Mode Power Supply (Buck Converter).
//!
//! Validates:
//! - Digital PWM generator (PeriodicClock with duty cycle)
//! - Floating D2A bridge driving high-side MOSFET switch gate-to-source
//! - Non-linear catch diode freewheeling during switch-off phases
//! - Continuous Conduction Mode (CCM) LC filtering
//! - A2D bridge monitoring output voltage regulation to assert POWER_GOOD
//! - Steady-state output voltage matching buck converter physics: Vout ~ D * Vin - (1 - D) * Vd.

use phonon_core::{CircuitGraph, LogicLevel, NodeId};
use phonon_models::mixed_signal::{A2dBridge, D2aBridge, DigitalNetwork, PeriodicClock};
use phonon_models::{DiodeModel, MosfetModel};
use phonon_solver::mixed_signal::{
    solve_mixed_signal, MixedSignalCircuit, MixedSignalOptions, MixedSignalSolution,
};
use phonon_solver::mna::ModelContext;
use phonon_solver::transient::TransientOptions;

#[test]
fn test_mixed_signal_buck_converter_ccm_and_power_good() {
    let mut graph = CircuitGraph::new();

    // 1. Input Power Supply: 12.0 V DC
    graph
        .add_voltage_source("Vin", "vin", "0", 12.0)
        .expect("Failed to add Vin");

    // 2. High-Side Power NMOS Switch: M1 (vin -> sw, gate = vgate, source/bulk = sw)
    let _n_vin = graph.get_or_create_node("vin");
    let n_sw = graph.get_or_create_node("sw");
    let n_vgate = graph.get_or_create_node("vgate");
    graph
        .add_mosfet("M1", "vin", "vgate", "sw", "sw")
        .expect("Failed to add M1");

    // 3. Freewheeling Catch Diode: D1 (anode = 0, cathode = sw)
    graph.add_diode("D1", "0", "sw").expect("Failed to add D1");

    // 4. LC Output Filter: L1 = 50 uH, C1 = 10 uF
    let n_vout = graph.get_or_create_node("vout");
    graph
        .add_inductor("L1", "sw", "vout", 50e-6, Some(0.0))
        .expect("Failed to add L1");
    graph
        .add_capacitor("C1", "vout", "0", 10e-6, Some(0.0))
        .expect("Failed to add C1");

    // 5. Load Resistor: Rload = 10 Ohms
    graph
        .add_resistor("Rload", "vout", "0", 10.0)
        .expect("Failed to add Rload");

    // Small snubber resistor at sw node to ensure well-conditioned node impedance
    graph
        .add_resistor("R_snub", "sw", "0", 1e5)
        .expect("Failed to add R_snub");

    // 6. Digital Network: 100 kHz PWM Generator with 40% Duty Cycle
    // Period = 10 us (4 us ON, 6 us OFF)
    let mut digital_net = DigitalNetwork::new();
    let d_pwm = digital_net.add_node(Some("PWM"));
    let d_pgood = digital_net.add_node(Some("POWER_GOOD"));

    let pwm_clock = PeriodicClock::new(d_pwm, 100_000.0).with_phase_duty(0.0, 0.40);
    digital_net.add_clock(pwm_clock);

    // 7. Boundary Bridges
    let mut circuit = MixedSignalCircuit::new(graph, digital_net);

    // A. Floating Gate Driver: D2A driving V(vgate) relative to V(sw)
    // When PWM=1, Vgate - Vsw = 10.0 V (hard ON)
    // When PWM=0, Vgate - Vsw = 0.0 V (hard OFF)
    let d2a_gate = D2aBridge::new(d_pwm, n_vgate, n_sw, 0.0, 10.0).with_timings(20e-9, 20e-9, 2.0);
    circuit.add_d2a(d2a_gate);

    // B. Power-Good Supervisor: A2D monitoring V(vout) relative to ground
    // Trip point: rising above 3.5 V asserts POWER_GOOD, falling below 2.5 V de-asserts
    let a2d_pgood = A2dBridge::new(n_vout, NodeId::GROUND, d_pgood, 2.5, 3.5);
    circuit.add_a2d(a2d_pgood);

    // 8. Device Physics Setup
    let mut context = ModelContext::new();
    // Power NMOS with low on-resistance (large W/L)
    let mos_model = MosfetModel {
        w: 500e-6,
        l: 0.18e-6,
        ..MosfetModel::default()
    };
    context.set_mosfet_model("M1", mos_model);

    // Schottky-like fast power diode
    let diode_model = DiodeModel {
        is: 1e-9,
        rs: 0.05,
        ..DiodeModel::default()
    };
    context.set_diode_model("D1", diode_model);

    // 9. Run Transient Mixed-Signal Simulation
    // 300 us duration = 30 switching cycles, allows LC filter to settle
    let transient_opts = TransientOptions {
        tstop: 300e-6,
        tstep: 0.25e-6,
        uic: true,
        ..TransientOptions::default()
    };

    let options = MixedSignalOptions {
        transient: transient_opts,
        max_delta_cycles: 100,
        crossing_tolerance: 1e-10,
    };

    let solution: MixedSignalSolution =
        solve_mixed_signal(&mut circuit, &context, &options).expect("Buck converter solve failed");

    // 10. Verification of Power Electronics Physics
    // A. Verify POWER_GOOD digital assertion
    let pgood_wave = solution.digital_waveform(d_pgood);
    assert!(
        pgood_wave.iter().any(|(_, lvl)| *lvl == LogicLevel::One),
        "POWER_GOOD supervisor should assert High as output voltage rises above 3.5V"
    );

    // B. Verify steady-state output voltage
    let vout_wave = solution.analog_node_waveform(n_vout);
    let steady_state_samples: Vec<f64> = vout_wave
        .iter()
        .filter(|(t, _)| *t >= 250e-6)
        .map(|(_, v)| *v)
        .collect();

    assert!(
        !steady_state_samples.is_empty(),
        "Expected steady-state samples after 250 us"
    );

    let avg_vout = steady_state_samples.iter().sum::<f64>() / steady_state_samples.len() as f64;
    // Expected Vout ~ D * Vin = 0.40 * 12.0 = 4.8 V (minus diode & switch drops ~ 0.5 V -> ~4.0 - 4.5 V)
    println!("Buck Converter Steady-State Avg Vout = {:.3} V", avg_vout);
    assert!(
        (3.8..=4.8).contains(&avg_vout),
        "Buck output voltage {:.3}V should match CCM law around 4.0 - 4.5 V",
        avg_vout
    );

    // C. Verify switching ripple is bounded (low ripple on LC filter)
    let max_v = steady_state_samples
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let min_v = steady_state_samples
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let ripple = max_v - min_v;
    println!("Buck Converter Output Ripple = {:.3} V", ripple);
    assert!(
        ripple < 0.5,
        "Output voltage ripple {:.3}V should be low for 50uH / 10uF filter",
        ripple
    );
}
