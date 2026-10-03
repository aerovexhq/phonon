#![deny(unsafe_code)]

//! EKV compact MOSFET parameter estimation engine and curve-fitting optimizer.

use super::curve_data::{MeasuredCurve, MeasurementPoint};
use super::polisher::{polish_parameters, GenericFittingResult, PolishableModel};

/// Target EKV physical parameters optimized during device parameter extraction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EkvTargetParams {
    /// Zero-bias nominal threshold voltage $V_{T0}$ in Volts ($V$), valid range [0.1, 1.5].
    pub vto: f64,
    /// Transconductance parameter $KP = \mu C_{ox}$ in $A / V^2$, valid range [1e-6, 1e-3].
    pub kp: f64,
    /// Body effect parameter $\gamma$ in $\sqrt{V}$, valid range [0.0, 2.0].
    pub gamma: f64,
    /// Channel mobility degradation parameter $\theta$ in $1 / V$, valid range [0.0, 0.5].
    pub theta: f64,
}

impl Default for EkvTargetParams {
    fn default() -> Self {
        Self {
            vto: 0.50,
            kp: 1.5e-4,
            gamma: 0.50,
            theta: 0.05,
        }
    }
}

impl EkvTargetParams {
    /// Clamps all parameter values strictly within physical boundaries.
    pub fn clamp(&mut self) {
        self.vto = self.vto.clamp(0.1, 1.5);
        self.kp = self.kp.clamp(1e-6, 1e-3);
        self.gamma = self.gamma.clamp(0.0, 2.0);
        self.theta = self.theta.clamp(0.0, 0.5);
    }

    /// Evaluates the compact EKV drain current $I_{ds}$ in Amperes ($A$).
    pub fn evaluate_ids(
        &self,
        v_ds: f64,
        v_gs: f64,
        v_bs: f64,
        w: f64,
        l: f64,
        temp_k: f64,
    ) -> f64 {
        let v_ds = v_ds.max(0.0);
        let w = w.max(1e-9);
        let l = l.max(1e-9);
        let temp_k = temp_k.max(1.0);
        let ut = phonon_core::thermal_voltage(temp_k).max(1e-4);

        let phi_0 = 0.6;
        let v_gb = v_gs - v_bs;
        let v_sb = -v_bs;
        let v_db = v_ds - v_bs;

        // Substrate slope factor n
        let rad = (phi_0 + (v_gb - self.vto).max(0.0)).max(1e-4);
        let n = 1.0 + self.gamma / (2.0 * rad.sqrt());

        // Pinch-off voltage Vp
        let vp = (v_gb - self.vto) / n;

        // Normalized forward and reverse overdrive
        let xf = (vp - v_sb) / (2.0 * ut);
        let xr = (vp - v_db) / (2.0 * ut);

        let softplus = |x: f64| -> f64 {
            if x > 40.0 {
                x
            } else if x < -40.0 {
                x.exp()
            } else {
                (1.0 + x.exp()).ln()
            }
        };

        let i_f = softplus(xf).powi(2);
        let i_r = softplus(xr).powi(2);

        // Specific current Ispec = 2 * n * kp * (W / L) * Ut^2
        let i_spec = 2.0 * n * self.kp * (w / l) * ut * ut;
        let ids0 = (i_spec * (i_f - i_r)).max(0.0);

        // Longitudinal vertical-field mobility reduction
        let degradation = 1.0 + self.theta * vp.max(0.0);
        (ids0 / degradation.max(0.1)).max(0.0)
    }
}

/// Bounding constraints for EKV MOSFET parameter optimization.
#[derive(Debug, Clone, PartialEq)]
pub struct EkvBounds {
    pub vto: (f64, f64),
    pub kp: (f64, f64),
    pub gamma: (f64, f64),
    pub theta: (f64, f64),
}

impl Default for EkvBounds {
    fn default() -> Self {
        Self {
            vto: (0.1, 1.5),
            kp: (1e-6, 1e-3),
            gamma: (0.0, 2.0),
            theta: (0.0, 0.5),
        }
    }
}

impl PolishableModel for EkvTargetParams {
    fn num_params() -> usize {
        4
    }

    fn to_normalized(&self) -> Vec<f64> {
        vec![
            ((self.vto - 0.1) / (1.5 - 0.1)).clamp(0.0, 1.0),
            ((self.kp - 1e-6) / (1e-3 - 1e-6)).clamp(0.0, 1.0),
            (self.gamma / 2.0).clamp(0.0, 1.0),
            (self.theta / 0.5).clamp(0.0, 1.0),
        ]
    }

    fn from_normalized(norm: &[f64]) -> Self {
        Self {
            vto: 0.1 + norm[0].clamp(0.0, 1.0) * (1.5 - 0.1),
            kp: 1e-6 + norm[1].clamp(0.0, 1.0) * (1e-3 - 1e-6),
            gamma: norm[2].clamp(0.0, 1.0) * 2.0,
            theta: norm[3].clamp(0.0, 1.0) * 0.5,
        }
    }

    fn evaluate_current(&self, pt: &MeasurementPoint, curve: &MeasuredCurve) -> f64 {
        self.evaluate_ids(
            pt.v_ds,
            pt.v_gs,
            pt.v_bs,
            curve.channel_width_m,
            curve.channel_length_m,
            curve.temperature_k,
        )
    }
}

/// Type alias for EKV fitting results.
pub type EkvFittingResult = GenericFittingResult<EkvTargetParams>;

/// Multi-island genetic algorithm and Levenberg-Marquardt hybrid optimizer for EKV parameters.
#[derive(Debug, Clone, Copy, Default)]
pub struct EkvOptimizer;

#[derive(Clone, Copy)]
struct EkvIndividual {
    genome: [f64; 4],
    loss: f64,
    rmse: f64,
    r_squared: f64,
}

struct EkvEvalContext<'a> {
    curve: &'a MeasuredCurve,
    bounds: &'a EkvBounds,
    ss_tot: f64,
    n_points: f64,
}

impl EkvOptimizer {
    /// Fits EKV model parameters to the given measurement curve with hybrid GA and LM polishing.
    pub fn fit_curve(
        curve: &MeasuredCurve,
        max_generations: usize,
        population_size: usize,
    ) -> EkvFittingResult {
        let default_bounds = EkvBounds::default();
        Self::fit_curve_with_bounds(curve, max_generations, population_size, &default_bounds)
    }

    /// Fits EKV model parameters within custom parameter bounds using GA and LM polishing.
    pub fn fit_curve_with_bounds(
        curve: &MeasuredCurve,
        max_generations: usize,
        population_size: usize,
        bounds: &EkvBounds,
    ) -> EkvFittingResult {
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
        bounds: &EkvBounds,
    ) -> EkvFittingResult {
        let n_points = curve.points.len();
        if n_points == 0 {
            return EkvFittingResult {
                params: EkvTargetParams::default(),
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

        let ctx = EkvEvalContext {
            curve,
            bounds,
            ss_tot,
            n_points: n_f64,
        };

        let pop_size = population_size.max(12);
        let generations = max_generations.max(5);

        let mut rng = EkvFastRng::new(0x454b_564d_4f53_3132);

        let num_islands = 3;
        let island_size = (pop_size / num_islands).max(4);
        let mut islands: Vec<Vec<EkvIndividual>> = Vec::with_capacity(num_islands);

        let heuristic_genome = extract_ekv_heuristic_genome(curve, bounds);
        let default_genome = ekv_params_to_genome(&EkvTargetParams::default(), bounds);

        for island_idx in 0..num_islands {
            let mut island = Vec::with_capacity(island_size);
            if island_idx == 0 {
                island.push(eval_ekv_individual(heuristic_genome, &ctx));
                island.push(eval_ekv_individual(default_genome, &ctx));
            }
            while island.len() < island_size {
                let g = [rng.next_f64(), rng.next_f64(), rng.next_f64(), rng.next_f64()];
                island.push(eval_ekv_individual(g, &ctx));
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
                    let p1 = fast_ekv_tournament(island, 3, &mut rng);
                    let p2 = fast_ekv_tournament(island, 3, &mut rng);

                    let alpha = -0.1 + 1.2 * rng.next_f64();
                    let mut child = [0.0; 4];
                    for k in 0..4 {
                        let val = alpha * p1.genome[k] + (1.0 - alpha) * p2.genome[k];
                        child[k] = val.clamp(0.0, 1.0);
                    }

                    for k in 0..4 {
                        if rng.next_f64() < mutation_rate {
                            let delta = rng.next_gaussian() * mutation_sigma;
                            child[k] = (child[k] + delta).clamp(0.0, 1.0);
                        }
                    }

                    new_island.push(eval_ekv_individual(child, &ctx));
                }

                *island = new_island;
            }

            let best_curr = islands[0][0];
            if gen >= 5 && (best_curr.loss < 1e-10 || best_curr.r_squared > 0.99999) {
                break;
            }

            if gen % 5 == 0 && num_islands > 1 {
                let best_migrants: Vec<EkvIndividual> = islands.iter().map(|isl| isl[0]).collect();
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

        let final_params = ekv_genome_to_params(&best_ind.genome, bounds);
        let converged = best_ind.r_squared >= 0.98 || best_ind.rmse < 1e-4;

        EkvFittingResult {
            params: final_params,
            rmse: best_ind.rmse,
            r_squared: best_ind.r_squared,
            iterations: completed_generations,
            converged,
        }
    }

    /// Polishes candidate EKV parameters using the Levenberg-Marquardt local solver.
    pub fn polish_parameters(
        params: &EkvTargetParams,
        curve: &MeasuredCurve,
    ) -> EkvFittingResult {
        polish_parameters(params, curve)
    }
}

#[inline(always)]
fn eval_ekv_individual(genome: [f64; 4], ctx: &EkvEvalContext<'_>) -> EkvIndividual {
    let params = ekv_genome_to_params(&genome, ctx.bounds);
    let (loss, rmse, r_squared) = fast_compute_ekv_loss(&params, ctx);
    EkvIndividual {
        genome,
        loss,
        rmse,
        r_squared,
    }
}

#[inline(always)]
fn fast_compute_ekv_loss(params: &EkvTargetParams, ctx: &EkvEvalContext<'_>) -> (f64, f64, f64) {
    let mut ss_res = 0.0;
    let curve = ctx.curve;
    let w = curve.channel_width_m;
    let l = curve.channel_length_m;
    let temp_k = curve.temperature_k;

    for pt in &curve.points {
        let i_pred = params.evaluate_ids(pt.v_ds, pt.v_gs, pt.v_bs, w, l, temp_k);
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
fn fast_ekv_tournament<'a>(
    island: &'a [EkvIndividual],
    k: usize,
    rng: &mut EkvFastRng,
) -> &'a EkvIndividual {
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

/// Translates normalized continuous genome vector $\mathbf{g} \in [0.0, 1.0]^4$ to `EkvTargetParams`.
pub fn ekv_genome_to_params(g: &[f64; 4], bounds: &EkvBounds) -> EkvTargetParams {
    EkvTargetParams {
        vto: bounds.vto.0 + g[0].clamp(0.0, 1.0) * (bounds.vto.1 - bounds.vto.0),
        kp: bounds.kp.0 + g[1].clamp(0.0, 1.0) * (bounds.kp.1 - bounds.kp.0),
        gamma: bounds.gamma.0 + g[2].clamp(0.0, 1.0) * (bounds.gamma.1 - bounds.gamma.0),
        theta: bounds.theta.0 + g[3].clamp(0.0, 1.0) * (bounds.theta.1 - bounds.theta.0),
    }
}

/// Encodes `EkvTargetParams` into normalized continuous genome vector $\mathbf{g} \in [0.0, 1.0]^4$.
pub fn ekv_params_to_genome(p: &EkvTargetParams, bounds: &EkvBounds) -> [f64; 4] {
    [
        ((p.vto - bounds.vto.0) / (bounds.vto.1 - bounds.vto.0).max(1e-12)).clamp(0.0, 1.0),
        ((p.kp - bounds.kp.0) / (bounds.kp.1 - bounds.kp.0).max(1e-12)).clamp(0.0, 1.0),
        ((p.gamma - bounds.gamma.0) / (bounds.gamma.1 - bounds.gamma.0).max(1e-12)).clamp(0.0, 1.0),
        ((p.theta - bounds.theta.0) / (bounds.theta.1 - bounds.theta.0).max(1e-12)).clamp(0.0, 1.0),
    ]
}

fn extract_ekv_heuristic_genome(curve: &MeasuredCurve, bounds: &EkvBounds) -> [f64; 4] {
    let mut default_params = EkvTargetParams::default();

    if curve.points.len() >= 4 {
        let max_ids = curve.points.iter().map(|p| p.i_ds).fold(0.0, f64::max);
        if max_ids > 1e-12 {
            let p_upper: Vec<_> = curve.points.iter().filter(|p| p.i_ds >= 0.20 * max_ids).collect();
            if p_upper.len() >= 2 {
                let first = p_upper[0];
                let last = p_upper[p_upper.len() - 1];

                let delta_vgs = (last.v_gs - first.v_gs).abs();
                let delta_vds = (last.v_ds - first.v_ds).abs();

                if delta_vgs > delta_vds && delta_vgs > 0.05 {
                    let sqrt1 = first.i_ds.max(0.0).sqrt();
                    let sqrt2 = last.i_ds.max(0.0).sqrt();
                    let diff_sqrt = sqrt2 - sqrt1;
                    if diff_sqrt.abs() > 1e-6 {
                        let vto_est = (last.v_gs * sqrt1 - first.v_gs * sqrt2) / (sqrt1 - sqrt2);
                        default_params.vto = vto_est.clamp(bounds.vto.0, bounds.vto.1);

                        let test_eval = default_params.evaluate_ids(
                            last.v_ds,
                            last.v_gs,
                            last.v_bs,
                            curve.channel_width_m,
                            curve.channel_length_m,
                            curve.temperature_k,
                        );
                        if test_eval > 1e-12 {
                            let ratio = last.i_ds / test_eval;
                            default_params.kp = (default_params.kp * ratio).clamp(bounds.kp.0, bounds.kp.1);
                        }
                    }
                } else if delta_vds > 0.05 {
                    let test_eval = default_params.evaluate_ids(
                        last.v_ds,
                        last.v_gs,
                        last.v_bs,
                        curve.channel_width_m,
                        curve.channel_length_m,
                        curve.temperature_k,
                    );
                    if test_eval > 1e-12 {
                        let ratio = last.i_ds / test_eval;
                        default_params.kp = (default_params.kp * ratio).clamp(bounds.kp.0, bounds.kp.1);
                    }
                }
            }
        }
    }

    ekv_params_to_genome(&default_params, bounds)
}

struct EkvFastRng {
    state: u64,
}

impl EkvFastRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x517c_c1b7_2722_0a95 } else { seed },
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
