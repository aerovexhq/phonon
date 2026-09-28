//! Integration test suite for Phase 42:
//! Sub-Millimeter Absorption Spectroscopy Engine & Multi-Threaded Comparative Benchmark.

use phonon_solver::quantum::{SubMillimeterSpectroscopyEngine, ThzQclBenchmarkRunner};

#[test]
fn test_sub_millimeter_molecular_rotational_spectroscopy() {
    let engine = SubMillimeterSpectroscopyEngine::atmospheric_catalog(1.0, 1.0); // 1.0 m path, 1.0 atm

    // 1. Water vapor (H2O) absorption at 0.557 THz:
    let res_h2o = engine.compute_transmission_spectrum(
        0.50, 0.65, 100, 4000.0, // 4,000 ppm water vapor
        0.0, 0.0,
    );

    // Deep transmission dip near 0.557 THz:
    assert!(res_h2o.min_transmission < 0.10);
    assert!(res_h2o.peak_absorbance > 2.0);
    assert!(
        (res_h2o.peak_absorption_frequency_thz - 0.557).abs() < 0.01,
        "Peak absorption should be at 0.557 THz, got: {} THz",
        res_h2o.peak_absorption_frequency_thz
    );

    // 2. Carbon Monoxide (CO) absorption ladder line at 0.576 THz:
    let res_co = engine.compute_transmission_spectrum(
        0.56, 0.60, 80, 0.0, 150.0, // 150 ppm CO
        0.0,
    );

    assert!(res_co.min_transmission < 0.95);
    assert!(
        (res_co.peak_absorption_frequency_thz - 0.576).abs() < 0.005,
        "Peak absorption should be at 0.576 THz, got: {} THz",
        res_co.peak_absorption_frequency_thz
    );

    // 3. Multi-species simultaneous absorption (H2O + CO + O3):
    let res_multi = engine.compute_transmission_spectrum(
        0.50, 1.20, 150, 3000.0, // 3000 ppm H2O
        50.0,   // 50 ppm CO
        10.0,   // 10 ppm O3
    );

    // High absorption across multiple resonance lines:
    assert!(res_multi.min_transmission < 0.05);
    assert!(res_multi.peak_absorbance > 3.0);
}

#[test]
fn test_multi_threaded_rayon_comparative_benchmark_engine() {
    // Run multi-threaded parameter sweep across 4 technologies:
    // 20 temperature points x 20 frequency points = 400 evaluations per technology = 1,600 total evaluations
    let report = ThzQclBenchmarkRunner::run_comparative_benchmark(20);

    assert_eq!(report.total_evaluations, 4 * 20 * 20);
    assert!(report.elapsed_ms > 0.0);
    assert!(
        report.evaluations_per_second > 10_000.0,
        "Rayon evaluation throughput should exceed 10,000 states/sec, got: {} states/sec",
        report.evaluations_per_second
    );

    // 1. Wall-plug efficiency (WPE):
    // THz QCL: 1 - 5%
    assert!(
        report.qcl_mean_wall_plug_efficiency >= 0.01
            && report.qcl_mean_wall_plug_efficiency <= 0.05,
        "QCL WPE should be 1-5%, got: {:.3}%",
        report.qcl_mean_wall_plug_efficiency * 100.0
    );

    // PCA WPE is ~1e-6 (10^-4 %):
    assert!(
        report.pca_mean_wall_plug_efficiency < 1e-4,
        "PCA WPE should be < 1e-4, got: {:.3e}",
        report.pca_mean_wall_plug_efficiency
    );

    // WPE advantage factor > 10,000x:
    assert!(
        report.wpe_advantage_qcl_vs_pca > 10_000.0,
        "QCL WPE advantage over PCA should exceed 10,000x, got: {:.1}x",
        report.wpe_advantage_qcl_vs_pca
    );

    // 2. Optical power:
    // Peak pulsed power > 100 mW:
    assert!(
        report.qcl_peak_power_mw >= 100.0,
        "QCL peak power should exceed 100 mW, got: {:.1} mW",
        report.qcl_peak_power_mw
    );

    // CW power > 10 mW:
    assert!(
        report.qcl_mean_cw_power_mw >= 10.0,
        "QCL CW power should exceed 10 mW, got: {:.1} mW",
        report.qcl_mean_cw_power_mw
    );

    // 3. Chip-scale integration density:
    // THz QCL: > 100 devices/cm^2:
    assert!(
        report.qcl_integration_density >= 100.0,
        "QCL density should exceed 100 devices/cm^2, got: {} devices/cm^2",
        report.qcl_integration_density
    );

    // Gas laser density < 0.01 devices/cm^2:
    assert!(report.gas_laser_integration_density < 0.01);
    assert!(report.density_advantage_qcl_vs_gas_laser > 10_000.0);

    // 4. Spectral purity / linewidth:
    // Sub-kilohertz comb line beat note (< 1000 Hz):
    assert!(
        report.qcl_comb_linewidth_hz < 1000.0,
        "QCL comb linewidth must be sub-kHz, got: {} Hz",
        report.qcl_comb_linewidth_hz
    );

    // 5. Maximum operating temperature:
    assert!(
        report.qcl_max_operating_temp_k > 200.0,
        "QCL T_max must exceed 200 K, got: {} K",
        report.qcl_max_operating_temp_k
    );

    // 6. Overall Phase 42 performance verification:
    assert!(report.target_performance_verified);
}
