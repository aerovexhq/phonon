#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic chiral fractional Chern-Simons hydrodynamics and anyonic
//! holographic edge viscometers across multi-threaded Rayon workers.

use phonon_solver::fractional_chern_simons_viscometer::ChernSimonsViscometerBenchmarkRunner;

#[test]
fn test_10k_fractional_chern_simons_viscometer_parallel_sweep() {
    let cycles = 10_000;
    let result = ChernSimonsViscometerBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 164 Quantum Acoustic Chiral Fractional Chern-Simons Hydrodynamics & Anyonic Holographic Edge Viscometers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Hall Viscosity Measurement Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_hall_viscosity_measurement_fidelity,
        result.min_hall_viscosity_measurement_fidelity,
        result.max_hall_viscosity_measurement_fidelity
    );
    println!(
        "Edge-to-Bulk Acoustic Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_edge_to_bulk_acoustic_isolation_db,
        result.min_edge_to_bulk_acoustic_isolation_db,
        result.max_edge_to_bulk_acoustic_isolation_db
    );
    println!(
        "Edge Mode Velocity Stability Fraction: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_edge_mode_velocity_stability_fraction,
        result.min_edge_mode_velocity_stability_fraction,
        result.max_edge_mode_velocity_stability_fraction
    );
    println!(
        "Anomalous Edge Acoustic Dissipation (dB/um): mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_anomalous_edge_acoustic_dissipation_db_per_um,
        result.min_anomalous_edge_acoustic_dissipation_db_per_um,
        result.max_anomalous_edge_acoustic_dissipation_db_per_um
    );
    println!(
        "Hydrodynamic Entropy Generation Rate (W/K): mean = {:.6e}, min = {:.6e}, max = {:.6e}",
        result.mean_hydrodynamic_entropy_generation_rate_w_per_k,
        result.min_hydrodynamic_entropy_generation_rate_w_per_k,
        result.max_hydrodynamic_entropy_generation_rate_w_per_k
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
        result.min_hall_viscosity_measurement_fidelity >= 0.9980,
        "Minimum Hall viscosity measurement fidelity must be >= 0.9980, got {:.6}",
        result.min_hall_viscosity_measurement_fidelity
    );
    assert!(
        result.min_edge_to_bulk_acoustic_isolation_db >= 55.0,
        "Minimum edge-to-bulk acoustic isolation must be >= 55.0 dB, got {:.4}",
        result.min_edge_to_bulk_acoustic_isolation_db
    );
    assert!(
        result.min_edge_mode_velocity_stability_fraction >= 0.9970,
        "Minimum edge mode velocity stability fraction must be >= 0.9970, got {:.6}",
        result.min_edge_mode_velocity_stability_fraction
    );
    assert!(
        result.max_anomalous_edge_acoustic_dissipation_db_per_um <= 0.0015,
        "Maximum anomalous edge acoustic dissipation must be <= 0.0015 dB/um, got {:.6}",
        result.max_anomalous_edge_acoustic_dissipation_db_per_um
    );
    assert!(
        result.max_hydrodynamic_entropy_generation_rate_w_per_k <= 1.0e-5,
        "Maximum hydrodynamic entropy generation rate must be <= 1.0e-5 W/K, got {:.6e}",
        result.max_hydrodynamic_entropy_generation_rate_w_per_k
    );
}
