#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for 3D topological acoustic higher-order axion insulators and chiral hinge
//! soliton networks across multi-threaded Rayon workers.

use phonon_solver::chiral_hinge_axion_soliton::HingeAxionBenchmarkRunner;

#[test]
fn test_10k_chiral_hinge_axion_soliton_parallel_sweep() {
    let cycles = 10_000;
    let result = HingeAxionBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 150 Topological Acoustic Higher-Order Axion Insulators & Chiral Hinge Soliton Networks Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Hinge State Transmission Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_hinge_state_transmission_fidelity,
        result.min_hinge_state_transmission_fidelity,
        result.max_hinge_state_transmission_fidelity
    );
    println!(
        "Topological Axion Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_axion_gap_mhz,
        result.min_topological_axion_gap_mhz,
        result.max_topological_axion_gap_mhz
    );
    println!(
        "Non-Linear Harmonic Distortion (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_non_linear_harmonic_distortion_db,
        result.min_non_linear_harmonic_distortion_db,
        result.max_non_linear_harmonic_distortion_db
    );
    println!(
        "Inter-Hinge Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_inter_hinge_crosstalk_isolation_db,
        result.min_inter_hinge_crosstalk_isolation_db,
        result.max_inter_hinge_crosstalk_isolation_db
    );
    println!(
        "Hinge Soliton Group Velocity (m/s): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_hinge_soliton_group_velocity_mps,
        result.min_hinge_soliton_group_velocity_mps,
        result.max_hinge_soliton_group_velocity_mps
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
        result.min_hinge_state_transmission_fidelity >= 0.9970,
        "Minimum hinge state transmission fidelity must be >= 0.9970, got {:.6}",
        result.min_hinge_state_transmission_fidelity
    );
    assert!(
        result.min_topological_axion_gap_mhz >= 25.0,
        "Minimum topological axion gap must be >= 25.0 MHz, got {:.4} MHz",
        result.min_topological_axion_gap_mhz
    );
    assert!(
        result.max_non_linear_harmonic_distortion_db <= -48.0,
        "Maximum non-linear harmonic distortion must be <= -48.0 dB, got {:.4} dB",
        result.max_non_linear_harmonic_distortion_db
    );
    assert!(
        result.min_inter_hinge_crosstalk_isolation_db >= 46.0,
        "Minimum inter-hinge crosstalk isolation must be >= 46.0 dB, got {:.4} dB",
        result.min_inter_hinge_crosstalk_isolation_db
    );
    assert!(
        result.min_hinge_soliton_group_velocity_mps >= 2200.0,
        "Minimum hinge soliton group velocity must be >= 2200.0 m/s, got {:.4} m/s",
        result.min_hinge_soliton_group_velocity_mps
    );
}
