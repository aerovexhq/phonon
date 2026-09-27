//! Integration Test: IEEE IRDS Roadmap Validation and Automated Target Compliance.
//!
//! Validates:
//! - Compliance assessment of evolved transistor candidates against IRDS 3nm, 2nm, and A14 specifications.
//! - Drive current per unit width $I_{on} / W_{eff}$, subthreshold swing $S$, intrinsic delay $\tau$, and self-heating $\Delta T$.
//! - Parallel roadmap filtering across multi-threaded inverse design populations.

use phonon_models::optimization::{evaluate_transistor_fitness, IrdsNodeTarget, TransistorGenome};
use phonon_solver::optimization::{EngineConfig, InverseDesignEngine};

#[test]
fn test_irds_roadmap_presets_compliance() {
    let target_3nm = IrdsNodeTarget::irds_3nm_node();
    let target_2nm = IrdsNodeTarget::irds_2nm_node();
    let target_a14 = IrdsNodeTarget::irds_a14_node();

    // 1. Validate 2nm GAA Nanosheet preset against 3nm and 2nm targets
    let n2_genome = TransistorGenome::n2_gaa_nanosheet_preset();
    let fit_n2 = evaluate_transistor_fitness(&n2_genome);

    let report_3nm = target_3nm.evaluate_compliance(&fit_n2);
    assert!(
        report_3nm.swing_satisfied,
        "2nm preset must meet 3nm swing target"
    );
    assert!(
        report_3nm.delay_satisfied,
        "2nm preset must meet 3nm delay target"
    );
    assert!(
        report_3nm.thermal_satisfied,
        "2nm preset must meet 3nm thermal target"
    );
    assert!(report_3nm.normalized_performance_score > 0.95);

    let report_2nm = target_2nm.evaluate_compliance(&fit_n2);
    assert!(
        report_2nm.swing_satisfied,
        "2nm preset must meet 2nm swing target"
    );
    assert!(
        report_2nm.delay_satisfied,
        "2nm preset must meet 2nm delay target"
    );
    assert!(
        report_2nm.thermal_satisfied,
        "2nm preset must meet 2nm thermal target"
    );

    // 2. Validate CFET preset against A14 target
    let cfet_genome = TransistorGenome::cfet_preset();
    let fit_cfet = evaluate_transistor_fitness(&cfet_genome);

    let report_a14 = target_a14.evaluate_compliance(&fit_cfet);
    assert!(
        report_a14.swing_satisfied,
        "CFET preset must meet A14 swing target"
    );
    assert!(
        report_a14.delay_satisfied,
        "CFET preset must meet A14 delay target"
    );
    assert!(
        report_a14.thermal_satisfied,
        "CFET preset must meet A14 thermal target"
    );
}

#[test]
fn test_automated_inverse_design_satisfies_roadmap_targets() {
    let mut config = EngineConfig::default();
    config.nsga2_config.population_size = 32;
    config.nsga2_config.max_generations = 5;
    config.enable_adjoint_refinement = true;
    config.adjoint_steps = 4;

    let engine = InverseDesignEngine::new(config);
    let pareto_front = engine.run_optimization(5, 54321);

    assert!(!pareto_front.is_empty(), "Must discover Pareto front");

    let target_3nm = IrdsNodeTarget::irds_3nm_node();
    let compliant_results = engine.evaluate_roadmap_compliance(&pareto_front, &target_3nm);

    // Filter solutions that satisfy roadmap subthreshold swing and thermal limits
    let viable_designs: Vec<_> = compliant_results
        .iter()
        .filter(|(_, rep)| rep.swing_satisfied && rep.thermal_satisfied)
        .collect();

    assert!(
        !viable_designs.is_empty(),
        "Evolutionary engine must yield designs satisfying 3nm swing and thermal criteria"
    );

    // Check that best candidate achieves strong performance score
    let max_score = compliant_results
        .iter()
        .map(|(_, rep)| rep.normalized_performance_score)
        .fold(0.0, f64::max);

    assert!(
        max_score > 0.80,
        "Best evolved candidate must score highly against IRDS metrics: max_score={}",
        max_score
    );
}
