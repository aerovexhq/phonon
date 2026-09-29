#![deny(unsafe_code)]

use phonon_solver::floquet_acoustic_chern::FloquetAcousticChernBenchmarkRunner;

#[test]
fn test_floquet_acoustic_chern_parallel_benchmark() {
    let report = FloquetAcousticChernBenchmarkRunner::run_benchmark(10_000);

    println!("\n=== Phase 103: Topological Floquet-Acoustic Chern Benchmark ===");
    println!("Total cycles: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/s",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean Chern number: {:.4} (target |C| = 1.0)",
        report.mean_chern_number
    );
    println!(
        "Mean topological minigap: {:.2} MHz (target >= 2.5 MHz)",
        report.mean_topological_minigap_mhz
    );
    println!(
        "Min topological minigap: {:.2} MHz",
        report.min_topological_minigap_mhz
    );
    println!(
        "Mean forward bend efficiency: {:.2}% (target >= 92.0%)",
        report.mean_forward_bend_efficiency_pct
    );
    println!(
        "Min forward bend efficiency: {:.2}%",
        report.min_forward_bend_efficiency_pct
    );
    println!(
        "Mean reverse isolation: {:.2} dB (target >= 30.0 dB)",
        report.mean_reverse_isolation_db
    );
    println!(
        "Min reverse isolation: {:.2} dB",
        report.min_reverse_isolation_db
    );
    println!(
        "Mean chiral edge velocity: {:.2} m/s",
        report.mean_chiral_edge_velocity_m_s
    );
    println!(
        "Mean beam steering angle: {:.2} deg",
        report.mean_beam_steering_angle_deg
    );
    println!(
        "Physical compliance fraction: {:.4}",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.throughput_cycles_per_sec > 100_000.0);
    assert_eq!(report.mean_chern_number, 1.0);
    assert!(report.min_topological_minigap_mhz >= 2.5);
    assert!(report.min_forward_bend_efficiency_pct >= 92.0);
    assert!(report.min_reverse_isolation_db >= 30.0);
    assert_eq!(report.compliance_fraction, 1.0);
}
