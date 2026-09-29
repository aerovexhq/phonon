#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for chiral quantum acoustic metamaterial circulators and multi-terminal
//! non-reciprocal router networks across multi-threaded Rayon workers.

use phonon_solver::chiral_acoustic_router::RouterBenchmarkRunner;

#[test]
fn test_10k_chiral_acoustic_router_parallel_sweep() {
    let cycles = 10_000;
    let result = RouterBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 129 Chiral Quantum Acoustic Metamaterial Circulators Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Non-Reciprocal Isolation (dB): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_non_reciprocal_isolation_db,
        result.min_non_reciprocal_isolation_db,
        result.max_non_reciprocal_isolation_db
    );
    println!(
        "Waveguide Insertion Loss (dB): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_insertion_loss_db,
        result.min_insertion_loss_db,
        result.max_insertion_loss_db
    );
    println!(
        "Phase Coherence Fidelity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_phase_coherence_fidelity,
        result.min_phase_coherence_fidelity,
        result.max_phase_coherence_fidelity
    );
    println!(
        "Cross-Talk Rejection (dB): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_cross_talk_rejection_db,
        result.min_cross_talk_rejection_db,
        result.max_cross_talk_rejection_db
    );
    println!(
        "Operating Bandwidth (MHz): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_operating_bandwidth_mhz,
        result.min_operating_bandwidth_mhz,
        result.max_operating_bandwidth_mhz
    );

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance fraction must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Assert non-reciprocal isolation >= 35.0 dB
    assert!(
        result.mean_non_reciprocal_isolation_db >= 35.0,
        "Mean non-reciprocal isolation must be >= 35.0 dB, got {:.5}",
        result.mean_non_reciprocal_isolation_db
    );
    assert!(
        result.min_non_reciprocal_isolation_db >= 35.0,
        "Min non-reciprocal isolation must be >= 35.0 dB, got {:.5}",
        result.min_non_reciprocal_isolation_db
    );

    // Assert insertion loss <= 0.40 dB
    assert!(
        result.mean_insertion_loss_db <= 0.40,
        "Mean insertion loss must be <= 0.40 dB, got {:.5}",
        result.mean_insertion_loss_db
    );
    assert!(
        result.max_insertion_loss_db <= 0.40,
        "Max insertion loss must be <= 0.40 dB, got {:.5}",
        result.max_insertion_loss_db
    );

    // Assert phase coherence fidelity >= 0.9920
    assert!(
        result.mean_phase_coherence_fidelity >= 0.9920,
        "Mean phase coherence fidelity must be >= 0.9920, got {:.5}",
        result.mean_phase_coherence_fidelity
    );
    assert!(
        result.min_phase_coherence_fidelity >= 0.9920,
        "Min phase coherence fidelity must be >= 0.9920, got {:.5}",
        result.min_phase_coherence_fidelity
    );

    // Assert cross-talk rejection >= 30.0 dB
    assert!(
        result.mean_cross_talk_rejection_db >= 30.0,
        "Mean cross-talk rejection must be >= 30.0 dB, got {:.5}",
        result.mean_cross_talk_rejection_db
    );
    assert!(
        result.min_cross_talk_rejection_db >= 30.0,
        "Min cross-talk rejection must be >= 30.0 dB, got {:.5}",
        result.min_cross_talk_rejection_db
    );

    // Assert operating bandwidth >= 12.0 MHz
    assert!(
        result.mean_operating_bandwidth_mhz >= 12.0,
        "Mean operating bandwidth must be >= 12.0 MHz, got {:.5}",
        result.mean_operating_bandwidth_mhz
    );
    assert!(
        result.min_operating_bandwidth_mhz >= 12.0,
        "Min operating bandwidth must be >= 12.0 MHz, got {:.5}",
        result.min_operating_bandwidth_mhz
    );
}
