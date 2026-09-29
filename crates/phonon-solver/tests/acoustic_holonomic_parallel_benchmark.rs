#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-Abelian quantum acoustic holonomic gates and geometric phase processors
//! across multi-threaded Rayon workers.

use phonon_solver::acoustic_holonomic_processor::HolonomicBenchmarkRunner;

#[test]
fn test_10k_acoustic_holonomic_parallel_sweep() {
    let cycles = 10_000;
    let result = HolonomicBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 126 Non-Abelian Acoustic Holonomic Gates Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Holonomic Gate Fidelity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_holonomic_gate_fidelity,
        result.min_holonomic_gate_fidelity,
        result.max_holonomic_gate_fidelity
    );
    println!(
        "Gate Operation Time (ns): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_gate_operation_time_ns,
        result.min_gate_operation_time_ns,
        result.max_gate_operation_time_ns
    );
    println!(
        "Gate Error Rate: mean = {:.5e}, min = {:.5e}, max = {:.5e}",
        result.mean_gate_error_rate,
        result.min_gate_error_rate,
        result.max_gate_error_rate
    );
    println!(
        "Two-Qubit Entangling Fidelity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_two_qubit_entangling_fidelity,
        result.min_two_qubit_entangling_fidelity,
        result.max_two_qubit_entangling_fidelity
    );
    println!(
        "Geometric Purity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_geometric_purity,
        result.min_geometric_purity,
        result.max_geometric_purity
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

    // Assert holonomic gate fidelity >= 0.9950
    assert!(
        result.mean_holonomic_gate_fidelity >= 0.9950,
        "Mean holonomic gate fidelity must be >= 0.9950, got {:.5}",
        result.mean_holonomic_gate_fidelity
    );
    assert!(
        result.min_holonomic_gate_fidelity >= 0.9950,
        "Min holonomic gate fidelity must be >= 0.9950, got {:.5}",
        result.min_holonomic_gate_fidelity
    );

    // Assert gate operation time <= 200.0 ns
    assert!(
        result.mean_gate_operation_time_ns <= 200.0,
        "Mean gate operation time must be <= 200.0 ns, got {:.5}",
        result.mean_gate_operation_time_ns
    );
    assert!(
        result.max_gate_operation_time_ns <= 200.0,
        "Max gate operation time must be <= 200.0 ns, got {:.5}",
        result.max_gate_operation_time_ns
    );

    // Assert gate error rate <= 1.0e-3
    assert!(
        result.mean_gate_error_rate <= 1.0e-3,
        "Mean gate error rate must be <= 1.0e-3, got {:.5e}",
        result.mean_gate_error_rate
    );
    assert!(
        result.max_gate_error_rate <= 1.0e-3,
        "Max gate error rate must be <= 1.0e-3, got {:.5e}",
        result.max_gate_error_rate
    );

    // Assert two-qubit entangling fidelity >= 0.9920
    assert!(
        result.mean_two_qubit_entangling_fidelity >= 0.9920,
        "Mean two-qubit entangling fidelity must be >= 0.9920, got {:.5}",
        result.mean_two_qubit_entangling_fidelity
    );
    assert!(
        result.min_two_qubit_entangling_fidelity >= 0.9920,
        "Min two-qubit entangling fidelity must be >= 0.9920, got {:.5}",
        result.min_two_qubit_entangling_fidelity
    );

    // Assert geometric purity >= 0.9900
    assert!(
        result.mean_geometric_purity >= 0.9900,
        "Mean geometric purity must be >= 0.9900, got {:.5}",
        result.mean_geometric_purity
    );
    assert!(
        result.min_geometric_purity >= 0.9900,
        "Min geometric purity must be >= 0.9900, got {:.5}",
        result.min_geometric_purity
    );
}
