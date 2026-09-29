//! Multi-threaded Rayon parallel benchmark runner for topological Floquet-acoustic
//! Chern insulators, verifying forward bend efficiency, reverse isolation, and quantized Chern numbers.

use phonon_models::floquet_acoustic_chern::{
    FloquetAcousticChernMetrics, FloquetAcousticChernParams,
};
use rayon::prelude::*;
use std::time::Instant;

use crate::floquet_acoustic_chern::FloquetAcousticChernSolver;

/// Single evaluated sweep point in the Floquet-acoustic Chern benchmark.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetAcousticChernSweepPoint {
    pub cycle_index: usize,
    pub params: FloquetAcousticChernParams,
    pub metrics: FloquetAcousticChernMetrics,
}

/// Comprehensive benchmark report aggregating 10,000 parameter sweeps.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetAcousticChernBenchmarkReport {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_cycles_per_sec: f64,
    pub mean_chern_number: f64,
    pub mean_topological_minigap_mhz: f64,
    pub min_topological_minigap_mhz: f64,
    pub mean_forward_bend_efficiency_pct: f64,
    pub min_forward_bend_efficiency_pct: f64,
    pub mean_reverse_isolation_db: f64,
    pub min_reverse_isolation_db: f64,
    pub mean_chiral_edge_velocity_m_s: f64,
    pub mean_beam_steering_angle_deg: f64,
    pub compliance_fraction: f64,
}

/// Benchmark runner executing multi-threaded sweeps.
pub struct FloquetAcousticChernBenchmarkRunner;

impl FloquetAcousticChernBenchmarkRunner {
    /// Executes a parallel benchmark across the specified number of cycles.
    pub fn run_benchmark(num_cycles: usize) -> FloquetAcousticChernBenchmarkReport {
        let start = Instant::now();

        let sweep_points: Vec<FloquetAcousticChernSweepPoint> = (0..num_cycles)
            .into_par_iter()
            .map(|idx| {
                let norm = (idx as f64) / (num_cycles.max(1) as f64);
                // Sweep strain amplitude [1.8e-4, 3.5e-4]
                let strain = 1.8e-4 + norm * 1.7e-4;
                // Sweep center frequency [42.0, 58.0 MHz]
                let f0 = 42.0 + norm * 16.0;
                // Sweep drive frequency [72.0, 95.0 MHz]
                let f_drive = 72.0 + (1.0 - norm) * 23.0;
                // Sweep velocity [3200.0, 3900.0 m/s]
                let v_ac = 3200.0 + norm * 700.0;
                // Sweep quality factor [3500.0, 7500.0]
                let q_factor = 3500.0 + norm * 4000.0;
                // Sweep bend angle [45.0, 120.0 deg]
                let bend_deg = 45.0 + norm * 75.0;
                // Sweep drive phase [0.0, 2*pi]
                let phase_rad = norm * 2.0 * std::f64::consts::PI;

                let params = FloquetAcousticChernParams::new(
                    50.0, v_ac, f0, f_drive, strain, phase_rad, q_factor, bend_deg, 500.0,
                );

                let solver = FloquetAcousticChernSolver::new(params);
                let metrics = solver.solve();

                FloquetAcousticChernSweepPoint {
                    cycle_index: idx,
                    params,
                    metrics,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (num_cycles as f64) / elapsed.max(1e-9);

        let mut sum_chern = 0.0;
        let mut sum_minigap = 0.0;
        let mut min_minigap = f64::MAX;
        let mut sum_bend_eff = 0.0;
        let mut min_bend_eff = f64::MAX;
        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut sum_v_edge = 0.0;
        let mut sum_steering = 0.0;
        let mut compliant_count = 0;

        for pt in &sweep_points {
            let m = &pt.metrics;
            sum_chern += m.chern_number;
            sum_minigap += m.topological_minigap_mhz;
            if m.topological_minigap_mhz < min_minigap {
                min_minigap = m.topological_minigap_mhz;
            }
            sum_bend_eff += m.forward_bend_efficiency_pct;
            if m.forward_bend_efficiency_pct < min_bend_eff {
                min_bend_eff = m.forward_bend_efficiency_pct;
            }
            sum_isolation += m.reverse_isolation_db;
            if m.reverse_isolation_db < min_isolation {
                min_isolation = m.reverse_isolation_db;
            }
            sum_v_edge += m.chiral_edge_velocity_m_s;
            sum_steering += m.beam_steering_angle_deg;
            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = num_cycles.max(1) as f64;

        FloquetAcousticChernBenchmarkReport {
            total_cycles: num_cycles,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_chern_number: sum_chern / n,
            mean_topological_minigap_mhz: sum_minigap / n,
            min_topological_minigap_mhz: min_minigap,
            mean_forward_bend_efficiency_pct: sum_bend_eff / n,
            min_forward_bend_efficiency_pct: min_bend_eff,
            mean_reverse_isolation_db: sum_isolation / n,
            min_reverse_isolation_db: min_isolation,
            mean_chiral_edge_velocity_m_s: sum_v_edge / n,
            mean_beam_steering_angle_deg: sum_steering / n,
            compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
