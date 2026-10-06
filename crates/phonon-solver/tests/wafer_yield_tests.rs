#![deny(unsafe_code)]

//! Unit and integration test suite for Wafer-Scale Yield, Spatially Correlated Process Variations & Harvesting Economics.

use phonon_solver::wafer_yield::{
    calculate_analytical_gross_dpw, classify_die_harvest, compute_wafer_economics,
    evaluate_die_process_parameters, generate_die_grid, generate_yield_curves, DefectYieldParams,
    DieArchitectureParams, HarvestSkuTier, ProcessVariationFieldParams,
    WaferEconomicsParams, WaferGeometryParams, WaferYieldCoSimulator,
};

#[test]
fn test_wafer_geometry_and_analytical_gross_dpw() {
    let wafer = WaferGeometryParams::default();
    assert_eq!(wafer.diameter_mm, 300.0);
    assert_eq!(wafer.wafer_radius_mm(), 150.0);
    assert_eq!(wafer.active_radius_mm(), 147.0);

    let gross_dpw = calculate_analytical_gross_dpw(&wafer);
    // For 300mm wafer with 12.5mm x 10mm die (1.25 cm^2), DPW is typically ~500
    assert!(gross_dpw >= 450 && gross_dpw <= 560, "Gross DPW should be ~500, got {}", gross_dpw);
}

#[test]
fn test_die_grid_generation_and_containment() {
    let wafer = WaferGeometryParams::default();
    let dies = generate_die_grid(&wafer);

    let fully_within: Vec<_> = dies.iter().filter(|d| d.is_fully_within_wafer).collect();
    assert!(!fully_within.is_empty(), "Should generate dies fully within active wafer");
    assert!(fully_within.len() >= 400 && fully_within.len() <= 560);

    for die in &fully_within {
        assert!(die.distance_from_center_mm <= wafer.active_radius_mm());
    }
}

#[test]
fn test_spatially_correlated_process_variations() {
    let wafer = WaferGeometryParams::default();
    let field = ProcessVariationFieldParams::default();
    let dies = generate_die_grid(&wafer);

    let center_die = dies.iter().min_by(|a, b| {
        a.distance_from_center_mm.partial_cmp(&b.distance_from_center_mm).unwrap()
    }).unwrap();

    let edge_die = dies.iter().filter(|d| d.is_fully_within_wafer).max_by(|a, b| {
        a.distance_from_center_mm.partial_cmp(&b.distance_from_center_mm).unwrap()
    }).unwrap();

    let center_proc = evaluate_die_process_parameters(center_die, &wafer, &field);
    let edge_proc = evaluate_die_process_parameters(edge_die, &wafer, &field);

    // Radial bowl causes higher Vth near the wafer edge
    assert!(edge_proc.vth_v > center_proc.vth_v - 0.05);
    // Gate length is positive and realistic
    assert!(center_proc.lg_nm > 12.0 && center_proc.lg_nm < 20.0);
    assert!(edge_proc.lg_nm > 12.0 && edge_proc.lg_nm < 20.0);
}

#[test]
fn test_defect_yield_models() {
    let params = DefectYieldParams::default();
    let d0 = params.d0_center_cm2;

    // At zero die area, yield should be 1.0 (100%)
    assert!((params.poisson_yield(d0, 0.0) - 1.0).abs() < 1e-6);
    assert!((params.murphy_yield(d0, 0.0) - 1.0).abs() < 1e-4);
    assert!((params.seeds_yield(d0, 0.0) - 1.0).abs() < 1e-6);
    assert!((params.negative_binomial_yield(d0, 0.0) - 1.0).abs() < 1e-6);

    let area = 1.5; // cm^2
    let y_pois = params.poisson_yield(d0, area);
    let y_murp = params.murphy_yield(d0, area);
    let y_seeds = params.seeds_yield(d0, area);
    let y_neg_bin = params.negative_binomial_yield(d0, area);

    // All yields should be in (0, 1)
    assert!(y_pois > 0.0 && y_pois < 1.0);
    assert!(y_murp > 0.0 && y_murp < 1.0);
    assert!(y_seeds > 0.0 && y_seeds < 1.0);
    assert!(y_neg_bin > 0.0 && y_neg_bin < 1.0);

    // Due to clustering (alpha = 2.0), Negative Binomial yield is strictly higher than Poisson
    assert!(y_neg_bin > y_pois, "Clustered Negative Binomial yield ({}) should exceed Poisson ({})", y_neg_bin, y_pois);

    // Radial defect density increases toward edge
    let r_active = 147.0;
    let d0_center = params.defect_density_at_radius(0.0, r_active);
    let d0_edge = params.defect_density_at_radius(r_active, r_active);
    assert!((d0_center - params.d0_center_cm2).abs() < 1e-6);
    assert!(d0_edge > d0_center * (1.0 + params.kappa_edge * 0.95));

    // Yield curves generation
    let curves = generate_yield_curves(&params, 3.0, 20);
    assert_eq!(curves.len(), 20);
    assert!(curves.first().unwrap().poisson_yield > curves.last().unwrap().poisson_yield);
}

#[test]
fn test_chiplet_harvesting_and_sku_binning() {
    let arch = DieArchitectureParams::default();

    // 0 defects -> Flagship
    let s0 = classify_die_harvest(0, 0, &arch);
    assert_eq!(s0.sku_tier, HarvestSkuTier::Flagship16Core);
    assert_eq!(s0.realized_value_usd, 850.0);

    // 1-2 core defects -> 12-Core Harvested
    let s1 = classify_die_harvest(0, 1, &arch);
    assert_eq!(s1.sku_tier, HarvestSkuTier::Harvested12Core);
    assert_eq!(s1.realized_value_usd, 550.0);

    let s2 = classify_die_harvest(0, 2, &arch);
    assert_eq!(s2.sku_tier, HarvestSkuTier::Harvested12Core);

    // 3-4 core defects -> 8-Core Salvage
    let s3 = classify_die_harvest(0, 3, &arch);
    assert_eq!(s3.sku_tier, HarvestSkuTier::Salvage8Core);
    assert_eq!(s3.realized_value_usd, 320.0);

    // >4 core defects -> Scrap
    let s5 = classify_die_harvest(0, 5, &arch);
    assert_eq!(s5.sku_tier, HarvestSkuTier::Scrap);
    assert_eq!(s5.realized_value_usd, 0.0);

    // Uncore defect -> Fatal scrap regardless of cores
    let s_uncore = classify_die_harvest(1, 0, &arch);
    assert_eq!(s_uncore.sku_tier, HarvestSkuTier::Scrap);
    assert_eq!(s_uncore.realized_value_usd, 0.0);
}

#[test]
fn test_wafer_economics_aggregation() {
    let arch = DieArchitectureParams::default();
    let econ = WaferEconomicsParams::default();

    let statuses = vec![
        classify_die_harvest(0, 0, &arch), // Tier 1: $850
        classify_die_harvest(0, 1, &arch), // Tier 2: $550
        classify_die_harvest(0, 3, &arch), // Tier 3: $320
        classify_die_harvest(1, 0, &arch), // Scrap: $0
    ];

    let report = compute_wafer_economics(&statuses, &arch, &econ);
    assert_eq!(report.gross_dpw, 4);
    assert_eq!(report.count_tier1_flagship, 1);
    assert_eq!(report.count_tier2_harvested, 1);
    assert_eq!(report.count_tier3_salvage, 1);
    assert_eq!(report.count_scrap, 1);
    assert_eq!(report.total_good_dies, 3);
    assert_eq!(report.functional_yield_pct, 75.0);
    assert_eq!(report.unharvested_yield_pct, 25.0);

    assert_eq!(report.gross_revenue_usd, 850.0 + 550.0 + 320.0);
    assert_eq!(report.unharvested_revenue_usd, 850.0);
    assert!(report.harvesting_revenue_uplift_pct > 100.0);
}

#[test]
fn test_wafer_yield_co_simulator_run() {
    let mut sim = WaferYieldCoSimulator::new_fast();
    assert!(sim.simulated_dies.is_empty());
    assert!(sim.latest_report.is_none());

    let report = sim.run_simulation();
    assert!(!sim.simulated_dies.is_empty());
    assert!(report.gross_dpw > 400);
    assert!(report.total_good_dies > 0);
    assert!(report.functional_yield_pct > report.unharvested_yield_pct);
    assert!(report.gross_revenue_usd > report.unharvested_revenue_usd);
    assert!(report.gross_margin_pct > 0.0);
    assert!(report.vth_mean_v > 0.25 && report.vth_mean_v < 0.45);
    assert!(report.vth_std_dev_mv > 5.0);
}
