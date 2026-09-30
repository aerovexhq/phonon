#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for chiral acoustic quantum Hall metamaterials and non-Abelian Moore-Read
//! Pfaffian edge waveguide synthesizers across multi-threaded Rayon workers.

use phonon_solver::chiral_quantum_hall_pfaffian::PfaffianQuantumHallBenchmarkRunner;

#[test]
fn test_10k_chiral_quantum_hall_pfaffian_parallel_sweep() {
    let cycles = 10_000;
    let result = PfaffianQuantumHallBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 155 Chiral Acoustic Quantum Hall Metamaterials & Non-Abelian Pfaffian Edge Waveguide Synthesizers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Pfaffian Topological State Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_pfaffian_topological_state_fidelity,
        result.min_pfaffian_topological_state_fidelity,
        result.max_pfaffian_topological_state_fidelity
    );
    println!(
        "Edge Channel Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_edge_channel_isolation_db,
        result.min_edge_channel_isolation_db,
        result.max_edge_channel_isolation_db
    );
    println!(
        "Neutral Mode Transmission Speed (m/s): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_neutral_mode_transmission_speed_mps,
        result.min_neutral_mode_transmission_speed_mps,
        result.max_neutral_mode_transmission_speed_mps
    );
    println!(
        "Thermal Hall Quantization Error: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_thermal_hall_quantization_error,
        result.min_thermal_hall_quantization_error,
        result.max_thermal_hall_quantization_error
    );
    println!(
        "Quasiparticle Braiding Visibility: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_quasiparticle_braiding_visibility,
        result.min_quasiparticle_braiding_visibility,
        result.max_quasiparticle_braiding_visibility
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
        result.min_pfaffian_topological_state_fidelity >= 0.9970,
        "Minimum Pfaffian state fidelity must be >= 0.9970, got {:.6}",
        result.min_pfaffian_topological_state_fidelity
    );
    assert!(
        result.min_edge_channel_isolation_db >= 46.0,
        "Minimum edge channel isolation must be >= 46.0 dB, got {:.4} dB",
        result.min_edge_channel_isolation_db
    );
    assert!(
        result.min_neutral_mode_transmission_speed_mps >= 1400.0,
        "Minimum neutral mode transmission speed must be >= 1400.0 m/s, got {:.4} m/s",
        result.min_neutral_mode_transmission_speed_mps
    );
    assert!(
        result.max_thermal_hall_quantization_error <= 0.0020,
        "Maximum thermal Hall quantization error must be <= 0.0020, got {:.6}",
        result.max_thermal_hall_quantization_error
    );
    assert!(
        result.min_quasiparticle_braiding_visibility >= 0.985,
        "Minimum quasiparticle braiding visibility must be >= 0.985, got {:.6}",
        result.min_quasiparticle_braiding_visibility
    );
}
