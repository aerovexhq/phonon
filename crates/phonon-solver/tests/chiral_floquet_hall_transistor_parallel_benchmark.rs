#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for chiral phononic Floquet-SBT gauge fields and dissipationless acoustic topological Hall transistors
//! across multi-threaded Rayon workers.

use phonon_solver::chiral_floquet_hall_transistor::FloquetHallTransistorBenchmarkRunner;

#[test]
fn test_10k_chiral_floquet_hall_transistor_parallel_sweep() {
    let cycles = 10_000;
    let result = FloquetHallTransistorBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 146 Chiral Phononic Floquet-SBT Gauge Fields & Acoustic Topological Hall Transistors Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Valley Hall Contrast Ratio (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_valley_hall_contrast_ratio_db,
        result.min_valley_hall_contrast_ratio_db,
        result.max_valley_hall_contrast_ratio_db
    );
    println!(
        "Topological Switching Time (ns): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_switching_time_ns,
        result.min_topological_switching_time_ns,
        result.max_topological_switching_time_ns
    );
    println!(
        "Cross-Talk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_cross_talk_isolation_db,
        result.min_cross_talk_isolation_db,
        result.max_cross_talk_isolation_db
    );
    println!(
        "Non-Adiabatic Insertion Loss (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_non_adiabatic_insertion_loss_db,
        result.min_non_adiabatic_insertion_loss_db,
        result.max_non_adiabatic_insertion_loss_db
    );
    println!(
        "Hall Transistor State Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_hall_transistor_state_fidelity,
        result.min_hall_transistor_state_fidelity,
        result.max_hall_transistor_state_fidelity
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
        result.min_valley_hall_contrast_ratio_db >= 35.0,
        "Minimum valley Hall contrast ratio must be >= 35.0 dB, got {:.4} dB",
        result.min_valley_hall_contrast_ratio_db
    );
    assert!(
        result.max_topological_switching_time_ns <= 15.0,
        "Maximum topological switching time must be <= 15.0 ns, got {:.4} ns",
        result.max_topological_switching_time_ns
    );
    assert!(
        result.min_cross_talk_isolation_db >= 40.0,
        "Minimum cross-talk isolation must be >= 40.0 dB, got {:.4} dB",
        result.min_cross_talk_isolation_db
    );
    assert!(
        result.max_non_adiabatic_insertion_loss_db <= 0.60,
        "Maximum non-adiabatic insertion loss must be <= 0.60 dB, got {:.4} dB",
        result.max_non_adiabatic_insertion_loss_db
    );
    assert!(
        result.min_hall_transistor_state_fidelity >= 0.9960,
        "Minimum hall transistor state fidelity must be >= 0.9960, got {:.6}",
        result.min_hall_transistor_state_fidelity
    );
}
