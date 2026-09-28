//! Integration test executing the 10,000 k-point parallel Rayon benchmark.

use phonon_solver::moire::run_10k_moire_benchmark;

#[test]
fn test_parallel_10k_moire_flatband_benchmark() {
    let threads = 4;
    let report = run_10k_moire_benchmark(threads);

    println!("\n=== Phase 55 Moir\u{00e9} Superlattice Benchmark Report ===");
    println!("Total k-points:                {}", report.total_k_points);
    println!("Elapsed Time (ms):             {:.3} ms", report.elapsed_ms);
    println!(
        "Throughput:                    {:.2} points/sec",
        report.points_per_second
    );
    println!(
        "Flat-Band Bandwidth:           {:.2} meV (limit: < 10 meV)",
        report.flatband_bandwidth_mev
    );
    println!(
        "Dirac Velocity Ratio:          {:.4} (limit: < 0.10)",
        report.dirac_velocity_ratio
    );
    println!(
        "Lower Remote Gap:              {:.2} meV (limit: > 10 meV)",
        report.lower_remote_gap_mev
    );
    println!(
        "Upper Remote Gap:              {:.2} meV (limit: > 10 meV)",
        report.upper_remote_gap_mev
    );
    println!(
        "Correlated Mott Gap (\u{03bd}=-2):     {:.2} meV (limit: > 2 meV)",
        report.correlated_mott_gap_mev
    );
    println!(
        "Superconducting Tc,max:        {:.2} K (limit: > 1.0 K)",
        report.superconducting_tc_max_k
    );
    println!(
        "Upper Critical Field B_c2:     {:.3} T",
        report.upper_critical_field_perp_tesla
    );
    println!(
        "Benchmark Verified:            {}",
        report.benchmark_verified
    );
    println!("========================================================\n");

    assert_eq!(report.total_k_points, 10_000);
    assert!(
        report.flatband_bandwidth_mev < 10.0,
        "Bandwidth {:.2} meV exceeds threshold 10 meV",
        report.flatband_bandwidth_mev
    );
    assert!(
        report.dirac_velocity_ratio < 0.10,
        "Dirac velocity ratio {:.4} exceeds threshold 0.10",
        report.dirac_velocity_ratio
    );
    assert!(
        report.lower_remote_gap_mev > 10.0,
        "Lower remote gap {:.2} meV below threshold 10 meV",
        report.lower_remote_gap_mev
    );
    assert!(
        report.correlated_mott_gap_mev > 2.0,
        "Correlated Mott gap {:.2} meV below threshold 2 meV",
        report.correlated_mott_gap_mev
    );
    assert!(
        report.superconducting_tc_max_k > 1.0,
        "Tc,max {:.2} K below threshold 1.0 K",
        report.superconducting_tc_max_k
    );

    let min_throughput = if cfg!(debug_assertions) {
        10_000.0
    } else {
        100_000.0
    };
    assert!(
        report.points_per_second > min_throughput,
        "Throughput {:.2} points/sec below required {:.0} points/sec",
        report.points_per_second,
        min_throughput
    );
    assert!(report.benchmark_verified);
}
