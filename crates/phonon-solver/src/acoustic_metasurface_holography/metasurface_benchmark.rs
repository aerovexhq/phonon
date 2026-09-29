//! Parallel parameter sweep benchmark suite for acoustic metasurface holography.

use crate::acoustic_metasurface_holography::AcousticMetasurfaceHolographySolver;
use phonon_models::acoustic_metasurface_holography::{
    MetasurfaceHolographyMetrics, MetasurfaceHolographyParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for acoustic metasurface holography parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetasurfaceBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_beam_steering_efficiency: f64,
    pub min_beam_steering_efficiency: f64,
    pub mean_inter_channel_crosstalk_db: f64,
    pub max_inter_channel_crosstalk_db: f64,
    pub mean_reconfiguration_latency_ns: f64,
    pub max_reconfiguration_latency_ns: f64,
    pub mean_insertion_loss_db: f64,
    pub max_insertion_loss_db: f64,
    pub mean_routing_channel_fidelity: f64,
    pub min_routing_channel_fidelity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct MetasurfaceBenchmarkRunner;

impl MetasurfaceBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> MetasurfaceBenchmarkResult {
        let sweep_params: Vec<MetasurfaceHolographyParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let freq = 1.0 + 9.0 * frac; // 1.0 to 10.0 GHz
                let pitch = 0.3 + 1.2 * ((i % 50) as f64 / 50.0); // 0.3 to 1.5 um
                let elements = 32 + (i % 97); // 32 to 128 elements
                let bits = 4 + (i % 7); // 4 to 10 bits
                let voltage = 1.5 + 6.5 * frac; // 1.5 to 8.0 V
                let r_ohms = 20.0 + 80.0 * ((i % 80) as f64 / 80.0); // 20.0 to 100.0 Ohms
                let c_pf = 0.02 + 0.48 * ((i % 50) as f64 / 50.0); // 0.02 to 0.50 pF
                let loss = 0.0005 + 0.0045 * frac; // 0.0005 to 0.0050 dB/um

                MetasurfaceHolographyParams::new(
                    freq, pitch, elements, bits, voltage, r_ohms, c_pf, loss,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<MetasurfaceHolographyMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = AcousticMetasurfaceHolographySolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_eff = 0.0;
        let mut min_eff = f64::MAX;
        let mut sum_xtalk = 0.0;
        let mut max_xtalk = f64::MIN;
        let mut sum_latency = 0.0;
        let mut max_latency = f64::MIN;
        let mut sum_il = 0.0;
        let mut max_il = f64::MIN;
        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_eff += m.beam_steering_efficiency;
            if m.beam_steering_efficiency < min_eff {
                min_eff = m.beam_steering_efficiency;
            }

            sum_xtalk += m.inter_channel_crosstalk_db;
            if m.inter_channel_crosstalk_db > max_xtalk {
                max_xtalk = m.inter_channel_crosstalk_db;
            }

            sum_latency += m.reconfiguration_latency_ns;
            if m.reconfiguration_latency_ns > max_latency {
                max_latency = m.reconfiguration_latency_ns;
            }

            sum_il += m.insertion_loss_db;
            if m.insertion_loss_db > max_il {
                max_il = m.insertion_loss_db;
            }

            sum_fid += m.routing_channel_fidelity;
            if m.routing_channel_fidelity < min_fid {
                min_fid = m.routing_channel_fidelity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        MetasurfaceBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_beam_steering_efficiency: sum_eff / n,
            min_beam_steering_efficiency: min_eff,
            mean_inter_channel_crosstalk_db: sum_xtalk / n,
            max_inter_channel_crosstalk_db: max_xtalk,
            mean_reconfiguration_latency_ns: sum_latency / n,
            max_reconfiguration_latency_ns: max_latency,
            mean_insertion_loss_db: sum_il / n,
            max_insertion_loss_db: max_il,
            mean_routing_channel_fidelity: sum_fid / n,
            min_routing_channel_fidelity: min_fid,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
