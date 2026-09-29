//! Parallel Rayon benchmark test for chiral phonon spin-mechanics and OAM multiplexers.

use phonon_solver::chiral_phonon_spin_mechanics::ChiralPhononSpinBenchmarkRunner;

#[test]
fn test_chiral_phonon_spin_mechanics_10k_benchmark() {
    let runner = ChiralPhononSpinBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("--- Chiral Phonon Spin-Mechanics Benchmark Report ---");
    println!("Total parameter sweeps: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/sec",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean OAM Mode Isolation: {:.2} dB (min: {:.2} dB, max: {:.2} dB)",
        report.mean_isolation_db, report.min_isolation_db, report.max_isolation_db
    );
    println!(
        "Mean Inter-Channel Crosstalk: {:.2} dB (worst: {:.2} dB)",
        report.mean_crosstalk_db, report.max_crosstalk_db
    );
    println!(
        "Mean Transduction Efficiency: {:.2}% (min: {:.2}%)",
        report.mean_efficiency_pct, report.min_efficiency_pct
    );
    println!(
        "Mean Insertion Loss: {:.3} dB (max: {:.3} dB)",
        report.mean_insertion_loss_db, report.max_insertion_loss_db
    );
    println!(
        "Mean Spin-Orbit Purity: {:.2}% (min: {:.2}%)",
        report.mean_purity_pct, report.min_purity_pct
    );
    println!(
        "Mean Router Extinction: {:.2} dB (min: {:.2} dB)",
        report.mean_router_extinction_db, report.min_router_extinction_db
    );
    println!(
        "Mean Multiplexed Capacity: {:.2} Gbps",
        report.mean_capacity_gbps
    );
    println!(
        "Compliance fraction: {:.4} ({}%)",
        report.compliance_fraction,
        report.compliance_fraction * 100.0
    );

    assert_eq!(
        report.total_cycles, 10_000,
        "Total sweeps must equal 10,000"
    );
    assert!(
        report.mean_isolation_db >= 25.0,
        "Mean isolation must be >= 25.0 dB, got {}",
        report.mean_isolation_db
    );
    assert!(
        report.min_isolation_db >= 25.0,
        "Min isolation must be >= 25.0 dB, got {}",
        report.min_isolation_db
    );
    assert!(
        report.mean_crosstalk_db <= -20.0,
        "Mean crosstalk must be <= -20.0 dB, got {}",
        report.mean_crosstalk_db
    );
    assert!(
        report.max_crosstalk_db <= -20.0,
        "Worst-case crosstalk must be <= -20.0 dB, got {}",
        report.max_crosstalk_db
    );
    assert!(
        report.mean_efficiency_pct >= 70.0,
        "Mean transduction efficiency must be >= 70.0%, got {}",
        report.mean_efficiency_pct
    );
    assert!(
        report.mean_insertion_loss_db <= 2.0,
        "Mean insertion loss must be <= 2.0 dB, got {}",
        report.mean_insertion_loss_db
    );
    assert!(
        report.mean_purity_pct >= 90.0,
        "Mean spin-orbit conversion purity must be >= 90.0%, got {}",
        report.mean_purity_pct
    );
    assert!(
        report.mean_router_extinction_db >= 25.0,
        "Mean router extinction ratio must be >= 25.0 dB, got {}",
        report.mean_router_extinction_db
    );
    assert!(
        report.mean_capacity_gbps >= 10.0,
        "Mean multiplexed capacity must be >= 10.0 Gbps, got {}",
        report.mean_capacity_gbps
    );
    assert!(
        (report.compliance_fraction - 1.0).abs() < 1e-6,
        "Compliance fraction must be 100%, got {}",
        report.compliance_fraction
    );
}
