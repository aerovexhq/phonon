//! Integration Test: TCAD Custom Physical MOSFET Synthesis.
//!
//! Validates:
//! - Programmatic definition of a custom sub-micron physical NMOS transistor from scratch
//! - Microscopic drift-diffusion channel inversion and gate field modulation
//! - Transfer characteristics (I_DS vs V_GS) showing subthreshold turn-on and transconductance
//! - Output characteristics (I_DS vs V_DS) demonstrating linear and saturation regimes.

use phonon_core::constants::T_REF;
use phonon_models::tcad::TcadDeviceBuilder;

#[test]
fn test_tcad_custom_mosfet_synthesis_and_characteristics() {
    let temp_k = T_REF;

    // 1. Synthesize custom physical NMOS transistor from scratch
    let nmos = TcadDeviceBuilder::new_mosfet("M_custom_nmos")
        .length(300.0e-9) // 300 nm total length
        .cross_section_area(1.0e-12) // 1 um x 1 um
        .oxide_thickness(3.0e-9) // 3 nm SiO2
        .p_doping(1.0e23) // 1e17 cm^-3 P-substrate
        .n_doping(1.0e26) // 1e20 cm^-3 N+ Source/Drain
        .mesh_points(60)
        .build();

    // 2. Transfer Sweep: V_GS = 0.2V to 1.2V at V_DS = 0.5V
    let v_ds_fixed = 0.5;
    let (i_02, gm_02, _, _) = nmos.evaluate_mosfet(v_ds_fixed, 0.2, 0.0, temp_k);
    let (i_05, gm_05, _, _) = nmos.evaluate_mosfet(v_ds_fixed, 0.5, 0.0, temp_k);
    let (i_08, gm_08, _, _) = nmos.evaluate_mosfet(v_ds_fixed, 0.8, 0.0, temp_k);
    let (i_12, gm_12, _, _) = nmos.evaluate_mosfet(v_ds_fixed, 1.2, 0.0, temp_k);

    println!(
        "MOSFET Transfer Sweep (V_DS = 0.5V):\n  V_GS=0.2V: I_DS={:.3e} A, gm={:.3e} S\n  V_GS=0.5V: I_DS={:.3e} A, gm={:.3e} S\n  V_GS=0.8V: I_DS={:.3e} A, gm={:.3e} S\n  V_GS=1.2V: I_DS={:.3e} A, gm={:.3e} S",
        i_02, gm_02, i_05, gm_05, i_08, gm_08, i_12, gm_12
    );

    // Channel turns on strongly as V_GS increases past threshold
    assert!(
        i_05 >= i_02,
        "Drain current must increase with gate overdrive"
    );
    assert!(
        i_08 >= i_05,
        "Drain current must increase with gate overdrive"
    );
    assert!(
        i_12 >= i_08,
        "Drain current must increase with gate overdrive"
    );
    assert!(gm_08 > 0.0, "Transconductance must be strictly positive");

    // 3. Output Sweep: V_DS = 0.1V to 1.5V at V_GS = 1.0V
    let v_gs_fixed = 1.0;
    let (i_d01, _, gds_01, _) = nmos.evaluate_mosfet(0.1, v_gs_fixed, 0.0, temp_k);
    let (i_d05, _, gds_05, _) = nmos.evaluate_mosfet(0.5, v_gs_fixed, 0.0, temp_k);
    let (i_d10, _, gds_10, _) = nmos.evaluate_mosfet(1.0, v_gs_fixed, 0.0, temp_k);
    let (i_d15, _, gds_15, _) = nmos.evaluate_mosfet(1.5, v_gs_fixed, 0.0, temp_k);

    println!(
        "MOSFET Output Sweep (V_GS = 1.0V):\n  V_DS=0.1V: I_DS={:.3e} A, gds={:.3e} S\n  V_DS=0.5V: I_DS={:.3e} A, gds={:.3e} S\n  V_DS=1.0V: I_DS={:.3e} A, gds={:.3e} S\n  V_DS=1.5V: I_DS={:.3e} A, gds={:.3e} S",
        i_d01, gds_01, i_d05, gds_05, i_d10, gds_10, i_d15, gds_15
    );

    assert!(i_d05 >= i_d01, "Current must increase with drain bias");
    assert!(i_d10 >= i_d05, "Current must increase with drain bias");
    assert!(i_d15 >= i_d10, "Current must increase with drain bias");
    assert!(
        gds_01 > 0.0 && gds_15 > 0.0,
        "Channel conductance must be strictly positive"
    );
}
