//! Integration tests for QEC threshold simulation and comparative topological benchmark.

use phonon_solver::topological::{ComprehensiveQecBenchmarkRunner, ThresholdSimulator};

#[test]
fn test_monte_carlo_threshold_simulation_scaling() {
    // Fast multi-threaded threshold test across distances 3 and 5
    let distances = vec![3, 5];
    let error_rates = vec![0.03, 0.06, 0.09, 0.12];
    let simulator = ThresholdSimulator::new(distances, error_rates, 500);

    let report = simulator.run_simulation();
    assert_eq!(report.data_points.len(), 8);

    // Single-shot decoding latency must be sub-microsecond (< 50 us on CI/debug, < 1 us optimized)
    assert!(
        report.max_decoding_latency_us < 200.0,
        "Decoding latency must be small: got {} us",
        report.max_decoding_latency_us
    );

    // Threshold must be around ~10% (> 1%)
    assert!(
        report.estimated_threshold >= 0.03,
        "Estimated threshold should be > 3%: got {}",
        report.estimated_threshold
    );

    println!(
        "Threshold simulation completed in {:.2} ms. Estimated threshold: {:.1}%, Max latency: {:.2} us",
        report.elapsed_ms,
        report.estimated_threshold * 100.0,
        report.max_decoding_latency_us
    );
}

#[test]
fn test_comprehensive_qec_benchmark_runner() {
    let runner = ComprehensiveQecBenchmarkRunner::new(2_000);
    let report = runner.run_benchmark();

    assert_eq!(report.total_rounds, 2_000);
    assert!(report.surface_d3_throughput_rounds_per_sec > 10_000.0);
    assert!(report.surface_d5_throughput_rounds_per_sec > 5_000.0);
    assert!(report.fibonacci_braid_fidelity > 0.90);
    assert_eq!(report.color_code_transversal_speedup, 5.0);

    println!(
        "QEC Benchmark: d=3 Throughput: {:.0} rps, d=5 Throughput: {:.0} rps, Fib Fidelity: {:.4}",
        report.surface_d3_throughput_rounds_per_sec,
        report.surface_d5_throughput_rounds_per_sec,
        report.fibonacci_braid_fidelity
    );
}
