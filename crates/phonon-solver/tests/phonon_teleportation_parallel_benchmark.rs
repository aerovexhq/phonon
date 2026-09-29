//! Parallel benchmark test for quantum phonon-mediated qubit state transfer sweeps.

use phonon_solver::quantum_phonon_teleportation::PhononTeleportationBenchmarkRunner;

#[test]
fn test_phonon_teleportation_parallel_benchmark() {
    let cycles = 10_000;
    let res = PhononTeleportationBenchmarkRunner::run_benchmark(cycles);

    println!("\n=== Phase 105: Quantum Phonon State Transfer Benchmark ===");
    println!("Total cycles: {}", res.total_cycles);
    println!("Elapsed time: {:.4} s", res.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/s", res.throughput_sweeps_per_sec);
    println!(
        "Mean state transfer fidelity: {:.4}% (target >= 96.0%)",
        res.mean_fidelity * 100.0
    );
    println!(
        "Min state transfer fidelity: {:.4}%",
        res.min_fidelity * 100.0
    );
    println!(
        "Mean Bell concurrence: {:.4} (target >= 0.92)",
        res.mean_concurrence
    );
    println!("Min Bell concurrence: {:.4}", res.min_concurrence);
    println!(
        "Mean phonon loss probability: {:.4}% (target <= 2.0%)",
        res.mean_phonon_loss * 100.0
    );
    println!(
        "Max phonon loss probability: {:.4}%",
        res.max_phonon_loss * 100.0
    );
    println!(
        "Mean link bandwidth: {:.2} MHz (target >= 50.0 MHz)",
        res.mean_bandwidth_mhz
    );
    println!("Min link bandwidth: {:.2} MHz", res.min_bandwidth_mhz);
    println!(
        "Physical compliance fraction: {:.4}",
        res.physical_compliance_fraction
    );

    assert_eq!(
        res.physical_compliance_fraction, 1.0,
        "All sweeps must be physically compliant"
    );
    assert!(
        res.mean_fidelity >= 0.960,
        "Mean fidelity must meet target >= 96.0%"
    );
    assert!(
        res.min_fidelity >= 0.960,
        "Min fidelity must meet target >= 96.0%"
    );
    assert!(
        res.mean_concurrence >= 0.920,
        "Mean concurrence must meet target >= 0.92"
    );
    assert!(
        res.min_concurrence >= 0.920,
        "Min concurrence must meet target >= 0.92"
    );
    assert!(
        res.mean_phonon_loss <= 0.020,
        "Mean phonon loss must be <= 0.02"
    );
    assert!(
        res.max_phonon_loss <= 0.020,
        "Max phonon loss must be <= 0.02"
    );
    assert!(
        res.mean_bandwidth_mhz >= 50.0,
        "Mean bandwidth must be >= 50.0 MHz"
    );
    assert!(
        res.min_bandwidth_mhz >= 50.0,
        "Min bandwidth must be >= 50.0 MHz"
    );
}
