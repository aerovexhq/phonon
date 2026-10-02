#![deny(unsafe_code)]

//! Verification test suite for Phonon Visual Studio Sensitivity Visualizer Dialog and worst-case UI widgets.

use egui::Color32;
use phonon_core::CircuitGraph;
use phonon_gui::widgets::sensitivity_dialog::SensitivityDialog;
use phonon_solver::mna::non_linear_solver::ModelContext;
use phonon_solver::sensitivity::{ObjectiveKind, SensitivityResult};
use phonon_solver::transient::TransientOptions;

#[test]
fn test_sensitivity_dialog_initialization_defaults() {
    let dialog = SensitivityDialog::new();

    assert!(!dialog.is_open, "Dialog should be closed by default");
    assert_eq!(dialog.selected_objective_idx, 0);
    assert_eq!(dialog.target_node, 1);
    assert!(dialog.show_badges_on_canvas);
    assert!(!dialog.run_requested);
    assert!(dialog.parameters.is_empty());
    assert!(dialog.sensitivity_results.is_empty());
    assert!(dialog.worst_case_summary.is_none());

    // Verify default objective is TerminalVoltage
    assert_eq!(
        dialog.current_objective(),
        ObjectiveKind::TerminalVoltage { node: 1 }
    );
}

#[test]
fn test_sensitivity_ranking_sorting() {
    let mut dialog = SensitivityDialog::new();

    dialog.sensitivity_results = vec![
        SensitivityResult {
            param_id: "R_LOW".to_string(),
            nominal_value: 100.0,
            gradient: 0.001,
            normalized_sensitivity: 0.02, // 2%
        },
        SensitivityResult {
            param_id: "R_HIGH".to_string(),
            nominal_value: 1000.0,
            gradient: -0.5,
            normalized_sensitivity: -0.85, // -85%
        },
        SensitivityResult {
            param_id: "C_MID".to_string(),
            nominal_value: 1.0e-6,
            gradient: -0.1,
            normalized_sensitivity: -0.25, // -25%
        },
    ];

    let sorted = dialog.sorted_results();
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0].param_id, "R_HIGH");
    assert_eq!(sorted[1].param_id, "C_MID");
    assert_eq!(sorted[2].param_id, "R_LOW");
}

#[test]
fn test_color_badge_generation() {
    // Crimson (>20%)
    let crimson = SensitivityDialog::badge_color(0.25);
    assert_eq!(crimson, Color32::from_rgb(220, 50, 50));

    let neg_crimson = SensitivityDialog::badge_color(-0.45);
    assert_eq!(neg_crimson, Color32::from_rgb(220, 50, 50));

    // Amber (5-20%)
    let amber = SensitivityDialog::badge_color(0.12);
    assert_eq!(amber, Color32::from_rgb(230, 160, 20));

    let neg_amber = SensitivityDialog::badge_color(-0.06);
    assert_eq!(neg_amber, Color32::from_rgb(230, 160, 20));

    // Cyan (<5%)
    let cyan = SensitivityDialog::badge_color(0.03);
    assert_eq!(cyan, Color32::from_rgb(50, 190, 220));

    let neg_cyan = SensitivityDialog::badge_color(-0.01);
    assert_eq!(neg_cyan, Color32::from_rgb(50, 190, 220));
}

#[test]
fn test_component_badge_lookup() {
    let mut dialog = SensitivityDialog::new();
    dialog.sensitivity_results = vec![
        SensitivityResult {
            param_id: "R1".to_string(),
            nominal_value: 1000.0,
            gradient: -0.05,
            normalized_sensitivity: -0.30, // 30% -> Crimson
        },
        SensitivityResult {
            param_id: "R2".to_string(),
            nominal_value: 500.0,
            gradient: 0.01,
            normalized_sensitivity: 0.08, // 8% -> Amber
        },
    ];

    let badge_r1 = dialog.get_badge_impact("R1");
    assert!(badge_r1.is_some());
    let (norm_sens, color) = badge_r1.unwrap();
    assert_eq!(norm_sens, -0.30);
    assert_eq!(color, Color32::from_rgb(220, 50, 50));

    let badge_r2 = dialog.get_badge_impact("R2");
    assert!(badge_r2.is_some());
    let (norm_sens2, color2) = badge_r2.unwrap();
    assert_eq!(norm_sens2, 0.08);
    assert_eq!(color2, Color32::from_rgb(230, 160, 20));

    assert!(dialog.get_badge_impact("UNKNOWN").is_none());
}

#[test]
fn test_run_analysis_populates_results_and_corners() {
    let mut dialog = SensitivityDialog::new();

    let mut graph = CircuitGraph::new();
    let _ = graph.add_voltage_source("V1", "VIN", "0", 5.0);
    let _ = graph.add_resistor("R1", "VIN", "VOUT", 1000.0);
    let _ = graph.add_capacitor("C1", "VOUT", "0", 1.0e-5, Some(0.0));

    let context = ModelContext::default();
    let mut options = TransientOptions::default();
    options.tstop = 0.005;
    options.tstep = 0.0001;
    options.uic = true;

    dialog.target_node = 2;
    dialog.selected_objective_idx = 0; // TerminalVoltage
    dialog.run_analysis(&graph, &context, &options);

    assert_eq!(dialog.sensitivity_results.len(), 2);
    assert!(dialog.worst_case_summary.is_some());
    let summary = dialog.worst_case_summary.as_ref().unwrap();
    assert!(summary.nominal.objective_value > 0.0);
    assert!(summary.max_degradation_percent >= 0.0);
    assert!(dialog.status_msg.contains("Adjoint analysis completed"));
}
