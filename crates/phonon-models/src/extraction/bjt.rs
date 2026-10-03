#![deny(unsafe_code)]

//! Gummel-Poon BJT parameter estimation engine and curve-fitting optimizer.

use super::curve_data::{MeasuredCurve, MeasurementPoint};
use super::polisher::{polish_parameters, GenericFittingResult, PolishableModel};

/// Target Gummel-Poon BJT parameters optimized during device parameter extraction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BjtTargetParams {
    /// Transport saturation current $I_S$ in Amperes ($A$), valid range [1e-18, 1e-12].
    pub is: f64,
    /// Ideal maximum forward current gain $BF$ ($\beta_F$), valid range [10.0, 500.0].
    pub bf: f64,
    /// Forward Early voltage $V_{AF}$ in Volts ($V$), valid range [10.0, 500.0].
    pub vaf: f64,
}

impl Default for BjtTargetParams {
    fn default() -> Self {
        Self {
            is: 1.0e-15,
            bf: 100.0,
            vaf: 100.0,
        }
    }
}

impl BjtTargetParams {
    /// Clamps all parameter values strictly within physical boundaries.
    pub fn clamp(&mut self) {
        self.is = self.is.clamp(1e-18, 1e-12);
        self.bf = self.bf.clamp(10.0, 500.0);
        self.vaf = self.vaf.clamp(10.0, 500.0);
    }

    /// Evaluates forward active collector current $I_c$ in Amperes ($A$).
    pub fn evaluate_ic(&self, v_ce: f64, v_be: f64, temp_k: f64) -> f64 {
        let vt = phonon_core::thermal_voltage(temp_k).max(1e-4);
        let v_ce = v_ce.max(0.0);
        let early_factor = 1.0 + v_ce / self.vaf.max(1.0);
        let x = v_be / vt;
        let exp_term = if x > 40.0 {
            40.0f64.exp() + (x - 40.0) * 40.0f64.exp()
        } else if x < -40.0 {
            -1.0
        } else {
            x.exp() - 1.0
        };
        (self.is * exp_term.max(0.0) * early_factor).max(0.0)
    }

    /// Evaluates forward active base current $I_b$ in Amperes ($A$).
    pub fn evaluate_ib(&self, v_be: f64, temp_k: f64) -> f64 {
        let vt = phonon_core::thermal_voltage(temp_k).max(1e-4);
        let x = v_be / vt;
        let exp_term = if x > 40.0 {
            40.0f64.exp() + (x - 40.0) * 40.0f64.exp()
        } else if x < -40.0 {
            -1.0
        } else {
            x.exp() - 1.0
        };
        (self.is / self.bf.max(1.0) * exp_term.max(0.0)).max(0.0)
    }
}

/// Bounding constraints for Gummel-Poon BJT parameter optimization.
#[derive(Debug, Clone, PartialEq)]
pub struct BjtBounds {
    pub is_range: (f64, f64),
    pub bf_range: (f64, f64),
    pub vaf_range: (f64, f64),
}

impl Default for BjtBounds {
    fn default() -> Self {
        Self {
            is_range: (1e-18, 1e-12),
            bf_range: (10.0, 500.0),
            vaf_range: (10.0, 500.0),
        }
    }
}

impl PolishableModel for BjtTargetParams {
    fn num_params() -> usize {
        3
    }

    fn to_normalized(&self) -> Vec<f64> {
        let is_clamped = self.is.clamp(1e-18, 1e-12);
        let log_is = is_clamped.log10();
        let norm_is = ((log_is - (-18.0)) / ((-12.0) - (-18.0))).clamp(0.0, 1.0);
        let norm_bf = ((self.bf - 10.0) / (500.0 - 10.0)).clamp(0.0, 1.0);
        let norm_vaf = ((self.vaf - 10.0) / (500.0 - 10.0)).clamp(0.0, 1.0);
        vec![norm_is, norm_bf, norm_vaf]
    }

    fn from_normalized(norm: &[f64]) -> Self {
        let g0 = norm[0].clamp(0.0, 1.0);
        let log_is = -18.0 + g0 * 6.0;
        let is_val = 10.0f64.powf(log_is);
        let bf_val = 10.0 + norm[1].clamp(0.0, 1.0) * 490.0;
        let vaf_val = 10.0 + norm[2].clamp(0.0, 1.0) * 490.0;
        Self {
            is: is_val,
            bf: bf_val,
            vaf: vaf_val,
        }
    }

    fn evaluate_current(&self, pt: &MeasurementPoint, curve: &MeasuredCurve) -> f64 {
        self.evaluate_ic(pt.v_ds, pt.v_gs, curve.temperature_k)
    }
}

/// Type alias for BJT fitting results.
pub type BjtFittingResult = GenericFittingResult<BjtTargetParams>;

/// Multi-island genetic algorithm and Levenberg-Marquardt hybrid optimizer for BJT parameters.
#[derive(Debug, Clone, Copy, Default)]
pub struct BjtOptimizer;

#[derive(Clone, Copy)]
struct BjtIndividual {
    genome: [f64; 3],
    loss: f64,
    rmse: f64,
    r_squared: f64,
}

struct BjtEvalContext<'a> {
    curve: &'a MeasuredCurve,
    bounds: &'a BjtBounds,
    ss_tot: f64,
    n_points: f64,
}

impl BjtOptimizer {
    /// Fits Gummel-Poon BJT model parameters to the given curve with hybrid GA and LM polishing.
    pub fn fit_curve(
        curve: &MeasuredCurve,
        max_generations: usize,
        population_size: usize,
    ) -> BjtFittingResult {
        let default_bounds = BjtBounds::default();
        Self::fit_curve_with_bounds(curve, max_generations, population_size, &default_bounds)
    }

    /// Fits BJT model parameters within custom parameter bounds using GA and LM polishing.
    pub fn fit_curve_with_bounds(
        curve: &MeasuredCurve,
        max_generations: usize,
        population_size: usize,
        bounds: &BjtBounds,
    ) -> BjtFittingResult {
        let ga_result = Self::fit_curve_ga_only(curve, max_generations, population_size, bounds);
        let polished = Self::polish_parameters(&ga_result.params, curve);
        if polished.rmse <= ga_result.rmse || polished.r_squared >= ga_result.r_squared {
            polished
        } else {
            ga_result
        }
    }

    /// Executes only the genetic algorithm phase without Levenberg-Marquardt polishing.
    pub fn fit_curve_ga_only(
        curve: &MeasuredCurve,
        max_generations: usize,
        population_size: usize,
        bounds: &BjtBounds,
    ) -> BjtFittingResult {
        let n_points = curve.points.len();
        if n_points == 0 {
            return BjtFittingResult {
                params: BjtTargetParams::default(),
                rmse: 0.0,
                r_squared: 0.0,
                iterations: 0,
                converged: false,
            };
        }

        let n_f64 = n_points as f64;
        let mut sum_meas = 0.0;
        for pt in &curve.points {
            sum_meas += pt.i_ds;
        }
        let mean_meas = sum_meas / n_f64;

        let mut ss_tot = 0.0;
        for pt in &curve.points {
            let dev = pt.i_ds - mean_meas;
            ss_tot += dev * dev;
        }

        let ctx = BjtEvalContext {
            curve,
            bounds,
            ss_tot,
            n_points: n_f64,
        };

        let pop_size = population_size.max(12);
        let generations = max_generations.max(5);

        let mut rng = BjtFastRng::new(0x424a_5447_5033_3236);

        let num_islands = 3;
        let island_size = (pop_size / num_islands).max(4);
        let mut islands: Vec<Vec<BjtIndividual>> = Vec::with_capacity(num_islands);

        let heuristic_genome = extract_bjt_heuristic_genome(curve, bounds);
        let default_genome = bjt_params_to_genome(&BjtTargetParams::default(), bounds);

        for island_idx in 0..num_islands {
            let mut island = Vec::with_capacity(island_size);
            if island_idx == 0 {
                island.push(eval_bjt_individual(heuristic_genome, &ctx));
                island.push(eval_bjt_individual(default_genome, &ctx));
            }
            while island.len() < island_size {
                let g = [rng.next_f64(), rng.next_f64(), rng.next_f64()];
                island.push(eval_bjt_individual(g, &ctx));
            }
            islands.push(island);
        }

        let mut completed_generations = 0;

        for gen in 0..generations {
            completed_generations = gen + 1;
            let mutation_rate = 0.25;
            let mutation_sigma = 0.08 * (1.0 - (gen as f64) / (generations as f64)).max(0.15);

            for island in &mut islands {
                island.sort_unstable_by(|a, b| a.loss.total_cmp(&b.loss));
                let mut new_island = Vec::with_capacity(island_size);

                new_island.push(island[0]);
                if island.len() > 1 {
                    new_island.push(island[1]);
                }

                while new_island.len() < island_size {
                    let p1 = fast_bjt_tournament(island, 3, &mut rng);
                    let p2 = fast_bjt_tournament(island, 3, &mut rng);

                    let alpha = -0.1 + 1.2 * rng.next_f64();
                    let mut child = [0.0; 3];
                    for k in 0..3 {
                        let val = alpha * p1.genome[k] + (1.0 - alpha) * p2.genome[k];
                        child[k] = val.clamp(0.0, 1.0);
                    }

                    for k in 0..3 {
                        if rng.next_f64() < mutation_rate {
                            let delta = rng.next_gaussian() * mutation_sigma;
                            child[k] = (child[k] + delta).clamp(0.0, 1.0);
                        }
                    }

                    new_island.push(eval_bjt_individual(child, &ctx));
                }

                *island = new_island;
            }

            let best_curr = islands[0][0];
            if gen >= 5 && (best_curr.loss < 1e-10 || best_curr.r_squared > 0.99999) {
                break;
            }

            if gen % 5 == 0 && num_islands > 1 {
                let best_migrants: Vec<BjtIndividual> = islands.iter().map(|isl| isl[0]).collect();
                for i in 0..num_islands {
                    let next_island = (i + 1) % num_islands;
                    let last_idx = islands[next_island].len() - 1;
                    islands[next_island][last_idx] = best_migrants[i];
                }
            }
        }

        let mut best_ind = islands[0][0];
        for island in &islands {
            for ind in island {
                if ind.loss < best_ind.loss {
                    best_ind = *ind;
                }
            }
        }

        let final_params = bjt_genome_to_params(&best_ind.genome, bounds);
        let converged = best_ind.r_squared >= 0.98 || best_ind.rmse < 1e-4;

        BjtFittingResult {
            params: final_params,
            rmse: best_ind.rmse,
            r_squared: best_ind.r_squared,
            iterations: completed_generations,
            converged,
        }
    }

    /// Polishes candidate BJT parameters using the Levenberg-Marquardt local solver.
    pub fn polish_parameters(
        params: &BjtTargetParams,
        curve: &MeasuredCurve,
    ) -> BjtFittingResult {
        polish_parameters(params, curve)
    }
}

#[inline(always)]
fn eval_bjt_individual(genome: [f64; 3], ctx: &BjtEvalContext<'_>) -> BjtIndividual {
    let params = bjt_genome_to_params(&genome, ctx.bounds);
    let (loss, rmse, r_squared) = fast_compute_bjt_loss(&params, ctx);
    BjtIndividual {
        genome,
        loss,
        rmse,
        r_squared,
    }
}

#[inline(always)]
fn fast_compute_bjt_loss(params: &BjtTargetParams, ctx: &BjtEvalContext<'_>) -> (f64, f64, f64) {
    let mut ss_res = 0.0;
    let curve = ctx.curve;
    let temp_k = curve.temperature_k;

    for pt in &curve.points {
        let i_pred = params.evaluate_ic(pt.v_ds, pt.v_gs, temp_k);
        let err = pt.i_ds - i_pred;
        ss_res += err * err;
    }

    let rmse = (ss_res / ctx.n_points).sqrt();
    let r_squared = if ctx.ss_tot > 1e-30 {
        (1.0 - (ss_res / ctx.ss_tot)).clamp(-1.0, 1.0)
    } else {
        1.0
    };

    let loss = ss_res / (ctx.ss_tot + 1e-24);
    (loss, rmse, r_squared)
}

#[inline(always)]
fn fast_bjt_tournament<'a>(
    island: &'a [BjtIndividual],
    k: usize,
    rng: &mut BjtFastRng,
) -> &'a BjtIndividual {
    let mut best_idx = (rng.next_u64() as usize) % island.len();
    let mut best_loss = island[best_idx].loss;

    for _ in 1..k {
        let candidate_idx = (rng.next_u64() as usize) % island.len();
        if island[candidate_idx].loss < best_loss {
            best_loss = island[candidate_idx].loss;
            best_idx = candidate_idx;
        }
    }

    &island[best_idx]
}

/// Translates normalized continuous genome vector $\mathbf{g} \in [0.0, 1.0]^3$ to `BjtTargetParams`.
pub fn bjt_genome_to_params(g: &[f64; 3], bounds: &BjtBounds) -> BjtTargetParams {
    let log_min = bounds.is_range.0.max(1e-24).log10();
    let log_max = bounds.is_range.1.max(1e-24).log10();
    let log_is = log_min + g[0].clamp(0.0, 1.0) * (log_max - log_min);
    let is_val = 10.0f64.powf(log_is);

    let bf_val = bounds.bf_range.0 + g[1].clamp(0.0, 1.0) * (bounds.bf_range.1 - bounds.bf_range.0);
    let vaf_val = bounds.vaf_range.0 + g[2].clamp(0.0, 1.0) * (bounds.vaf_range.1 - bounds.vaf_range.0);

    BjtTargetParams {
        is: is_val,
        bf: bf_val,
        vaf: vaf_val,
    }
}

/// Encodes `BjtTargetParams` into normalized continuous genome vector $\mathbf{g} \in [0.0, 1.0]^3$.
pub fn bjt_params_to_genome(p: &BjtTargetParams, bounds: &BjtBounds) -> [f64; 3] {
    let log_min = bounds.is_range.0.max(1e-24).log10();
    let log_max = bounds.is_range.1.max(1e-24).log10();
    let log_is = p.is.max(1e-24).log10();
    let norm_is = ((log_is - log_min) / (log_max - log_min).max(1e-12)).clamp(0.0, 1.0);

    let norm_bf = ((p.bf - bounds.bf_range.0) / (bounds.bf_range.1 - bounds.bf_range.0).max(1e-12)).clamp(0.0, 1.0);
    let norm_vaf = ((p.vaf - bounds.vaf_range.0) / (bounds.vaf_range.1 - bounds.vaf_range.0).max(1e-12)).clamp(0.0, 1.0);

    [norm_is, norm_bf, norm_vaf]
}

fn extract_bjt_heuristic_genome(curve: &MeasuredCurve, bounds: &BjtBounds) -> [f64; 3] {
    let mut default_params = BjtTargetParams::default();

    if curve.points.len() >= 3 {
        let max_ic = curve.points.iter().map(|p| p.i_ds).fold(0.0, f64::max);
        if max_ic > 1e-15 {
            let vt = phonon_core::thermal_voltage(curve.temperature_k).max(1e-4);
            // Search for bias point where Vbe is between 0.4 and 0.8 V
            if let Some(pt) = curve.points.iter().find(|p| p.v_gs >= 0.45 && p.i_ds > 1e-12) {
                let x = (pt.v_gs / vt).min(40.0);
                let exp_factor = x.exp() - 1.0;
                if exp_factor > 1e-6 {
                    let is_est = pt.i_ds / (exp_factor * (1.0 + pt.v_ds / 100.0));
                    default_params.is = is_est.clamp(bounds.is_range.0, bounds.is_range.1);
                }
            }

            // Estimate Early voltage if Vce varies
            let p_first = &curve.points[0];
            let p_last = &curve.points[curve.points.len() - 1];
            let delta_vce = (p_last.v_ds - p_first.v_ds).abs();
            let delta_vbe = (p_last.v_gs - p_first.v_gs).abs();

            if delta_vce > 0.5 && delta_vbe < 0.05 && max_ic > 0.0 {
                let slope = (p_last.i_ds - p_first.i_ds) / delta_vce;
                if slope > 1e-12 {
                    let vaf_est = max_ic / slope;
                    default_params.vaf = vaf_est.clamp(bounds.vaf_range.0, bounds.vaf_range.1);
                }
            }
        }
    }

    bjt_params_to_genome(&default_params, bounds)
}

struct BjtFastRng {
    state: u64,
}

impl BjtFastRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x626a_745f_726e_6731 } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn next_gaussian(&mut self) -> f64 {
        let u1 = self.next_f64().max(1e-15);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}
