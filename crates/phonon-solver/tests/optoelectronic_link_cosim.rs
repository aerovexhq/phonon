//! Integration test for end-to-end Optoelectronic Integrated Circuit (PIC) co-simulation.
//!
//! Validates:
//! - Complete optoelectronic link:
//!   CW Laser -> MZI Electro-Optic Modulator -> Optical Waveguide -> Micro-Ring Resonator -> PIN Photodetector -> Electrical Load.
//! - Non-linear MNA electrical receiver circuit simulation and KCL conservation.
//! - High-speed digital telemetry: Eye diagram folding across 2-UI, Q-factor, Bit Error Rate (BER), and energy efficiency in fJ/bit.

use phonon_core::{CircuitGraph, OpticalSignal, T_REF};
use phonon_models::photonic::{
    ElectroOpticModulatorModel, MicroRingResonatorModel, OpticalWaveguideModel, PhotodetectorModel,
    TelecomAnalyzer,
};
use phonon_solver::mna::non_linear_solver::{solve_dc_non_linear, ModelContext, NewtonOptions};

#[test]
fn test_end_to_end_optoelectronic_link_cosim() {
    // 1. Optical Transmitter: CW Laser (10 mW at 1550 nm) + MZI Modulator (V_pi = 3.0 V)
    let cw_laser = OpticalSignal::new(1.55e-6, 10.0e-3, 0.0);
    let mzi = ElectroOpticModulatorModel::silicon_mzi_quadrature(1.0e-3, 3.0);

    // 2. Optical Transmission Channel: 1 cm SOI waveguide + Micro-Ring Resonator
    let waveguide = OpticalWaveguideModel::silicon_strip(0.01);
    let mrr = MicroRingResonatorModel::add_drop_silicon(10e-6, 0.2);

    // 3. Modulate optical signal with a 2-level digital voltage sequence (Bit 1 = -1.5V, Bit 0 = +1.5V)
    let bit_rate_bps = 50.0e9; // 50 Gbps
    let bit_period_s = 1.0 / bit_rate_bps; // 20 ps
    let bit_pattern = [1, 0, 1, 1, 0, 0, 1, 0, 1, 1, 1, 0, 0, 0, 1, 0];

    let mut optical_trace = Vec::new();
    let mut ones_power = Vec::new();
    let mut zeros_power = Vec::new();

    let samples_per_bit = 8;
    let dt = bit_period_s / samples_per_bit as f64;
    let mut sim_time = 0.0;

    for &bit in &bit_pattern {
        let v_drive = if bit == 1 { -1.5 } else { 1.5 };
        // Modulate CW laser
        let mod_sig = mzi.propagate(&cw_laser, v_drive, T_REF);
        // Propagate through waveguide
        let wg_sig = waveguide.propagate(&mod_sig, T_REF);
        // Filter through micro-ring resonator (through port)
        let (thru_sig, _) = mrr.propagate(&wg_sig, T_REF);

        if bit == 1 {
            ones_power.push(thru_sig.power_watts);
        } else {
            zeros_power.push(thru_sig.power_watts);
        }

        for _ in 0..samples_per_bit {
            optical_trace.push((sim_time, thru_sig.power_watts));
            sim_time += dt;
        }
    }

    // 4. Photodetector and Electrical Receiver Circuit (MNA)
    let pin_detector = PhotodetectorModel::ge_on_si_pin();
    let r_load = 50.0; // 50 Ohm transimpedance load resistor

    // Evaluate electrical receiver circuit using MNA non-linear solver
    let mut graph = CircuitGraph::new();
    let n_out = graph.get_or_create_node("V_OUT");
    let opt_port = graph.get_or_create_optical_port("OPT_RX");

    // Reverse bias voltage source (2.0 V)
    graph
        .add_voltage_source("VBIAS", "V_BIAS", "0", 2.0)
        .unwrap();
    // Load resistor from V_OUT to ground
    graph.add_resistor("RLOAD", "V_OUT", "0", r_load).unwrap();
    // PIN photodetector: Cathode connects to V_BIAS, Anode connects to V_OUT
    // Under reverse bias, photocurrent flows cathode -> anode -> RLOAD -> GND
    let resp = pin_detector.responsivity(1.55e-6);
    graph
        .add_photodetector("PD1", "OPT_RX", "V_OUT", "V_BIAS", resp)
        .unwrap();

    let mut context = ModelContext::new();
    context.set_photodetector("PD1", pin_detector.clone());

    // Test receiver state at logical '1' optical power level
    let avg_p_one = ones_power.iter().sum::<f64>() / ones_power.len() as f64;
    context.set_optical_signal(opt_port, OpticalSignal::new(1.55e-6, avg_p_one, 0.0));

    let sol_one = solve_dc_non_linear(&graph, &context, &NewtonOptions::default())
        .expect("MNA receiver solve at bit 1 must converge");
    let v_out_one = sol_one.node_voltage(n_out);

    // KCL verification on receiver circuit
    let empty_caps = std::collections::HashMap::new();
    let kcl_one = phonon_solver::verification::kcl_probe::verify_kcl_dynamic(
        &graph,
        &sol_one.node_voltages,
        &sol_one.branch_currents,
        &empty_caps,
        Some(&context),
        1e-4,
        1e-6,
    );
    assert!(
        kcl_one.is_valid,
        "KCL must be preserved in receiver at bit 1"
    );

    // Test receiver state at logical '0' optical power level
    let avg_p_zero = zeros_power.iter().sum::<f64>() / zeros_power.len() as f64;
    context.set_optical_signal(opt_port, OpticalSignal::new(1.55e-6, avg_p_zero, 0.0));

    let sol_zero = solve_dc_non_linear(&graph, &context, &NewtonOptions::default())
        .expect("MNA receiver solve at bit 0 must converge");
    let v_out_zero = sol_zero.node_voltage(n_out);

    // Dynamic electrical voltage swing: V_out = I_ph * R_load = R0 * P_opt * R_load
    assert!(v_out_one > v_out_zero);
    assert!(v_out_one > 0.01); // Substantial millivolt output signal

    // 5. Digital Optical Telemetry Analysis
    let eye_metrics = TelecomAnalyzer::analyze_levels(&ones_power, &zeros_power);
    assert!(eye_metrics.eye_height > 0.0);
    assert!(eye_metrics.q_factor > 6.0); // Excellent signal quality
    assert!(eye_metrics.ber < 1e-9); // Error-free optical link (< 1e-9)

    // Generate folded 2-UI eye diagram samples
    let eye_diagram = TelecomAnalyzer::generate_eye_diagram(&optical_trace, bit_rate_bps);
    assert_eq!(eye_diagram.len(), optical_trace.len());
    for s in &eye_diagram {
        assert!(s.time_ui >= 0.0 && s.time_ui < 2.0);
    }

    // Energy efficiency calculation:
    // Total transmitter + receiver power ~ 20 mW at 50 Gbps
    let e_bit_fj = TelecomAnalyzer::energy_per_bit_fj(20.0e-3, bit_rate_bps);
    // 20 mW / 50 Gbps = 400 fJ/bit
    assert!((e_bit_fj - 400.0).abs() < 1e-3);
}
