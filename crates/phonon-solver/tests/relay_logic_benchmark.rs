//! Integration tests for autonomous relay logic synthesis, zero standby leakage, and parallel multi-core benchmarking.

use phonon_models::relay::MultiBitRelayAdder;
use phonon_solver::{AutonomousRelaySynthesizer, RelayBenchmarkRunner, RelaySynthesisTarget};

#[test]
fn test_autonomous_relay_logic_synthesis_and_component_reduction() {
    let synth = AutonomousRelaySynthesizer::new(0.085);

    // 1. Synthesize 2-input NAND
    let nand = synth.synthesize(RelaySynthesisTarget::Nand2);
    assert_eq!(nand.truth_table_fidelity, 1.0);
    assert_eq!(nand.relay_count, 3);
    assert!(nand.transistor_savings_percent >= 25.0); // 3 relays vs 4 CMOS transistors

    // 2. Synthesize 2-input XOR
    let xor = synth.synthesize(RelaySynthesisTarget::Xor2);
    assert_eq!(xor.truth_table_fidelity, 1.0);
    assert_eq!(xor.relay_count, 4);
    assert!(xor.transistor_savings_percent >= 65.0); // 4 relays vs 12 CMOS transistors

    // 3. Synthesize 1-bit Full Adder
    let fa = synth.synthesize(RelaySynthesisTarget::FullAdder);
    assert_eq!(fa.truth_table_fidelity, 1.0);
    assert_eq!(fa.relay_count, 10);
    assert!(fa.transistor_savings_percent >= 64.0); // 10 relays vs 28 CMOS transistors
    assert!(
        fa.standby_leakage_w < 1.0e-15,
        "Full adder standby leakage must be practically zero"
    );
}

#[test]
fn test_multibit_relay_adder_arithmetic_exactness() {
    let adder16 = MultiBitRelayAdder::new(16);
    let test_cases = [
        (0u64, 0u64, 0u64, false),
        (100, 200, 300, false),
        (32767, 1, 32768, false),
        (65535, 1, 0, true), // 16-bit overflow
    ];

    for &(a, b, expected_sum, expected_cout) in &test_cases {
        let (sum, cout) = adder16.add_u64(a, b);
        assert_eq!(sum, expected_sum, "16-bit sum mismatch for {} + {}", a, b);
        assert_eq!(
            cout, expected_cout,
            "16-bit cout mismatch for {} + {}",
            a, b
        );
    }

    // 32-bit adder test
    let adder32 = MultiBitRelayAdder::new(32);
    let (sum32, cout32) = adder32.add_u64(1_000_000_000, 2_000_000_000);
    assert_eq!(sum32, 3_000_000_000);
    assert!(!cout32);

    // Standby quiescent power of 32-bit adder must be < 1 pW (practically zero)
    let p_leak = adder32.total_static_leakage_w(0.085);
    assert!(
        p_leak < 1.0e-12,
        "32-bit adder leakage power was {:.2e} W, expected < 1 pW",
        p_leak
    );
}

#[test]
fn test_parallel_relay_vs_cmos_benchmark_across_cpu_cores() {
    let runner = RelayBenchmarkRunner::new(0.085, 0.70);
    let report = runner.run_benchmark(5000);

    // 1. Standby quiescent power leakage must be reduced by > 99.999%
    assert!(
        report.standby_power_reduction_percent > 99.999,
        "Standby power reduction was only {:.4}%, expected > 99.999%",
        report.standby_power_reduction_percent
    );
    assert!(
        report.relay_standby_leakage_uw < 1.0e-6,
        "Relay standby leakage {:.2e} uW exceeds 1 fW threshold",
        report.relay_standby_leakage_uw
    );

    // 2. Subthreshold swing steepness improvement factor > 10x
    assert!(
        report.swing_steepness_factor > 10.0,
        "Swing steepness factor was {:.1}x, expected > 10x",
        report.swing_steepness_factor
    );

    // 3. Dynamic switching energy reduction at sub-100mV actuation > 80%
    assert!(
        report.dynamic_energy_savings_percent > 80.0,
        "Dynamic energy savings was {:.1}%, expected > 80%",
        report.dynamic_energy_savings_percent
    );
    assert!(
        report.relay_dynamic_energy_per_op_fj < 5.0,
        "Relay dynamic energy per 32-bit op was {:.2} fJ",
        report.relay_dynamic_energy_per_op_fj
    );

    // 4. Verification across all parallel Rayon operations
    assert_eq!(report.operations_count, 5000);
    assert!(
        report.all_operations_exact,
        "All parallel arithmetic operations must be bit-exact"
    );
    assert!(report.parallel_duration_s > 0.0);
}
