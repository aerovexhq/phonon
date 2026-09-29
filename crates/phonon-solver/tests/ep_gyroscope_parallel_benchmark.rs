#![deny(unsafe_code)]

use phonon_solver::non_hermitian_ep_gyroscope::EpGyroscopeBenchmarkRunner;

#[test]
fn test_ep_gyroscope_parallel_benchmark() {
    let report = EpGyroscopeBenchmarkRunner::run_benchmark(10_000);

    println!("\n=== Phase 104: Non-Hermitian EP Gyroscope Benchmark ===");
    println!("Total cycles: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/s",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean scale enhancement: {:.2}x (target >= 15.0x)",
        report.mean_scale_factor_enhancement
    );
    println!(
        "Min scale enhancement: {:.2}x",
        report.min_scale_factor_enhancement
    );
    println!(
        "Mean dynamic range: {:.2} dB (target >= 120.0 dB)",
        report.mean_dynamic_range_db
    );
    println!("Min dynamic range: {:.2} dB", report.min_dynamic_range_db);
    println!(
        "Mean angle random walk: {:.4e} deg/sqrt(hr) (target <= 0.001)",
        report.mean_angle_random_walk
    );
    println!(
        "Max angle random walk: {:.4e} deg/sqrt(hr)",
        report.max_angle_random_walk
    );
    println!(
        "Mean bias stability: {:.4e} deg/hr (target <= 0.005)",
        report.mean_bias_stability
    );
    println!(
        "Max bias stability: {:.4e} deg/hr",
        report.max_bias_stability
    );
    println!("Mean Petermann factor: {:.2}", report.mean_petermann_factor);
    println!(
        "Physical compliance fraction: {:.4}",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.throughput_cycles_per_sec > 100_000.0);
    assert!(report.min_scale_factor_enhancement >= 15.0);
    assert!(report.min_dynamic_range_db >= 120.0);
    assert!(report.max_angle_random_walk <= 0.001);
    assert!(report.max_bias_stability <= 0.005);
    assert_eq!(report.compliance_fraction, 1.0);
}
