#![deny(unsafe_code)]

//! High-throughput genetic algorithm curve-fitting engine and compact BSIM4 parameter optimizer.

use super::curve_data::MeasuredCurve;

/// Target BSIM4 physical parameters optimized during device parameter extraction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bsim4TargetParams {
    /// Zero-bias threshold voltage $V_{th0}$ in Volts ($V$), valid range [0.1, 1.2].
    pub vth0: f64,
    /// Low-field carrier mobility $\mu_0$ in $m^2 / (V \cdot s)$, valid range [0.01, 0.15].
    pub u0: f64,
    /// Carrier saturation velocity $v_{sat}$ in $m/s$, valid range [5e4, 2e5].
    pub vsat: f64,
    /// Short-channel effect coefficient $DVT0$, valid range [0.1, 4.0].
    pub dvt0: f64,
    /// Drain-Induced Barrier Lowering (DIBL) coefficient $\eta_0$, valid range [0.0, 0.5].
    pub eta0: f64,
    /// Source/drain series resistance $RDSW$ in $\Omega \cdot \mu m$, valid range [0.0, 250.0].
    pub rdsw: f64,
    /// Subthreshold swing $S$ in $mV / \text{decade}$, valid range [60.0, 150.0].
    pub subthreshold_swing_mv_dec: f64,
}

impl Default for Bsim4TargetParams {
    fn default() -> Self {
        Self {
            vth0: 0.45,
            u0: 0.05,
            vsat: 1.0e5,
            dvt0: 1.0,
            eta0: 0.08,
            rdsw: 100.0,
            subthreshold_swing_mv_dec: 80.0,
        }
    }
}

impl Bsim4TargetParams {
    /// Clamps all parameter values strictly within valid physical bounds.
    pub fn clamp(&mut self) {
        self.vth0 = self.vth0.clamp(0.1, 1.2);
        self.u0 = self.u0.clamp(0.01, 0.15);
        self.vsat = self.vsat.clamp(5e4, 2e5);
        self.dvt0 = self.dvt0.clamp(0.1, 4.0);
        self.eta0 = self.eta0.clamp(0.0, 0.5);
        self.rdsw = self.rdsw.clamp(0.0, 250.0);
        self.subthreshold_swing_mv_dec = self.subthreshold_swing_mv_dec.clamp(60.0, 150.0);
    }

    /// Evaluates the compact BSIM4 drain current $I_{ds}$ in Amperes ($A$).
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

        let vt = phonon_core::thermal_voltage(temp_k);
        let c_ox = (phonon_core::EPSILON_0 * phonon_core::EPSILON_R_OX) / 3.0e-9;

        // Temperature adjustments
        let delta_t = temp_k - 300.0;
        let vth_t = self.vth0 - 1.2e-3 * delta_t;
        let t_ratio = (temp_k / 300.0).max(0.01);
        let mu_t = self.u0 * t_ratio.powf(-1.5);

        // Body effect and DIBL
        let phi_s = 0.65;
        let gamma = 0.4;
        let v_bs_eff = v_bs.min(0.5 * phi_s);
        let body_term = (phi_s - v_bs_eff).max(1e-4).sqrt() - phi_s.sqrt();
        let delta_vth_body = gamma * body_term;

        // DIBL & short-channel effect rolloff
        let l_ref = 100e-9;
        let delta_vth_sce = self.dvt0 * 0.02 * (l_ref / l).min(3.0);
        let delta_vth_dibl = self.eta0 * v_ds;

        let vth_eff = vth_t - delta_vth_sce - delta_vth_dibl + delta_vth_body;

        // Subthreshold swing factor
        let s_volts_per_dec = self.subthreshold_swing_mv_dec * 1e-3;
        let ln10 = std::f64::consts::LN_10;
        let n_sub = (s_volts_per_dec / (ln10 * vt)).max(1.0);
        let two_n_vt = 2.0 * n_sub * vt;

        // Unified effective overdrive voltage V_gsteff
        let v_ov = v_gs - vth_eff;
        let x = v_ov / two_n_vt;
        let v_gsteff = if x > 40.0 {
            v_ov
        } else if x < -40.0 {
            two_n_vt * x.exp()
        } else {
            two_n_vt * (1.0 + x.exp()).ln()
        };

        // Velocity saturation and saturation voltage V_dsat
        let e_sat = (2.0 * self.vsat / mu_t).max(1e3);
        let e_sat_l = e_sat * l;
        let v_dsat = (e_sat_l * v_gsteff) / (e_sat_l + v_gsteff + 1e-12);

        // Smooth transition to saturation voltage V_dseff
        let delta_sat = 0.02;
        let term_diff = v_dsat - v_ds - delta_sat;
        let root_term = (term_diff * term_diff + 4.0 * delta_sat * v_dsat).sqrt();
        let v_dseff = v_dsat - 0.5 * (term_diff + root_term);

        // Series resistance RDSW
        let r_ds = (self.rdsw * 1e-6) / w;
        let beta = mu_t * c_ox * (w / l);
        let denominator = 1.0 + v_dseff / e_sat_l + beta * r_ds * v_gsteff;

        // Current before CLM
        let i_ds0 = (beta * (v_gsteff - 0.5 * v_dseff).max(0.0) * v_dseff) / denominator.max(1e-6);

        // Channel-length modulation
        let lambda_clm = 0.03 * (l_ref / l).min(5.0);
        let clm_factor = 1.0 + lambda_clm * (v_ds - v_dseff).max(0.0);

        (i_ds0 * clm_factor).max(0.0)
    }
}

/// Evaluation and convergence telemetry from a completed curve fitting execution.
#[derive(Debug, Clone, PartialEq)]
pub struct FittingResult {
    pub params: Bsim4TargetParams,
    pub rmse: f64,
    pub r_squared: f64,
    pub iterations: usize,
    pub converged: bool,
}

/// Multi-island genetic algorithm curve-fitting engine with local refinement.
#[derive(Debug, Clone, Copy, Default)]
pub struct GaOptimizer;

#[derive(Clone, Copy)]
struct Individual {
    genome: [f64; 7],
    loss: f64,
    rmse: f64,
    r_squared: f64,
}

struct EvalContext<'a> {
    curve: &'a MeasuredCurve,
    ss_tot: f64,
    n_points: f64,
}

impl GaOptimizer {
    /// Fits compact BSIM4 model parameters to the given measurement curve.
    pub fn fit_curve(
        curve: &MeasuredCurve,
        max_generations: usize,
        population_size: usize,
    ) -> FittingResult {
        let n_points = curve.points.len();
        if n_points == 0 {
            return FittingResult {
                params: Bsim4TargetParams::default(),
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

        let ctx = EvalContext {
            curve,
            ss_tot,
            n_points: n_f64,
        };

        let pop_size = population_size.max(12);
        let generations = max_generations.max(5);

        let mut rng = FastRng::new(0x4d59_5350_4943_4531);

        let num_islands = 3;
        let island_size = (pop_size / num_islands).max(4);

        let mut islands: Vec<Vec<Individual>> = Vec::with_capacity(num_islands);

        let heuristic_genome = extract_heuristic_genome(curve);
        let default_genome = params_to_genome(&Bsim4TargetParams::default());

        for island_idx in 0..num_islands {
            let mut island = Vec::with_capacity(island_size);
            if island_idx == 0 {
                island.push(eval_individual(heuristic_genome, &ctx));
                island.push(eval_individual(default_genome, &ctx));
            }
            while island.len() < island_size {
                let g = [
                    rng.next_f64(),
                    rng.next_f64(),
                    rng.next_f64(),
                    rng.next_f64(),
                    rng.next_f64(),
                    rng.next_f64(),
                    rng.next_f64(),
                ];
                island.push(eval_individual(g, &ctx));
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

                // Elitism: preserve top 2
                new_island.push(island[0]);
                if island.len() > 1 {
                    new_island.push(island[1]);
                }

                while new_island.len() < island_size {
                    let p1 = fast_tournament_select(island, 3, &mut rng);
                    let p2 = fast_tournament_select(island, 3, &mut rng);

                    let alpha = -0.1 + 1.2 * rng.next_f64();
                    let mut child = [0.0; 7];
                    for k in 0..7 {
                        let val = alpha * p1.genome[k] + (1.0 - alpha) * p2.genome[k];
                        child[k] = val.clamp(0.0, 1.0);
                    }

                    for k in 0..7 {
                        if rng.next_f64() < mutation_rate {
                            let delta = rng.next_gaussian() * mutation_sigma;
                            child[k] = (child[k] + delta).clamp(0.0, 1.0);
                        }
                    }

                    new_island.push(eval_individual(child, &ctx));
                }

                *island = new_island;
            }

            // Check if elite has already converged with near-perfect fit after at least 5 generations
            let best_curr = islands[0][0];
            if gen >= 5 && (best_curr.loss < 1e-10 || best_curr.r_squared > 0.99999) {
                break;
            }

            // Periodic migration
            if gen % 5 == 0 && num_islands > 1 {
                let best_migrants: Vec<Individual> = islands.iter().map(|isl| isl[0]).collect();
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

        // Local coordinate descent refinement with 2 passes
        let mut best_genome = best_ind.genome;
        let mut best_loss = best_ind.loss;
        let mut best_rmse = best_ind.rmse;
        let mut best_r2 = best_ind.r_squared;

        let step_sizes = [0.02, 0.008, 0.002, 0.0005];
        for &step in &step_sizes {
            for _pass in 0..2 {
                for param_idx in 0..7 {
                    let mut test_plus = best_genome;
                    test_plus[param_idx] = (test_plus[param_idx] + step).clamp(0.0, 1.0);
                    let (loss_plus, rmse_p, r2_p) = fast_compute_loss(&genome_to_params(&test_plus), &ctx);
                    if loss_plus < best_loss {
                        best_loss = loss_plus;
                        best_rmse = rmse_p;
                        best_r2 = r2_p;
                        best_genome = test_plus;
                        continue;
                    }

                    let mut test_minus = best_genome;
                    test_minus[param_idx] = (test_minus[param_idx] - step).clamp(0.0, 1.0);
                    let (loss_minus, rmse_m, r2_m) = fast_compute_loss(&genome_to_params(&test_minus), &ctx);
                    if loss_minus < best_loss {
                        best_loss = loss_minus;
                        best_rmse = rmse_m;
                        best_r2 = r2_m;
                        best_genome = test_minus;
                    }
                }
            }
        }

        let final_params = genome_to_params(&best_genome);
        let converged = best_r2 >= 0.98 || best_rmse < 1e-4;

        FittingResult {
            params: final_params,
            rmse: best_rmse,
            r_squared: best_r2,
            iterations: completed_generations,
            converged,
        }
    }
}

#[inline(always)]
fn eval_individual(genome: [f64; 7], ctx: &EvalContext<'_>) -> Individual {
    let params = genome_to_params(&genome);
    let (loss, rmse, r_squared) = fast_compute_loss(&params, ctx);
    Individual {
        genome,
        loss,
        rmse,
        r_squared,
    }
}

#[inline(always)]
fn fast_compute_loss(params: &Bsim4TargetParams, ctx: &EvalContext<'_>) -> (f64, f64, f64) {
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
fn fast_tournament_select<'a>(
    island: &'a [Individual],
    k: usize,
    rng: &mut FastRng,
) -> &'a Individual {
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

/// Translates normalized continuous genome vector $\mathbf{g} \in [0.0, 1.0]^7$ to `Bsim4TargetParams`.
pub fn genome_to_params(g: &[f64; 7]) -> Bsim4TargetParams {
    Bsim4TargetParams {
        vth0: 0.1 + g[0].clamp(0.0, 1.0) * (1.2 - 0.1),
        u0: 0.01 + g[1].clamp(0.0, 1.0) * (0.15 - 0.01),
        vsat: 5e4 + g[2].clamp(0.0, 1.0) * (2e5 - 5e4),
        dvt0: 0.1 + g[3].clamp(0.0, 1.0) * (4.0 - 0.1),
        eta0: 0.0 + g[4].clamp(0.0, 1.0) * 0.5,
        rdsw: 0.0 + g[5].clamp(0.0, 1.0) * 250.0,
        subthreshold_swing_mv_dec: 60.0 + g[6].clamp(0.0, 1.0) * (150.0 - 60.0),
    }
}

/// Encodes `Bsim4TargetParams` into normalized continuous genome vector $\mathbf{g} \in [0.0, 1.0]^7$.
pub fn params_to_genome(p: &Bsim4TargetParams) -> [f64; 7] {
    [
        ((p.vth0 - 0.1) / (1.2 - 0.1)).clamp(0.0, 1.0),
        ((p.u0 - 0.01) / (0.15 - 0.01)).clamp(0.0, 1.0),
        ((p.vsat - 5e4) / (2e5 - 5e4)).clamp(0.0, 1.0),
        ((p.dvt0 - 0.1) / (4.0 - 0.1)).clamp(0.0, 1.0),
        (p.eta0 / 0.5).clamp(0.0, 1.0),
        (p.rdsw / 250.0).clamp(0.0, 1.0),
        ((p.subthreshold_swing_mv_dec - 60.0) / (150.0 - 60.0)).clamp(0.0, 1.0),
    ]
}

/// Estimates initial parameters from measurement curve heuristics.
fn extract_heuristic_genome(curve: &MeasuredCurve) -> [f64; 7] {
    let mut default_params = Bsim4TargetParams::default();

    if curve.points.len() >= 4 {
        let max_ids = curve.points.iter().map(|p| p.i_ds).fold(0.0, f64::max);
        if max_ids > 1e-12 {
            let p_upper: Vec<_> = curve.points.iter().filter(|p| p.i_ds >= 0.25 * max_ids).collect();
            if p_upper.len() >= 2 {
                let first = p_upper[0];
                let last = p_upper[p_upper.len() - 1];

                let delta_vgs = (last.v_gs - first.v_gs).abs();
                let delta_vds = (last.v_ds - first.v_ds).abs();

                if delta_vgs > delta_vds && delta_vgs > 0.05 {
                    // Transfer curve: use sqrt(Ids) linear extrapolation
                    let sqrt1 = first.i_ds.max(0.0).sqrt();
                    let sqrt2 = last.i_ds.max(0.0).sqrt();
                    let diff_sqrt = sqrt2 - sqrt1;
                    if diff_sqrt.abs() > 1e-6 {
                        let vth_eff_est = (last.v_gs * sqrt1 - first.v_gs * sqrt2) / (sqrt1 - sqrt2);
                        let vth0_est = vth_eff_est
                            + default_params.dvt0 * 0.02
                            + default_params.eta0 * last.v_ds;
                        default_params.vth0 = vth0_est.clamp(0.1, 1.2);

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
                            default_params.u0 = (default_params.u0 * ratio).clamp(0.01, 0.15);
                        }
                    }
                } else if delta_vds > 0.05 {
                    // Output curve
                    let gds_sat = (last.i_ds - first.i_ds) / delta_vds;
                    if max_ids > 0.0 {
                        let eta_est = (gds_sat / max_ids).clamp(0.0, 0.3);
                        default_params.eta0 = eta_est;
                    }
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
                        default_params.u0 = (default_params.u0 * ratio).clamp(0.01, 0.15);
                    }
                }
            }
        }
    }

    params_to_genome(&default_params)
}

/// Ultra-fast pure safe PRNG based on 64-bit XorShift algorithm.
struct FastRng {
    state: u64,
}

impl FastRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c_49e6_748f_ea9b } else { seed },
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
