//! Integration Test: TCAD First-Principles 1D P-N Junction Diode Synthesis.
//!
//! Validates:
//! - Physical silicon p-n diode synthesized from doping profiles (Na = 1e17 cm^-3, Nd = 1e16 cm^-3)
//! - Microscopic Poisson-Drift-Diffusion solution with Scharfetter-Gummel discretization
//! - Analytical built-in potential verification: V_bi = V_t * ln(Na * Nd / ni^2)
//! - Forward exponential conduction and reverse blocking characteristics.

use phonon_core::constants::T_REF;
use phonon_models::tcad::{MaterialProperties, TcadDeviceBuilder};

#[test]
fn test_tcad_pn_junction_synthesis_and_characteristics() {
    let temp_k = T_REF;
    let mat = MaterialProperties::silicon();
    let vt = MaterialProperties::thermal_voltage(temp_k);
    let ni = mat.intrinsic_carrier_concentration(temp_k);

    let na = 1.0e23; // 1e17 cm^-3
    let nd = 1.0e22; // 1e16 cm^-3
    let area = 1.0e-8; // 100 um x 100 um = 1e-8 m^2
    let length = 2.0e-6; // 2 um length

    // 1. Synthesize physical TCAD diode from scratch
    let diode = TcadDeviceBuilder::new_pn_junction("D_physical")
        .length(length)
        .cross_section_area(area)
        .p_doping(na)
        .n_doping(nd)
        .mesh_points(60)
        .build();

    // 2. Verify Analytical Built-In Potential V_bi
    let expected_vbi = vt * (na * nd / (ni * ni)).ln();
    println!("Theoretical V_bi = {:.4} V", expected_vbi);

    // In thermal equilibrium (0V applied), potential step across junction
    let eq_state = diode
        .solver
        .initialize_equilibrium(&diode.mesh, &mat, temp_k);
    let p_side_psi = eq_state.psi[0];
    let n_side_psi = *eq_state.psi.last().unwrap();
    let simulated_vbi = n_side_psi - p_side_psi;

    println!(
        "Simulated V_bi: N-side ({:.4}V) - P-side ({:.4}V) = {:.4} V",
        n_side_psi, p_side_psi, simulated_vbi
    );

    let vbi_error = (simulated_vbi - expected_vbi).abs() / expected_vbi;
    assert!(
        vbi_error < 0.05,
        "V_bi discrepancy too large: sim = {:.4} V, expected = {:.4} V (rel err: {:.2}%)",
        simulated_vbi,
        expected_vbi,
        vbi_error * 100.0
    );

    // 3. Forward Bias Sweep: 0.1 V to 0.5 V
    let (i_01, g_01) = diode.evaluate_diode(0.10, temp_k);
    let (i_02, g_02) = diode.evaluate_diode(0.20, temp_k);
    let (i_03, g_03) = diode.evaluate_diode(0.30, temp_k);
    let (i_04, g_04) = diode.evaluate_diode(0.40, temp_k);

    println!(
        "Forward Bias:\n  V=0.10V: I={:.3e} A, g={:.3e} S\n  V=0.20V: I={:.3e} A, g={:.3e} S\n  V=0.30V: I={:.3e} A, g={:.3e} S\n  V=0.40V: I={:.3e} A, g={:.3e} S",
        i_01, g_01, i_02, g_02, i_03, g_03, i_04, g_04
    );

    assert!(
        i_02 > i_01,
        "Current must increase monotonically with forward bias"
    );
    assert!(
        i_03 > i_02,
        "Current must increase monotonically with forward bias"
    );
    assert!(
        i_04 > i_03,
        "Current must increase monotonically with forward bias"
    );
    assert!(
        g_01 > 0.0 && g_04 > 0.0,
        "Conductance must be strictly positive"
    );

    // 4. Reverse Bias: -1.0 V
    let (i_rev, g_rev) = diode.evaluate_diode(-1.0, temp_k);
    println!(
        "Reverse Bias:\n  V=-1.0V: I={:.3e} A, g={:.3e} S",
        i_rev, g_rev
    );

    assert!(
        i_rev.abs() < 1e-8,
        "Reverse leakage current must remain minimal (< 10 nA)"
    );
    assert!(g_rev > 0.0, "Dynamic conductance must remain non-negative");
}
