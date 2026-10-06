#![deny(unsafe_code)]

//! Analytical verification and test suite for Silicon Aging, Reliability & Electromigration Engine.
//!
//! Validates:
//! 1. BTI reaction-diffusion power-law growth, Arrhenius activation, and dynamic AC recovery.
//! 2. HCI lateral field impact ionization, substrate current, and transconductance degradation.
//! 3. TDDB Weibull cumulative failure distribution, area Poisson scaling, and FIT rates.
//! 4. Electromigration Black's equation, Joule self-heating, and Blech length immortality limits across M1-M15.
//! 5. Unified 10-year datacenter derating telemetry, frequency degradation, and guardband synthesis.

use phonon_solver::silicon_aging::{
    build_default_m1_to_m15_stack, calculate_bti_ac_recovery_factor, calculate_bti_dc_vth_shift,
    calculate_bti_effective_vth_shift, calculate_fit_rate, calculate_hci_gm_degradation,
    calculate_hci_vth_shift, calculate_mean_free_path_nm, calculate_progressive_gate_leakage_density,
    calculate_stage_delay_penalty, calculate_substrate_current,
    calculate_subthreshold_swing_degradation, calculate_t63_eta_sec,
    calculate_weibull_failure_probability, calculate_weibull_plot_w, evaluate_full_metal_stack,
    evaluate_metal_layer_em, generate_weibull_reliability_curve, BlacksEquationParams, BtiParams,
    HciParams, MetalLayerId, SiliconAgingCoSimulator, TddbParams, TransistorPolarity,
    HOURS_PER_YEAR, SECONDS_PER_YEAR,
};

#[test]
fn test_bti_reaction_diffusion_and_ac_recovery() {
    let params = BtiParams::default();
    let v_dd = 0.85;
    let temp_room_k = 298.15; // 25 C
    let temp_hot_k = 358.15;  // 85 C
    let temp_extreme_k = 378.15; // 105 C

    let one_year_sec = SECONDS_PER_YEAR;
    let ten_years_sec = 10.0 * SECONDS_PER_YEAR;

    // 1. Time growth: 10-year shift must exceed 1-year shift by approximately (10)^(1/6) ~ 1.467
    let shift_1yr = calculate_bti_dc_vth_shift(
        &params,
        v_dd,
        temp_hot_k,
        one_year_sec,
        TransistorPolarity::PmosNbti,
    );
    let shift_10yr = calculate_bti_dc_vth_shift(
        &params,
        v_dd,
        temp_hot_k,
        ten_years_sec,
        TransistorPolarity::PmosNbti,
    );

    assert!(shift_1yr > 0.010, "1-year shift should exceed 10 mV");
    assert!(shift_10yr > shift_1yr, "Shift must grow monotonically with time");
    let ratio = shift_10yr / shift_1yr;
    let expected_ratio = 10.0_f64.powf(params.time_exponent_n);
    assert!(
        (ratio - expected_ratio).abs() < 1.0e-3,
        "Time scaling ratio ({:.4}) must match expected t^n ({:.4})",
        ratio,
        expected_ratio
    );

    // 2. Arrhenius temperature acceleration
    let shift_room = calculate_bti_dc_vth_shift(
        &params,
        v_dd,
        temp_room_k,
        ten_years_sec,
        TransistorPolarity::PmosNbti,
    );
    let shift_extreme = calculate_bti_dc_vth_shift(
        &params,
        v_dd,
        temp_extreme_k,
        ten_years_sec,
        TransistorPolarity::PmosNbti,
    );

    assert!(
        shift_extreme > shift_10yr && shift_10yr > shift_room,
        "BTI shift must accelerate monotonically with temperature"
    );

    // 3. Polarity comparison: pMOS NBTI must exceed nMOS PBTI
    let pmos_shift = calculate_bti_dc_vth_shift(
        &params,
        v_dd,
        temp_hot_k,
        ten_years_sec,
        TransistorPolarity::PmosNbti,
    );
    let nmos_shift = calculate_bti_dc_vth_shift(
        &params,
        v_dd,
        temp_hot_k,
        ten_years_sec,
        TransistorPolarity::NmosPbti,
    );
    assert!(
        pmos_shift > 2.0 * nmos_shift,
        "pMOS NBTI shift ({:.3} V) must exceed nMOS PBTI shift ({:.3} V) by > 2x",
        pmos_shift,
        nmos_shift
    );

    // 4. Dynamic AC stress recovery
    let beta_dc = calculate_bti_ac_recovery_factor(1.0, params.time_exponent_n);
    let beta_ac_50 = calculate_bti_ac_recovery_factor(0.5, params.time_exponent_n);
    let beta_ac_20 = calculate_bti_ac_recovery_factor(0.2, params.time_exponent_n);

    assert_eq!(beta_dc, 1.0, "DC stress recovery factor must be 1.0");
    assert!(
        beta_ac_50 < 0.75 && beta_ac_50 > 0.55,
        "AC 50% duty cycle factor ({:.3}) should yield ~30-40% recovery reduction",
        beta_ac_50
    );
    assert!(
        beta_ac_20 < beta_ac_50,
        "Lower duty cycle must yield lower effective degradation"
    );

    let effective_ac = calculate_bti_effective_vth_shift(
        &params,
        v_dd,
        temp_hot_k,
        ten_years_sec,
        0.50,
        TransistorPolarity::PmosNbti,
    );
    assert!(effective_ac < pmos_shift);

    // 5. Stage delay penalty
    let delay_penalty = calculate_stage_delay_penalty(effective_ac, v_dd, params.v_th0);
    assert!(delay_penalty > 0.0 && delay_penalty < 30.0);
}

#[test]
fn test_hci_impact_ionization_and_transconductance() {
    let params = HciParams::default();
    let temp_k = 358.15; // 85 C

    // 1. Substrate current behavior in linear vs saturation
    let i_sub_subthreshold = calculate_substrate_current(&params, 0.20, 0.85, temp_k);
    let i_sub_saturation = calculate_substrate_current(&params, 0.85, 0.85, temp_k);

    assert!(
        i_sub_saturation > 1.0e-7,
        "Saturation substrate current ({:.2e} A) should be measurable",
        i_sub_saturation
    );
    assert!(
        i_sub_saturation > 1000.0 * i_sub_subthreshold,
        "Saturation substrate current must drastically exceed linear regime"
    );

    // 2. Mean free path temperature dependence
    let lambda_300 = calculate_mean_free_path_nm(&params, 300.0);
    let lambda_400 = calculate_mean_free_path_nm(&params, 400.0);
    assert!(
        lambda_300 > lambda_400,
        "Carrier mean free path must decrease at higher temperature due to phonon scattering"
    );

    // 3. HCI V_th shift over time
    let time_1yr = SECONDS_PER_YEAR;
    let time_10yr = 10.0 * SECONDS_PER_YEAR;
    let hci_1yr = calculate_hci_vth_shift(&params, 0.85, 0.425, temp_k, time_1yr);
    let hci_10yr = calculate_hci_vth_shift(&params, 0.85, 0.425, temp_k, time_10yr);

    assert!(hci_10yr > hci_1yr, "HCI shift must accumulate with time");
    assert!(hci_10yr < 0.10, "HCI shift should remain physical (< 100 mV)");

    // 4. Transconductance loss and subthreshold swing degradation
    let delta_gm_pct = calculate_hci_gm_degradation(hci_10yr, &params);
    let delta_ss = calculate_subthreshold_swing_degradation(hci_10yr, temp_k, 1.15);

    assert!(delta_gm_pct > 0.0 && delta_gm_pct < 25.0);
    assert!(delta_ss > 0.0 && delta_ss < 20.0);
}

#[test]
fn test_tddb_weibull_statistics_and_area_scaling() {
    let params = TddbParams::default();
    let v_dd = 0.85;
    let temp_k = 358.15; // 85 C

    // 1. Reference structure vs chip-level area scaling
    let a_ref = params.reference_area_um2; // 10 um^2
    let a_chip = params.chip_gate_area_mm2 * 1.0e6; // 45 mm^2 = 4.5e7 um^2

    let eta_ref = calculate_t63_eta_sec(&params, v_dd, temp_k, a_ref);
    let eta_chip = calculate_t63_eta_sec(&params, v_dd, temp_k, a_chip);

    assert!(
        eta_chip < eta_ref,
        "Chip-level characteristic lifetime must be shorter than test structure due to defect area scaling"
    );
    let area_ratio = a_chip / a_ref;
    let expected_scale = (1.0 / area_ratio).powf(1.0 / params.weibull_beta);
    let measured_scale = eta_chip / eta_ref;
    assert!(
        (measured_scale - expected_scale).abs() < 1.0e-3 * expected_scale,
        "Area scaling must follow Poisson percolation exponent 1/beta"
    );

    // 2. Cumulative failure probability
    let time_10yr_sec = 10.0 * SECONDS_PER_YEAR;
    let f_10yr = calculate_weibull_failure_probability(time_10yr_sec, eta_chip, params.weibull_beta);
    assert!(f_10yr >= 0.0 && f_10yr < 0.05, "10-year chip failure prob should be < 5%");

    // 3. FIT rate
    let mission_hours = 10.0 * HOURS_PER_YEAR;
    let fit = calculate_fit_rate(f_10yr, mission_hours);
    assert!(fit >= 0.0 && fit < 1000.0, "FIT rate must be well-bounded");

    // 4. Weibull plot linearization
    let w = calculate_weibull_plot_w(f_10yr);
    assert!(w.is_finite());

    // 5. Pre-breakdown progressive gate leakage
    let initial_leak = params.initial_leakage_density_na_per_um2;
    let degraded_leak = calculate_progressive_gate_leakage_density(&params, time_10yr_sec, eta_chip);
    assert!(
        degraded_leak > initial_leak,
        "Gate leakage density must increase with dielectric wear-out"
    );

    // 6. Multi-point Weibull reliability curve
    let (probs, ws) = generate_weibull_reliability_curve(&params, v_dd, temp_k, &[time_10yr_sec]);
    assert_eq!(probs.len(), 1);
    assert_eq!(ws.len(), 1);
}

#[test]
fn test_electromigration_blacks_equation_and_blech_immortality() {
    let stack = build_default_m1_to_m15_stack();
    assert_eq!(stack.len(), 15, "Metal stack must contain exactly 15 layers (M1..M15)");

    let params = BlacksEquationParams::default();
    let temp_sub_k = 358.15; // 85 C

    // Test single layer evaluation
    let single_em = evaluate_metal_layer_em(&stack[0], &params, temp_sub_k);
    assert_eq!(single_em.layer_id, MetalLayerId::M1);

    let results = evaluate_full_metal_stack(&stack, &params, temp_sub_k);
    assert_eq!(results.len(), 15);

    // Verify fine-pitch local metal layers (M1, M2) benefit from Blech length short-wire arrest
    let m1 = &results[0];
    assert_eq!(m1.layer_id, MetalLayerId::M1);
    assert!(
        m1.is_blech_immortal,
        "M1 short segment ({:.1} um) must satisfy Blech immortality limit",
        stack[0].length_um
    );
    assert!(m1.mttf_years > 1000.0, "Immortal wire has effectively infinite MTTF");

    // Verify global power layers have significant current and Joule self-heating
    let m15 = &results[14];
    assert_eq!(m15.layer_id, MetalLayerId::M15);
    assert!(m15.current_density_a_per_cm2 > 1.0e5);
    assert!(m15.joule_heating_delta_t_k > 0.01);
    assert!(m15.effective_metal_temp_k > temp_sub_k);
    assert!(m15.mttf_years >= 10.0, "M15 must meet 10-year MTTF requirement");
}

#[test]
fn test_silicon_aging_co_simulator_10yr_datacenter_derating() {
    let sim = SiliconAgingCoSimulator::new_fast();

    // 1. Verify snapshot evaluation
    let snap_1yr = sim.evaluate_snapshot(1.0);
    let snap_10yr = sim.evaluate_snapshot(10.0);

    assert!(snap_10yr.total_vth_shift_mv > snap_1yr.total_vth_shift_mv);
    assert!(snap_10yr.frequency_degradation_pct > snap_1yr.frequency_degradation_pct);
    assert!(snap_10yr.operating_frequency_ghz < sim.nominal_freq_ghz);

    // 2. Telemetry report
    let report = sim.generate_telemetry_report();
    assert_eq!(report.v_dd_nominal_v, 0.85);
    assert_eq!(report.nominal_freq_ghz, 3.50);
    assert!(report.ten_year_total_shift_mv > 15.0 && report.ten_year_total_shift_mv < 85.0);
    assert!(report.recommended_guardband_mv > 0.0);
    assert!(report.min_em_mttf_years >= 10.0);
    assert!(report.is_10yr_qualified, "Default configuration must pass 10-year sign-off");

    // 3. Multi-year curve generation
    let curves = sim.generate_time_curves(25);
    assert_eq!(curves.time_years.len(), 25);
    assert_eq!(curves.total_shifts_mv.len(), 25);
    assert_eq!(curves.freq_ghz.len(), 25);
    assert_eq!(curves.required_guardband_mv.len(), 25);

    // Time points must be strictly increasing
    for i in 1..25 {
        assert!(curves.time_years[i] > curves.time_years[i - 1]);
        assert!(curves.total_shifts_mv[i] >= curves.total_shifts_mv[i - 1]);
    }
}
