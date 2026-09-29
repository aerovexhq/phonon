#![deny(unsafe_code)]

//! Integration Test: 10,000-Sweep Parallel Rayon Benchmark for Quantum Acoustic Systems.

use phonon_solver::quantum_acoustic::run_quantum_acoustic_benchmark;

#[test]
fn test_quantum_acoustic_parallel_benchmark_execution() {
    let num_sweeps = 10_000;
    let report = run_quantum_acoustic_benchmark(num_sweeps);

    assert_eq!(report.total_sweeps, num_sweeps);
    assert!(report.elapsed_seconds > 0.0);
    assert!(
        report.throughput_sweeps_per_sec > 50_000.0,
        "Parallel throughput {} sweeps/sec is below threshold",
        report.throughput_sweeps_per_sec
    );

    // Physical metrics
    assert!(
        report.mean_conversion_efficiency > 0.05,
        "Mean IDT conversion efficiency {} too low",
        report.mean_conversion_efficiency
    );
    assert!(
        report.mean_coupling_rate_mhz > 1.0,
        "Mean coupling rate {} MHz too low",
        report.mean_coupling_rate_mhz
    );
    assert!(
        report.strong_coupling_fraction >= 0.99,
        "Expected >= 99% strong coupling, got {:.2}%",
        report.strong_coupling_fraction * 100.0
    );

    // Fidelity assertions
    assert!(
        report.mean_swap_fidelity >= 0.95,
        "Mean SWAP fidelity must exceed 95%, got {:.4}",
        report.mean_swap_fidelity
    );
    assert!(
        report.min_swap_fidelity >= 0.90,
        "Min SWAP fidelity must exceed 90%, got {:.4}",
        report.min_swap_fidelity
    );
    assert!(
        report.mean_bell_fidelity >= 0.95,
        "Mean Bell state fidelity must exceed 95%, got {:.4}",
        report.mean_bell_fidelity
    );
    assert!(
        report.min_bell_fidelity >= 0.90,
        "Min Bell state fidelity must exceed 90%, got {:.4}",
        report.min_bell_fidelity
    );
    assert!(
        report.mean_hom_visibility >= 0.90,
        "Mean HOM visibility must exceed 90%, got {:.4}",
        report.mean_hom_visibility
    );
}
