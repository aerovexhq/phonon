//! Parallel Rayon benchmark verifying quantum axion electrodynamics,
//! haloscope SNR (>= 15.0 dB), and topological magnetoplasmon isolation (>= 25.0 dB) across 10,000 parameter sweeps.

use phonon_solver::axion_electrodynamics::AxionElectrodynamicsBenchmarkRunner;

#[test]
fn test_axion_electrodynamics_parallel_benchmark_10k_sweeps() {
    let runner = AxionElectrodynamicsBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("==================================================================");
    println!("PHASE 82: QUANTUM AXION ELECTRODYNAMICS & MAGNETOPLASMONS");
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
        "Mean Haloscope SNR:              {:.2} dB (>= 15.0 dB required)",
        report.mean_snr_db
    );
    println!(
        "Min Haloscope SNR:               {:.2} dB",
        report.min_snr_db
    );
    println!(
        "Max Haloscope SNR:               {:.2} dB",
        report.max_snr_db
    );
    println!(
        "Mean Magnetoplasmon Isolation:   {:.2} dB (>= 25.0 dB required)",
        report.mean_isolation_db
    );
    println!(
        "Min Magnetoplasmon Isolation:    {:.2} dB",
        report.min_isolation_db
    );
    println!(
        "Max Magnetoplasmon Isolation:    {:.2} dB",
        report.max_isolation_db
    );
    println!(
        "Mean Converted Signal Power:     {:.2} dBm",
        report.mean_conversion_power_dbm
    );
    println!(
        "Mean Witten Conductance:         {:.4e} S",
        report.mean_witten_conductance_siemens
    );
    println!(
        "Mean Polariton Anti-Crossing:    {:.2} GHz",
        report.mean_polariton_gap_ghz
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
        report.mean_snr_db >= 15.0,
        "Mean SNR must be >= 15.0 dB, got {:.2} dB",
        report.mean_snr_db
    );
    assert!(
        report.min_snr_db >= 15.0,
        "Min SNR must be >= 15.0 dB, got {:.2} dB",
        report.min_snr_db
    );
    assert!(
        report.mean_isolation_db >= 25.0,
        "Mean isolation contrast must be >= 25.0 dB, got {:.2} dB",
        report.mean_isolation_db
    );
    assert!(
        report.min_isolation_db >= 25.0,
        "Min isolation contrast must be >= 25.0 dB, got {:.2} dB",
        report.min_isolation_db
    );
    assert!(
        report.compliance_fraction >= 1.0,
        "Compliance fraction must be 100%, got {:.2}%",
        report.compliance_fraction * 100.0
    );
}
