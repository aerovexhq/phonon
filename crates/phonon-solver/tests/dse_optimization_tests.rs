#![deny(unsafe_code)]

//! Unit and integration test suite for Automated Multi-Objective PPA-C Design Space Exploration (DSE).

use phonon_solver::dse_optimization::{
    assign_crowding_distance, evaluate_packaging_ppac, generate_gp_slice,
    non_dominated_sort, polynomial_mutation, sbx_crossover, DesignGenome, DseCoSimulator,
    GaussianProcessParams, GaussianProcessRegressor, GeneticRng, Individual, ObjectiveValues,
    PackagingTechnology,
};

#[test]
fn test_packaging_architecture_ppac_trades() {
    let wafer_cost = 16500.0;
    let mono = evaluate_packaging_ppac(PackagingTechnology::MonolithicSoc, 16, 32.0, 3.2, 0.85, wafer_cost);
    let mcm = evaluate_packaging_ppac(PackagingTechnology::OrganicMcm, 16, 32.0, 3.2, 0.85, wafer_cost);
    let cowos = evaluate_packaging_ppac(PackagingTechnology::SiliconInterposerCoWoS, 16, 32.0, 3.2, 0.85, wafer_cost);
    let soic = evaluate_packaging_ppac(PackagingTechnology::HybridBonding3D, 16, 32.0, 3.2, 0.85, wafer_cost);

    // Monolithic has zero D2D latency degradation (full baseline IPC)
    assert!(mono.effective_throughput_ipc > mcm.effective_throughput_ipc);
    assert!(mono.effective_throughput_ipc >= cowos.effective_throughput_ipc);

    // 3D Hybrid Bonding has lowest D2D energy per bit and highest effective IPC among multi-die
    assert!(soic.effective_throughput_ipc > mcm.effective_throughput_ipc);
    assert_eq!(PackagingTechnology::HybridBonding3D.d2d_energy_pj_per_bit(), 0.08);

    // Thermal resistance penalty is highest in 3D due to vertical heat stacking
    assert!(soic.thermal_headroom_c < mono.thermal_headroom_c);

    // All power and cost values must be positive and physically realistic
    for res in [&mono, &mcm, &cowos, &soic] {
        assert!(res.total_power_w > 10.0 && res.total_power_w < 500.0);
        assert!(res.unit_manufacturing_cost_usd > 20.0 && res.unit_manufacturing_cost_usd < 1000.0);
        assert!(res.total_silicon_area_mm2 > 20.0);
    }
}

#[test]
fn test_gaussian_process_and_expected_improvement() {
    let mut gp = GaussianProcessRegressor::new(GaussianProcessParams {
        signal_variance: 1.0,
        lengthscale: 0.5,
        noise_variance: 1e-4,
    });

    let x_train = vec![vec![1.0], vec![2.0], vec![3.0]];
    let y_train = vec![1.5, 3.2, 2.1];

    let fitted = gp.fit(x_train, y_train);
    assert!(fitted);

    // Prediction at training point x=2.0 should be very close to 3.2 with low variance
    let (mu_train, var_train) = gp.predict(&[2.0]);
    assert!((mu_train - 3.2).abs() < 0.1, "Predictive mean at training point should be close to observed value");
    assert!(var_train < 0.01, "Variance at training point should be small");

    // Prediction far away (e.g. x=6.0) should have high variance (approaching signal variance)
    let (_, var_far) = gp.predict(&[6.0]);
    assert!(var_far > var_train);

    // Expected improvement should be evaluated without panicking
    let ei = gp.expected_improvement(&[2.5], 3.2, 0.01);
    assert!(ei >= 0.0);

    // 1D GP slice generation
    let slice = generate_gp_slice(&gp, 0.5, 4.5, 20, 3.2);
    assert_eq!(slice.len(), 20);
    for pt in &slice {
        assert!(pt.gp_uncertainty_upper >= pt.gp_mean);
        assert!(pt.gp_uncertainty_lower <= pt.gp_mean);
    }
}

#[test]
fn test_nsga2_pareto_dominance_and_sorting() {
    let ind_a = ObjectiveValues {
        power_w: 45.0,
        latency_ns: 0.30,
        silicon_area_mm2: 120.0,
        unit_cost_usd: 85.0,
    };

    let ind_b = ObjectiveValues {
        power_w: 60.0,
        latency_ns: 0.35,
        silicon_area_mm2: 140.0,
        unit_cost_usd: 110.0,
    };

    // A dominates B on all objectives
    assert!(ind_a.dominates(&ind_b));
    assert!(!ind_b.dominates(&ind_a));

    let ind_c = ObjectiveValues {
        power_w: 40.0,
        latency_ns: 0.40, // Tradeoff: lower power but higher latency
        silicon_area_mm2: 110.0,
        unit_cost_usd: 80.0,
    };

    // Mutual non-dominance between A and C
    assert!(!ind_a.dominates(&ind_c));
    assert!(!ind_c.dominates(&ind_a));

    let mut population = vec![
        Individual {
            genome: DesignGenome::default(),
            objectives: ind_a,
            rank: 0,
            crowding_distance: 0.0,
        },
        Individual {
            genome: DesignGenome::default(),
            objectives: ind_b,
            rank: 0,
            crowding_distance: 0.0,
        },
        Individual {
            genome: DesignGenome::default(),
            objectives: ind_c,
            rank: 0,
            crowding_distance: 0.0,
        },
    ];

    let fronts = non_dominated_sort(&mut population);
    assert_eq!(fronts.len(), 2);
    // Front 1 should contain A (0) and C (2)
    assert_eq!(fronts[0].len(), 2);
    // Front 2 should contain dominated B (1)
    assert_eq!(fronts[1].len(), 1);

    // Crowding distance assignment
    assign_crowding_distance(&fronts[0], &mut population);
    assert!(population[fronts[0][0]].crowding_distance.is_infinite());
    assert!(population[fronts[0][1]].crowding_distance.is_infinite());
}

#[test]
fn test_genetic_crossover_and_mutation() {
    let mut rng = GeneticRng::new(0x12345678);
    let p1 = DesignGenome {
        vdd_v: 0.75,
        clock_freq_ghz: 2.4,
        gate_length_nm: 14.0,
        nanosheet_width_nm: 25.0,
        num_nanosheets: 3,
        num_cores: 8,
        cache_l3_mb: 24.0,
        packaging: PackagingTechnology::MonolithicSoc,
    };
    let p2 = DesignGenome {
        vdd_v: 0.95,
        clock_freq_ghz: 3.8,
        gate_length_nm: 18.0,
        nanosheet_width_nm: 40.0,
        num_nanosheets: 4,
        num_cores: 32,
        cache_l3_mb: 64.0,
        packaging: PackagingTechnology::SiliconInterposerCoWoS,
    };

    let (mut c1, c2) = sbx_crossover(&p1, &p2, &mut rng);

    // Crossover offspring must remain in bounds
    assert!(c1.vdd_v >= 0.65 && c1.vdd_v <= 1.10);
    assert!(c2.vdd_v >= 0.65 && c2.vdd_v <= 1.10);
    assert!(c1.clock_freq_ghz >= 1.2 && c1.clock_freq_ghz <= 4.5);
    assert!(c2.clock_freq_ghz >= 1.2 && c2.clock_freq_ghz <= 4.5);

    polynomial_mutation(&mut c1, &mut rng);
    assert!(c1.gate_length_nm >= 12.0 && c1.gate_length_nm <= 22.0);
    assert!(c1.nanosheet_width_nm >= 15.0 && c1.nanosheet_width_nm <= 50.0);
}

#[test]
fn test_dse_co_simulator_execution() {
    let mut sim = DseCoSimulator::new_fast();
    assert!(sim.population.is_empty());
    assert!(sim.pareto_frontier.is_empty());

    let report = sim.run_optimization();

    assert!(!sim.population.is_empty());
    assert!(!sim.pareto_frontier.is_empty());
    assert_eq!(report.pareto_frontier_count, sim.pareto_frontier.len());
    assert!(report.min_power_w > 0.0);
    assert!(report.max_freq_ghz > 1.5);
    assert!(report.best_energy_efficiency_gflops_per_w > 0.0);
    assert!(!sim.gp_slice.is_empty());
    assert_eq!(sim.packaging_comparisons.len(), 4);
}
