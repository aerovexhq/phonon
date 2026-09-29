//! Parallel benchmark integration test for quantum valley phononics,
//! verifying 10,000 parameter sweeps across Rayon threads.

use phonon_solver::valley_acoustic::ValleyPhononBenchmarkRunner;

#[test]
fn test_valley_phonon_parallel_benchmark_10000_sweeps() {
    let runner = ValleyPhononBenchmarkRunner::new(10_000);
    let report = runner.run_parallel_benchmark();

    println!("\n=== Valley Phononics Parallel Benchmark Report ===");
    println!("Total sweeps: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/sec",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean pseudomagnetic field: {:.2} T",
        report.mean_pseudomagnetic_field_t
    );
    println!(
        "Min pseudomagnetic field: {:.2} T",
        report.min_pseudomagnetic_field_t
    );
    println!(
        "Mean valley contrast: {:.2} dB",
        report.mean_valley_contrast_db
    );
    println!(
        "Min valley contrast: {:.2} dB",
        report.min_valley_contrast_db
    );
    println!("Mean Purcell factor: {:.2}", report.mean_purcell_factor);
    println!("Min Purcell factor: {:.2}", report.min_purcell_factor);
    println!(
        "Mean corner transmission: {:.4}",
        report.mean_corner_transmission
    );
    println!(
        "Topological compliance fraction: {:.4}",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(
        report.throughput_cycles_per_sec > 10_000.0,
        "Throughput must exceed 10,000 sweeps/sec, achieved {:.2}",
        report.throughput_cycles_per_sec
    );
    assert!(
        report.mean_pseudomagnetic_field_t >= 100.0,
        "Mean pseudomagnetic field must be >= 100 T, got {:.2} T",
        report.mean_pseudomagnetic_field_t
    );
    assert!(
        report.min_pseudomagnetic_field_t >= 100.0,
        "Min pseudomagnetic field must be >= 100 T, got {:.2} T",
        report.min_pseudomagnetic_field_t
    );
    assert!(
        report.mean_valley_contrast_db >= 20.0,
        "Mean valley contrast must be >= 20 dB, got {:.2} dB",
        report.mean_valley_contrast_db
    );
    assert!(
        report.min_valley_contrast_db >= 20.0,
        "Min valley contrast must be >= 20 dB, got {:.2} dB",
        report.min_valley_contrast_db
    );
    assert!(
        report.mean_purcell_factor > 10.0,
        "Mean Purcell factor must exceed 10.0, got {:.2}",
        report.mean_purcell_factor
    );
    assert!(
        report.min_purcell_factor > 10.0,
        "Min Purcell factor must exceed 10.0, got {:.2}",
        report.min_purcell_factor
    );
    assert!(
        report.mean_corner_transmission >= 0.90,
        "Mean corner transmission must be >= 0.90, got {:.4}",
        report.mean_corner_transmission
    );
    assert_eq!(
        report.compliance_fraction, 1.0,
        "All sweeps must be 100% compliant with valley phononics criteria"
    );
}
