//! Integration Test: Unconstrained Gate Topology Synthesis.
//!
//! Validates:
//! - Automated synthesis of 2-input XOR2 and NAND2 logic cells from unconstrained topologies.
//! - Transistor count reduction: 4T PTL/TG XOR vs 12T Static CMOS XOR.
//! - 100% truth table correctness across all $2^N$ Boolean input combinations.
//! - Static noise margin validation ($NM_H, NM_L \ge 0.10\text{ V}$) and threshold drop diagnostics.

use phonon_models::synthesis::{CircuitTopology, TruthTable};
use phonon_solver::synthesis::{SynthesisConfig, TopologyEvolver, TransientGateVerifier};

#[test]
fn test_unconstrained_xor2_synthesis_and_transistor_reduction() {
    let tt_xor = TruthTable::xor2();

    let config = SynthesisConfig {
        population_size: 32,
        max_generations: 6,
        target_truth_table: tt_xor.clone(),
        ..Default::default()
    };

    let evolver = TopologyEvolver::new(config);
    let viable_candidates = evolver.run_synthesis(6, 4242);

    assert!(
        !viable_candidates.is_empty(),
        "Evolutionary synthesis must discover valid XOR2 topologies"
    );

    let best = &viable_candidates[0];

    // Verify 100% Boolean logic correctness
    assert_eq!(
        best.metrics.logic_correctness, 1.0,
        "Discovered XOR2 must achieve 100% truth table correctness"
    );

    // Standard static CMOS XOR requires 12 transistors.
    // Synthesized PTL/TG XOR must require at most 6 transistors (typically 4T).
    assert!(
        best.metrics.transistor_count <= 6,
        "Synthesized XOR transistor count ({}) must be significantly lower than 12T CMOS",
        best.metrics.transistor_count
    );

    // Verify noise margins
    assert!(
        best.metrics.noise_margin_high_v >= 0.10,
        "High noise margin ({} V) must be >= 0.10 V",
        best.metrics.noise_margin_high_v
    );
    assert!(
        best.metrics.noise_margin_low_v >= 0.10,
        "Low noise margin ({} V) must be >= 0.10 V",
        best.metrics.noise_margin_low_v
    );

    // Verify transient dynamics
    let verifier = TransientGateVerifier::default();
    let transient_res = verifier.verify_transient(&best.topology, &tt_xor);
    assert!(transient_res.is_hazard_free);
    assert!(transient_res.average_delay_ps < 40.0);
}

#[test]
fn test_unconstrained_nand2_synthesis_and_structural_sanity() {
    let tt_nand = TruthTable::nand2();

    let config = SynthesisConfig {
        population_size: 30,
        max_generations: 5,
        target_truth_table: tt_nand.clone(),
        ..Default::default()
    };

    let evolver = TopologyEvolver::new(config);
    let viable_candidates = evolver.run_synthesis(5, 7777);

    assert!(
        !viable_candidates.is_empty(),
        "Must discover physically viable NAND2 topologies"
    );

    let best = &viable_candidates[0];
    assert_eq!(best.metrics.logic_correctness, 1.0);
    assert!(best.metrics.transistor_count <= 4);
    assert!(best.topology.is_structurally_sane());
    assert!(!best.metrics.has_threshold_drop);
}

#[test]
fn test_ptl_xor_vs_cmos_nand_metrics() {
    let ptl_xor = CircuitTopology::ptl_xor2();
    let cmos_nand = CircuitTopology::static_cmos_nand2();

    assert_eq!(ptl_xor.total_transistors(), 4);
    assert_eq!(cmos_nand.total_transistors(), 4);

    assert!(ptl_xor.is_structurally_sane());
    assert!(cmos_nand.is_structurally_sane());

    let verifier = TransientGateVerifier::default();
    let res_xor = verifier.verify_transient(&ptl_xor, &TruthTable::xor2());
    let res_nand = verifier.verify_transient(&cmos_nand, &TruthTable::nand2());

    assert!(res_xor.is_hazard_free);
    assert!(res_nand.is_hazard_free);
    assert!(res_xor.total_switching_energy_fj > 0.0);
    assert!(res_nand.total_switching_energy_fj > 0.0);
}
