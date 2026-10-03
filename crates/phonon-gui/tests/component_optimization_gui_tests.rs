#![deny(unsafe_code)]

//! Automated GUI test suite for Phonon Visual Studio SPICE Component Optimization and Parameter Estimation.

use egui::Pos2;
use phonon_gui::{ComponentKind, DeviceModelKind, ExtractionWizardDialog, SchematicComponent};

#[test]
fn test_wizard_model_selection_and_defaults() {
    let mut wizard = ExtractionWizardDialog::new();

    assert!(!wizard.is_open);
    assert_eq!(wizard.model_kind, DeviceModelKind::Bsim4);
    assert_eq!(wizard.model_name, "NMOS_BSIM4");
    assert!(wizard.is_nmos);
    assert!(wizard.is_npn);
    assert!(wizard.enable_lm_polishing);

    // Switch to EKV
    wizard.set_model_kind(DeviceModelKind::Ekv);
    assert_eq!(wizard.model_kind, DeviceModelKind::Ekv);
    assert_eq!(wizard.model_name, "NMOS_EKV");

    // Switch EKV to PMOS
    wizard.is_nmos = false;
    wizard.set_model_kind(DeviceModelKind::Bsim4);
    wizard.set_model_kind(DeviceModelKind::Ekv);
    assert_eq!(wizard.model_name, "PMOS_EKV");

    // Switch to BJT NPN
    wizard.is_npn = true;
    wizard.set_model_kind(DeviceModelKind::GummelPoonBjt);
    assert_eq!(wizard.model_kind, DeviceModelKind::GummelPoonBjt);
    assert_eq!(wizard.model_name, "NPN_BJT");

    // Switch to BJT PNP
    wizard.is_npn = false;
    wizard.set_model_kind(DeviceModelKind::Ekv);
    wizard.set_model_kind(DeviceModelKind::GummelPoonBjt);
    assert_eq!(wizard.model_name, "PNP_BJT");
}

#[test]
fn test_wizard_curve_loading_all_models() {
    let mut wizard = ExtractionWizardDialog::new();

    // 1. BSIM4 Curves
    wizard.set_model_kind(DeviceModelKind::Bsim4);
    wizard.load_sample_nmos();
    assert!(wizard.current_curve.is_some());
    assert_eq!(wizard.current_curve.as_ref().unwrap().points.len(), 50);

    wizard.load_sample_output();
    assert!(wizard.current_curve.as_ref().unwrap().name.contains("Output"));

    wizard.load_sample_cryo();
    assert_eq!(wizard.current_curve.as_ref().unwrap().temperature_k, 4.2);

    // 2. EKV Curves
    wizard.set_model_kind(DeviceModelKind::Ekv);
    wizard.load_sample_ekv_transfer();
    assert!(wizard.current_curve.is_some());
    assert_eq!(wizard.current_curve.as_ref().unwrap().points.len(), 50);
    assert!(wizard.current_curve.as_ref().unwrap().name.contains("EKV_Transfer"));

    wizard.load_sample_ekv_output();
    assert!(wizard.current_curve.as_ref().unwrap().name.contains("EKV_Output"));

    // 3. BJT Curves
    wizard.set_model_kind(DeviceModelKind::GummelPoonBjt);
    wizard.load_sample_bjt_forward_active();
    assert!(wizard.current_curve.is_some());
    assert_eq!(wizard.current_curve.as_ref().unwrap().points.len(), 50);
    assert!(wizard.current_curve.as_ref().unwrap().name.contains("Forward_Active"));

    wizard.load_sample_bjt_gummel();
    assert!(wizard.current_curve.as_ref().unwrap().name.contains("Gummel"));

    wizard.load_sample_bjt_output();
    assert!(wizard.current_curve.as_ref().unwrap().name.contains("Output"));
}

#[test]
fn test_wizard_bounds_adjustments() {
    let mut wizard = ExtractionWizardDialog::new();

    // EKV bounds adjustments
    wizard.ekv_bounds.vto = (0.25, 0.95);
    wizard.ekv_bounds.kp = (5e-5, 5e-4);
    wizard.ekv_bounds.gamma = (0.1, 1.2);
    wizard.ekv_bounds.theta = (0.01, 0.3);

    assert_eq!(wizard.ekv_bounds.vto, (0.25, 0.95));
    assert_eq!(wizard.ekv_bounds.kp, (5e-5, 5e-4));

    // BJT bounds adjustments
    wizard.bjt_bounds.is_range = (1e-17, 1e-13);
    wizard.bjt_bounds.bf_range = (40.0, 350.0);
    wizard.bjt_bounds.vaf_range = (30.0, 250.0);

    assert_eq!(wizard.bjt_bounds.is_range, (1e-17, 1e-13));
    assert_eq!(wizard.bjt_bounds.bf_range, (40.0, 350.0));
}

#[test]
fn test_wizard_optimization_triggering_and_deck_generation() {
    let mut wizard = ExtractionWizardDialog::new();

    // 1. BSIM4 Fitting
    wizard.set_model_kind(DeviceModelKind::Bsim4);
    wizard.load_sample_nmos();
    wizard.model_name = "NMOS_BSIM4_TEST".to_string();
    wizard.run_fitting();

    assert!(wizard.fitting_result.is_some());
    let bsim_res = wizard.fitting_result.as_ref().unwrap();
    assert!(bsim_res.r_squared > 0.985);
    assert!(bsim_res.converged);
    assert!(wizard.generated_deck.is_some());
    let bsim_deck = wizard.generated_deck.as_ref().unwrap();
    assert!(bsim_deck.contains(".MODEL NMOS_BSIM4_TEST NMOS"));
    assert!(bsim_deck.contains("LEVEL=54"));
    let parsed_bsim = phonon_netlist::parse_netlist(bsim_deck);
    assert!(parsed_bsim.is_ok(), "BSIM4 deck parsed: {:?}", parsed_bsim.err());

    // 2. EKV Fitting
    wizard.set_model_kind(DeviceModelKind::Ekv);
    wizard.load_sample_ekv_transfer();
    wizard.model_name = "EKV_CORE_TEST".to_string();
    wizard.run_fitting();

    assert!(wizard.ekv_result.is_some());
    let ekv_res = wizard.ekv_result.as_ref().unwrap();
    assert!(ekv_res.r_squared > 0.985);
    assert!(ekv_res.converged);
    assert!(wizard.generated_deck.is_some());
    let ekv_deck = wizard.generated_deck.as_ref().unwrap();
    assert!(ekv_deck.contains(".MODEL EKV_CORE_TEST NMOS"));
    assert!(ekv_deck.contains("LEVEL=55"));
    let parsed_ekv = phonon_netlist::parse_netlist(ekv_deck);
    assert!(parsed_ekv.is_ok(), "EKV deck parsed: {:?}", parsed_ekv.err());

    // 3. BJT Fitting
    wizard.set_model_kind(DeviceModelKind::GummelPoonBjt);
    wizard.load_sample_bjt_forward_active();
    wizard.model_name = "BJT_NPN_TEST".to_string();
    wizard.run_fitting();

    assert!(wizard.bjt_result.is_some());
    let bjt_res = wizard.bjt_result.as_ref().unwrap();
    assert!(bjt_res.r_squared > 0.985);
    assert!(bjt_res.converged);
    assert!(wizard.generated_deck.is_some());
    let bjt_deck = wizard.generated_deck.as_ref().unwrap();
    assert!(bjt_deck.contains(".MODEL BJT_NPN_TEST NPN"));
    assert!(bjt_deck.contains("IS="));
    assert!(bjt_deck.contains("BF="));
    let parsed_bjt = phonon_netlist::parse_netlist(bjt_deck);
    assert!(parsed_bjt.is_ok(), "BJT deck parsed: {:?}", parsed_bjt.err());
}

#[test]
fn test_wizard_lm_refinement_toggle_and_triggering() {
    let mut wizard = ExtractionWizardDialog::new();
    wizard.set_model_kind(DeviceModelKind::Ekv);
    wizard.load_sample_ekv_transfer();

    // Disable LM polishing initially
    wizard.enable_lm_polishing = false;
    wizard.max_generations = 10;
    wizard.run_fitting();

    assert!(wizard.ekv_result.is_some());
    let initial_rmse = wizard.ekv_result.as_ref().unwrap().rmse;

    // Trigger explicit LM local refinement
    wizard.refine_with_lm();

    let refined_rmse = wizard.ekv_result.as_ref().unwrap().rmse;
    assert!(
        refined_rmse <= initial_rmse,
        "LM refinement must maintain or improve RMSE (initial: {:.4e}, refined: {:.4e})",
        initial_rmse,
        refined_rmse
    );
    assert!(wizard.status_msg.contains("LM Polished"));
}

#[test]
fn test_wizard_apply_to_schematic_component() {
    let mut wizard = ExtractionWizardDialog::new();

    // 1. Apply BSIM4 to NMOS component
    wizard.set_model_kind(DeviceModelKind::Bsim4);
    wizard.load_sample_nmos();
    wizard.model_name = "NMOS_EXTRACTED_V1".to_string();
    wizard.run_fitting();

    let mut comp_nmos = SchematicComponent::new(1, ComponentKind::Nmos, Pos2::new(100.0, 100.0), 1);
    assert!(comp_nmos.model_name.is_none());

    wizard.apply_to_component(&mut comp_nmos);

    assert_eq!(comp_nmos.model_name, Some("NMOS_EXTRACTED_V1".to_string()));
    assert_eq!(comp_nmos.value_str, "NMOS_EXTRACTED_V1");
    assert!(comp_nmos.properties.iter().any(|(k, _)| k == "MODEL_DECK"));
    assert!(comp_nmos.properties.iter().any(|(k, _)| k == "VTH0"));
    assert!(comp_nmos.properties.iter().any(|(k, _)| k == "U0"));
    assert!(comp_nmos.properties.iter().any(|(k, _)| k == "RMSE"));
    assert!(wizard.status_msg.contains("Applied model"));

    // 2. Apply EKV to another component
    wizard.set_model_kind(DeviceModelKind::Ekv);
    wizard.load_sample_ekv_transfer();
    wizard.model_name = "EKV_DEVICE_V2".to_string();
    wizard.run_fitting();

    let mut comp_ekv = SchematicComponent::new(2, ComponentKind::Nmos, Pos2::new(200.0, 100.0), 2);
    wizard.apply_to_component(&mut comp_ekv);

    assert_eq!(comp_ekv.model_name, Some("EKV_DEVICE_V2".to_string()));
    assert_eq!(comp_ekv.value_str, "EKV_DEVICE_V2");
    assert!(comp_ekv.properties.iter().any(|(k, _)| k == "VTO"));
    assert!(comp_ekv.properties.iter().any(|(k, _)| k == "KP"));
    assert!(comp_ekv.properties.iter().any(|(k, _)| k == "GAMMA"));
    assert!(comp_ekv.properties.iter().any(|(k, _)| k == "THETA"));

    // 3. Apply BJT to BjtNpn component
    wizard.set_model_kind(DeviceModelKind::GummelPoonBjt);
    wizard.load_sample_bjt_forward_active();
    wizard.model_name = "BJT_RF_V3".to_string();
    wizard.run_fitting();

    let mut comp_bjt = SchematicComponent::new(3, ComponentKind::BjtNpn, Pos2::new(300.0, 100.0), 1);
    wizard.apply_to_component(&mut comp_bjt);

    assert_eq!(comp_bjt.model_name, Some("BJT_RF_V3".to_string()));
    assert_eq!(comp_bjt.value_str, "BJT_RF_V3");
    assert!(comp_bjt.properties.iter().any(|(k, _)| k == "IS"));
    assert!(comp_bjt.properties.iter().any(|(k, _)| k == "BF"));
    assert!(comp_bjt.properties.iter().any(|(k, _)| k == "VAF"));
}
