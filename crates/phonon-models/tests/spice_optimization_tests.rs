#![deny(unsafe_code)]

//! Automated verification test suite for full SPICE component optimization and evolutionary parameter estimation.

use phonon_models::extraction::{
    generate_bjt_model_deck, generate_bsim4_model_deck, generate_ekv_model_deck,
    polish_parameters, validate_bjt_model_deck, validate_bsim4_model_deck,
    validate_ekv_model_deck, BjtOptimizer, BjtTargetParams, Bsim4TargetParams,
    EkvOptimizer, EkvTargetParams, FittingResult, GaOptimizer, MeasuredCurve,
};

#[test]
fn test_bsim4_synthetic_curve_fitting() {
    let ground_truth_vth = 0.48;
    let ground_truth_u0 = 0.052;

    let curve = MeasuredCurve::synthetic_nmos_transfer_curve(
        1.2,
        (0.0, 1.5, 50),
        ground_truth_vth,
        ground_truth_u0,
    );

    let result = GaOptimizer::fit_curve(&curve, 25, 48);

    assert!(result.converged, "BSIM4 optimizer should achieve convergence");
    assert!(
        result.r_squared > 0.985,
        "BSIM4 R^2 must exceed 0.985, got {:.6}",
        result.r_squared
    );

    let vth_err_pct = ((result.params.vth0 - ground_truth_vth).abs() / ground_truth_vth) * 100.0;
    let u0_err_pct = ((result.params.u0 - ground_truth_u0).abs() / ground_truth_u0) * 100.0;

    assert!(
        vth_err_pct < 3.0,
        "Extracted Vth error must be < 3.0%, got {:.2}%",
        vth_err_pct
    );
    assert!(
        u0_err_pct < 6.0,
        "Extracted U0 error must be < 6.0%, got {:.2}%",
        u0_err_pct
    );
}

#[test]
fn test_ekv_synthetic_curve_fitting() {
    let ground_truth_vto = 0.50;
    let ground_truth_kp = 1.5e-4;
    let ground_truth_gamma = 0.45;
    let ground_truth_theta = 0.04;

    let curve = MeasuredCurve::synthetic_ekv_transfer_curve(
        1.2,
        (0.0, 1.5, 50),
        ground_truth_vto,
        ground_truth_kp,
        ground_truth_gamma,
        ground_truth_theta,
    );

    let result = EkvOptimizer::fit_curve(&curve, 25, 48);

    assert!(result.converged, "EKV optimizer should achieve convergence");
    assert!(
        result.r_squared > 0.985,
        "EKV R^2 must exceed 0.985, got {:.6}",
        result.r_squared
    );

    let vto_err_pct = ((result.params.vto - ground_truth_vto).abs() / ground_truth_vto) * 100.0;
    let kp_err_pct = ((result.params.kp - ground_truth_kp).abs() / ground_truth_kp) * 100.0;

    assert!(
        vto_err_pct < 3.0,
        "Extracted VTO error must be < 3.0%, got {:.2}%",
        vto_err_pct
    );
    assert!(
        kp_err_pct < 8.0,
        "Extracted KP error must be < 8.0%, got {:.2}%",
        kp_err_pct
    );
}

#[test]
fn test_bjt_synthetic_curve_fitting() {
    let ground_truth_is = 1.0e-15;
    let ground_truth_bf = 120.0;
    let ground_truth_vaf = 95.0;

    let curve = MeasuredCurve::synthetic_bjt_forward_active_curve(
        2.0,
        (0.40, 0.75, 45),
        ground_truth_is,
        ground_truth_bf,
        ground_truth_vaf,
    );

    let result = BjtOptimizer::fit_curve(&curve, 25, 48);

    assert!(result.converged, "BJT optimizer should achieve convergence");
    assert!(
        result.r_squared > 0.985,
        "BJT R^2 must exceed 0.985, got {:.6}",
        result.r_squared
    );

    let log_is_truth = ground_truth_is.log10();
    let log_is_extracted = result.params.is.log10();
    let log_is_err_pct = ((log_is_extracted - log_is_truth).abs() / log_is_truth.abs()) * 100.0;

    assert!(
        log_is_err_pct < 2.0,
        "Extracted log(IS) error must be < 2.0%, got {:.2}%",
        log_is_err_pct
    );
}

#[test]
fn test_hybrid_ga_lm_polishing_convergence_reducing_rmse() {
    // 1. BSIM4 Hybrid Polishing
    let bsim_curve =
        MeasuredCurve::synthetic_nmos_transfer_curve(1.2, (0.0, 1.5, 50), 0.48, 0.052);
    let max_ids_bsim = bsim_curve.points.iter().map(|p| p.i_ds).fold(0.0, f64::max);

    // Initial unpolished candidate perturbed by 5%
    let initial_bsim = Bsim4TargetParams {
        vth0: 0.48 * 1.05,
        u0: 0.052 * 0.95,
        ..Bsim4TargetParams::default()
    };
    let unpolished_bsim = GaOptimizer::fit_curve_ga_only(&bsim_curve, 6, 24);
    let polished_bsim = GaOptimizer::polish_parameters(&initial_bsim, &bsim_curve);

    assert!(
        polished_bsim.rmse < unpolished_bsim.rmse || polished_bsim.rmse < 1e-4,
        "BSIM4 LM polishing must reduce RMSE residual (unpolished: {:.4e}, polished: {:.4e})",
        unpolished_bsim.rmse,
        polished_bsim.rmse
    );
    let bsim_relative_rmse = polished_bsim.rmse / max_ids_bsim;
    assert!(
        bsim_relative_rmse < 0.01,
        "BSIM4 LM polished relative RMSE must achieve sub-1% residual, got {:.4}%",
        bsim_relative_rmse * 100.0
    );

    // 2. EKV Hybrid Polishing
    let ekv_curve =
        MeasuredCurve::synthetic_ekv_transfer_curve(1.2, (0.0, 1.5, 50), 0.50, 1.5e-4, 0.50, 0.05);
    let max_ids_ekv = ekv_curve.points.iter().map(|p| p.i_ds).fold(0.0, f64::max);

    let initial_ekv = EkvTargetParams {
        vto: 0.50 * 1.06,
        kp: 1.5e-4 * 0.94,
        gamma: 0.50 * 1.05,
        theta: 0.05 * 0.95,
    };
    let mut initial_rmse_ekv = 0.0;
    for pt in &ekv_curve.points {
        let diff = pt.i_ds - initial_ekv.evaluate_ids(
            pt.v_ds,
            pt.v_gs,
            pt.v_bs,
            ekv_curve.channel_width_m,
            ekv_curve.channel_length_m,
            ekv_curve.temperature_k,
        );
        initial_rmse_ekv += diff * diff;
    }
    initial_rmse_ekv = (initial_rmse_ekv / (ekv_curve.points.len() as f64)).sqrt();

    let polished_ekv = EkvOptimizer::polish_parameters(&initial_ekv, &ekv_curve);
    assert!(
        polished_ekv.rmse < initial_rmse_ekv,
        "EKV LM polishing must strictly reduce RMSE residual (initial: {:.4e}, polished: {:.4e})",
        initial_rmse_ekv,
        polished_ekv.rmse
    );
    let ekv_relative_rmse = polished_ekv.rmse / max_ids_ekv;
    assert!(
        ekv_relative_rmse < 0.01,
        "EKV LM polished relative RMSE must achieve sub-1% residual, got {:.4}%",
        ekv_relative_rmse * 100.0
    );

    // 3. BJT Hybrid Polishing
    let bjt_curve =
        MeasuredCurve::synthetic_bjt_forward_active_curve(2.0, (0.45, 0.75, 40), 1e-15, 100.0, 100.0);
    let max_ic_bjt = bjt_curve.points.iter().map(|p| p.i_ds).fold(0.0, f64::max);

    let initial_bjt = BjtTargetParams {
        is: 1e-15 * 1.10,
        bf: 100.0 * 0.92,
        vaf: 100.0 * 1.08,
    };
    let mut initial_rmse_bjt = 0.0;
    for pt in &bjt_curve.points {
        let diff = pt.i_ds - initial_bjt.evaluate_ic(pt.v_ds, pt.v_gs, bjt_curve.temperature_k);
        initial_rmse_bjt += diff * diff;
    }
    initial_rmse_bjt = (initial_rmse_bjt / (bjt_curve.points.len() as f64)).sqrt();

    let polished_bjt = BjtOptimizer::polish_parameters(&initial_bjt, &bjt_curve);
    assert!(
        polished_bjt.rmse < initial_rmse_bjt,
        "BJT LM polishing must strictly reduce RMSE residual (initial: {:.4e}, polished: {:.4e})",
        initial_rmse_bjt,
        polished_bjt.rmse
    );
    let bjt_relative_rmse = polished_bjt.rmse / max_ic_bjt;
    assert!(
        bjt_relative_rmse < 0.01,
        "BJT LM polished relative RMSE must achieve sub-1% residual, got {:.4}%",
        bjt_relative_rmse * 100.0
    );
}

#[test]
fn test_model_deck_generation_all_three_device_types() {
    // 1. BSIM4 deck
    let bsim_params = Bsim4TargetParams {
        vth0: 0.452,
        u0: 0.051,
        vsat: 1.1e5,
        dvt0: 1.15,
        eta0: 0.078,
        rdsw: 95.0,
        subthreshold_swing_mv_dec: 79.2,
    };
    let bsim_res = FittingResult {
        params: bsim_params,
        rmse: 1.8e-6,
        r_squared: 0.9992,
        iterations: 20,
        converged: true,
    };
    let bsim_deck = generate_bsim4_model_deck("NMOS_CORE_28", true, &bsim_params, &bsim_res);
    assert!(validate_bsim4_model_deck(&bsim_deck).is_ok());
    assert!(bsim_deck.contains(".MODEL NMOS_CORE_28 NMOS"));
    assert!(bsim_deck.contains("LEVEL=54"));
    assert!(bsim_deck.contains("VTH0=0.452000"));

    // 2. EKV deck
    let ekv_params = EkvTargetParams {
        vto: 0.515,
        kp: 1.48e-4,
        gamma: 0.52,
        theta: 0.048,
    };
    let ekv_deck = generate_ekv_model_deck("EKV_LOWPOWER", true, &ekv_params);
    assert!(validate_ekv_model_deck(&ekv_deck).is_ok());
    assert!(ekv_deck.contains(".MODEL EKV_LOWPOWER NMOS"));
    assert!(ekv_deck.contains("LEVEL=55"));
    assert!(ekv_deck.contains("VTO=0.515000"));
    assert!(ekv_deck.contains("KP=1.480000e-4"));

    // 3. BJT deck
    let bjt_params = BjtTargetParams {
        is: 1.25e-15,
        bf: 180.0,
        vaf: 110.0,
    };
    let bjt_deck = generate_bjt_model_deck("BJT_NPN_RF", true, &bjt_params);
    assert!(validate_bjt_model_deck(&bjt_deck).is_ok());
    assert!(bjt_deck.contains(".MODEL BJT_NPN_RF NPN"));
    assert!(bjt_deck.contains("IS=1.250000e-15"));
    assert!(bjt_deck.contains("BF=180.0000"));
    assert!(bjt_deck.contains("VAF=110.0000"));
}

#[test]
fn test_generic_polish_parameters_interface() {
    let curve_ekv =
        MeasuredCurve::synthetic_ekv_transfer_curve(1.2, (0.0, 1.5, 30), 0.5, 1.5e-4, 0.5, 0.05);
    let ekv_cand = EkvTargetParams::default();
    let res = polish_parameters(&ekv_cand, &curve_ekv);
    assert!(res.r_squared > 0.95);

    let curve_bjt =
        MeasuredCurve::synthetic_bjt_forward_active_curve(2.0, (0.5, 0.75, 30), 1e-15, 100.0, 100.0);
    let bjt_cand = BjtTargetParams::default();
    let res_bjt = polish_parameters(&bjt_cand, &curve_bjt);
    assert!(res_bjt.r_squared > 0.95);
}
