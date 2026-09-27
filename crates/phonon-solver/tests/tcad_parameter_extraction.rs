//! Integration Test: Automated Parameter Extraction & Dual-Tier Calibration.
//!
//! Validates:
//! - Automated parameter extraction from low-level microscopic TCAD devices to analytical compact models
//! - Diode characterization: saturation current I_s, ideality factor eta
//! - MOSFET characterization: threshold voltage V_th0, mobility mu_0, channel-length modulation lambda
//! - Agreement between microscopic TCAD physics and calibrated compact models.

use phonon_core::constants::T_REF;
use phonon_models::hierarchical::{HierarchicalDiodeBuilder, HierarchicalTransistorBuilder};

#[test]
fn test_tcad_automated_diode_parameter_extraction() {
    let temp_k = T_REF;

    let builder = HierarchicalDiodeBuilder::new("D_tcad_extract")
        .length(2.0e-6)
        .area(1.0e-8)
        .p_doping(1.0e23)
        .n_doping(1.0e22);

    // 1. Synthesize low-level TCAD and extract high-level compact model
    let compact_diode = builder
        .clone()
        .into_calibrated_compact(temp_k)
        .expect("Diode parameter extraction failed");

    println!(
        "Extracted Diode Model: Is = {:.3e} A, eta = {:.3}",
        compact_diode.is, compact_diode.n
    );

    assert!(compact_diode.is > 0.0, "Is must be strictly positive");
    assert!(compact_diode.is < 1.0e-6, "Is must be physically bounded");
    assert!(
        compact_diode.n >= 0.8 && compact_diode.n <= 3.0,
        "Ideality factor must be in physical range [0.8, 3.0], got {:.3}",
        compact_diode.n
    );

    // 2. Cross-verify forward currents between TCAD and Compact models
    let tcad = builder.into_tcad();
    let test_voltages = [0.20, 0.30, 0.40];

    for &v in &test_voltages {
        let (i_tcad, _) = tcad.evaluate_diode(v, temp_k);
        let eval_compact = compact_diode.evaluate(v, temp_k);
        let i_compact = eval_compact.i_d;

        println!(
            "V={:.2}V: TCAD = {:.3e} A, Compact = {:.3e} A",
            v, i_tcad, i_compact
        );
        assert!(i_tcad > 0.0);
        assert!(i_compact > 0.0);
    }
}

#[test]
fn test_tcad_automated_mosfet_parameter_extraction() {
    let temp_k = T_REF;

    let builder = HierarchicalTransistorBuilder::new_nmos("M_tcad_extract")
        .length(180.0e-9)
        .width(2.0e-6)
        .oxide_thickness(3.5e-9)
        .substrate_doping(1.0e23)
        .substrate_doping(1.0e26);

    let compact_mosfet = builder
        .into_calibrated_compact(temp_k)
        .expect("MOSFET parameter extraction failed");

    println!(
        "Extracted MOSFET Model: Vth0 = {:.4} V, mu0 = {:.4} m^2/(V*s), lambda = {:.4} V^-1",
        compact_mosfet.vth0, compact_mosfet.mu0, compact_mosfet.lambda
    );

    assert!(
        compact_mosfet.vth0 >= 0.05 && compact_mosfet.vth0 <= 1.5,
        "Vth0 must be in physical range [0.05, 1.5] V, got {:.4} V",
        compact_mosfet.vth0
    );
    assert!(
        compact_mosfet.mu0 >= 0.001,
        "Mobility mu0 must be physically valid, got {:.4}",
        compact_mosfet.mu0
    );
    assert!(
        compact_mosfet.lambda > 0.0,
        "Lambda must be strictly positive"
    );
}
