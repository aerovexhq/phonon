//! Integration tests for Multi-Valued Logic (MVL) physical semiconductor devices
//! and static noise margins (SNM).
//!
//! Validates:
//! 1. Multi-threshold MOSFET gate workfunction tuning (\(V_{th}\) shifts across 6 flavors).
//! 2. Multi-peak resonant tunneling diode (RTD) dual negative differential conductance (NDR)
//!    and 3-state stable operating point solution.
//! 3. Chirality-tuned Carbon Nanotube FET (CNTFET) bandgap and threshold scaling.
//! 4. 3-state Simple Ternary Inverter (STI) Voltage Transfer Characteristic (VTC) and TSNM extraction.
//! 5. Static state thermal retention stability across temperature.

use phonon_models::mvl::{
    CntfetTernaryModel, MosfetFlavor, MultiPeakRtdModel, MultiPeakRtdParams, MultiThresholdMosfet,
    MultiThresholdMosfetParams, TernaryNoiseMarginAnalyzer,
};

#[test]
fn test_mvl_multi_threshold_mosfet_flavors() {
    let eot = 0.9e-9;
    let l_g = 18e-9;
    let w_ch = 120e-9;
    let temp = 300.0;

    // NMOS Flavors
    let n_low = MultiThresholdMosfet::new(
        MultiThresholdMosfetParams::from_flavor(MosfetFlavor::LowN, eot, l_g, w_ch),
        temp,
    );
    let n_std = MultiThresholdMosfet::new(
        MultiThresholdMosfetParams::from_flavor(MosfetFlavor::StandardN, eot, l_g, w_ch),
        temp,
    );
    let n_high = MultiThresholdMosfet::new(
        MultiThresholdMosfetParams::from_flavor(MosfetFlavor::HighN, eot, l_g, w_ch),
        temp,
    );

    // PMOS Flavors
    let p_low = MultiThresholdMosfet::new(
        MultiThresholdMosfetParams::from_flavor(MosfetFlavor::LowP, eot, l_g, w_ch),
        temp,
    );
    let p_std = MultiThresholdMosfet::new(
        MultiThresholdMosfetParams::from_flavor(MosfetFlavor::StandardP, eot, l_g, w_ch),
        temp,
    );
    let p_high = MultiThresholdMosfet::new(
        MultiThresholdMosfetParams::from_flavor(MosfetFlavor::HighP, eot, l_g, w_ch),
        temp,
    );

    let eval_n_low = n_low.evaluate(0.45, 0.9, 0.0, temp);
    let eval_n_std = n_std.evaluate(0.45, 0.9, 0.0, temp);
    let eval_n_high = n_high.evaluate(0.45, 0.9, 0.0, temp);

    // Verify monotonic threshold ordering engineered via workfunction tuning
    assert!(
        eval_n_low.v_th < eval_n_std.v_th,
        "Low-Vth NMOS must have smaller threshold than Standard"
    );
    assert!(
        eval_n_std.v_th < eval_n_high.v_th,
        "Standard NMOS must have smaller threshold than High-Vth"
    );

    // At intermediate voltage (0.45 V = Vdd/2), LowN conducts significantly more than HighN
    assert!(
        eval_n_low.ids > eval_n_high.ids * 2.0,
        "Low-Vth NMOS must conduct substantially more than High-Vth at half-rail"
    );

    // Check PMOS magnitudes
    let eval_p_low = p_low.evaluate(-0.45, -0.9, 0.0, temp);
    let eval_p_std = p_std.evaluate(-0.45, -0.9, 0.0, temp);
    let eval_p_high = p_high.evaluate(-0.45, -0.9, 0.0, temp);

    assert!(eval_p_low.v_th.abs() < eval_p_std.v_th.abs());
    assert!(eval_p_std.v_th.abs() < eval_p_high.v_th.abs());
}

#[test]
fn test_mvl_multi_peak_rtd_characteristics_and_states() {
    let rtd = MultiPeakRtdModel::new(MultiPeakRtdParams::default());
    let (pvcr1, pvcr2) = rtd.pvcr();

    // Verify strong Peak-to-Valley Current Ratios
    assert!(pvcr1 >= 7.0, "PVCR1 must be >= 7.0, actual: {pvcr1}");
    assert!(pvcr2 >= 7.0, "PVCR2 must be >= 7.0, actual: {pvcr2}");

    // Verify negative conductance regions (NDR)
    let ev_p1 = rtd.evaluate(0.22, 300.0);
    let ev_ndr1 = rtd.evaluate(0.30, 300.0);
    let ev_v1 = rtd.evaluate(0.38, 300.0);
    let ev_p2 = rtd.evaluate(0.58, 300.0);
    let ev_ndr2 = rtd.evaluate(0.66, 300.0);
    let ev_v2 = rtd.evaluate(0.74, 300.0);
    let ev_post = rtd.evaluate(0.85, 300.0);

    // First resonance peak and valley
    assert!(ev_p1.current > ev_v1.current);
    assert!(ev_ndr1.is_ndr, "Must be NDR in branch 1-2");
    assert!(ev_ndr1.conductance < 0.0);

    // Second resonance peak and valley
    assert!(ev_p2.current > ev_v2.current);
    assert!(ev_ndr2.is_ndr, "Must be NDR in branch 3-4");
    assert!(ev_ndr2.conductance < 0.0);

    // Post-valley positive conduction
    assert!(!ev_post.is_ndr);
    assert!(ev_post.conductance > 0.0);

    // Solve for stable states on 800 ohm load line at 1.0 V
    let states = rtd.solve_stable_states(1.0, 800.0, 300.0);
    assert_eq!(
        states.len(),
        3,
        "Cascaded RTD must produce exactly 3 stable equilibrium states, got: {states:?}"
    );

    // State 0: Low voltage (< 0.3 V)
    assert!(states[0] < 0.30);
    // State 1: Intermediate plateau (0.35 V to 0.55 V)
    assert!(states[1] > 0.35 && states[1] < 0.55);
    // State 2: High voltage (> 0.70 V)
    assert!(states[2] > 0.70);
}

#[test]
fn test_mvl_cntfet_chirality_threshold_tuning() {
    let cnt_low = CntfetTernaryModel::low_vth();
    let cnt_std = CntfetTernaryModel::standard_vth();
    let cnt_high = CntfetTernaryModel::high_vth();

    // Verify inverse relationship between diameter and bandgap
    assert!(cnt_low.diameter_m > cnt_std.diameter_m);
    assert!(cnt_std.diameter_m > cnt_high.diameter_m);

    assert!(cnt_low.bandgap_ev < cnt_std.bandgap_ev);
    assert!(cnt_std.bandgap_ev < cnt_high.bandgap_ev);

    // Verify pristine threshold voltages
    assert!(cnt_low.v_th > 0.20 && cnt_low.v_th < 0.35);
    assert!(cnt_std.v_th > 0.35 && cnt_std.v_th < 0.48);
    assert!(cnt_high.v_th > 0.48 && cnt_high.v_th < 0.65);
}

#[test]
fn test_mvl_ternary_noise_margin_and_thermal_retention() {
    let analyzer = TernaryNoiseMarginAnalyzer::new(0.9);
    let margins = analyzer
        .compute_sti_margins(400)
        .expect("Must extract noise margins from STI VTC");

    // All four multi-state noise margins must exceed 75 mV
    assert!(
        margins.nm_l0 > 0.075,
        "NM_L0 must be > 75 mV, got: {}",
        margins.nm_l0
    );
    assert!(
        margins.nm_h0 > 0.075,
        "NM_H0 must be > 75 mV, got: {}",
        margins.nm_h0
    );
    assert!(
        margins.nm_l1 > 0.075,
        "NM_L1 must be > 75 mV, got: {}",
        margins.nm_l1
    );
    assert!(
        margins.nm_h1 > 0.075,
        "NM_H1 must be > 75 mV, got: {}",
        margins.nm_h1
    );

    // Overall TSNM must be >= 80 mV
    assert!(
        margins.tsnm >= 0.080,
        "TSNM must be >= 80 mV, got: {}",
        margins.tsnm
    );

    // Evaluate thermal retention from 250 K (cryogenic-edge) to 380 K (hot junction)
    let rep_250 = analyzer.evaluate_thermal_retention(margins.tsnm, 250.0);
    let rep_300 = analyzer.evaluate_thermal_retention(margins.tsnm, 300.0);
    let rep_380 = analyzer.evaluate_thermal_retention(margins.tsnm, 380.0);

    assert!(rep_250.is_stable);
    assert!(rep_300.is_stable);
    assert!(rep_300.safety_factor > 1.05);

    // Physical sanity: safety factor must decrease monotonically as temperature increases
    assert!(rep_250.safety_factor > rep_300.safety_factor);
    assert!(rep_300.safety_factor > rep_380.safety_factor);
}
