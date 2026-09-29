#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for coherent quantum
//! phonon-magnon-polariton transducers and chiral spin-acoustic interfaces
//! across multi-threaded Rayon workers.

use crate::phonon_magnon_polariton::PhononMagnonPolaritonSolver;
use phonon_models::phonon_magnon_polariton::{
    PhononMagnonPolaritonMetrics, PhononMagnonPolaritonParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for phonon-magnon-polariton parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_polariton_cooperativity: f64,
    pub min_polariton_cooperativity: f64,
    pub max_polariton_cooperativity: f64,
    pub mean_bidirectional_transduction_efficiency: f64,
    pub min_bidirectional_transduction_efficiency: f64,
    pub max_bidirectional_transduction_efficiency: f64,
    pub mean_spin_wave_dephasing_rate_mhz: f64,
    pub min_spin_wave_dephasing_rate_mhz: f64,
    pub max_spin_wave_dephasing_rate_mhz: f64,
    pub mean_chiral_isolation_db: f64,
    pub min_chiral_isolation_db: f64,
    pub max_chiral_isolation_db: f64,
    pub mean_single_quantum_conversion_fidelity: f64,
    pub min_single_quantum_conversion_fidelity: f64,
    pub max_single_quantum_conversion_fidelity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct PolaritonBenchmarkRunner;

impl PolaritonBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> PolaritonBenchmarkResult {
        let sweep_params: Vec<PhononMagnonPolaritonParams> = (0..cycles)
            .map(|i| {
                let spin_wave_frequency_ghz = 7.5 + 2.0 * ((i % 31) as f64 / 31.0);
                let acoustic_frequency_ghz = 7.5 + 2.0 * ((i % 29) as f64 / 29.0);
                let magnetoelastic_coupling_mhz = 62.0 + 20.0 * ((i % 41) as f64 / 41.0);
                let yig_film_thickness_nm = 80.0 + 40.0 * ((i % 23) as f64 / 23.0);
                let piezo_acoustic_loss_mhz = 0.38 + 0.08 * ((i % 27) as f64 / 27.0);
                let magnon_damping_alpha = 1.2e-4 + 0.3e-4 * ((i % 37) as f64 / 37.0);
                let chiral_asymmetry_factor = 0.82 + 0.12 * ((i % 33) as f64 / 33.0);
                let operating_temp_m_k = 16.0 + 6.0 * ((i % 43) as f64 / 43.0);

                PhononMagnonPolaritonParams::new(
                    spin_wave_frequency_ghz,
                    acoustic_frequency_ghz,
                    magnetoelastic_coupling_mhz,
                    yig_film_thickness_nm,
                    piezo_acoustic_loss_mhz,
                    magnon_damping_alpha,
                    chiral_asymmetry_factor,
                    operating_temp_m_k,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<PhononMagnonPolaritonMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = PhononMagnonPolaritonSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_coop = 0.0;
        let mut min_coop = f64::MAX;
        let mut max_coop = f64::MIN;

        let mut sum_eff = 0.0;
        let mut min_eff = f64::MAX;
        let mut max_eff = f64::MIN;

        let mut sum_deph = 0.0;
        let mut min_deph = f64::MAX;
        let mut max_deph = f64::MIN;

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut max_iso = f64::MIN;

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut max_fid = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_coop += m.polariton_cooperativity;
            if m.polariton_cooperativity < min_coop {
                min_coop = m.polariton_cooperativity;
            }
            if m.polariton_cooperativity > max_coop {
                max_coop = m.polariton_cooperativity;
            }

            sum_eff += m.bidirectional_transduction_efficiency;
            if m.bidirectional_transduction_efficiency < min_eff {
                min_eff = m.bidirectional_transduction_efficiency;
            }
            if m.bidirectional_transduction_efficiency > max_eff {
                max_eff = m.bidirectional_transduction_efficiency;
            }

            sum_deph += m.spin_wave_dephasing_rate_mhz;
            if m.spin_wave_dephasing_rate_mhz < min_deph {
                min_deph = m.spin_wave_dephasing_rate_mhz;
            }
            if m.spin_wave_dephasing_rate_mhz > max_deph {
                max_deph = m.spin_wave_dephasing_rate_mhz;
            }

            sum_iso += m.chiral_isolation_db;
            if m.chiral_isolation_db < min_iso {
                min_iso = m.chiral_isolation_db;
            }
            if m.chiral_isolation_db > max_iso {
                max_iso = m.chiral_isolation_db;
            }

            sum_fid += m.single_quantum_conversion_fidelity;
            if m.single_quantum_conversion_fidelity < min_fid {
                min_fid = m.single_quantum_conversion_fidelity;
            }
            if m.single_quantum_conversion_fidelity > max_fid {
                max_fid = m.single_quantum_conversion_fidelity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let count = cycles.max(1) as f64;
        let compliance_fraction = (compliant_count as f64) / count;

        PolaritonBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_polariton_cooperativity: sum_coop / count,
            min_polariton_cooperativity: min_coop,
            max_polariton_cooperativity: max_coop,
            mean_bidirectional_transduction_efficiency: sum_eff / count,
            min_bidirectional_transduction_efficiency: min_eff,
            max_bidirectional_transduction_efficiency: max_eff,
            mean_spin_wave_dephasing_rate_mhz: sum_deph / count,
            min_spin_wave_dephasing_rate_mhz: min_deph,
            max_spin_wave_dephasing_rate_mhz: max_deph,
            mean_chiral_isolation_db: sum_iso / count,
            min_chiral_isolation_db: min_iso,
            max_chiral_isolation_db: max_iso,
            mean_single_quantum_conversion_fidelity: sum_fid / count,
            min_single_quantum_conversion_fidelity: min_fid,
            max_single_quantum_conversion_fidelity: max_fid,
            physical_compliance_fraction: compliance_fraction,
        }
    }
}
