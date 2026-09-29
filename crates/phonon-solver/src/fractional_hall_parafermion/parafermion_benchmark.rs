#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for fractional quantum Hall acoustic
//! metamaterials and non-Abelian parafermion interferometers across multi-threaded Rayon workers.

use crate::fractional_hall_parafermion::parafermion_solver::FractionalHallParafermionSolver;
use phonon_models::fractional_hall_parafermion::{
    FractionalHallParafermionMetrics, FractionalHallParafermionParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for fractional Hall parafermion parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParafermionBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_braid_phase_fidelity: f64,
    pub min_braid_phase_fidelity: f64,
    pub max_braid_phase_fidelity: f64,
    pub mean_fractional_state_fidelity: f64,
    pub min_fractional_state_fidelity: f64,
    pub max_fractional_state_fidelity: f64,
    pub mean_fractional_quantization_error: f64,
    pub min_fractional_quantization_error: f64,
    pub max_fractional_quantization_error: f64,
    pub mean_topological_fractional_gap_mhz: f64,
    pub min_topological_fractional_gap_mhz: f64,
    pub max_topological_fractional_gap_mhz: f64,
    pub mean_braiding_visibility: f64,
    pub min_braiding_visibility: f64,
    pub max_braiding_visibility: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct ParafermionBenchmarkRunner;

impl ParafermionBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> ParafermionBenchmarkResult {
        let sweep_params: Vec<FractionalHallParafermionParams> = (0..cycles)
            .map(|i| {
                let acoustic_resonance_freq_ghz = 4.0 + 0.5 * ((i % 13) as f64 / 13.0);
                let synthetic_lorentz_coupling_mhz = 64.0 + 8.0 * ((i % 17) as f64 / 17.0);
                let fractional_filling_factor = 1.0 / 3.0;
                let parafermion_order_z_m = 3;
                let interferometer_arm_length_um = 40.0 + 8.0 * ((i % 19) as f64 / 19.0);
                let acoustic_damping_rate_khz = 1.4 + 0.4 * ((i % 23) as f64 / 23.0);
                let operating_temp_m_k = 10.0 + 2.0 * ((i % 29) as f64 / 29.0);
                let quasiparticle_tunneling_mhz = 19.0 + 4.0 * ((i % 31) as f64 / 31.0);

                FractionalHallParafermionParams::new(
                    acoustic_resonance_freq_ghz,
                    synthetic_lorentz_coupling_mhz,
                    fractional_filling_factor,
                    parafermion_order_z_m,
                    interferometer_arm_length_um,
                    acoustic_damping_rate_khz,
                    operating_temp_m_k,
                    quasiparticle_tunneling_mhz,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<FractionalHallParafermionMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = FractionalHallParafermionSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_braid_fid = 0.0;
        let mut min_braid_fid = f64::MAX;
        let mut max_braid_fid = f64::MIN;

        let mut sum_frac_fid = 0.0;
        let mut min_frac_fid = f64::MAX;
        let mut max_frac_fid = f64::MIN;

        let mut sum_quant_err = 0.0;
        let mut min_quant_err = f64::MAX;
        let mut max_quant_err = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut max_gap = f64::MIN;

        let mut sum_vis = 0.0;
        let mut min_vis = f64::MAX;
        let mut max_vis = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_braid_fid += m.braid_phase_fidelity;
            if m.braid_phase_fidelity < min_braid_fid {
                min_braid_fid = m.braid_phase_fidelity;
            }
            if m.braid_phase_fidelity > max_braid_fid {
                max_braid_fid = m.braid_phase_fidelity;
            }

            sum_frac_fid += m.fractional_state_fidelity;
            if m.fractional_state_fidelity < min_frac_fid {
                min_frac_fid = m.fractional_state_fidelity;
            }
            if m.fractional_state_fidelity > max_frac_fid {
                max_frac_fid = m.fractional_state_fidelity;
            }

            sum_quant_err += m.fractional_quantization_error;
            if m.fractional_quantization_error < min_quant_err {
                min_quant_err = m.fractional_quantization_error;
            }
            if m.fractional_quantization_error > max_quant_err {
                max_quant_err = m.fractional_quantization_error;
            }

            sum_gap += m.topological_fractional_gap_mhz;
            if m.topological_fractional_gap_mhz < min_gap {
                min_gap = m.topological_fractional_gap_mhz;
            }
            if m.topological_fractional_gap_mhz > max_gap {
                max_gap = m.topological_fractional_gap_mhz;
            }

            sum_vis += m.braiding_visibility;
            if m.braiding_visibility < min_vis {
                min_vis = m.braiding_visibility;
            }
            if m.braiding_visibility > max_vis {
                max_vis = m.braiding_visibility;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        ParafermionBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_braid_phase_fidelity: sum_braid_fid / n,
            min_braid_phase_fidelity: min_braid_fid,
            max_braid_phase_fidelity: max_braid_fid,
            mean_fractional_state_fidelity: sum_frac_fid / n,
            min_fractional_state_fidelity: min_frac_fid,
            max_fractional_state_fidelity: max_frac_fid,
            mean_fractional_quantization_error: sum_quant_err / n,
            min_fractional_quantization_error: min_quant_err,
            max_fractional_quantization_error: max_quant_err,
            mean_topological_fractional_gap_mhz: sum_gap / n,
            min_topological_fractional_gap_mhz: min_gap,
            max_topological_fractional_gap_mhz: max_gap,
            mean_braiding_visibility: sum_vis / n,
            min_braiding_visibility: min_vis,
            max_braiding_visibility: max_vis,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
