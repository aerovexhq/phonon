//! Parallel parameter sweep benchmark suite for quantum acoustic waveguide QED.

use crate::quantum_acoustic_waveguide::QuantumAcousticWaveguideSolver;
use phonon_models::quantum_acoustic_waveguide::{WaveguideQedMetrics, WaveguideQedParams};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for waveguide QED sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveguideQedBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_directionality: f64,
    pub min_directionality: f64,
    pub mean_purcell_factor: f64,
    pub min_purcell_factor: f64,
    pub mean_lifetime_extension: f64,
    pub min_lifetime_extension: f64,
    pub mean_concurrence: f64,
    pub min_concurrence: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct WaveguideQedBenchmarkRunner;

impl WaveguideQedBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> WaveguideQedBenchmarkResult {
        let sweep_params: Vec<WaveguideQedParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let f_a = 4.0 + 1.5 * frac; // 4.0 to 5.5 GHz
                let f_bg = f_a; // resonance
                let bg_w = 0.15 + 0.10 * frac; // 0.15 to 0.25
                let v_saw = 3488.0 + 50.0 * (frac - 0.5); // LiNbO3 acoustic velocity
                let d_um = 3.5 + 0.8 * ((i % 50) as f64 / 50.0); // 3.5 to 4.3 um
                let fingers = 10 + (i % 8); // 10 to 17 fingers
                let gamma_0 = 6.0 + 4.0 * ((i % 60) as f64 / 60.0); // 6 to 10 MHz
                let phase = std::f64::consts::FRAC_PI_2 + 0.05 * (frac - 0.5); // near pi/2
                let l_um = 120.0 + 100.0 * ((i % 80) as f64 / 80.0); // 120 to 220 um
                let t1 = 60.0 + 40.0 * ((i % 70) as f64 / 70.0); // 60 to 100 us
                let tphi = 50.0 + 40.0 * ((i % 90) as f64 / 90.0); // 50 to 90 us
                let temp = 10.0 + 20.0 * ((i % 40) as f64 / 40.0); // 10 to 30 mK

                WaveguideQedParams::new(
                    f_a, f_bg, bg_w, v_saw, d_um, fingers, gamma_0, phase, l_um, t1, tphi, temp,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<WaveguideQedMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = QuantumAcousticWaveguideSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_dir = 0.0;
        let mut min_dir = f64::MAX;
        let mut sum_pur = 0.0;
        let mut min_pur = f64::MAX;
        let mut sum_life = 0.0;
        let mut min_life = f64::MAX;
        let mut sum_conc = 0.0;
        let mut min_conc = f64::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_dir += m.chiral_acoustic_directionality;
            if m.chiral_acoustic_directionality < min_dir {
                min_dir = m.chiral_acoustic_directionality;
            }

            sum_pur += m.waveguide_purcell_factor;
            if m.waveguide_purcell_factor < min_pur {
                min_pur = m.waveguide_purcell_factor;
            }

            sum_life += m.bound_state_lifetime_extension;
            if m.bound_state_lifetime_extension < min_life {
                min_life = m.bound_state_lifetime_extension;
            }

            sum_conc += m.acoustic_entanglement_concurrence;
            if m.acoustic_entanglement_concurrence < min_conc {
                min_conc = m.acoustic_entanglement_concurrence;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        WaveguideQedBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_directionality: sum_dir / n,
            min_directionality: min_dir,
            mean_purcell_factor: sum_pur / n,
            min_purcell_factor: min_pur,
            mean_lifetime_extension: sum_life / n,
            min_lifetime_extension: min_life,
            mean_concurrence: sum_conc / n,
            min_concurrence: min_conc,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
