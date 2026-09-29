#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-Abelian anyon braiding in chiral acoustic Chern metamaterials and
//! fault-tolerant phononic topological qubits across multi-threaded Rayon workers.

use phonon_solver::chiral_chern_anyon_braiding::BraidingBenchmarkRunner;

#[test]
fn test_10k_chiral_chern_anyon_braiding_parallel_sweep() {
    let cycles = 10_000;
    let result = BraidingBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 132 Non-Abelian Anyon Braiding in Chiral Acoustic Chern Metamaterials Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Braiding Gate Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_braiding_gate_fidelity,
        result.min_braiding_gate_fidelity,
        result.max_braiding_gate_fidelity
    );
    println!(
        "Topological Protection Gap (MHz): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Anyon Collision Visibility: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_anyon_collision_visibility,
        result.min_anyon_collision_visibility,
        result.max_anyon_collision_visibility
    );
    println!(
        "Non-Adiabatic Leakage Rate: mean = {:.6e}, min = {:.6e}, max = {:.6e}",
        result.mean_non_adiabatic_leakage_rate,
        result.min_non_adiabatic_leakage_rate,
        result.max_non_adiabatic_leakage_rate
    );
    println!(
        "Topological Qubit Coherence (ms): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_topological_qubit_coherence_ms,
        result.min_topological_qubit_coherence_ms,
        result.max_topological_qubit_coherence_ms
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

    // Assert braiding gate fidelity >= 0.9980
    assert!(
        result.mean_braiding_gate_fidelity >= 0.9980,
        "Mean braiding gate fidelity must be >= 0.9980, got {:.6}",
        result.mean_braiding_gate_fidelity
    );
    assert!(
        result.min_braiding_gate_fidelity >= 0.9980,
        "Min braiding gate fidelity must be >= 0.9980, got {:.6}",
        result.min_braiding_gate_fidelity
    );

    // Assert topological protection gap >= 18.0 MHz
    assert!(
        result.mean_topological_protection_gap_mhz >= 18.0,
        "Mean topological protection gap must be >= 18.0 MHz, got {:.5}",
        result.mean_topological_protection_gap_mhz
    );
    assert!(
        result.min_topological_protection_gap_mhz >= 18.0,
        "Min topological protection gap must be >= 18.0 MHz, got {:.5}",
        result.min_topological_protection_gap_mhz
    );

    // Assert anyon collision visibility >= 0.950
    assert!(
        result.mean_anyon_collision_visibility >= 0.950,
        "Mean anyon collision visibility must be >= 0.950, got {:.5}",
        result.mean_anyon_collision_visibility
    );
    assert!(
        result.min_anyon_collision_visibility >= 0.950,
        "Min anyon collision visibility must be >= 0.950, got {:.5}",
        result.min_anyon_collision_visibility
    );

    // Assert non-adiabatic leakage rate <= 1.0e-5
    assert!(
        result.mean_non_adiabatic_leakage_rate <= 1.0e-5,
        "Mean non-adiabatic leakage rate must be <= 1.0e-5, got {:.6e}",
        result.mean_non_adiabatic_leakage_rate
    );
    assert!(
        result.max_non_adiabatic_leakage_rate <= 1.0e-5,
        "Max non-adiabatic leakage rate must be <= 1.0e-5, got {:.6e}",
        result.max_non_adiabatic_leakage_rate
    );

    // Assert topological qubit coherence >= 12.0 ms
    assert!(
        result.mean_topological_qubit_coherence_ms >= 12.0,
        "Mean topological qubit coherence must be >= 12.0 ms, got {:.5}",
        result.mean_topological_qubit_coherence_ms
    );
    assert!(
        result.min_topological_qubit_coherence_ms >= 12.0,
        "Min topological qubit coherence must be >= 12.0 ms, got {:.5}",
        result.min_topological_qubit_coherence_ms
    );
}
