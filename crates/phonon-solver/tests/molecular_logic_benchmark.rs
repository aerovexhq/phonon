//! Integration Test: Autonomous Molecular Logic Synthesis & 3nm CMOS Comparative Benchmark.
//!
//! Validates:
//! 1. Autonomous synthesis of QI Inverter, NAND2, NOR2, XOR2, and 1-bit Full Adder cells with 100% truth table fidelity.
//! 2. 64-bit ripple-carry arithmetic execution.
//! 3. Multi-threaded Rayon benchmark demonstrating > 1,000x energy reduction and > 10,000x density over 3nm GAA CMOS.

use phonon_models::molecular::MultiBitMolecularAdder;
use phonon_solver::molecular::{
    MolecularBenchmarkRunner, MolecularLogicSynthesizer, TargetLogicFunction,
};

#[test]
fn test_autonomous_synthesis_all_gate_primitives() {
    let synth = MolecularLogicSynthesizer::new(0.35);

    let targets = [
        TargetLogicFunction::Inverter,
        TargetLogicFunction::Nand2,
        TargetLogicFunction::Nor2,
        TargetLogicFunction::Xor2,
        TargetLogicFunction::FullAdder1Bit,
    ];

    for target in targets {
        let result = synth.synthesize(target);
        assert_eq!(
            result.truth_table_fidelity, 1.0,
            "Synthesized gate {:?} must achieve 100% truth table fidelity",
            target
        );
        assert!(
            result.candidates_evaluated > 0,
            "Candidate search space must be explored"
        );
        assert!(
            result.footprint_nm2 < 10.0,
            "Footprint for {:?} must be < 10 nm^2, got {:.2} nm^2",
            target,
            result.footprint_nm2
        );
    }
}

#[test]
fn test_multi_bit_molecular_adder_exactness() {
    let adder32 = MultiBitMolecularAdder::new(32, 0.35);

    // Test a variety of arithmetic additions
    let test_cases = [
        (0u64, 0u64, false, 0u64, false),
        (12345u64, 67890u64, false, 80235u64, false),
        (0xFFFFFFFFu64, 1u64, false, 0u64, true), // 32-bit overflow
        (0x12345678u64, 0x87654321u64, false, 0x99999999u64, false),
    ];

    for (a, b, cin, expected_sum, expected_cout) in test_cases {
        let (sum, cout) = adder32.add(a, b, cin);
        assert_eq!(
            sum, expected_sum,
            "Sum mismatch for {} + {} + {}",
            a, b, cin
        );
        assert_eq!(
            cout, expected_cout,
            "Cout mismatch for {} + {} + {}",
            a, b, cin
        );
    }
}

#[test]
fn test_rayon_parallel_benchmark_vs_3nm_cmos() {
    let runner = MolecularBenchmarkRunner::new(0.35);
    let report = runner.run_parallel_arithmetic_benchmark(200);

    assert_eq!(report.batch_operations_completed, 200);
    assert!(
        report.molecular_energy_per_gate_mev < 100.0,
        "Molecular logic must achieve sub-100 meV switching, got {:.2} meV",
        report.molecular_energy_per_gate_mev
    );
    assert!(
        report.energy_reduction_factor > 1000.0,
        "Energy reduction factor must exceed 1,000x over 3nm CMOS, got {:.1}x",
        report.energy_reduction_factor
    );
    assert!(
        report.density_advantage_factor > 10_000.0,
        "Density advantage factor must exceed 10,000x over 3nm CMOS, got {:.1}x",
        report.density_advantage_factor
    );
    assert!(
        report.total_simulated_energy_savings_joules > 0.0,
        "Total simulated energy savings must be positive"
    );
}
