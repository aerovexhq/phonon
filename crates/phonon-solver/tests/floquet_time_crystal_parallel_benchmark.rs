//! Parallel Rayon benchmark verifying Floquet quantum time crystals,
//! lifetime tau >= 1000 cycles, and spectral rigidity contrast >= 20.0 dB across 10,000 parameter sweeps.

use phonon_solver::quantum_time_crystal::TimeCrystalBenchmarkRunner;

#[test]
fn test_floquet_time_crystal_parallel_benchmark_10k_sweeps() {
    let runner = TimeCrystalBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("==================================================================");
    println!("PHASE 84: FLOQUET QUANTUM TIME CRYSTALS & SUBHARMONIC PHONONS");
    println!("==================================================================");
    println!("Total Parameter Sweeps:          {}", report.total_cycles);
    println!(
        "Elapsed Time:                    {:.4} s",
        report.elapsed_seconds
    );
    println!(
        "Throughput:                      {:.2} sweeps/sec",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean DTC Lifetime:               {:.1} cycles (>= 1000 cycles required)",
        report.mean_lifetime_cycles
    );
    println!(
        "Min DTC Lifetime:                {:.1} cycles",
        report.min_lifetime_cycles
    );
    println!(
        "Max DTC Lifetime:                {:.1} cycles",
        report.max_lifetime_cycles
    );
    println!(
        "Mean Rigidity Contrast:          {:.2} dB (>= 20.0 dB required)",
        report.mean_contrast_db
    );
    println!(
        "Min Rigidity Contrast:           {:.2} dB",
        report.min_contrast_db
    );
    println!(
        "Max Rigidity Contrast:           {:.2} dB",
        report.max_contrast_db
    );
    println!(
        "Mean Memory State Fidelity:      {:.2}% (>= 90.0% required)",
        report.mean_memory_fidelity * 100.0
    );
    println!(
        "Min Memory State Fidelity:       {:.2}%",
        report.min_memory_fidelity * 100.0
    );
    println!(
        "Mean Subharmonic Quality Factor: {:.1}",
        report.mean_quality_factor
    );
    println!(
        "Compliance Fraction:             {:.2}%",
        report.compliance_fraction * 100.0
    );
    println!("==================================================================");

    assert_eq!(
        report.total_cycles, 10_000,
        "Total sweeps must equal 10,000"
    );
    assert!(
        report.mean_lifetime_cycles >= 1000.0,
        "Mean lifetime must be >= 1000 cycles, got {:.1} cycles",
        report.mean_lifetime_cycles
    );
    assert!(
        report.min_lifetime_cycles >= 1000.0,
        "Min lifetime must be >= 1000 cycles, got {:.1} cycles",
        report.min_lifetime_cycles
    );
    assert!(
        report.mean_contrast_db >= 20.0,
        "Mean contrast must be >= 20.0 dB, got {:.2} dB",
        report.mean_contrast_db
    );
    assert!(
        report.min_contrast_db >= 20.0,
        "Min contrast must be >= 20.0 dB, got {:.2} dB",
        report.min_contrast_db
    );
    assert!(
        report.mean_memory_fidelity >= 0.90,
        "Mean memory fidelity must be >= 90.0%, got {:.2}%",
        report.mean_memory_fidelity * 100.0
    );
    assert!(
        report.min_memory_fidelity >= 0.90,
        "Min memory fidelity must be >= 90.0%, got {:.2}%",
        report.min_memory_fidelity * 100.0
    );
    assert!(
        report.compliance_fraction >= 1.0,
        "Compliance fraction must be 100%, got {:.2}%",
        report.compliance_fraction * 100.0
    );
}
