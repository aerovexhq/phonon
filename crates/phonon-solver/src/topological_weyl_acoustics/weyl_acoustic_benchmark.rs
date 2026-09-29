//! Parallel parameter sweep benchmark suite for topological Weyl acoustics.

use crate::topological_weyl_acoustics::TopologicalWeylAcousticSolver;
use phonon_models::topological_weyl_acoustics::{WeylAcousticMetrics, WeylAcousticParams};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for Weyl acoustic sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeylAcousticBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_weyl_separation: f64,
    pub min_weyl_separation: f64,
    pub mean_fermi_arc_transmission: f64,
    pub min_fermi_arc_transmission: f64,
    pub mean_dislocation_purity: f64,
    pub min_dislocation_purity: f64,
    pub mean_bulk_isolation_db: f64,
    pub min_bulk_isolation_db: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct WeylAcousticBenchmarkRunner;

impl WeylAcousticBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> WeylAcousticBenchmarkResult {
        let sweep_params: Vec<WeylAcousticParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let a_um = 350.0 + 200.0 * frac; // 350 to 550 um
                let f_w = 700.0 + 300.0 * ((i % 100) as f64 / 100.0); // 700 to 1000 MHz
                let trs = 0.20 + 0.25 * ((i % 60) as f64 / 60.0); // 0.20 to 0.45
                let inv = 0.25 + 0.25 * frac; // 0.25 to 0.50
                let burgers = 1.0; // integer Burgers vector
                let disorder = 0.02 + 0.06 * ((i % 80) as f64 / 80.0); // 0.02 to 0.08
                let angle = 0.0; // (001) surface
                let q_bulk = 8.0e3 + 8.0e3 * frac; // 8e3 to 1.6e4
                let temp = 2.0 + 10.0 * ((i % 50) as f64 / 50.0); // 2 to 12 K

                WeylAcousticParams::new(
                    a_um, f_w, trs, inv, burgers, disorder, angle, q_bulk, temp,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<WeylAcousticMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = TopologicalWeylAcousticSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_sep = 0.0;
        let mut min_sep = f64::MAX;
        let mut sum_arc = 0.0;
        let mut min_arc = f64::MAX;
        let mut sum_pur = 0.0;
        let mut min_pur = f64::MAX;
        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_sep += m.weyl_point_separation_norm;
            if m.weyl_point_separation_norm < min_sep {
                min_sep = m.weyl_point_separation_norm;
            }

            sum_arc += m.fermi_arc_transmission;
            if m.fermi_arc_transmission < min_arc {
                min_arc = m.fermi_arc_transmission;
            }

            sum_pur += m.dislocation_mode_purity;
            if m.dislocation_mode_purity < min_pur {
                min_pur = m.dislocation_mode_purity;
            }

            sum_iso += m.bulk_bandgap_isolation_db;
            if m.bulk_bandgap_isolation_db < min_iso {
                min_iso = m.bulk_bandgap_isolation_db;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        WeylAcousticBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_weyl_separation: sum_sep / n,
            min_weyl_separation: min_sep,
            mean_fermi_arc_transmission: sum_arc / n,
            min_fermi_arc_transmission: min_arc,
            mean_dislocation_purity: sum_pur / n,
            min_dislocation_purity: min_pur,
            mean_bulk_isolation_db: sum_iso / n,
            min_bulk_isolation_db: min_iso,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
