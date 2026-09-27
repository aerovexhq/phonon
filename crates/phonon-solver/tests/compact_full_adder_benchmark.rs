//! Integration Test: Compact Full Adder Benchmark against 28-T Static CMOS.
//!
//! Benchmarks:
//! - 28T Static CMOS Mirror Adder.
//! - 20T Transmission-Gate Adder (TGA).
//! - 14T Low-Power Hybrid PTL/TG Adder.
//! - 10T Pass-Transistor Logic (PTL) Adder.
//! - 4T + 4-NDR Resonant Tunneling MOBILE Adder.
//!
//! Validates:
//! - 100% truth table correctness across all 8 Boolean input vectors.
//! - >50% transistor count reduction in 14T, 10T, and RTD adders.
//! - Significant switching energy and active silicon area savings over standard 28T CMOS.
//! - Glitch immunity and transient timing verification.

use phonon_models::synthesis::{CircuitMetricsEvaluator, CircuitTopology, TruthTable};
use phonon_solver::synthesis::TransientGateVerifier;

#[test]
fn test_all_adder_architectures_satisfy_truth_table() {
    let tt_fa = TruthTable::full_adder_1bit();
    let evaluator = CircuitMetricsEvaluator::default();

    let adders = vec![
        CircuitTopology::static_cmos_full_adder_28t(),
        CircuitTopology::tga_full_adder_20t(),
        CircuitTopology::hybrid_full_adder_14t(),
        CircuitTopology::ptl_full_adder_10t(),
        CircuitTopology::rtd_mobile_full_adder(),
    ];

    for adder in &adders {
        let metrics = evaluator.evaluate(adder, &tt_fa);
        assert!(
            adder.is_structurally_sane(),
            "Adder {} must be structurally sane",
            adder.name
        );
        assert_eq!(
            metrics.logic_correctness, 1.0,
            "Adder {} must achieve 100% truth table correctness",
            adder.name
        );
        assert!(
            metrics.is_physically_viable,
            "Adder {} must be physically viable",
            adder.name
        );
    }
}

#[test]
fn test_component_count_and_area_reduction_against_28t_cmos() {
    let cmos_28t = CircuitTopology::static_cmos_full_adder_28t();
    let tga_20t = CircuitTopology::tga_full_adder_20t();
    let hybrid_14t = CircuitTopology::hybrid_full_adder_14t();
    let ptl_10t = CircuitTopology::ptl_full_adder_10t();
    let rtd_adder = CircuitTopology::rtd_mobile_full_adder();

    // 1. Verify exact transistor counts
    assert_eq!(cmos_28t.total_transistors(), 28);
    assert_eq!(tga_20t.total_transistors(), 20);
    assert_eq!(hybrid_14t.total_transistors(), 14);
    assert_eq!(ptl_10t.total_transistors(), 10);
    assert_eq!(rtd_adder.total_transistors(), 4);

    // 2. Transistor reduction percentages relative to 28T CMOS
    let red_14t = (28 - 14) as f64 / 28.0;
    assert_eq!(
        red_14t, 0.50,
        "14T adder must achieve 50% transistor reduction"
    );

    let red_10t = (28 - 10) as f64 / 28.0;
    assert!(
        red_10t > 0.60,
        "10T adder must achieve >60% transistor reduction"
    );

    let red_rtd = (28 - 4) as f64 / 28.0;
    assert!(
        red_rtd > 0.85,
        "RTD adder must achieve >85% transistor reduction"
    );

    // 3. Active silicon area reduction
    let area_28t = cmos_28t.total_active_area_um2();
    let area_14t = hybrid_14t.total_active_area_um2();
    let area_10t = ptl_10t.total_active_area_um2();
    let area_rtd = rtd_adder.total_active_area_um2();

    assert!(
        area_14t < area_28t * 0.75,
        "14T area must be < 75% of 28T area"
    );
    assert!(
        area_10t < area_28t * 0.50,
        "10T area must be < 50% of 28T area"
    );
    assert!(
        area_rtd < area_28t * 0.35,
        "RTD area must be < 35% of 28T area"
    );
}

#[test]
fn test_transient_delay_energy_and_glitch_immunity() {
    let tt = TruthTable::full_adder_1bit();
    let verifier = TransientGateVerifier::default();

    let cmos_28t = CircuitTopology::static_cmos_full_adder_28t();
    let hybrid_14t = CircuitTopology::hybrid_full_adder_14t();
    let ptl_10t = CircuitTopology::ptl_full_adder_10t();
    let rtd_adder = CircuitTopology::rtd_mobile_full_adder();

    let res_28t = verifier.verify_transient(&cmos_28t, &tt);
    let res_14t = verifier.verify_transient(&hybrid_14t, &tt);
    let res_10t = verifier.verify_transient(&ptl_10t, &tt);
    let res_rtd = verifier.verify_transient(&rtd_adder, &tt);

    // All adders must be hazard-free under standard input transitions
    assert!(res_28t.is_hazard_free);
    assert!(res_14t.is_hazard_free);
    assert!(res_10t.is_hazard_free);
    assert!(res_rtd.is_hazard_free);

    // Compact adders must exhibit lower dynamic switching energy due to fewer internal parasitic nodes
    assert!(
        res_14t.total_switching_energy_fj < res_28t.total_switching_energy_fj,
        "14T energy ({} fJ) must be lower than 28T ({} fJ)",
        res_14t.total_switching_energy_fj,
        res_28t.total_switching_energy_fj
    );
    assert!(
        res_10t.total_switching_energy_fj < res_28t.total_switching_energy_fj,
        "10T energy ({} fJ) must be lower than 28T ({} fJ)",
        res_10t.total_switching_energy_fj,
        res_28t.total_switching_energy_fj
    );
    assert!(
        res_rtd.total_switching_energy_fj < res_28t.total_switching_energy_fj,
        "RTD energy ({} fJ) must be lower than 28T ({} fJ)",
        res_rtd.total_switching_energy_fj,
        res_28t.total_switching_energy_fj
    );

    // Energy-Delay Product (EDP) improvement
    assert!(res_14t.energy_delay_product_js < res_28t.energy_delay_product_js);
    assert!(res_rtd.energy_delay_product_js < res_28t.energy_delay_product_js);
}
