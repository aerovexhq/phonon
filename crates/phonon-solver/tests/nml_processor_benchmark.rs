//! Integration Test: Autonomous NML Logic Synthesis & 3nm CMOS Comparative Benchmark.
//!
//! Validates:
//! 1. Autonomous geometric synthesis of NML Inverter, Majority-3, AND2, OR2, and Full Adder cells.
//! 2. 64-bit multi-bit arithmetic addition exactness.
//! 3. Parallel Rayon benchmark demonstrating 100% static leakage elimination and radiation hardness over 3nm GAA CMOS.

use phonon_models::spintronics::MultiBitNmlAdder;
use phonon_solver::spintronics::{NmlBenchmarkRunner, NmlLogicSynthesizer, TargetNmlFunction};

#[test]
fn test_autonomous_synthesis_all_nml_primitives() {
    let synth = NmlLogicSynthesizer::default();

    let targets = [
        TargetNmlFunction::Inverter,
        TargetNmlFunction::Majority3,
        TargetNmlFunction::And2,
        TargetNmlFunction::Or2,
        TargetNmlFunction::FullAdder1Bit,
    ];

    for target in targets {
        let result = synth.synthesize(target);
        assert_eq!(
            result.truth_table_fidelity, 1.0,
            "Synthesized gate {:?} must achieve 100% truth table fidelity",
            target
        );
        assert_eq!(
            result.static_power_w, 0.0,
            "Static standby leakage must be strictly zero"
        );
        assert!(
            result.thermal_stability_factor >= 40.0,
            "Thermal stability factor Delta must be >= 40 for 10-year retention"
        );
        assert!(
            result.footprint_nm2 < 50_000.0,
            "Footprint must be compact, got {:.1} nm^2",
            result.footprint_nm2
        );
    }
}

#[test]
fn test_multi_bit_nml_adder_exactness() {
    let adder64 = MultiBitNmlAdder::new(64);

    let test_cases = [
        (0u64, 0u64, false, 0u64, false),
        (123456789u64, 987654321u64, false, 1111111110u64, false),
        (0xFFFFFFFFFFFFFFFFu64, 1u64, false, 0u64, true), // 64-bit overflow carry
        (
            0x0123456789ABCDEFu64,
            0xFEDCBA9876543210u64,
            false,
            0xFFFFFFFFFFFFFFFFu64,
            false,
        ),
    ];

    for (a, b, cin, exp_sum, exp_cout) in test_cases {
        let (sum, cout) = adder64.add(a, b, cin);
        assert_eq!(sum, exp_sum, "Sum mismatch for {} + {} + {}", a, b, cin);
        assert_eq!(cout, exp_cout, "Cout mismatch for {} + {} + {}", a, b, cin);
    }
}

#[test]
fn test_rayon_parallel_nml_vs_3nm_cmos_benchmark() {
    let runner = NmlBenchmarkRunner::default();
    let report = runner.run_parallel_arithmetic_benchmark(200);

    assert_eq!(report.batch_operations_completed, 200);
    assert_eq!(
        report.static_power_elimination_pct, 100.0,
        "NML must achieve 100% static leakage elimination"
    );
    assert_eq!(
        report.nml_static_power_w, 0.0,
        "NML static power must be strictly zero"
    );
    assert!(
        report.component_count_reduction_pct > 40.0,
        "Component count reduction must exceed 40%, got {:.1}%",
        report.component_count_reduction_pct
    );
    assert!(
        report.non_volatile_retention,
        "NML states must be non-volatile"
    );
    assert!(
        report.radiation_hardened,
        "NML logic must be radiation-hardened"
    );
    assert!(
        report.throughput_ops_per_sec > 0.0,
        "Throughput must be positive"
    );
    assert!(
        report.total_static_energy_saved_joules > 0.0,
        "Static energy savings must be positive"
    );
}
