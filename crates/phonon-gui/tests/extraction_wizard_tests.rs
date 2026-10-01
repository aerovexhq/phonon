#![deny(unsafe_code)]

//! Verification test suite for Phonon Visual Studio SPICE model parameter extraction wizard.

use phonon_gui::ExtractionWizardDialog;
use std::time::Instant;

#[test]
fn test_wizard_initialization_and_sample_curve_loading() {
    let mut wizard = ExtractionWizardDialog::new();

    assert!(!wizard.is_open, "Wizard dialog should be closed by default");
    assert!(wizard.current_curve.is_none());
    assert!(wizard.fitting_result.is_none());
    assert!(wizard.generated_deck.is_none());

    wizard.load_sample_nmos();
    assert!(wizard.current_curve.is_some());
    let curve = wizard.current_curve.as_ref().unwrap();
    assert_eq!(curve.points.len(), 50);
    assert_eq!(curve.temperature_k, 300.0);
    assert!(wizard.status_msg.contains("50 points"));
}

#[test]
fn test_parameter_extraction_trigger_and_model_export() {
    let mut wizard = ExtractionWizardDialog::new();
    wizard.load_sample_nmos();
    wizard.model_name = "BSIM4_TEST_DEV".to_string();

    wizard.run_fitting();

    assert!(
        wizard.fitting_result.is_some(),
        "Fitting result must be populated after run_fitting"
    );
    let result = wizard.fitting_result.as_ref().unwrap();
    assert!(
        result.r_squared > 0.985,
        "R^2 must exceed 0.985, got {:.6}",
        result.r_squared
    );
    assert!(
        result.converged,
        "Optimization should achieve convergence status"
    );

    assert!(
        wizard.generated_deck.is_some(),
        "SPICE .MODEL deck must be generated"
    );
    let deck = wizard.generated_deck.as_ref().unwrap();

    assert!(deck.contains(".MODEL BSIM4_TEST_DEV NMOS"));
    assert!(deck.contains("LEVEL=54"));
    assert!(deck.contains("VERSION=4.8.2"));
    assert!(deck.contains("VTH0="));
    assert!(deck.contains("U0="));
    assert!(deck.contains("VSAT="));

    // Validate 100% syntactical AST parsing through phonon_netlist
    let parsed = phonon_netlist::parse_netlist(deck);
    assert!(
        parsed.is_ok(),
        "Netlist parser rejected generated deck: {:?}",
        parsed.err()
    );
    let netlist = parsed.unwrap();
    assert!(netlist.models.contains_key("BSIM4_TEST_DEV"));
}

#[test]
fn test_output_curve_and_cryo_loading() {
    let mut wizard = ExtractionWizardDialog::new();

    wizard.load_sample_output();
    assert!(wizard.current_curve.is_some());
    let output_curve = wizard.current_curve.as_ref().unwrap();
    assert!(output_curve.name.contains("Output"));
    assert_eq!(output_curve.points.len(), 50);

    wizard.load_sample_cryo();
    assert!(wizard.current_curve.is_some());
    let cryo_curve = wizard.current_curve.as_ref().unwrap();
    assert_eq!(cryo_curve.temperature_k, 4.2);
    assert!(cryo_curve.name.contains("Cryo"));
}

#[test]
fn test_csv_ingestion_in_wizard() {
    let mut wizard = ExtractionWizardDialog::new();

    wizard.raw_csv_input = "\
v_ds,v_gs,v_bs,i_ds
1.0,0.2,0.0,1.5e-8
1.0,0.5,0.0,2.5e-5
1.0,0.8,0.0,1.2e-4
1.0,1.2,0.0,4.8e-4
"
    .to_string();

    wizard.parse_csv_input();

    assert!(
        wizard.current_curve.is_some(),
        "Curve must be parsed from valid CSV"
    );
    let curve = wizard.current_curve.as_ref().unwrap();
    assert_eq!(curve.points.len(), 4);
    assert!((curve.points[0].i_ds - 1.5e-8).abs() < 1e-12);
    assert!((curve.points[3].v_gs - 1.2).abs() < 1e-6);
}

#[test]
fn test_ui_dialog_operations_benchmark_sub_1ms() {
    let mut wizard = ExtractionWizardDialog::new();

    let iterations = 1000;
    let start = Instant::now();

    for i in 0..iterations {
        wizard.is_open = i % 2 == 0;
        if i % 3 == 0 {
            wizard.load_sample_nmos();
        } else if i % 3 == 1 {
            wizard.load_sample_output();
        } else {
            wizard.load_sample_cryo();
        }
    }

    let elapsed = start.elapsed();
    let per_op_us = (elapsed.as_secs_f64() * 1e6) / (iterations as f64);

    println!(
        "Completed {} UI state transitions in {:.3} ms ({:.2} us/op)",
        iterations,
        elapsed.as_secs_f64() * 1000.0,
        per_op_us
    );

    assert!(
        per_op_us < 1000.0,
        "UI dialog operation took {:.2} us (limit 1000 us = 1 ms)",
        per_op_us
    );
}
