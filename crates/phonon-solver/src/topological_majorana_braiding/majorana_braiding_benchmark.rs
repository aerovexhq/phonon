//! Parallel parameter sweep benchmark suite for topological Majorana braiding.

use crate::topological_majorana_braiding::TopologicalMajoranaBraidingSolver;
use phonon_models::topological_majorana_braiding::{
    MajoranaBraidingMetrics, MajoranaBraidingParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for Majorana braiding sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaBraidingBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_braiding_fidelity: f64,
    pub min_braiding_fidelity: f64,
    pub mean_phase_error_rad: f64,
    pub max_phase_error_rad: f64,
    pub mean_parity_contrast: f64,
    pub min_parity_contrast: f64,
    pub mean_braiding_period_ns: f64,
    pub max_braiding_period_ns: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct MajoranaBraidingBenchmarkRunner;

impl MajoranaBraidingBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> MajoranaBraidingBenchmarkResult {
        let sweep_params: Vec<MajoranaBraidingParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let n_junc = 4 + (i % 6); // 4 to 9 junctions
                let gap_uev = 180.0 + 150.0 * frac; // 180 to 330 ueV
                let strain = 3.0e-4 + 3.0e-4 * ((i % 50) as f64 / 50.0); // 3e-4 to 6e-4
                let tau_ns = 15.0 + 25.0 * ((i % 80) as f64 / 80.0); // 15 to 40 ns
                let gamma_qp_khz = 0.40 + 0.80 * frac; // 0.40 to 1.20 kHz
                let overlap_nev = 6.0 + 12.0 * ((i % 60) as f64 / 60.0); // 6 to 18 neV
                let q_res = 2.5e4 + 3.0e4 * frac; // 2.5e4 to 5.5e4
                let temp_mk = 15.0 + 20.0 * ((i % 40) as f64 / 40.0); // 15 to 35 mK

                MajoranaBraidingParams::new(
                    n_junc,
                    gap_uev,
                    strain,
                    tau_ns,
                    gamma_qp_khz,
                    overlap_nev,
                    q_res,
                    temp_mk,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<MajoranaBraidingMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = TopologicalMajoranaBraidingSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut sum_err = 0.0;
        let mut max_err = 0.0;
        let mut sum_con = 0.0;
        let mut min_con = f64::MAX;
        let mut sum_tau = 0.0;
        let mut max_tau = 0.0;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_fid += m.braiding_gate_fidelity;
            if m.braiding_gate_fidelity < min_fid {
                min_fid = m.braiding_gate_fidelity;
            }

            sum_err += m.non_abelian_phase_error_rad;
            if m.non_abelian_phase_error_rad > max_err {
                max_err = m.non_abelian_phase_error_rad;
            }

            sum_con += m.parity_readout_contrast;
            if m.parity_readout_contrast < min_con {
                min_con = m.parity_readout_contrast;
            }

            sum_tau += m.braiding_cycle_period_ns;
            if m.braiding_cycle_period_ns > max_tau {
                max_tau = m.braiding_cycle_period_ns;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        MajoranaBraidingBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_braiding_fidelity: sum_fid / n,
            min_braiding_fidelity: min_fid,
            mean_phase_error_rad: sum_err / n,
            max_phase_error_rad: max_err,
            mean_parity_contrast: sum_con / n,
            min_parity_contrast: min_con,
            mean_braiding_period_ns: sum_tau / n,
            max_braiding_period_ns: max_tau,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
