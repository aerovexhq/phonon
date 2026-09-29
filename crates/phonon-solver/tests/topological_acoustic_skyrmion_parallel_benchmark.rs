#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for topological acoustic skyrmion lattices and chiral phononic neuromorphic
//! processing engines across multi-threaded Rayon workers.

use phonon_solver::topological_acoustic_skyrmion::SkyrmionBenchmarkRunner;

#[test]
fn test_10k_topological_acoustic_skyrmion_parallel_sweep() {
    let cycles = 10_000;
    let result = SkyrmionBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 142 Topological Acoustic Skyrmion Lattices & Chiral Phononic Neuromorphic Processing Engines Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Synaptic State Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_synaptic_state_fidelity,
        result.min_synaptic_state_fidelity,
        result.max_synaptic_state_fidelity
    );
    println!(
        "Skyrmion Propagation Velocity (m/s): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_skyrmion_propagation_velocity_mps,
        result.min_skyrmion_propagation_velocity_mps,
        result.max_skyrmion_propagation_velocity_mps
    );
    println!(
        "Topological Charge Quantization Error: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_topological_charge_quantization_error,
        result.min_topological_charge_quantization_error,
        result.max_topological_charge_quantization_error
    );
    println!(
        "Neuromorphic Energy Dissipation (aJ): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_neuromorphic_energy_dissipation_aj,
        result.min_neuromorphic_energy_dissipation_aj,
        result.max_neuromorphic_energy_dissipation_aj
    );
    println!(
        "State Retention Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_state_retention_isolation_db,
        result.min_state_retention_isolation_db,
        result.max_state_retention_isolation_db
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
        result.min_synaptic_state_fidelity >= 0.9960,
        "Minimum synaptic state fidelity must be >= 0.9960"
    );
    assert!(
        result.min_skyrmion_propagation_velocity_mps >= 850.0,
        "Minimum skyrmion propagation velocity must be >= 850.0 m/s"
    );
    assert!(
        result.max_topological_charge_quantization_error <= 0.0030,
        "Maximum topological charge error must be <= 0.0030"
    );
    assert!(
        result.max_neuromorphic_energy_dissipation_aj <= 15.0,
        "Maximum neuromorphic energy dissipation must be <= 15.0 aJ"
    );
    assert!(
        result.min_state_retention_isolation_db >= 42.0,
        "Minimum state retention isolation must be >= 42.0 dB"
    );
}
