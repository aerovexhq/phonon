//! Integration tests benchmarking Multi-Valued Balanced Ternary arithmetic
//! against 64-bit Binary CMOS baselines across Rayon multi-core execution.
//!
//! Validates:
//! 1. Information-theoretic and pin count reductions (41 trits > 64 bits, 35.9% pin reduction; 32 trits, 50% reduction).
//! 2. Transistor count reduction (>65% fewer transistors than static CMOS 28T).
//! 3. Critical path delay speedup and dynamic switching energy reduction (>70%).
//! 4. Interconnect wiring capacitance and Rent's rule routing congestion alleviation.
//! 5. Zero-overhead subtraction latency advantage (sign-free balanced ternary).
//! 6. Multi-core parallel vector additions across thousands of operations.

use phonon_solver::mvl::{MvlBenchmarkRunner, TernaryAdderEngine};

#[test]
fn test_mvl_ternary_vs_binary_arithmetic_benchmark() {
    let runner = MvlBenchmarkRunner::default();
    let reports = runner.run_comparison_suite();

    assert_eq!(reports.len(), 3);
    let bin_64 = &reports[0];
    let ter_41 = &reports[1];
    let ter_32 = &reports[2];

    // 1. Word length & Pin Count Reductions
    assert_eq!(bin_64.pin_count, 64);
    assert_eq!(ter_41.pin_count, 41);
    assert_eq!(ter_32.pin_count, 32);

    assert!(
        ter_41.pin_reduction_percent >= 35.0,
        "41-trit must achieve >= 35% pin reduction over 64-bit binary, actual: {}",
        ter_41.pin_reduction_percent
    );
    assert!(
        (ter_32.pin_reduction_percent - 50.0).abs() < 1e-3,
        "32-trit must achieve 50% pin reduction over 64-bit binary"
    );

    // 2. Transistor Count Reduction (> 65% reduction for 41-trit)
    assert_eq!(bin_64.transistor_count, 1792); // 64 * 28T
    assert_eq!(ter_41.transistor_count, 574); // 41 * 14T
    assert_eq!(ter_32.transistor_count, 448); // 32 * 14T

    assert!(
        ter_41.transistor_reduction_percent > 65.0,
        "41-trit TFA must reduce transistor count by > 65%, actual: {}",
        ter_41.transistor_reduction_percent
    );
    assert!(
        ter_32.transistor_reduction_percent > 70.0,
        "32-trit TFA must reduce transistor count by > 70%, actual: {}",
        ter_32.transistor_reduction_percent
    );

    // 3. Propagation Delay Speedup
    assert!(
        ter_41.delay_speedup_percent >= 30.0,
        "41-trit must have >= 30% lower ripple delay than 64-bit binary"
    );
    assert!(
        ter_32.delay_speedup_percent >= 55.0,
        "32-trit must have >= 55% lower delay than 64-bit binary"
    );

    // 4. Dynamic Switching Energy Reduction
    assert!(
        ter_41.energy_reduction_percent >= 75.0,
        "41-trit must reduce dynamic energy by >= 75% due to fewer FETs and Vdd/2 swing, actual: {}",
        ter_41.energy_reduction_percent
    );

    // 5. Energy-Delay Product (EDP) improvement
    assert!(
        ter_41.edp_zjs < bin_64.edp_zjs * 0.20,
        "41-trit EDP must be at least 5x lower than 64-bit binary"
    );

    // 6. Subtraction speedup (Zero-overhead negation)
    assert!(
        ter_41.subtraction_speedup_percent > ter_41.delay_speedup_percent,
        "Ternary subtraction must outperform binary 2's complement subtraction"
    );
}

#[test]
fn test_mvl_interconnect_capacitance_and_wiring_alleviation() {
    let runner = MvlBenchmarkRunner::default();
    let reports = runner.run_comparison_suite();

    let bin_cap = reports[0].interconnect_capacitance_ff;
    let ter41_cap = reports[1].interconnect_capacitance_ff;
    let ter32_cap = reports[2].interconnect_capacitance_ff;

    // Bus wiring capacitance must be significantly reduced due to fewer tracks and shorter Rent wirelength
    assert!(
        ter41_cap < bin_cap * 0.65,
        "41-trit bus capacitance must be < 65% of 64-bit binary bus, actual: {} vs {}",
        ter41_cap,
        bin_cap
    );
    assert!(
        ter32_cap < bin_cap * 0.50,
        "32-trit bus capacitance must be < 50% of 64-bit binary bus, actual: {} vs {}",
        ter32_cap,
        bin_cap
    );
}

#[test]
fn test_mvl_parallel_vector_arithmetic_across_cpu_cores() {
    let runner = MvlBenchmarkRunner::default();

    // Run 10,000 parallel additions and subtractions across all CPU cores
    let verified = runner.run_parallel_vector_additions(10_000);
    assert!(
        verified,
        "All 10,000 multi-core parallel vector additions/subtractions must match exact math"
    );
}

#[test]
fn test_mvl_zero_overhead_subtraction_equality() {
    let engine = TernaryAdderEngine::new(32);

    let a_val: i64 = 987_654_321;
    let b_val: i64 = 123_456_789;

    let a = engine.i64_to_trits(a_val);
    let b = engine.i64_to_trits(b_val);

    // A - B
    let (diff_trits, _) = engine.sub(&a, &b).unwrap();
    let diff = engine.trits_to_i64(&diff_trits);
    assert_eq!(diff, a_val - b_val);

    // B - A (negative result, naturally signed in balanced ternary)
    let (neg_diff_trits, _) = engine.sub(&b, &a).unwrap();
    let neg_diff = engine.trits_to_i64(&neg_diff_trits);
    assert_eq!(neg_diff, b_val - a_val);

    // Verify exact negation symmetry
    for i in 0..32 {
        assert_eq!(diff_trits[i], -neg_diff_trits[i]);
    }
}
