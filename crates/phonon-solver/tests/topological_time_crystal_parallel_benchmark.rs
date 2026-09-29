#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic topological time crystals and Floquet-symmetry-enriched
//! phononic memories across multi-threaded Rayon workers.

use phonon_solver::topological_time_crystal::TimeCrystalBenchmarkRunner;

#[test]
fn test_10k_topological_time_crystal_parallel_sweep() {
    let cycles = 10_000;
    let result = TimeCrystalBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 140 Quantum Acoustic Topological Time Crystals & Floquet-Symmetry-Enriched Phononic Memories Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Time-Crystalline Order Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_time_crystalline_order_fidelity,
        result.min_time_crystalline_order_fidelity,
        result.max_time_crystalline_order_fidelity
    );
    println!(
        "Subharmonic Locking Error: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_subharmonic_locking_error,
        result.min_subharmonic_locking_error,
        result.max_subharmonic_locking_error
    );
    println!(
        "Temporal Crystalline Lifetime (ms): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_temporal_crystalline_lifetime_ms,
        result.min_temporal_crystalline_lifetime_ms,
        result.max_temporal_crystalline_lifetime_ms
    );
    println!(
        "Memory Retention Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_memory_retention_isolation_db,
        result.min_memory_retention_isolation_db,
        result.max_memory_retention_isolation_db
    );
    println!(
        "Many-Body Localization Ratio: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_many_body_localization_ratio,
        result.min_many_body_localization_ratio,
        result.max_many_body_localization_ratio
    );

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Verify roadmap thresholds
    assert!(
        result.min_time_crystalline_order_fidelity >= 0.9960,
        "Minimum time-crystalline order fidelity must be >= 0.9960"
    );
    assert!(
        result.max_subharmonic_locking_error <= 0.0020,
        "Maximum subharmonic locking error must be <= 0.0020"
    );
    assert!(
        result.min_temporal_crystalline_lifetime_ms >= 100.0,
        "Minimum temporal crystalline lifetime must be >= 100.0 ms"
    );
    assert!(
        result.min_memory_retention_isolation_db >= 45.0,
        "Minimum memory retention isolation must be >= 45.0 dB"
    );
    assert!(
        result.min_many_body_localization_ratio >= 0.920,
        "Minimum many-body localization ratio must be >= 0.920"
    );
}
