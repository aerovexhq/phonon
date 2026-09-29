//! Multi-threaded Rayon parallel benchmark runner for non-Hermitian phononic
//! exceptional point gyroscopes, verifying scale enhancement, dynamic range, and bias stability.

use phonon_models::non_hermitian_ep_gyroscope::{EpGyroscopeMetrics, EpGyroscopeParams};
use rayon::prelude::*;
use std::time::Instant;

use crate::non_hermitian_ep_gyroscope::EpGyroscopeSolver;

/// Single evaluated sweep point in the EP gyroscope benchmark.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EpGyroscopeSweepPoint {
    pub cycle_index: usize,
    pub params: EpGyroscopeParams,
    pub metrics: EpGyroscopeMetrics,
}

/// Comprehensive benchmark report aggregating 10,000 parameter sweeps.
#[derive(Debug, Clone, PartialEq)]
pub struct EpGyroscopeBenchmarkReport {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_cycles_per_sec: f64,
    pub mean_scale_factor_enhancement: f64,
    pub min_scale_factor_enhancement: f64,
    pub mean_dynamic_range_db: f64,
    pub min_dynamic_range_db: f64,
    pub mean_angle_random_walk: f64,
    pub max_angle_random_walk: f64,
    pub mean_bias_stability: f64,
    pub max_bias_stability: f64,
    pub mean_petermann_factor: f64,
    pub compliance_fraction: f64,
}

/// Benchmark runner executing multi-threaded sweeps.
pub struct EpGyroscopeBenchmarkRunner;

impl EpGyroscopeBenchmarkRunner {
    /// Executes a parallel benchmark across the specified number of cycles.
    pub fn run_benchmark(num_cycles: usize) -> EpGyroscopeBenchmarkReport {
        let start = Instant::now();

        let sweep_points: Vec<EpGyroscopeSweepPoint> = (0..num_cycles)
            .into_par_iter()
            .map(|idx| {
                let norm = (idx as f64) / (num_cycles.max(1) as f64);
                // Sweep ring radius [150.0, 350.0 um]
                let radius = 150.0 + norm * 200.0;
                // Sweep center frequency [400.0, 600.0 MHz]
                let f0 = 400.0 + norm * 200.0;
                // Sweep velocity [3500.0, 4200.0 m/s]
                let v_ac = 3500.0 + norm * 700.0;
                // Sweep coupling [1.5, 3.0 MHz]
                let kappa = 1.5 + norm * 1.5;
                // Sweep gain/loss close to kappa (ratio 0.985 - 0.998)
                let gamma = kappa * (0.985 + norm * 0.013);
                // Sweep rotation rate [0.1, 50.0 deg/s]
                let rot_rate = 0.1 + norm * 49.9;
                // Sweep temperature [77.0, 300.0 K]
                let temp = 77.0 + norm * 223.0;
                // Sweep quality factor [15000.0, 45000.0]
                let q_factor = 15000.0 + norm * 30000.0;

                let params = EpGyroscopeParams::new(
                    radius, f0, v_ac, kappa, gamma, rot_rate, temp, q_factor,
                );

                let solver = EpGyroscopeSolver::new(params);
                let metrics = solver.solve();

                EpGyroscopeSweepPoint {
                    cycle_index: idx,
                    params,
                    metrics,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (num_cycles as f64) / elapsed.max(1e-9);

        let mut sum_enhancement = 0.0;
        let mut min_enhancement = f64::MAX;
        let mut sum_dr = 0.0;
        let mut min_dr = f64::MAX;
        let mut sum_arw = 0.0;
        let mut max_arw = f64::MIN;
        let mut sum_bias = 0.0;
        let mut max_bias = f64::MIN;
        let mut sum_petermann = 0.0;
        let mut compliant_count = 0;

        for pt in &sweep_points {
            let m = &pt.metrics;
            sum_enhancement += m.scale_factor_enhancement;
            if m.scale_factor_enhancement < min_enhancement {
                min_enhancement = m.scale_factor_enhancement;
            }
            sum_dr += m.dynamic_range_db;
            if m.dynamic_range_db < min_dr {
                min_dr = m.dynamic_range_db;
            }
            sum_arw += m.angle_random_walk_deg_sqrthr;
            if m.angle_random_walk_deg_sqrthr > max_arw {
                max_arw = m.angle_random_walk_deg_sqrthr;
            }
            sum_bias += m.bias_stability_deg_hr;
            if m.bias_stability_deg_hr > max_bias {
                max_bias = m.bias_stability_deg_hr;
            }
            sum_petermann += m.petermann_factor;
            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = num_cycles.max(1) as f64;

        EpGyroscopeBenchmarkReport {
            total_cycles: num_cycles,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_scale_factor_enhancement: sum_enhancement / n,
            min_scale_factor_enhancement: min_enhancement,
            mean_dynamic_range_db: sum_dr / n,
            min_dynamic_range_db: min_dr,
            mean_angle_random_walk: sum_arw / n,
            max_angle_random_walk: max_arw,
            mean_bias_stability: sum_bias / n,
            max_bias_stability: max_bias,
            mean_petermann_factor: sum_petermann / n,
            compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
