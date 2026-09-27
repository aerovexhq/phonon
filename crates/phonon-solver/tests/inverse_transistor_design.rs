//! Integration Test: Inverse Transistor Device Design and Multi-Objective Optimization.
//!
//! Validates:
//! - Multi-objective NSGA-II Pareto optimization across GAA Nanosheet, CFET, and FinFET architectures.
//! - Non-dominated front extraction with diverse trade-offs ($I_{on}$, $I_{off}$, $\tau$, $EDP$, $\Delta T_{SH}$).
//! - Quantum confinement shifts $\Delta E_c \propto T_{ch}^{-2}$ in sub-5nm channels.
//! - Adjoint local sensitivity gradient refinement on Pareto candidates.

use phonon_models::optimization::{
    evaluate_transistor_fitness, AdjointRefiner, ArchitectureType, GeneBounds, OptimizationTarget,
    TransistorGenome,
};
use phonon_solver::optimization::{EngineConfig, InverseDesignEngine};

#[test]
fn test_multi_objective_pareto_front_diversity() {
    let mut config = EngineConfig::default();
    config.nsga2_config.population_size = 36;
    config.nsga2_config.max_generations = 6;
    config.enable_adjoint_refinement = true;
    config.adjoint_steps = 4;

    let engine = InverseDesignEngine::new(config);
    let front = engine.run_optimization(6, 12345);

    assert!(
        !front.is_empty(),
        "Pareto front must not be empty after evolutionary inverse design"
    );

    // Verify all Pareto solutions are physically viable
    for ind in &front {
        assert!(
            ind.fitness.is_physically_viable,
            "All Pareto solutions must satisfy physical constraints: genome={:?}",
            ind.genome
        );
        assert!(
            ind.fitness.i_on_a > 1.0e-6,
            "Drive current must be positive"
        );
        assert!(ind.fitness.i_off_a < 1.0e-3, "Off leakage must be bounded");
        assert!(
            ind.fitness.subthreshold_swing_mv_per_dec < 90.0,
            "Subthreshold swing must demonstrate good electrostatic control"
        );
        assert!(
            ind.fitness.self_heating_delta_t_k < 120.0,
            "Self-heating temperature rise must not exceed thermal breakdown limit"
        );
    }

    // Verify trade-off spectrum: maximum Ion design vs minimum delay design
    let best_ion = engine
        .find_best_by(&front, |fit| fit.i_on_a)
        .expect("Must have best Ion candidate");
    let best_delay = engine
        .find_best_by(&front, |fit| -fit.intrinsic_delay_ps)
        .expect("Must have fastest delay candidate");

    assert!(best_ion.fitness.i_on_a >= best_delay.fitness.i_on_a);
    assert!(best_delay.fitness.intrinsic_delay_ps <= best_ion.fitness.intrinsic_delay_ps);
}

#[test]
fn test_architecture_comparison_gaa_vs_finfet() {
    // GAA Nanosheet provides full 4-sided gate surround, whereas FinFET has 3-sided gate surround.
    // Therefore, at ultra-short gate lengths (e.g. 10nm), GAA must exhibit lower DIBL and steeper swing.
    let mut gaa_genome = TransistorGenome::n2_gaa_nanosheet_preset();
    gaa_genome.gate_length_nm = 10.0;
    gaa_genome.channel_thickness_nm = 4.0;
    gaa_genome.channel_width_nm = 20.0;

    let mut finfet_genome = gaa_genome.clone();
    finfet_genome.architecture = ArchitectureType::FinFet;

    let fit_gaa = evaluate_transistor_fitness(&gaa_genome);
    let fit_finfet = evaluate_transistor_fitness(&finfet_genome);

    // GAA nanosheet must have steeper or equal subthreshold swing compared to FinFET due to superior gate control
    assert!(
        fit_gaa.subthreshold_swing_mv_per_dec <= fit_finfet.subthreshold_swing_mv_per_dec,
        "GAA swing ({} mV/dec) should be superior to FinFET ({} mV/dec)",
        fit_gaa.subthreshold_swing_mv_per_dec,
        fit_finfet.subthreshold_swing_mv_per_dec
    );

    // GAA must exhibit lower normalized off-state leakage due to improved scale length lambda
    let ioff_norm_gaa = fit_gaa.i_off_a / (fit_gaa.effective_width_nm * 1e-9);
    let ioff_norm_finfet = fit_finfet.i_off_a / (fit_finfet.effective_width_nm * 1e-9);
    assert!(
        ioff_norm_gaa <= ioff_norm_finfet,
        "GAA normalized off-current ({} A/m) must be lower than FinFET ({} A/m)",
        ioff_norm_gaa,
        ioff_norm_finfet
    );
}

#[test]
fn test_quantum_confinement_and_subband_splitting() {
    // As channel thickness is scaled from 8nm down to 3nm, quantum confinement shift
    // Delta Ec = hbar^2 * pi^2 / (2 * m * Tch^2) must scale inversely quadratically
    let mut thick_channel = TransistorGenome::n2_gaa_nanosheet_preset();
    thick_channel.channel_thickness_nm = 8.0;

    let mut thin_channel = TransistorGenome::n2_gaa_nanosheet_preset();
    thin_channel.channel_thickness_nm = 3.0;

    let fit_thick = evaluate_transistor_fitness(&thick_channel);
    let fit_thin = evaluate_transistor_fitness(&thin_channel);

    // Theoretical ratio: (8 / 3)^2 = 64 / 9 = ~7.11
    let ratio = fit_thin.quantum_confinement_shift_ev / fit_thick.quantum_confinement_shift_ev;
    assert!(
        (ratio - 7.11).abs() < 0.25,
        "Quantum confinement shift ratio ({}) should closely track (8/3)^2 ~ 7.11",
        ratio
    );

    // Confinement increase raises the effective bandgap and shifts threshold voltage Vth
    assert!(fit_thin.quantum_confinement_shift_ev > fit_thick.quantum_confinement_shift_ev);
}

#[test]
fn test_adjoint_gradient_ascent_refinement() {
    let mut initial_genome = TransistorGenome::n2_gaa_nanosheet_preset();
    initial_genome.gate_length_nm = 14.0;
    initial_genome.channel_width_nm = 25.0;

    let refiner = AdjointRefiner::new(GeneBounds::default(), 6);
    let initial_fit = evaluate_transistor_fitness(&initial_genome);

    // Refine for maximum drive current
    let refined_genome =
        refiner.refine_candidate(&initial_genome, OptimizationTarget::MaximizeDriveCurrent);
    let refined_fit = evaluate_transistor_fitness(&refined_genome);

    assert!(
        refined_fit.i_on_a >= initial_fit.i_on_a,
        "Adjoint gradient ascent must maintain or increase drive current (before: {}, after: {})",
        initial_fit.i_on_a,
        refined_fit.i_on_a
    );
    assert!(
        refined_fit.is_physically_viable,
        "Refined candidate must remain physically viable"
    );
}
