#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum opto-electro-phononic
//! frequency translators across multi-threaded Rayon workers.

use crate::opto_electro_phononic_translator::OptoElectroPhononicTranslatorSolver;
use phonon_models::opto_electro_phononic_translator::{
    OptoElectroPhononicTranslatorMetrics, OptoElectroPhononicTranslatorParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for quantum opto-electro-phononic frequency translator parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TranslatorBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_transduction_efficiency: f64,
    pub min_transduction_efficiency: f64,
    pub max_transduction_efficiency: f64,
    pub mean_added_thermal_noise_quanta: f64,
    pub min_added_thermal_noise_quanta: f64,
    pub max_added_thermal_noise_quanta: f64,
    pub mean_conversion_bandwidth_mhz: f64,
    pub min_conversion_bandwidth_mhz: f64,
    pub max_conversion_bandwidth_mhz: f64,
    pub mean_quantum_state_transfer_fidelity: f64,
    pub min_quantum_state_transfer_fidelity: f64,
    pub max_quantum_state_transfer_fidelity: f64,
    pub mean_ground_state_cooling_occupancy: f64,
    pub min_ground_state_cooling_occupancy: f64,
    pub max_ground_state_cooling_occupancy: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct TranslatorBenchmarkRunner;

impl TranslatorBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> TranslatorBenchmarkResult {
        let sweep_params: Vec<OptoElectroPhononicTranslatorParams> = (0..cycles)
            .map(|i| {
                let mmwave_frequency_ghz = 40.0 + 20.0 * ((i % 31) as f64 / 31.0);
                let telecom_wavelength_nm = 1540.0 + 20.0 * ((i % 29) as f64 / 29.0);
                let piezoelectric_cooperativity = 30.0 + 20.0 * ((i % 37) as f64 / 37.0);
                let optomechanical_cooperativity = 30.0 + 20.0 * ((i % 41) as f64 / 41.0);
                let acoustic_damping_rate_mhz = 1.0 + 0.8 * ((i % 23) as f64 / 23.0);
                let optical_q_factor = 2.0e6 + 1.0e6 * ((i % 19) as f64 / 19.0);
                let operating_temp_m_k = 10.0 + 8.0 * ((i % 43) as f64 / 43.0);
                let pump_laser_power_mw = 3.5 + 3.0 * ((i % 17) as f64 / 17.0);

                OptoElectroPhononicTranslatorParams::new(
                    mmwave_frequency_ghz,
                    telecom_wavelength_nm,
                    piezoelectric_cooperativity,
                    optomechanical_cooperativity,
                    acoustic_damping_rate_mhz,
                    optical_q_factor,
                    operating_temp_m_k,
                    pump_laser_power_mw,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<OptoElectroPhononicTranslatorMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = OptoElectroPhononicTranslatorSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_eta = 0.0;
        let mut min_eta = f64::MAX;
        let mut max_eta = f64::MIN;

        let mut sum_n_add = 0.0;
        let mut min_n_add = f64::MAX;
        let mut max_n_add = f64::MIN;

        let mut sum_bw = 0.0;
        let mut min_bw = f64::MAX;
        let mut max_bw = f64::MIN;

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut max_fid = f64::MIN;

        let mut sum_n_cool = 0.0;
        let mut min_n_cool = f64::MAX;
        let mut max_n_cool = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_eta += m.transduction_efficiency;
            if m.transduction_efficiency < min_eta {
                min_eta = m.transduction_efficiency;
            }
            if m.transduction_efficiency > max_eta {
                max_eta = m.transduction_efficiency;
            }

            sum_n_add += m.added_thermal_noise_quanta;
            if m.added_thermal_noise_quanta < min_n_add {
                min_n_add = m.added_thermal_noise_quanta;
            }
            if m.added_thermal_noise_quanta > max_n_add {
                max_n_add = m.added_thermal_noise_quanta;
            }

            sum_bw += m.conversion_bandwidth_mhz;
            if m.conversion_bandwidth_mhz < min_bw {
                min_bw = m.conversion_bandwidth_mhz;
            }
            if m.conversion_bandwidth_mhz > max_bw {
                max_bw = m.conversion_bandwidth_mhz;
            }

            sum_fid += m.quantum_state_transfer_fidelity;
            if m.quantum_state_transfer_fidelity < min_fid {
                min_fid = m.quantum_state_transfer_fidelity;
            }
            if m.quantum_state_transfer_fidelity > max_fid {
                max_fid = m.quantum_state_transfer_fidelity;
            }

            sum_n_cool += m.ground_state_cooling_occupancy;
            if m.ground_state_cooling_occupancy < min_n_cool {
                min_n_cool = m.ground_state_cooling_occupancy;
            }
            if m.ground_state_cooling_occupancy > max_n_cool {
                max_n_cool = m.ground_state_cooling_occupancy;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let count = cycles.max(1) as f64;
        let compliance_fraction = (compliant_count as f64) / count;

        TranslatorBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_transduction_efficiency: sum_eta / count,
            min_transduction_efficiency: min_eta,
            max_transduction_efficiency: max_eta,
            mean_added_thermal_noise_quanta: sum_n_add / count,
            min_added_thermal_noise_quanta: min_n_add,
            max_added_thermal_noise_quanta: max_n_add,
            mean_conversion_bandwidth_mhz: sum_bw / count,
            min_conversion_bandwidth_mhz: min_bw,
            max_conversion_bandwidth_mhz: max_bw,
            mean_quantum_state_transfer_fidelity: sum_fid / count,
            min_quantum_state_transfer_fidelity: min_fid,
            max_quantum_state_transfer_fidelity: max_fid,
            mean_ground_state_cooling_occupancy: sum_n_cool / count,
            min_ground_state_cooling_occupancy: min_n_cool,
            max_ground_state_cooling_occupancy: max_n_cool,
            physical_compliance_fraction: compliance_fraction,
        }
    }
}
