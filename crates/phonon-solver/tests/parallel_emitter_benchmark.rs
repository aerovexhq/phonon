//! Integration Benchmark for Parallel Discrete RF Emitters & Transceiver Links

use phonon_solver::em::EmitterBenchmarkRunner;

#[test]
fn test_parallel_discrete_emitter_benchmark_10k_cases() {
    let runner = EmitterBenchmarkRunner::new();

    // 5,000 synthesized emitters + 5,000 transceiver links = 10,000 total evaluations
    let report = runner.run_benchmark(5000, 5000);

    println!(
        "Emitter Benchmark: {} emitters, {} links in {:.2} ms ({:.0} evals/sec), avg Friis error: {:.4}%",
        report.total_emitters_synthesized,
        report.total_transceiver_links_evaluated,
        report.elapsed_millis,
        report.throughput_evaluations_per_sec,
        report.average_friis_error_percent,
    );

    assert_eq!(report.total_emitters_synthesized, 5000);
    assert_eq!(report.total_transceiver_links_evaluated, 5000);
    assert!(
        report.all_valid,
        "All benchmark evaluations must be physically valid"
    );
    assert!(
        report.average_friis_error_percent < 1.0,
        "Average Friis discrepancy should be < 1.0%, got {:.4}%",
        report.average_friis_error_percent
    );
    assert!(
        report.throughput_evaluations_per_sec > 10_000.0,
        "Throughput should exceed 10,000 evals/sec"
    );
}
