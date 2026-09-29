//! Parallel parameter sweep benchmark suite for quantum phonon-mediated state transfer.

use crate::quantum_phonon_teleportation::PhononTeleportationSolver;
use phonon_models::quantum_phonon_teleportation::{
    PhononTeleportationMetrics, PhononTeleportationParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for quantum phonon teleportation sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononTeleportationBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_fidelity: f64,
    pub min_fidelity: f64,
    pub mean_concurrence: f64,
    pub min_concurrence: f64,
    pub mean_phonon_loss: f64,
    pub max_phonon_loss: f64,
    pub mean_bandwidth_mhz: f64,
    pub min_bandwidth_mhz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct PhononTeleportationBenchmarkRunner;

impl PhononTeleportationBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> PhononTeleportationBenchmarkResult {
        let sweep_params: Vec<PhononTeleportationParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let f_q = 3.5 + 3.0 * frac; // 3.5 to 6.5 GHz
                let alpha = -300.0 + 100.0 * (1.0 - frac); // -300 to -200 MHz
                let t1 = 45.0 + 50.0 * ((i % 100) as f64 / 100.0); // 45 to 95 us
                let tphi = 35.0 + 40.0 * ((i % 120) as f64 / 120.0); // 35 to 75 us
                let g = 25.0 + 15.0 * ((i % 80) as f64 / 80.0); // 25 to 40 MHz
                let v_saw = 3488.0 + 100.0 * (frac - 0.5); // 3438 to 3538 m/s
                let link_um = 150.0 + 350.0 * ((i % 150) as f64 / 150.0); // 150 to 500 um
                let atten = 0.15 + 0.25 * frac; // 0.15 to 0.40 dB/cm
                let temp_mk = 10.0 + 50.0 * ((i % 50) as f64 / 50.0); // 10 to 60 mK
                let shaping = 0.990 + 0.009 * (1.0 - frac); // 0.990 to 0.999
                let idt_bw = 55.0 + 30.0 * ((i % 60) as f64 / 60.0); // 55 to 85 MHz

                PhononTeleportationParams::new(
                    f_q, alpha, t1, tphi, g, v_saw, link_um, atten, temp_mk, shaping, idt_bw,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<PhononTeleportationMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = PhononTeleportationSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut sum_conc = 0.0;
        let mut min_conc = f64::MAX;
        let mut sum_loss = 0.0;
        let mut max_loss = f64::MIN;
        let mut sum_bw = 0.0;
        let mut min_bw = f64::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_fid += m.state_transfer_fidelity;
            if m.state_transfer_fidelity < min_fid {
                min_fid = m.state_transfer_fidelity;
            }

            sum_conc += m.acoustic_bell_concurrence;
            if m.acoustic_bell_concurrence < min_conc {
                min_conc = m.acoustic_bell_concurrence;
            }

            sum_loss += m.phonon_loss_probability;
            if m.phonon_loss_probability > max_loss {
                max_loss = m.phonon_loss_probability;
            }

            sum_bw += m.quantum_link_bandwidth_mhz;
            if m.quantum_link_bandwidth_mhz < min_bw {
                min_bw = m.quantum_link_bandwidth_mhz;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        PhononTeleportationBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_fidelity: sum_fid / n,
            min_fidelity: min_fid,
            mean_concurrence: sum_conc / n,
            min_concurrence: min_conc,
            mean_phonon_loss: sum_loss / n,
            max_phonon_loss: max_loss,
            mean_bandwidth_mhz: sum_bw / n,
            min_bandwidth_mhz: min_bw,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
