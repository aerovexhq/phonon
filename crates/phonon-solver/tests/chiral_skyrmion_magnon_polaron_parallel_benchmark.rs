#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for topological acoustic chiral skyrmion-lattice transducers and non-reciprocal
//! magnon-polaron interconnects across multi-threaded Rayon workers.

use phonon_solver::chiral_skyrmion_magnon_polaron::SkyrmionPolaronBenchmarkRunner;

#[test]
fn test_10k_chiral_skyrmion_magnon_polaron_parallel_sweep() {
    let cycles = 10_000;
    let result = SkyrmionPolaronBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 153 Topological Acoustic Chiral Skyrmion-Lattice Transducers & Non-Reciprocal Magnon-Polaron Interconnects Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Topological Hall Angle (deg): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_hall_angle_deg,
        result.min_topological_hall_angle_deg,
        result.max_topological_hall_angle_deg
    );
    println!(
        "Magnon-Polaron Transfer Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_magnon_polaron_transfer_fidelity,
        result.min_magnon_polaron_transfer_fidelity,
        result.max_magnon_polaron_transfer_fidelity
    );
    println!(
        "Non-Reciprocal Acoustic Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_non_reciprocal_acoustic_isolation_db,
        result.min_non_reciprocal_acoustic_isolation_db,
        result.max_non_reciprocal_acoustic_isolation_db
    );
    println!(
        "Skyrmion Drift Velocity (m/s): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_skyrmion_drift_velocity_mps,
        result.min_skyrmion_drift_velocity_mps,
        result.max_skyrmion_drift_velocity_mps
    );
    println!(
        "Topological Charge Stability Ratio: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_topological_charge_stability_ratio,
        result.min_topological_charge_stability_ratio,
        result.max_topological_charge_stability_ratio
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
        result.min_topological_hall_angle_deg >= 18.0,
        "Minimum topological Hall angle must be >= 18.0 deg, got {:.4} deg",
        result.min_topological_hall_angle_deg
    );
    assert!(
        result.min_magnon_polaron_transfer_fidelity >= 0.9970,
        "Minimum magnon-polaron transfer fidelity must be >= 0.9970, got {:.6}",
        result.min_magnon_polaron_transfer_fidelity
    );
    assert!(
        result.min_non_reciprocal_acoustic_isolation_db >= 48.0,
        "Minimum non-reciprocal acoustic isolation must be >= 48.0 dB, got {:.4} dB",
        result.min_non_reciprocal_acoustic_isolation_db
    );
    assert!(
        result.min_skyrmion_drift_velocity_mps >= 180.0,
        "Minimum skyrmion drift velocity must be >= 180.0 m/s, got {:.4} m/s",
        result.min_skyrmion_drift_velocity_mps
    );
    assert!(
        result.min_topological_charge_stability_ratio >= 0.990,
        "Minimum topological charge stability ratio must be >= 0.990, got {:.6}",
        result.min_topological_charge_stability_ratio
    );
}
