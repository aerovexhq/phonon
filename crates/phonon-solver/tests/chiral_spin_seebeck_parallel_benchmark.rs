//! Parallel Rayon benchmark verifying chiral phonon-magnon spin Seebeck cascades,
//! thermal rectification ratio (>= 10.0x), and spin Seebeck voltage (>= 5.0 uV) across 10,000 parameter sweeps.

use phonon_solver::chiral_spin_seebeck::ChiralSpinSeebeckBenchmarkRunner;

#[test]
fn test_chiral_spin_seebeck_parallel_benchmark_10k_sweeps() {
    let runner = ChiralSpinSeebeckBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("==================================================================");
    println!("PHASE 83: CHIRAL PHONON-MAGNON SPIN SEEBECK & THERMAL RECTIFIERS");
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
        "Mean Rectification Ratio:        {:.2}x (>= 10.0x required)",
        report.mean_rectification_ratio
    );
    println!(
        "Min Rectification Ratio:         {:.2}x",
        report.min_rectification_ratio
    );
    println!(
        "Max Rectification Ratio:         {:.2}x",
        report.max_rectification_ratio
    );
    println!(
        "Mean Spin Seebeck Voltage:       {:.2} uV (>= 5.0 uV required)",
        report.mean_seebeck_voltage_uv
    );
    println!(
        "Min Spin Seebeck Voltage:        {:.2} uV",
        report.min_seebeck_voltage_uv
    );
    println!(
        "Max Spin Seebeck Voltage:        {:.2} uV",
        report.max_seebeck_voltage_uv
    );
    println!(
        "Mean Thermocell Power Output:    {:.2} pW",
        report.mean_power_output_pw
    );
    println!(
        "Mean Thermocell Efficiency:      {:.6}%",
        report.mean_efficiency_percent
    );
    println!(
        "Mean Injected Spin Current:      {:.2e} A/m^2",
        report.mean_spin_current_density_a_m2
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
        report.mean_rectification_ratio >= 10.0,
        "Mean rectification ratio must be >= 10.0x, got {:.2}x",
        report.mean_rectification_ratio
    );
    assert!(
        report.min_rectification_ratio >= 10.0,
        "Min rectification ratio must be >= 10.0x, got {:.2}x",
        report.min_rectification_ratio
    );
    assert!(
        report.mean_seebeck_voltage_uv >= 5.0,
        "Mean Seebeck voltage must be >= 5.0 uV, got {:.2} uV",
        report.mean_seebeck_voltage_uv
    );
    assert!(
        report.min_seebeck_voltage_uv >= 5.0,
        "Min Seebeck voltage must be >= 5.0 uV, got {:.2} uV",
        report.min_seebeck_voltage_uv
    );
    assert!(
        report.compliance_fraction >= 1.0,
        "Compliance fraction must be 100%, got {:.2}%",
        report.compliance_fraction * 100.0
    );
}
