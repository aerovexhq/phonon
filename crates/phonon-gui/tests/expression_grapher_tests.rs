#![deny(unsafe_code)]

//! Verification test suite for Phase 355: Mathematical Expression Waveform Graphing Engine.
//!
//! Tests expression parsing, evaluation across time arrays, SPICE-style node voltage
//! referencing, and WaveformTrace generation for the virtual oscilloscope.

use egui::Color32;
use phonon_gui::scripting::expression_grapher::{
    evaluate_expression, generate_trace, parse_expression, ExpressionGrapher,
};
use std::collections::HashMap;

#[test]
fn test_expression_ast_parsing() {
    let expr = "2.5 + 2.5 * sin(2 * pi * 1000 * t)";
    let ast = parse_expression(expr).expect("Expression parse failed");

    let voltages = HashMap::new();
    let val_at_0 = ast.eval(0.0, &voltages).expect("Eval at 0 failed");
    // at t = 0: 2.5 + 2.5 * sin(0) = 2.5
    assert!((val_at_0 - 2.5).abs() < 1e-6);

    // at t = 0.00025 (1/4000 s -> quarter cycle of 1kHz -> 2*pi*1000*0.00025 = pi/2):
    // 2.5 + 2.5 * sin(pi/2) = 5.0
    let val_at_quarter = ast.eval(0.00025, &voltages).expect("Eval at quarter cycle failed");
    assert!((val_at_quarter - 5.0).abs() < 1e-4);
}

#[test]
fn test_spice_node_voltage_differencing() {
    let expr = "V(VIN) - V(VOUT)";
    let ast = parse_expression(expr).expect("Parse failed");

    let mut voltages = HashMap::new();
    voltages.insert("VIN".to_string(), 5.0);
    voltages.insert("VOUT".to_string(), 1.75);

    let diff = ast.eval(0.0, &voltages).expect("Eval failed");
    assert!((diff - 3.25).abs() < 1e-6);
}

#[test]
fn test_evaluate_expression_time_array() {
    let voltages = HashMap::new();
    let samples = evaluate_expression(
        "5.0 * cos(2 * pi * 500 * t)",
        0.0,
        0.002, // 1 full cycle of 500 Hz (T = 2 ms)
        101,
        &voltages,
    )
    .expect("Array eval failed");

    assert_eq!(samples.len(), 101);
    assert_eq!(samples[0][0], 0.0);
    assert!((samples[0][1] - 5.0).abs() < 1e-4);

    // End of 1 full cycle: cos(2*pi) = 1.0 -> 5.0
    let last = samples.last().unwrap();
    assert!((last[0] - 0.002).abs() < 1e-6);
    assert!((last[1] - 5.0).abs() < 1e-4);
}

#[test]
fn test_generate_waveform_trace_for_oscilloscope() {
    let mut voltages = HashMap::new();
    voltages.insert("VCC".to_string(), 12.0);

    let trace = generate_trace(
        "Damped Response",
        Color32::from_rgb(80, 200, 255),
        "V(VCC) * exp(-1000 * t) * sin(2 * pi * 2000 * t)",
        0.0,
        0.005,
        500,
        &voltages,
    )
    .expect("Trace generation failed");

    assert_eq!(trace.name, "Damped Response");
    assert_eq!(trace.color, Color32::from_rgb(80, 200, 255));
    assert_eq!(trace.samples.len(), 500);

    // Peak-to-peak should be non-zero
    assert!(trace.v_pp() > 0.0);
}

#[test]
fn test_expression_grapher_builder_configuration() {
    let mut voltages = HashMap::new();
    voltages.insert("VIN".to_string(), 3.3);

    let grapher = ExpressionGrapher::new()
        .with_voltages(voltages)
        .with_time_range(0.0, 0.001, 200);

    let trace = grapher
        .plot("V_VIN Step", Color32::GREEN, "V(VIN) * 0.5 + 0.1")
        .expect("Plot failed");

    assert_eq!(trace.samples.len(), 200);
    assert!((trace.samples[0][1] - (3.3 * 0.5 + 0.1)).abs() < 1e-6);
}
