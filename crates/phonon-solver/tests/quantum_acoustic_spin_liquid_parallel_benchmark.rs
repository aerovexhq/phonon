#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-Abelian quantum acoustic fractional spin liquids and topological
//! resonating valence bond networks across multi-threaded Rayon workers.

use phonon_solver::quantum_acoustic_spin_liquid::SpinLiquidBenchmarkRunner;

#[test]
fn test_10k_quantum_acoustic_spin_liquid_parallel_sweep() {
    let cycles = 10_000;
    let result = SpinLiquidBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 152 Non-Abelian Quantum Acoustic Fractional Spin Liquids & Topological Resonating Valence Bond Networks Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Spinon Excitation Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_spinon_excitation_fidelity,
        result.min_spinon_excitation_fidelity,
        result.max_spinon_excitation_fidelity
    );
    println!(
        "Topological Entanglement Entropy: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_topological_entanglement_entropy,
        result.min_topological_entanglement_entropy,
        result.max_topological_entanglement_entropy
    );
    println!(
        "Topological Entropy Error: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_topological_entropy_error,
        result.min_topological_entropy_error,
        result.max_topological_entropy_error
    );
    println!(
        "Spin-Mechanical Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_spin_mechanical_crosstalk_isolation_db,
        result.min_spin_mechanical_crosstalk_isolation_db,
        result.max_spin_mechanical_crosstalk_isolation_db
    );
    println!(
        "Ground-State Degeneracy Protection (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_ground_state_degeneracy_protection_db,
        result.min_ground_state_degeneracy_protection_db,
        result.max_ground_state_degeneracy_protection_db
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
        result.min_spinon_excitation_fidelity >= 0.9960,
        "Minimum spinon excitation fidelity must be >= 0.9960, got {:.6}",
        result.min_spinon_excitation_fidelity
    );
    assert!(
        result.min_topological_entanglement_entropy >= 0.6793,
        "Minimum topological entanglement entropy must be >= 0.6793, got {:.6}",
        result.min_topological_entanglement_entropy
    );
    assert!(
        result.max_topological_entropy_error <= 0.0020,
        "Maximum topological entropy error must be <= 0.0020, got {:.6}",
        result.max_topological_entropy_error
    );
    assert!(
        result.min_spin_mechanical_crosstalk_isolation_db >= 44.0,
        "Minimum spin-mechanical crosstalk isolation must be >= 44.0 dB, got {:.4} dB",
        result.min_spin_mechanical_crosstalk_isolation_db
    );
    assert!(
        result.min_ground_state_degeneracy_protection_db >= 40.0,
        "Minimum ground-state degeneracy protection must be >= 40.0 dB, got {:.4} dB",
        result.min_ground_state_degeneracy_protection_db
    );
}
