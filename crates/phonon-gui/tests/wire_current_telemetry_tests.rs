#![deny(unsafe_code)]

//! Test suite for wire current computation, telemetry readouts, zoom-aware pill badges,
//! and Logisim-style dual-tab palette navigation.

use egui::{Color32, Pos2};
use phonon_gui::schematic::{
    compile_schematic, compute_wire_telemetry, ComponentKind, SchematicComponent, SchematicWire,
    WireSegment,
};
use phonon_gui::widgets::palette::{ComponentPalette, SidebarTab};
use phonon_gui::widgets::pill_badge::{proportional_zoom_scale, PillBadgeStyle};
use std::collections::HashMap;

#[test]
fn test_wire_current_telemetry_resistor_circuit() {
    // Construct circuit: R1 (1000 Ohm) at (100.0, 100.0)
    // Pin 1 is at (100.0, 60.0), Pin 2 is at (100.0, 140.0)
    let mut comps = Vec::new();
    let mut r1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    r1.value_str = "1000".to_string();
    comps.push(r1.clone());

    // Wire 10 connects to Pin 1 at (100.0, 60.0)
    // Wire 20 connects to Pin 2 at (100.0, 140.0)
    let wires = vec![
        SchematicWire::new(
            10,
            vec![WireSegment::new(Pos2::new(50.0, 60.0), Pos2::new(100.0, 60.0))],
        ),
        SchematicWire::new(
            20,
            vec![WireSegment::new(Pos2::new(100.0, 140.0), Pos2::new(150.0, 140.0))],
        ),
    ];

    let compiled = compile_schematic(&comps, &wires).expect("Circuit compilation should succeed");

    // DC solution: Pin 1 = 5.0V, Pin 2 = 0.0V
    let pin1_net = compiled
        .pin_to_net
        .get(&(r1.name.clone(), "1".to_string()))
        .unwrap()
        .clone();
    let pin2_net = compiled
        .pin_to_net
        .get(&(r1.name.clone(), "2".to_string()))
        .unwrap()
        .clone();

    let mut dc_voltages = HashMap::new();
    dc_voltages.insert(pin1_net, 5.0);
    dc_voltages.insert(pin2_net, 0.0);

    let (wire_voltages, wire_currents) = compute_wire_telemetry(&comps, &wires, &compiled, &dc_voltages);

    assert_eq!(wire_voltages.len(), wires.len());
    assert_eq!(wire_currents.len(), wires.len());

    // Wire connected to 1k resistor with 5V drop should carry 5mA (0.005 A)
    let i_wire10 = wire_currents.get(&10);
    assert!(i_wire10.is_some());
    let current_val = i_wire10.unwrap();
    assert!((current_val - 0.005).abs() < 1e-4);
}

#[test]
fn test_pill_badge_styles_and_presets() {
    let volt_style = PillBadgeStyle::voltage(Color32::from_rgb(60, 160, 240));
    assert_eq!(volt_style.text_color, Color32::from_rgb(200, 240, 255));
    assert_eq!(volt_style.border_color, Color32::from_rgb(60, 160, 240));

    let temp_normal = PillBadgeStyle::temperature(Color32::from_rgb(100, 200, 100), false);
    assert_eq!(temp_normal.border_color, Color32::from_rgb(100, 200, 100));

    let temp_hot = PillBadgeStyle::temperature(Color32::from_rgb(255, 80, 80), true);
    assert_eq!(temp_hot.border_color, Color32::from_rgb(255, 60, 60));

    let wire_style = PillBadgeStyle::wire_telemetry();
    assert!(wire_style.bg_color.a() > 200);

    let sens_style = PillBadgeStyle::sensitivity(Color32::from_rgb(220, 120, 40));
    assert_eq!(sens_style.border_color, Color32::from_rgb(220, 120, 40));
}

#[test]
fn test_proportional_zoom_scaling() {
    // Low zoom should clamp to minimum scale 0.75
    let scale_zoomed_out = proportional_zoom_scale(0.1);
    assert!((scale_zoomed_out - 0.75).abs() < 1e-3);

    // High zoom should clamp to maximum scale 2.0
    let scale_zoomed_in = proportional_zoom_scale(10.0);
    assert!((scale_zoomed_in - 2.0).abs() < 1e-3);

    // Standard 1.0 zoom should return exactly 1.0
    let scale_normal = proportional_zoom_scale(1.0);
    assert!((scale_normal - 1.0).abs() < 1e-3);

    // Monotonic scaling between 0.75 and 2.0
    let s_05 = proportional_zoom_scale(0.5);
    let s_15 = proportional_zoom_scale(1.5);
    assert!(s_05 < scale_normal);
    assert!(scale_normal < s_15);
}

#[test]
fn test_palette_tabs_and_search_query() {
    let mut palette = ComponentPalette::new();
    assert_eq!(palette.active_tab, SidebarTab::Library);
    assert_eq!(palette.search_query, "");

    palette.search_query = "resistor".to_string();
    assert_eq!(palette.search_query, "resistor");

    palette.search_query.clear();
    assert_eq!(palette.search_query, "");

    palette.active_tab = SidebarTab::Hierarchy;
    assert_eq!(palette.active_tab, SidebarTab::Hierarchy);
}
