#![deny(unsafe_code)]

//! Verification test suite for Phonon SPICE model parameter extraction and GA curve fitting.

use phonon_models::extraction::{
    generate_bsim4_model_deck, validate_bsim4_model_deck, Bsim4TargetParams, FittingResult,
    GaOptimizer, MeasuredCurve, MeasurementPoint,
};
use std::time::Instant;

#[test]
fn test_synthetic_transfer_curve_generation() {
    let v_ds = 1.2;
    let v_th = 0.48;
    let u0 = 0.052;
    let curve = MeasuredCurve::synthetic_nmos_transfer_curve(v_ds, (0.0, 1.5, 50), v_th, u0);

    assert_eq!(curve.points.len(), 50);
    assert_eq!(curve.temperature_k, 300.0);
    assert_eq!(curve.channel_width_m, 10e-6);
    assert_eq!(curve.channel_length_m, 100e-9);

    // Verify first and last bias points
    assert!((curve.points[0].v_gs - 0.0).abs() < 1e-6);
    assert!((curve.points[49].v_gs - 1.5).abs() < 1e-6);

    // Conduction in saturation should be monotonically increasing
    for i in 25..49 {
        assert!(
            curve.points[i + 1].i_ds >= curve.points[i].i_ds,
            "Transfer curve must increase monotonically above threshold: point {} vs {}",
            i,
            i + 1
        );
    }

    // Gate capacitance should be positive
    for pt in &curve.points {
        assert!(pt.c_gg.is_some());
        assert!(pt.c_gg.unwrap() > 0.0);
    }
}

#[test]
fn test_ga_parameter_recovery_vth_and_mobility() {
    let ground_truth_vth = 0.45;
    let ground_truth_u0 = 0.055;

    let curve = MeasuredCurve::synthetic_nmos_transfer_curve(
        0.1,
        (0.0, 1.5, 50),
        ground_truth_vth,
        ground_truth_u0,
    );

    let result = GaOptimizer::fit_curve(&curve, 25, 48);

    assert!(result.converged, "Optimizer should converge on synthetic data");
    assert!(
        result.r_squared > 0.99,
        "R^2 must exceed 0.99 on noise-free data, got {:.6}",
        result.r_squared
    );

    let vth_error_pct = ((result.params.vth0 - ground_truth_vth).abs() / ground_truth_vth) * 100.0;
    let u0_error_pct = ((result.params.u0 - ground_truth_u0).abs() / ground_truth_u0) * 100.0;

    println!(
        "Ground Truth: Vth = {:.4} V, U0 = {:.5} m^2/V-s | Extracted: Vth = {:.4} V ({:.2}% err), U0 = {:.5} ({:.2}% err) | R^2 = {:.6}",
        ground_truth_vth, ground_truth_u0, result.params.vth0, vth_error_pct, result.params.u0, u0_error_pct, result.r_squared
    );

    assert!(
        vth_error_pct < 2.0,
        "Vth error must be within 2.0%, got {:.2}%",
        vth_error_pct
    );
    assert!(
        u0_error_pct < 3.0,
        "U0 mobility error must be within 3.0%, got {:.2}%",
        u0_error_pct
    );
}

#[test]
fn test_output_curve_fitting_with_dibl() {
    let ground_truth_vth = 0.42;
    let ground_truth_u0 = 0.048;

    let curve = MeasuredCurve::synthetic_nmos_output_curve(
        1.0,
        (0.0, 1.8, 40),
        ground_truth_vth,
        ground_truth_u0,
    );

    let result = GaOptimizer::fit_curve(&curve, 25, 48);

    assert!(result.converged, "Optimizer should converge on output curve");
    assert!(
        result.r_squared > 0.985,
        "R^2 must exceed 0.985 on output curve, got {:.6}",
        result.r_squared
    );
    assert!(result.rmse < 1e-3, "RMSE must be small, got {:.6e}", result.rmse);
}

#[test]
fn test_noisy_measurement_curve_fitting_robustness() {
    let base_curve =
        MeasuredCurve::synthetic_nmos_transfer_curve(1.2, (0.0, 1.5, 50), 0.46, 0.050);

    // Inject synthetic pseudorandom noise (+/- 2.5%) into measured current
    let mut noisy_points = Vec::with_capacity(base_curve.points.len());
    let mut rng_state: u64 = 0x1234_5678_9abc_def0;
    for pt in &base_curve.points {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let norm = ((rng_state >> 33) as f64) / (1u64 << 31) as f64; // [-1.0, 1.0] approx
        let noise_factor = 1.0 + 0.025 * (norm - 0.5) * 2.0;
        let noisy_ids = (pt.i_ds * noise_factor).max(0.0);
        noisy_points.push(MeasurementPoint::new(
            pt.v_ds, pt.v_gs, pt.v_bs, noisy_ids, pt.c_gg,
        ));
    }

    let noisy_curve = MeasuredCurve::new(
        "Noisy_NMOS_Transfer",
        base_curve.temperature_k,
        base_curve.channel_length_m,
        base_curve.channel_width_m,
        noisy_points,
    );

    let result = GaOptimizer::fit_curve(&noisy_curve, 30, 48);

    println!(
        "Noisy Curve Fitting: RMSE = {:.6e}, R^2 = {:.6}",
        result.rmse, result.r_squared
    );

    assert!(
        result.r_squared > 0.98,
        "Robustness test must yield R^2 > 0.98 even with 2.5% noise, got {:.6}",
        result.r_squared
    );
}

#[test]
fn test_spice_model_deck_formatting_and_netlist_parsing() {
    let params = Bsim4TargetParams {
        vth0: 0.465,
        u0: 0.052,
        vsat: 1.15e5,
        dvt0: 1.25,
        eta0: 0.075,
        rdsw: 110.0,
        subthreshold_swing_mv_dec: 78.5,
    };

    let result = FittingResult {
        params,
        rmse: 2.15e-6,
        r_squared: 0.9991,
        iterations: 25,
        converged: true,
    };

    let deck = generate_bsim4_model_deck("NMOS_CRYOCMOS_300K", true, &params, &result);

    // Validate structure
    assert!(deck.contains(".MODEL NMOS_CRYOCMOS_300K NMOS"));
    assert!(deck.contains("LEVEL=54"));
    assert!(deck.contains("VERSION=4.8.2"));
    assert!(deck.contains("TNOM=300.0"));
    assert!(deck.contains("VTH0=0.465000"));
    assert!(deck.contains("U0=0.052000"));
    assert!(deck.contains("VSAT=115000.0"));
    assert!(validate_bsim4_model_deck(&deck).is_ok());

    // Validate syntax and parameters using model deck validator
    let val_res = validate_bsim4_model_deck(&deck);
    assert!(
        val_res.is_ok(),
        "BSIM4 model deck validation failed: {:?}",
        val_res.err()
    );
    assert!(deck.contains("DVT0=1.250000"));
    assert!(deck.contains("ETA0=0.075000"));
    assert!(deck.contains("RDSW=110.00"));
    assert!(deck.contains("RMSE = 2.150000e-6 A"));
    assert!(deck.contains("R^2 = 0.999100"));
}

#[test]
fn test_throughput_benchmark_100_curves_under_500ms() {
    let curve =
        MeasuredCurve::synthetic_nmos_transfer_curve(1.2, (0.0, 1.5, 50), 0.45, 0.050);

    let count = 100;
    let start = Instant::now();

    for _ in 0..count {
        let result = GaOptimizer::fit_curve(&curve, 20, 36);
        assert!(result.r_squared > 0.98);
    }

    let elapsed = start.elapsed();
    let fits_per_sec = (count as f64) / elapsed.as_secs_f64();

    println!(
        "Completed {} curve fittings in {:.2} ms ({:.1} fits/sec)",
        count,
        elapsed.as_secs_f64() * 1000.0,
        fits_per_sec
    );

    assert!(
        elapsed.as_millis() < 500,
        "Benchmark failed: 100 curve fits took {} ms (limit 500 ms)",
        elapsed.as_millis()
    );
    assert!(
        fits_per_sec >= 200.0,
        "Throughput below threshold: got {:.1} fits/sec (target >= 200)",
        fits_per_sec
    );
}
