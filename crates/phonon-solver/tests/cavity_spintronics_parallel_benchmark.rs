//! Parallel Multi-Cycle Benchmark Integration Test.

use phonon_solver::cavity_spintronics::{CavityBenchmarkConfig, CavityBenchmarkRunner};

#[test]
fn test_cavity_spintronics_10000_cycles_parallel_benchmark() {
    let runner = CavityBenchmarkRunner::new();
    let config = CavityBenchmarkConfig {
        total_cycles: 10_000,
        center_freq_hz: 10.0e9,
        coupling_rate_hz: 40.0e6,
        drive_field_t: 15.0e-6,
    };

    let report = runner.run(&config);

    println!(
        "Cavity Spintronics Benchmark: {} cycles in {:.2} ms ({:.0} cycles/sec)",
        report.total_cycles, report.elapsed_ms, report.cycles_per_sec
    );
    println!(
        "  Mean Cooperativity C_mp: {:.1}",
        report.mean_cooperativity
    );
    println!("  Min Cooperativity:       {:.1}", report.min_cooperativity);
    println!(
        "  Anti-crossing Splitting: {:.2} MHz",
        report.anticrossing_splitting_hz / 1e6
    );
    println!(
        "  Mean ISHE Voltage:       {:.2} uV",
        report.mean_ishe_voltage_volts * 1e6
    );
    println!(
        "  Max ISHE Voltage:        {:.2} uV",
        report.max_ishe_voltage_volts * 1e6
    );
    println!(
        "  Strong Coupling Frac:    {:.1}%",
        report.strong_coupling_fraction * 100.0
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(
        report.mean_cooperativity > 100.0,
        "Cooperativity must exceed 100"
    );
    assert!(report.min_cooperativity > 50.0);
    assert!(
        report.anticrossing_splitting_hz > 20.0e6,
        "Splitting must exceed 20 MHz"
    );
    assert!(
        report.mean_ishe_voltage_volts > 1.0e-6,
        "Mean ISHE voltage must exceed 1.0 uV"
    );
    assert!(
        report.strong_coupling_fraction > 0.95,
        "Over 95% of configurations must be strongly coupled"
    );
    assert!(
        report.cycles_per_sec > 5_000.0,
        "Benchmark throughput should be high"
    );
}
