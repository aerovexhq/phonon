#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for topological moire acoustic polaritonic lattices and flat-band phonon
//! superfluidity across multi-threaded Rayon workers.

use phonon_solver::topological_moire_polariton::MoirePolaritonBenchmarkRunner;

#[test]
fn test_10k_topological_moire_polariton_parallel_sweep() {
    let cycles = 10_000;
    let result = MoirePolaritonBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 134 Topological Moire Acoustic Polaritonic Lattices Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Superfluid Velocity (m/s): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_superfluid_velocity_m_per_s,
        result.min_superfluid_velocity_m_per_s,
        result.max_superfluid_velocity_m_per_s
    );
    println!(
        "Propagation Loss (dB/cm): mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_propagation_loss_db_per_cm,
        result.min_propagation_loss_db_per_cm,
        result.max_propagation_loss_db_per_cm
    );
    println!(
        "Condensation Threshold Density (m^-2): mean = {:.5e}, min = {:.5e}, max = {:.5e}",
        result.mean_condensation_threshold_density,
        result.min_condensation_threshold_density,
        result.max_condensation_threshold_density
    );
    println!(
        "Chern Number: mean = {:.1}, min = {}, max = {}",
        result.mean_chern_number, result.min_chern_number, result.max_chern_number
    );
    println!(
        "Flat-Band Bandwidth (MHz): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_flat_band_bandwidth_mhz,
        result.min_flat_band_bandwidth_mhz,
        result.max_flat_band_bandwidth_mhz
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

    // Assert superfluid velocity >= 2500.0 m/s
    assert!(
        result.mean_superfluid_velocity_m_per_s >= 2500.0,
        "Mean superfluid velocity must be >= 2500.0 m/s, got {:.5}",
        result.mean_superfluid_velocity_m_per_s
    );
    assert!(
        result.min_superfluid_velocity_m_per_s >= 2500.0,
        "Min superfluid velocity must be >= 2500.0 m/s, got {:.5}",
        result.min_superfluid_velocity_m_per_s
    );

    // Assert propagation loss <= 0.020 dB/cm
    assert!(
        result.mean_propagation_loss_db_per_cm <= 0.020,
        "Mean propagation loss must be <= 0.020 dB/cm, got {:.6}",
        result.mean_propagation_loss_db_per_cm
    );
    assert!(
        result.max_propagation_loss_db_per_cm <= 0.020,
        "Max propagation loss must be <= 0.020 dB/cm, got {:.6}",
        result.max_propagation_loss_db_per_cm
    );

    // Assert condensation threshold density <= 5.0e12 m^-2
    assert!(
        result.mean_condensation_threshold_density <= 5.0e12,
        "Mean condensation threshold density must be <= 5.0e12, got {:.5e}",
        result.mean_condensation_threshold_density
    );
    assert!(
        result.max_condensation_threshold_density <= 5.0e12,
        "Max condensation threshold density must be <= 5.0e12, got {:.5e}",
        result.max_condensation_threshold_density
    );

    // Assert quantized Chern number C == 1
    assert_eq!(
        result.min_chern_number, 1,
        "Min Chern number must be 1, got {}",
        result.min_chern_number
    );
    assert_eq!(
        result.max_chern_number, 1,
        "Max Chern number must be 1, got {}",
        result.max_chern_number
    );

    // Assert flat-band bandwidth <= 2.0 MHz
    assert!(
        result.mean_flat_band_bandwidth_mhz <= 2.0,
        "Mean flat-band bandwidth must be <= 2.0 MHz, got {:.5}",
        result.mean_flat_band_bandwidth_mhz
    );
    assert!(
        result.max_flat_band_bandwidth_mhz <= 2.0,
        "Max flat-band bandwidth must be <= 2.0 MHz, got {:.5}",
        result.max_flat_band_bandwidth_mhz
    );
}
