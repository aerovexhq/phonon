//! Integration tests for multi-threaded Rayon benchmark comparing Topological Qubits,
//! physical Transmons, Surface-Code Transmons, and CMOS control baselines.

use phonon_solver::topological::TopologicalBenchmarkRunner;

#[test]
fn test_parallel_topological_benchmark_10k_circuits() {
    let runner = TopologicalBenchmarkRunner::new(10_000, 10);
    let report = runner.run_benchmark();

    assert_eq!(report.total_circuits_simulated, 10_000);

    // Topological fidelity must be near unity (> 0.9999)
    assert!(
        report.topological_avg_fidelity > 0.9999,
        "Topological fidelity must remain extremely high: got {}",
        report.topological_avg_fidelity
    );

    // Physical unprotected transmons accumulate gate errors across 10 gates (0.999^10 ~ 0.990)
    assert!(
        report.transmon_unprotected_avg_fidelity < 0.995,
        "Unprotected transmons must show error accumulation: got {}",
        report.transmon_unprotected_avg_fidelity
    );

    // Footprint reduction factor must exceed 1,000,000x:
    // Surface-code d=3 requires 17 transmons * 0.25 mm^2 = 4.25 mm^2 = 4,250,000 um^2 vs 1 um^2
    assert!(
        report.footprint_reduction_factor >= 4_000_000.0,
        "Topological qubits must achieve massive footprint reduction: got {}x",
        report.footprint_reduction_factor
    );

    // Coherence advantage: 200 ms / 100 us = 2000x
    assert!(
        report.coherence_advantage_factor >= 2000.0,
        "Coherence lifetime advantage must be at least 2000x: got {}x",
        report.coherence_advantage_factor
    );

    // Fermion parity conservation rate across 10,000 circuits must exceed 99.9%
    assert!(
        report.parity_conservation_rate >= 0.999,
        "Parity conservation rate must be near 100%: got {}",
        report.parity_conservation_rate
    );

    println!(
        "Topological Benchmark Completed: 10,000 circuits in {:.2} ms. Topo Fidelity: {:.6}, Footprint Reduction: {:.0}x",
        report.elapsed_wallclock_ms,
        report.topological_avg_fidelity,
        report.footprint_reduction_factor
    );
}
