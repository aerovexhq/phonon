#![deny(unsafe_code)]

//! Wafer-Scale Yield, Spatially Correlated Process Variations & Harvesting Economics Engine.
//!
//! Integrates 300mm wafer geometry, gross die layout, WID/D2D Gaussian field variations,
//! Poisson/Murphy/Seeds/Negative Binomial yield models, core harvesting, and wafer gross margin economics.

pub mod defect_yield;
pub mod harvesting_binning;
pub mod process_variation;

pub use defect_yield::{
    generate_yield_curves, DefectRng, DefectYieldParams, YieldCurvePoint,
};
pub use harvesting_binning::{
    classify_die_harvest, compute_wafer_economics, DieArchitectureParams, DieHarvestStatus,
    HarvestSkuTier, WaferEconomicsParams, WaferEconomicsReport,
};
pub use process_variation::{
    calculate_analytical_gross_dpw, evaluate_die_process_parameters, generate_die_grid,
    DieGridPosition, DieProcessParameters, ProcessVariationFieldParams, WaferGeometryParams,
};

/// Combined state for an individual die on the wafer grid after simulation.
#[derive(Debug, Clone)]
pub struct SimulatedDie {
    pub pos: DieGridPosition,
    pub process: DieProcessParameters,
    pub harvest: DieHarvestStatus,
    pub local_d0_cm2: f64,
}

/// Comprehensive telemetry summary report from a wafer yield simulation run.
#[derive(Debug, Clone)]
pub struct WaferYieldTelemetryReport {
    pub gross_dpw: usize,
    pub analytical_gross_dpw: usize,
    pub total_good_dies: usize,
    pub functional_yield_pct: f64,
    pub unharvested_yield_pct: f64,
    pub yield_poisson_pct: f64,
    pub yield_murphy_pct: f64,
    pub yield_seeds_pct: f64,
    pub yield_neg_bin_pct: f64,
    pub gross_revenue_usd: f64,
    pub unharvested_revenue_usd: f64,
    pub harvesting_revenue_uplift_pct: f64,
    pub total_mfg_cost_usd: f64,
    pub gross_profit_usd: f64,
    pub gross_margin_pct: f64,
    pub cost_per_good_die_usd: f64,
    pub count_tier1_flagship: usize,
    pub count_tier2_harvested: usize,
    pub count_tier3_salvage: usize,
    pub count_scrap: usize,
    pub vth_mean_v: f64,
    pub vth_std_dev_mv: f64,
    pub lg_mean_nm: f64,
    pub lg_std_dev_nm: f64,
}

/// Co-simulator managing wafer geometry, process variations, defect yield, and SKU harvesting economics.
#[derive(Debug, Clone)]
pub struct WaferYieldCoSimulator {
    pub geometry: WaferGeometryParams,
    pub process: ProcessVariationFieldParams,
    pub defect: DefectYieldParams,
    pub arch: DieArchitectureParams,
    pub economics: WaferEconomicsParams,
    pub simulated_dies: Vec<SimulatedDie>,
    pub yield_curves: Vec<YieldCurvePoint>,
    pub latest_report: Option<WaferYieldTelemetryReport>,
}

impl Default for WaferYieldCoSimulator {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl WaferYieldCoSimulator {
    /// Sub-microsecond cold boot constructor initializing baseline parameters without executing simulation.
    pub fn new_fast() -> Self {
        Self {
            geometry: WaferGeometryParams::default(),
            process: ProcessVariationFieldParams::default(),
            defect: DefectYieldParams::default(),
            arch: DieArchitectureParams::default(),
            economics: WaferEconomicsParams::default(),
            simulated_dies: Vec::new(),
            yield_curves: Vec::new(),
            latest_report: None,
        }
    }

    /// Pre-seeds simulator with an instant full run for GUI initialization.
    pub fn new_with_baseline() -> Self {
        let mut sim = Self::new_fast();
        sim.run_simulation();
        sim
    }

    /// Executes complete wafer yield and harvesting simulation across the wafer grid.
    pub fn run_simulation(&mut self) -> WaferYieldTelemetryReport {
        let analytical_dpw = calculate_analytical_gross_dpw(&self.geometry);
        let die_grid = generate_die_grid(&self.geometry);

        // Filter only dies fully within the wafer active boundary for gross DPW
        let active_dies: Vec<DieGridPosition> = die_grid
            .into_iter()
            .filter(|d| d.is_fully_within_wafer)
            .collect();

        let gross_dpw = active_dies.len();
        let die_area_cm2 = self.geometry.die_area_cm2();
        let uncore_fraction = self.arch.uncore_area_fraction;
        let core_fraction = 1.0 - uncore_fraction;

        let mut rng = DefectRng::new(0xdeadbeef12345678);
        let mut simulated_dies = Vec::with_capacity(gross_dpw);
        let mut die_statuses = Vec::with_capacity(gross_dpw);

        let mut vth_sum = 0.0;
        let mut vth_sq_sum = 0.0;
        let mut lg_sum = 0.0;
        let mut lg_sq_sum = 0.0;

        for pos in active_dies {
            let proc = evaluate_die_process_parameters(&pos, &self.geometry, &self.process);

            vth_sum += proc.vth_v;
            vth_sq_sum += proc.vth_v * proc.vth_v;
            lg_sum += proc.lg_nm;
            lg_sq_sum += proc.lg_nm * proc.lg_nm;

            // Radial defect density at this die coordinate
            let local_d0 = self.defect.defect_density_at_radius(
                pos.distance_from_center_mm,
                self.geometry.active_radius_mm(),
            );

            // Poisson-distributed defect counts for uncore and core regions
            let uncore_lambda = local_d0 * (die_area_cm2 * uncore_fraction) * self.defect.critical_area_factor;
            let core_lambda = local_d0 * (die_area_cm2 * core_fraction) * self.defect.critical_area_factor;

            let uncore_defects = rng.sample_poisson(uncore_lambda);
            let core_defects = rng.sample_poisson(core_lambda);

            let harvest = classify_die_harvest(uncore_defects, core_defects, &self.arch);
            die_statuses.push(harvest.clone());

            simulated_dies.push(SimulatedDie {
                pos,
                process: proc,
                harvest,
                local_d0_cm2: local_d0,
            });
        }

        let econ_report = compute_wafer_economics(&die_statuses, &self.arch, &self.economics);
        self.yield_curves = generate_yield_curves(&self.defect, 3.5, 30);

        let n = gross_dpw.max(1) as f64;
        let vth_mean = vth_sum / n;
        let vth_variance = (vth_sq_sum / n - vth_mean * vth_mean).max(0.0);
        let vth_std_dev_mv = vth_variance.sqrt() * 1000.0;

        let lg_mean = lg_sum / n;
        let lg_variance = (lg_sq_sum / n - lg_mean * lg_mean).max(0.0);
        let lg_std_dev = lg_variance.sqrt();

        // Theoretical center-wafer model yield percentages
        let d0_center = self.defect.d0_center_cm2;
        let yield_poisson_pct = self.defect.poisson_yield(d0_center, die_area_cm2) * 100.0;
        let yield_murphy_pct = self.defect.murphy_yield(d0_center, die_area_cm2) * 100.0;
        let yield_seeds_pct = self.defect.seeds_yield(d0_center, die_area_cm2) * 100.0;
        let yield_neg_bin_pct = self.defect.negative_binomial_yield(d0_center, die_area_cm2) * 100.0;

        let report = WaferYieldTelemetryReport {
            gross_dpw,
            analytical_gross_dpw: analytical_dpw,
            total_good_dies: econ_report.total_good_dies,
            functional_yield_pct: econ_report.functional_yield_pct,
            unharvested_yield_pct: econ_report.unharvested_yield_pct,
            yield_poisson_pct,
            yield_murphy_pct,
            yield_seeds_pct,
            yield_neg_bin_pct,
            gross_revenue_usd: econ_report.gross_revenue_usd,
            unharvested_revenue_usd: econ_report.unharvested_revenue_usd,
            harvesting_revenue_uplift_pct: econ_report.harvesting_revenue_uplift_pct,
            total_mfg_cost_usd: econ_report.total_mfg_cost_usd,
            gross_profit_usd: econ_report.gross_profit_usd,
            gross_margin_pct: econ_report.gross_margin_pct,
            cost_per_good_die_usd: econ_report.cost_per_good_die_usd,
            count_tier1_flagship: econ_report.count_tier1_flagship,
            count_tier2_harvested: econ_report.count_tier2_harvested,
            count_tier3_salvage: econ_report.count_tier3_salvage,
            count_scrap: econ_report.count_scrap,
            vth_mean_v: vth_mean,
            vth_std_dev_mv,
            lg_mean_nm: lg_mean,
            lg_std_dev_nm: lg_std_dev,
        };

        self.simulated_dies = simulated_dies;
        self.latest_report = Some(report.clone());
        report
    }
}
