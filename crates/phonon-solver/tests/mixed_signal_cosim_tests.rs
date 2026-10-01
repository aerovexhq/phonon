#![deny(unsafe_code)]

//! Verification and benchmark suite for Phase 314 Mixed-Signal Mixed-Domain Co-Simulation,
//! Event-Driven Digital Verilog/VHDL Interface, and Continuous-Time Analog Synchronizer.

use std::time::Instant;

use phonon_solver::mixed_signal::{
    parse_verilog_module, BoundaryAdc, BoundaryDac, DacSmoothing, DigitalEngine, DigitalEvent,
    DigitalLogicGate, GateKind, LogicState, MixedSignalSynchronizer,
};

#[test]
fn test_discrete_event_gate_logic_all_kinds() {
    let mut engine = DigitalEngine::new(16);

    // Setup input nodes: 0 (A), 1 (B)
    // Setup combinational gates
    let delay = 1.0e-11; // 10 ps
    engine.add_gate(DigitalLogicGate::new(0, GateKind::And, vec![0, 1], 2, delay));
    engine.add_gate(DigitalLogicGate::new(1, GateKind::Or, vec![0, 1], 3, delay));
    engine.add_gate(DigitalLogicGate::new(2, GateKind::Xor, vec![0, 1], 4, delay));
    engine.add_gate(DigitalLogicGate::new(3, GateKind::Not, vec![0], 5, delay));
    engine.add_gate(DigitalLogicGate::new(4, GateKind::Nand, vec![0, 1], 6, delay));
    engine.add_gate(DigitalLogicGate::new(5, GateKind::Nor, vec![0, 1], 7, delay));
    engine.add_gate(DigitalLogicGate::new(6, GateKind::Buffer, vec![0], 8, delay));
    engine.add_gate(DigitalLogicGate::new(7, GateKind::Xnor, vec![0, 1], 9, delay));

    // Sequential DFF: node 10 (CLK), node 11 (D), node 12 (Q)
    engine.add_gate(DigitalLogicGate::new(8, GateKind::Dff, vec![10, 11], 12, delay));

    // Initially all nodes Low
    assert_eq!(engine.get_state(0), LogicState::Low);
    assert_eq!(engine.get_state(1), LogicState::Low);

    // Transition A to High at t = 1.0 ns
    engine.schedule_event(DigitalEvent {
        time_s: 1.0e-9,
        node_id: 0,
        new_state: LogicState::High,
    });

    let events = engine.step_until(1.05e-9);
    assert!(!events.is_empty());
    assert_eq!(engine.get_state(0), LogicState::High);
    assert_eq!(engine.get_state(2), LogicState::Low); // AND(1,0) = 0
    assert_eq!(engine.get_state(3), LogicState::High); // OR(1,0) = 1
    assert_eq!(engine.get_state(4), LogicState::High); // XOR(1,0) = 1
    assert_eq!(engine.get_state(5), LogicState::Low); // NOT(1) = 0
    assert_eq!(engine.get_state(6), LogicState::High); // NAND(1,0) = 1
    assert_eq!(engine.get_state(7), LogicState::Low); // NOR(1,0) = 0
    assert_eq!(engine.get_state(8), LogicState::High); // BUF(1) = 1
    assert_eq!(engine.get_state(9), LogicState::Low); // XNOR(1,0) = 0

    // Transition B to High at t = 2.0 ns
    engine.schedule_event(DigitalEvent {
        time_s: 2.0e-9,
        node_id: 1,
        new_state: LogicState::High,
    });

    engine.step_until(2.05e-9);
    assert_eq!(engine.get_state(2), LogicState::High); // AND(1,1) = 1
    assert_eq!(engine.get_state(3), LogicState::High); // OR(1,1) = 1
    assert_eq!(engine.get_state(4), LogicState::Low); // XOR(1,1) = 0
    assert_eq!(engine.get_state(6), LogicState::Low); // NAND(1,1) = 0
    assert_eq!(engine.get_state(9), LogicState::High); // XNOR(1,1) = 1

    // D-Flip-Flop test:
    // Set D (node 11) to High at 3.0 ns while CLK is Low
    engine.schedule_event(DigitalEvent {
        time_s: 3.0e-9,
        node_id: 11,
        new_state: LogicState::High,
    });
    engine.step_until(3.05e-9);
    assert_eq!(engine.get_state(12), LogicState::Low); // Q remains Low

    // Rising edge on CLK (node 10) at 4.0 ns
    engine.schedule_event(DigitalEvent {
        time_s: 4.0e-9,
        node_id: 10,
        new_state: LogicState::High,
    });
    engine.step_until(4.05e-9);
    assert_eq!(engine.get_state(12), LogicState::High); // Q captures D -> High

    // Change D to Low at 5.0 ns while CLK is still High
    engine.schedule_event(DigitalEvent {
        time_s: 5.0e-9,
        node_id: 11,
        new_state: LogicState::Low,
    });
    engine.step_until(5.05e-9);
    assert_eq!(engine.get_state(12), LogicState::High); // Q stays High (no posedge)

    // Pulse CLK Low at 5.5 ns, then High at 6.0 ns
    engine.schedule_event(DigitalEvent {
        time_s: 5.5e-9,
        node_id: 10,
        new_state: LogicState::Low,
    });
    engine.schedule_event(DigitalEvent {
        time_s: 6.0e-9,
        node_id: 10,
        new_state: LogicState::High,
    });
    engine.step_until(6.05e-9);
    assert_eq!(engine.get_state(12), LogicState::Low); // Q captures new D -> Low
}

#[test]
fn test_boundary_adc_threshold_crossing_and_interpolation() {
    let mut adc = BoundaryAdc::new(0, 1)
        .with_thresholds(0.8, 2.0, 0.1)
        .with_initial_state(LogicState::Low, 0.0, 0.0);

    // v_th_high = 2.0 + 0.1 = 2.1 V
    // v_th_low = 0.8 - 0.1 = 0.7 V

    // Step 1: Sub-threshold ramp (0.0 V to 1.0 V over 1.0 ns)
    let ev1 = adc.update(1.0, 1.0e-9);
    assert!(ev1.is_none());
    assert_eq!(adc.current_state, LogicState::Low);

    // Step 2: Ramp across upper threshold (1.0 V to 3.0 V from 1.0 ns to 2.0 ns)
    // Threshold is 2.1 V.
    // Interpolated t* = 1.0e-9 + (2.1 - 1.0)/(3.0 - 1.0) * 1.0e-9 = 1.0e-9 + 0.55 * 1.0e-9 = 1.55e-9
    let ev2 = adc.update(3.0, 2.0e-9);
    assert!(ev2.is_some());
    let ev2 = ev2.unwrap();
    assert_eq!(ev2.new_state, LogicState::High);
    assert_eq!(ev2.node_id, 1);
    assert!((ev2.time_s - 1.55e-9).abs() < 1e-12);
    assert_eq!(adc.current_state, LogicState::High);

    // Step 3: Voltage dips to 1.5 V (above lower threshold 0.7 V) -> hysteresis holds High
    let ev3 = adc.update(1.5, 3.0e-9);
    assert!(ev3.is_none());
    assert_eq!(adc.current_state, LogicState::High);

    // Step 4: Voltage drops across lower threshold (1.5 V to 0.5 V from 3.0 ns to 4.0 ns)
    // Threshold is 0.7 V.
    // Interpolated t* = 3.0e-9 + (0.7 - 1.5)/(0.5 - 1.5) * 1.0e-9 = 3.0e-9 + 0.8 * 1.0e-9 = 3.8e-9
    let ev4 = adc.update(0.5, 4.0e-9);
    assert!(ev4.is_some());
    let ev4 = ev4.unwrap();
    assert_eq!(ev4.new_state, LogicState::Low);
    assert_eq!(ev4.node_id, 1);
    assert!((ev4.time_s - 3.8e-9).abs() < 1e-12);
    assert_eq!(adc.current_state, LogicState::Low);
}

#[test]
fn test_boundary_dac_continuous_dynamics_and_smoothing() {
    let mut dac = BoundaryDac::new(0, 0)
        .with_levels(0.0, 3.3)
        .with_timing(1.0e-9, 2.0e-9) // 1 ns rise, 2 ns fall
        .with_impedance(50.0);

    assert_eq!(dac.voltage_at(0.0), 0.0);
    assert_eq!(dac.r_out, 50.0);

    // Transition to High at t = 1.0 ns
    dac.on_digital_event(&DigitalEvent {
        time_s: 1.0e-9,
        node_id: 0,
        new_state: LogicState::High,
    });

    // At t = 1.0 ns: 0.0 V
    assert_eq!(dac.voltage_at(1.0e-9), 0.0);

    // At t = 1.5 ns (halfway through 1.0 ns rise): 1.65 V
    let v_mid_rise = dac.voltage_at(1.5e-9);
    assert!((v_mid_rise - 1.65).abs() < 1e-9);

    // At t = 2.0 ns (rise complete): 3.3 V
    let v_top = dac.voltage_at(2.0e-9);
    assert!((v_top - 3.3).abs() < 1e-9);

    // At t = 2.5 ns (plateau): 3.3 V
    assert!((dac.voltage_at(2.5e-9) - 3.3).abs() < 1e-9);

    // Transition to Low at t = 3.0 ns (fall time = 2.0 ns)
    dac.on_digital_event(&DigitalEvent {
        time_s: 3.0e-9,
        node_id: 0,
        new_state: LogicState::Low,
    });

    // At t = 3.0 ns: 3.3 V
    assert!((dac.voltage_at(3.0e-9) - 3.3).abs() < 1e-9);

    // At t = 4.0 ns (halfway through 2.0 ns fall): 1.65 V
    let v_mid_fall = dac.voltage_at(4.0e-9);
    assert!((v_mid_fall - 1.65).abs() < 1e-9);

    // At t = 5.0 ns (fall complete): 0.0 V
    let v_bottom = dac.voltage_at(5.0e-9);
    assert!(v_bottom.abs() < 1e-9);

    // Test SmoothStep mode
    let mut dac_smooth = BoundaryDac::new(0, 0)
        .with_levels(0.0, 3.3)
        .with_timing(1.0e-9, 1.0e-9)
        .with_smoothing(DacSmoothing::SmoothStep);

    dac_smooth.on_digital_event(&DigitalEvent {
        time_s: 0.0,
        node_id: 0,
        new_state: LogicState::High,
    });

    // At u = 0.25 (t = 0.25 ns), linear is 0.25 * 3.3 = 0.825 V.
    // SmoothStep is u^2 * (3 - 2u) = 0.0625 * 2.5 = 0.15625 -> 0.15625 * 3.3 = 0.515625 V.
    let v_smooth_quarter = dac_smooth.voltage_at(0.25e-9);
    assert!((v_smooth_quarter - 0.515625).abs() < 1e-6);
}

#[test]
fn test_closed_loop_mixed_signal_cosim_oscillator() {
    let mut digital_engine = DigitalEngine::new(4);
    // Inverter gate: digital node 1 (in) -> digital node 2 (out), delay 0.5 ns
    digital_engine.add_gate(DigitalLogicGate::new(
        0,
        GateKind::Not,
        vec![1],
        2,
        0.5e-9,
    ));

    // DAC connects digital node 2 -> analog node 0
    let dac = BoundaryDac::new(2, 0)
        .with_levels(0.0, 3.3)
        .with_timing(0.2e-9, 0.2e-9);

    // ADC connects analog node 0 -> digital node 1
    let adc = BoundaryAdc::new(0, 1)
        .with_thresholds(1.0, 2.2, 0.1)
        .with_initial_state(LogicState::Low, 0.0, 0.0);

    let mut sync = MixedSignalSynchronizer::new(digital_engine, 1);
    sync.add_dac(dac);
    sync.add_adc(adc);

    // Seed initial event: digital node 2 (DAC input) starts High at 0.0s, starting oscillation
    sync.digital_engine.schedule_event(DigitalEvent {
        time_s: 0.0,
        node_id: 2,
        new_state: LogicState::High,
    });

    // RC low-pass filter analog simulation: tau = 1.0 ns
    let tau = 1.0e-9;
    let duration = 30.0e-9; // 30 ns co-simulation
    let dt = 0.05e-9; // 50 ps time steps

    let reports = sync.run_cosim(duration, dt, |_t_start, _t_end, cur_v, dac_v| {
        let v_dac = dac_v[0];
        let v_cur = cur_v[0];
        // Exponential decay step: v_next = v_dac + (v_cur - v_dac) * exp(-dt / tau)
        let v_next = v_dac + (v_cur - v_dac) * (-dt / tau).exp();
        vec![v_next]
    });

    assert!(!reports.is_empty());

    // Verify oscillation: count transitions on ADC events
    let mut total_adc_transitions = 0usize;
    for rep in &reports {
        total_adc_transitions += rep.adc_events.len();
        // Zero numerical divergence guarantee: voltages remain bounded in [0.0, 3.3]
        for &v in &rep.analog_voltages {
            assert!(v >= -1e-6 && v <= 3.3 + 1e-6);
        }
    }

    // Over 30 ns with period ~ (0.5ns + RC charging ~1.5ns)*2 ~ 4ns, expect at least 6 transitions
    assert!(
        total_adc_transitions >= 6,
        "Expected at least 6 ADC transitions in ring oscillator, got {}",
        total_adc_transitions
    );
}

#[test]
fn test_structural_and_behavioral_verilog_parser() {
    let verilog_code = r#"
        // 1-bit full adder slice with registered output
        module FullAdderSlice (
            input a,
            input b,
            input cin,
            input clk,
            output sum,
            output cout,
            output reg_sum
        );
            wire w_xor1;
            wire w_and1;
            wire w_and2;

            assign w_xor1 = a ^ b;
            assign sum = w_xor1 ^ cin;
            assign w_and1 = a & b;
            assign w_and2 = w_xor1 & cin;
            assign cout = w_and1 | w_and2;

            always @(posedge clk) begin
                reg_sum <= sum;
            end
        endmodule
    "#;

    let parsed = parse_verilog_module(verilog_code).expect("Failed to parse Verilog module");

    assert_eq!(parsed.name, "FullAdderSlice");
    assert!(parsed.inputs.contains(&"a".to_string()));
    assert!(parsed.inputs.contains(&"b".to_string()));
    assert!(parsed.inputs.contains(&"cin".to_string()));
    assert!(parsed.inputs.contains(&"clk".to_string()));
    assert!(parsed.outputs.contains(&"sum".to_string()));
    assert!(parsed.outputs.contains(&"cout".to_string()));
    assert!(parsed.outputs.contains(&"reg_sum".to_string()));

    let node_a = parsed.get_net_id("a").unwrap();
    let node_b = parsed.get_net_id("b").unwrap();
    let node_cin = parsed.get_net_id("cin").unwrap();
    let node_clk = parsed.get_net_id("clk").unwrap();
    let node_sum = parsed.get_net_id("sum").unwrap();
    let node_cout = parsed.get_net_id("cout").unwrap();
    let node_reg_sum = parsed.get_net_id("reg_sum").unwrap();
    let mut engine = parsed.engine;

    // Test Vector 1: a=1, b=0, cin=0 -> sum=1, cout=0
    engine.schedule_event(DigitalEvent {
        time_s: 1.0e-9,
        node_id: node_a,
        new_state: LogicState::High,
    });
    engine.step_until(1.05e-9);
    assert_eq!(engine.get_state(node_sum), LogicState::High);
    assert_eq!(engine.get_state(node_cout), LogicState::Low);

    // Clock edge at 2.0 ns captures sum into reg_sum
    engine.schedule_event(DigitalEvent {
        time_s: 2.0e-9,
        node_id: node_clk,
        new_state: LogicState::High,
    });
    engine.step_until(2.05e-9);
    assert_eq!(engine.get_state(node_reg_sum), LogicState::High);

    // Test Vector 2: a=1, b=1, cin=0 -> sum=0, cout=1
    engine.schedule_event(DigitalEvent {
        time_s: 3.0e-9,
        node_id: node_b,
        new_state: LogicState::High,
    });
    engine.step_until(3.05e-9);
    assert_eq!(engine.get_state(node_sum), LogicState::Low);
    assert_eq!(engine.get_state(node_cout), LogicState::High);

    // Prior to next clock edge, reg_sum remains High
    assert_eq!(engine.get_state(node_reg_sum), LogicState::High);

    // New clock posedge at 4.0 ns captures sum=0 into reg_sum
    engine.schedule_event(DigitalEvent {
        time_s: 3.5e-9,
        node_id: node_clk,
        new_state: LogicState::Low,
    });
    engine.schedule_event(DigitalEvent {
        time_s: 4.0e-9,
        node_id: node_clk,
        new_state: LogicState::High,
    });
    engine.step_until(4.05e-9);
    assert_eq!(engine.get_state(node_reg_sum), LogicState::Low);

    // Test Vector 3: a=1, b=1, cin=1 -> sum=1, cout=1
    engine.schedule_event(DigitalEvent {
        time_s: 4.5e-9,
        node_id: node_cin,
        new_state: LogicState::High,
    });
    engine.step_until(4.55e-9);
    assert_eq!(engine.get_state(node_sum), LogicState::High);
    assert_eq!(engine.get_state(node_cout), LogicState::High);
}

#[test]
fn test_mixed_signal_cosim_throughput_benchmark() {
    // Benchmark 500,000 mixed-signal co-sim events per second threshold
    let num_events = 100_000usize;
    let mut engine = DigitalEngine::new(8);

    // Setup an odd-length inverter chain ring oscillator: 0 -> 1 -> 2 -> 0 (3 stages)
    let delay = 1.0e-11; // 10 ps
    engine.add_gate(DigitalLogicGate::new(0, GateKind::Not, vec![0], 1, delay));
    engine.add_gate(DigitalLogicGate::new(1, GateKind::Not, vec![1], 2, delay));
    engine.add_gate(DigitalLogicGate::new(2, GateKind::Not, vec![2], 0, delay));

    // Set node 1 initially High (so inverter 0 input 0 Low -> 1 High is in balance, but 0 going High breaks balance)
    engine.set_state(1, LogicState::High);

    // Kick off oscillation: transition node 0 from Low to High at t = 0
    engine.schedule_event(DigitalEvent {
        time_s: 0.0,
        node_id: 0,
        new_state: LogicState::High,
    });

    let start = Instant::now();
    let mut target_t = 0.0;
    let dt = 1.0e-7;
    let mut total_processed = 0usize;

    while total_processed < num_events {
        target_t += dt;
        let events = engine.step_until(target_t);
        if events.is_empty() {
            break;
        }
        total_processed += events.len();
    }

    let elapsed = start.elapsed();
    let events_per_sec = (total_processed as f64) / elapsed.as_secs_f64();

    println!(
        "Mixed-signal co-sim throughput: {:.2} events/sec (processed {} events in {:.3} ms)",
        events_per_sec,
        total_processed,
        elapsed.as_secs_f64() * 1000.0
    );

    assert!(
        events_per_sec > 500_000.0,
        "Co-simulation throughput ({:.2} ev/s) must exceed 500,000 events/second threshold",
        events_per_sec
    );
}
