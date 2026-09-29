#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for non-Abelian anyon braiding in
//! chiral acoustic Chern metamaterials across Rayon workers.

use crate::chiral_chern_anyon_braiding::ChiralChernAnyonBraidingSolver;
use phonon_models::chiral_chern_anyon_braiding::{
    ChiralChernAnyonBraidingMetrics, ChiralChernAnyonBraidingParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for chiral Chern anyon braiding parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BraidingBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_braiding_gate_fidelity: f64,
    pub min_braiding_gate_fidelity: f64,
    pub max_braiding_gate_fidelity: f64,
    pub mean_topological_protection_gap_mhz: f64,
    pub min_topological_protection_gap_mhz: f64,
    pub max_topological_protection_gap_mhz: f64,
    pub mean_anyon_collision_visibility: f64,
    pub min_anyon_collision_visibility: f64,
    pub max_anyon_collision_visibility: f64,
    pub mean_non_adiabatic_leakage_rate: f64,
    pub min_non_adiabatic_leakage_rate: f64,
    pub max_non_adiabatic_leakage_rate: f64,
    pub mean_topological_qubit_coherence_ms: f64,
    pub min_topological_qubit_coherence_ms: f64,
    pub max_topological_qubit_coherence_ms: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct BraidingBenchmarkRunner;

impl BraidingBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> BraidingBenchmarkResult {
        let sweep_params: Vec<ChiralChernAnyonBraidingParams> = (0..cycles)
            .map(|i| {
                let acoustic_center_freq_ghz = 4.0 + 1.6 * ((i % 19) as f64 / 19.0);
                let chern_bandgap_mhz = 75.0 + 15.0 * ((i % 23) as f64 / 23.0);
                let strain_modulation_amplitude_mhz = 20.0 + 5.0 * ((i % 29) as f64 / 29.0);
                let braiding_arm_length_um = 45.0 + 10.0 * ((i % 31) as f64 / 31.0);
                let anyon_wavepacket_speed_m_per_s = 3200.0 + 400.0 * ((i % 37) as f64 / 37.0);
                let acoustic_loss_rate_khz = 2.0 + 0.6 * ((i % 41) as f64 / 41.0);
                let operating_temp_m_k = 12.0 + 3.5 * ((i % 43) as f64 / 43.0);
                let anyon_type_parafermion_order = 2 + (i % 3);

                ChiralChernAnyonBraidingParams::new(
                    acoustic_center_freq_ghz,
                    chern_bandgap_mhz,
                    strain_modulation_amplitude_mhz,
                    braiding_arm_length_um,
                    anyon_wavepacket_speed_m_per_s,
                    acoustic_loss_rate_khz,
                    operating_temp_m_k,
                    anyon_type_parafermion_order,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<ChiralChernAnyonBraidingMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = ChiralChernAnyonBraidingSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut max_fid = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut max_gap = f64::MIN;

        let mut sum_vis = 0.0;
        let mut min_vis = f64::MAX;
        let mut max_vis = f64::MIN;

        let mut sum_leak = 0.0;
        let mut min_leak = f64::MAX;
        let mut max_leak = f64::MIN;

        let mut sum_coh = 0.0;
        let mut min_coh = f64::MAX;
        let mut max_coh = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fid += m.braiding_gate_fidelity;
            if m.braiding_gate_fidelity < min_fid {
                min_fid = m.braiding_gate_fidelity;
            }
            if m.braiding_gate_fidelity > max_fid {
                max_fid = m.braiding_gate_fidelity;
            }

            sum_gap += m.topological_protection_gap_mhz;
            if m.topological_protection_gap_mhz < min_gap {
                min_gap = m.topological_protection_gap_mhz;
            }
            if m.topological_protection_gap_mhz > max_gap {
                max_gap = m.topological_protection_gap_mhz;
            }

            sum_vis += m.anyon_collision_visibility;
            if m.anyon_collision_visibility < min_vis {
                min_vis = m.anyon_collision_visibility;
            }
            if m.anyon_collision_visibility > max_vis {
                max_vis = m.anyon_collision_visibility;
            }

            sum_leak += m.non_adiabatic_leakage_rate;
            if m.non_adiabatic_leakage_rate < min_leak {
                min_leak = m.non_adiabatic_leakage_rate;
            }
            if m.non_adiabatic_leakage_rate > max_leak {
                max_leak = m.non_adiabatic_leakage_rate;
            }

            sum_coh += m.topological_qubit_coherence_ms;
            if m.topological_qubit_coherence_ms < min_coh {
                min_coh = m.topological_qubit_coherence_ms;
            }
            if m.topological_qubit_coherence_ms > max_coh {
                max_coh = m.topological_qubit_coherence_ms;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let count = cycles.max(1) as f64;
        let compliance_fraction = (compliant_count as f64) / count;

        BraidingBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_braiding_gate_fidelity: sum_fid / count,
            min_braiding_gate_fidelity: min_fid,
            max_braiding_gate_fidelity: max_fid,
            mean_topological_protection_gap_mhz: sum_gap / count,
            min_topological_protection_gap_mhz: min_gap,
            max_topological_protection_gap_mhz: max_gap,
            mean_anyon_collision_visibility: sum_vis / count,
            min_anyon_collision_visibility: min_vis,
            max_anyon_collision_visibility: max_vis,
            mean_non_adiabatic_leakage_rate: sum_leak / count,
            min_non_adiabatic_leakage_rate: min_leak,
            max_non_adiabatic_leakage_rate: max_leak,
            mean_topological_qubit_coherence_ms: sum_coh / count,
            min_topological_qubit_coherence_ms: min_coh,
            max_topological_qubit_coherence_ms: max_coh,
            physical_compliance_fraction: compliance_fraction,
        }
    }
}
