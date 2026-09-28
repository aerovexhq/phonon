//! Integration Test Suite:
//! Cold Atom Gravimetry, Gravity Gradiometry & Multi-Threaded Rayon Benchmark.

use phonon_models::quantum::{MachZehnderInterferometer, RB87_D2_WAVELENGTH_METERS};
use phonon_solver::quantum::{
    BayesianPhaseEstimator, ColdAtomBenchmarkRunner, ColdAtomFringeFitter, GravimeterTechnology,
    HybridVibrationCanceller,
};

#[test]
fn test_multi_threaded_rayon_comparative_benchmark_engine() {
    let evaluation_count = 4_000;
    let report = ColdAtomBenchmarkRunner::run_comparative_benchmark(evaluation_count);

    assert_eq!(report.total_evaluations, evaluation_count);
    assert!(report.elapsed_ms > 0.0);
    assert!(report.evaluations_per_second > 10_000.0);

    // 1. Absolute drift-free bias stability: 0.0 nm/s^2/day vs > 100 nm/s^2/day
    assert_eq!(report.cold_atom_bias_drift_nm_s2_per_day, 0.0);
    assert!(report.classical_spring_drift_nm_s2_per_day > 100.0);

    // 2. Gravitational acceleration sensitivity <= 1e-8 m/s^2 / sqrt(Hz)
    assert!(report.cold_atom_acceleration_sensitivity <= 1.0e-8);
    assert!(report.cold_atom_acceleration_sensitivity > 1.0e-10);

    // 3. Relativistic geodetic height elevation resolution < 1.0 cm
    assert!(report.geodetic_height_resolution_cm < 1.0);
    assert!(report.geodetic_height_resolution_cm > 0.0);

    // 4. Hybrid classical accelerometer vibration noise cancellation
    assert!(report.hybrid_vibration_contrast_improvement_factor > 4.0);

    // 5. Verification flag
    assert!(report.target_performance_verified);
}

#[test]
fn test_hybrid_classical_accelerometer_correlation_and_contrast_restoration() {
    let t_inter = 0.08; // 80 ms interrogation time
    let dt = 1.0e-4; // 10 kHz classical accelerometer sample rate
    let n_steps = (2.0 * t_inter / dt) as usize;

    // Simulate multi-tone ground vibration: seismic micro-tremors at 5 Hz, 15 Hz, and 45 Hz
    let mut accel_time_series = vec![0.0; n_steps];
    for (i, sample) in accel_time_series.iter_mut().enumerate() {
        let t = (i as f64) * dt;
        *sample = 2.0e-5 * (2.0 * std::f64::consts::PI * 5.0 * t).sin()
            + 1.0e-5 * (2.0 * std::f64::consts::PI * 15.0 * t).cos()
            + 0.5e-5 * (2.0 * std::f64::consts::PI * 45.0 * t).sin();
    }

    let k_eff = 2.0 * (2.0 * std::f64::consts::PI / RB87_D2_WAVELENGTH_METERS);
    let phi_corr =
        HybridVibrationCanceller::compute_correlation_phase(&accel_time_series, dt, k_eff, t_inter);
    assert!(phi_corr.is_finite());
    assert!(phi_corr.abs() > 0.0);

    // Uncompensated vibration phase noise sigma_raw = 2.8 radians
    let sigma_raw: f64 = 2.8;
    let intrinsic_contrast: f64 = 0.92;
    let raw_contrast: f64 = intrinsic_contrast * (-0.5 * sigma_raw * sigma_raw).exp();
    // Raw contrast severely washed out: C_raw < 0.05
    assert!(raw_contrast < 0.05);

    // With 98.5% classical-quantum correlation
    let compensated_contrast =
        HybridVibrationCanceller::compensated_contrast(intrinsic_contrast, sigma_raw, 0.985);
    // Contrast restored to > 0.70
    assert!(compensated_contrast > 0.70);

    // Contrast improvement factor > 10x
    let improvement_factor = compensated_contrast / raw_contrast;
    assert!(improvement_factor > 10.0);
}

#[test]
fn test_dual_cloud_gravity_gradiometer_fringe_fitting_and_inversion() {
    let t_interrogation = 0.06; // 60 ms
    let mut mz = MachZehnderInterferometer::standard_rb87_gravimeter(t_interrogation);
    mz.gravity_gradient_zz = 3.086e-6; // 3086 Eötvös
    mz.baseline_distance = 1.0; // 1 meter

    let k_eff = mz.effective_k_magnitude();
    let dphi_grad = mz.gravity_gradient_phase_diff();

    // Scan applied laser phase theta_i over 32 points in [0, 2pi]
    let n_points = 32;
    let mut scan_phases = Vec::with_capacity(n_points);
    let mut pop_cloud1 = Vec::with_capacity(n_points);
    let mut pop_cloud2 = Vec::with_capacity(n_points);

    for i in 0..n_points {
        let theta = (i as f64) * 2.0 * std::f64::consts::PI / (n_points as f64);
        scan_phases.push(theta);

        // Intrinsic phase for Cloud 1
        let phi1 = mz.gravitational_phase() + theta;
        let p1 = 0.5 * (1.0 - mz.contrast * phi1.cos());
        pop_cloud1.push(p1);

        // Intrinsic phase for Cloud 2 (shifted by dphi_grad)
        let phi2 = phi1 + dphi_grad;
        let p2 = 0.5 * (1.0 - mz.contrast * phi2.cos());
        pop_cloud2.push(p2);
    }

    // Fit sinusoids to both cloud fringe patterns
    let fit1 = ColdAtomFringeFitter::fit_sinusoid(&scan_phases, &pop_cloud1);
    let fit2 = ColdAtomFringeFitter::fit_sinusoid(&scan_phases, &pop_cloud2);

    assert!((fit1.contrast - mz.contrast).abs() < 1e-4);
    assert!((fit2.contrast - mz.contrast).abs() < 1e-4);

    // Extract differential phase
    let mut diff_phase = fit2.phase_rad - fit1.phase_rad;
    while diff_phase < -std::f64::consts::PI {
        diff_phase += 2.0 * std::f64::consts::PI;
    }
    while diff_phase > std::f64::consts::PI {
        diff_phase -= 2.0 * std::f64::consts::PI;
    }
    assert!((diff_phase - dphi_grad).abs() < 1e-4);

    // Extract vertical gravity gradient T_zz
    let extracted_tzz = ColdAtomFringeFitter::extract_gravity_gradient(
        fit2.phase_rad,
        fit1.phase_rad,
        k_eff,
        mz.baseline_distance,
        t_interrogation,
    );
    assert!((extracted_tzz - mz.gravity_gradient_zz).abs() < 1e-7);
}

#[test]
fn test_bayesian_phase_estimation_reaching_cramer_rao_limit() {
    let true_phase: f64 = 0.825; // radians
    let contrast: f64 = 0.90;
    let atom_count: f64 = 500_000.0;

    // Detection shot-noise variance per launch: sigma^2 = 1 / (C^2 * N)
    let shot_noise_var: f64 = 1.0 / (contrast * contrast * atom_count);
    let cr_limit_sigma: f64 = shot_noise_var.sqrt();

    let mut estimator = BayesianPhaseEstimator::with_prior(0.0, 1.0); // uninformative prior
    assert_eq!(estimator.measurement_count, 0);

    // Simulate 200 sequential Bayesian updates
    for i in 1..=200 {
        // Pseudo-random deterministic noise
        let noise = ((i as f64).sin() * 0.95) * cr_limit_sigma;
        let obs = true_phase + noise;
        estimator.update(obs, shot_noise_var);
    }

    assert_eq!(estimator.measurement_count, 200);
    // Estimated phase converged to within a fraction of single-shot sigma
    assert!((estimator.mean_phase - true_phase).abs() < cr_limit_sigma);

    // Posterior uncertainty scales as sigma_0 / sqrt(M)
    let expected_post_sigma = cr_limit_sigma / (200.0_f64).sqrt();
    assert!((estimator.uncertainty_rad() - expected_post_sigma).abs() < 1e-7);
}

#[test]
fn test_gravimeter_technologies_drift_and_sensitivity_comparison() {
    let interrogation_t = 0.15;
    let seismic_noise = 2.0e-6; // 2 micro-g

    let p_quantum = ColdAtomBenchmarkRunner::evaluate_state(
        GravimeterTechnology::ColdAtomQuantum,
        interrogation_t,
        seismic_noise,
    );
    let p_super = ColdAtomBenchmarkRunner::evaluate_state(
        GravimeterTechnology::Superconducting,
        interrogation_t,
        seismic_noise,
    );
    let p_spring = ColdAtomBenchmarkRunner::evaluate_state(
        GravimeterTechnology::ClassicalSpring,
        interrogation_t,
        seismic_noise,
    );
    let p_mems = ColdAtomBenchmarkRunner::evaluate_state(
        GravimeterTechnology::MemsGravimeter,
        interrogation_t,
        seismic_noise,
    );

    // Cold Atom has exactly 0.0 drift
    assert_eq!(p_quantum.bias_drift_nm_s2_per_day, 0.0);
    assert!((0.0..=1.0).contains(&p_super.bias_drift_nm_s2_per_day));
    assert!(p_spring.bias_drift_nm_s2_per_day > 100.0);
    assert!(p_mems.bias_drift_nm_s2_per_day > 1000.0);

    // Cold Atom sensitivity <= 1e-8
    assert!(p_quantum.acceleration_sensitivity <= 1.0e-8);
    // Classical spring sensitivity is ~1e-7 (worse)
    assert!(p_spring.acceleration_sensitivity > p_quantum.acceleration_sensitivity);
    // MEMS sensitivity is ~1e-6 (much worse)
    assert!(p_mems.acceleration_sensitivity > p_spring.acceleration_sensitivity);

    // Geodetic resolution
    assert!(p_quantum.geodetic_height_resolution_cm < 1.0);
    assert_eq!(p_super.geodetic_height_resolution_cm, 999.0);
    assert_eq!(p_spring.geodetic_height_resolution_cm, 999.0);
}
