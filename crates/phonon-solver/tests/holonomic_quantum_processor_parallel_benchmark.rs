#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum non-Abelian holonomic acoustic gate processors and braided
//! phonon circuit architectures across multi-threaded Rayon workers.

use phonon_solver::holonomic_quantum_processor::HolonomicProcessorBenchmarkRunner;

#[test]
fn test_10k_holonomic_quantum_processor_parallel_sweep() {
    let cycles = 10_000;
    let result = HolonomicProcessorBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 138 Holonomic Quantum Processor & Braided Circuits Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Holonomic Gate Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_holonomic_gate_fidelity,
        result.min_holonomic_gate_fidelity,
        result.max_holonomic_gate_fidelity
    );
    println!(
        "Two-Qubit Gate Duration (ns): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_two_qubit_gate_duration_ns,
        result.min_two_qubit_gate_duration_ns,
        result.max_two_qubit_gate_duration_ns
    );
    println!(
        "Geometric Phase Error: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_geometric_phase_error,
        result.min_geometric_phase_error,
        result.max_geometric_phase_error
    );
    println!(
        "Fault-Tolerant Logic Depth: mean = {:.2}, min = {}, max = {}",
        result.mean_fault_tolerant_logic_depth,
        result.min_fault_tolerant_logic_depth,
        result.max_fault_tolerant_logic_depth
    );
    println!(
        "Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_crosstalk_isolation_db,
        result.min_crosstalk_isolation_db,
        result.max_crosstalk_isolation_db
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

    // Assert holonomic gate fidelity >= 0.9960
    assert!(
        result.mean_holonomic_gate_fidelity >= 0.9960,
        "Mean gate fidelity must be >= 0.9960, got {:.6}",
        result.mean_holonomic_gate_fidelity
    );
    assert!(
        result.min_holonomic_gate_fidelity >= 0.9960,
        "Min gate fidelity must be >= 0.9960, got {:.6}",
        result.min_holonomic_gate_fidelity
    );

    // Assert two-qubit gate duration <= 35.0 ns
    assert!(
        result.mean_two_qubit_gate_duration_ns <= 35.0,
        "Mean gate duration must be <= 35.0 ns, got {:.4}",
        result.mean_two_qubit_gate_duration_ns
    );
    assert!(
        result.max_two_qubit_gate_duration_ns <= 35.0,
        "Max gate duration must be <= 35.0 ns, got {:.4}",
        result.max_two_qubit_gate_duration_ns
    );

    // Assert geometric phase error <= 0.0050
    assert!(
        result.mean_geometric_phase_error <= 0.0050,
        "Mean geometric phase error must be <= 0.0050, got {:.6}",
        result.mean_geometric_phase_error
    );
    assert!(
        result.max_geometric_phase_error <= 0.0050,
        "Max geometric phase error must be <= 0.0050, got {:.6}",
        result.max_geometric_phase_error
    );

    // Assert fault-tolerant logic depth >= 100
    assert!(
        result.mean_fault_tolerant_logic_depth >= 100.0,
        "Mean logic depth must be >= 100, got {:.2}",
        result.mean_fault_tolerant_logic_depth
    );
    assert!(
        result.min_fault_tolerant_logic_depth >= 100,
        "Min logic depth must be >= 100, got {}",
        result.min_fault_tolerant_logic_depth
    );

    // Assert crosstalk isolation >= 40.0 dB
    assert!(
        result.mean_crosstalk_isolation_db >= 40.0,
        "Mean crosstalk isolation must be >= 40.0 dB, got {:.4}",
        result.mean_crosstalk_isolation_db
    );
    assert!(
        result.min_crosstalk_isolation_db >= 40.0,
        "Min crosstalk isolation must be >= 40.0 dB, got {:.4}",
        result.min_crosstalk_isolation_db
    );
}
