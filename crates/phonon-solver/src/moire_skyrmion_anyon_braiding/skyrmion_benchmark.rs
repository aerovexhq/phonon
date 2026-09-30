#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for non-Abelian quantum acoustic anyonic
//! braiding in moire skyrmion crystals and chiral topological spin-Peierls transducers
//! across multi-threaded Rayon workers.

use crate::moire_skyrmion_anyon_braiding::MoireSkyrmionAnyonBraidingSolver;
use phonon_models::moire_skyrmion_anyon_braiding::{
    MoireSkyrmionAnyonBraidingMetrics, MoireSkyrmionAnyonBraidingParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for moire skyrmion anyon braiding parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoireSkyrmionBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_anyonic_braiding_phase_fidelity: f64,
    pub min_anyonic_braiding_phase_fidelity: f64,
    pub max_anyonic_braiding_phase_fidelity: f64,
    pub mean_topological_protection_gap_mhz: f64,
    pub min_topological_protection_gap_mhz: f64,
    pub max_topological_protection_gap_mhz: f64,
    pub mean_skyrmion_topological_stability_fraction: f64,
    pub min_skyrmion_topological_stability_fraction: f64,
    pub max_skyrmion_topological_stability_fraction: f64,
    pub mean_inter_skyrmion_crosstalk_isolation_db: f64,
    pub min_inter_skyrmion_crosstalk_isolation_db: f64,
    pub max_inter_skyrmion_crosstalk_isolation_db: f64,
    pub mean_topological_mode_dephasing_rate_hz: f64,
    pub min_topological_mode_dephasing_rate_hz: f64,
    pub max_topological_mode_dephasing_rate_hz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct MoireSkyrmionBenchmarkRunner;

impl MoireSkyrmionBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> MoireSkyrmionBenchmarkResult {
        let sweep_params: Vec<MoireSkyrmionAnyonBraidingParams> = (0..cycles)
            .map(|i| {
                let twist_angle_degrees = 0.85 + 1.60 * (((i * 7) % 50) as f64 / 50.0);
                let spin_peierls_coupling_constant =
                    0.15 + 0.78 * (((i * 13) % 45) as f64 / 45.0);
                let dzyaloshinskii_moriya_interaction_mev =
                    1.2 + 13.5 * (((i * 11) % 40) as f64 / 40.0);
                let heisenberg_exchange_coupling_j_mev =
                    6.0 + 33.0 * (((i * 23) % 35) as f64 / 35.0);
                let surface_acoustic_wave_frequency_ghz =
                    1.2 + 13.5 * (((i * 19) % 45) as f64 / 45.0);
                let cryogenic_temperature_mk =
                    2.0 + 46.0 * (((i * 31) % 40) as f64 / 40.0);
                let inter_skyrmion_pitch_nm =
                    35.0 + 260.0 * (((i * 17) % 50) as f64 / 50.0);
                let braiding_path_length_um =
                    0.6 + 7.2 * (((i * 29) % 45) as f64 / 45.0);

                MoireSkyrmionAnyonBraidingParams::new(
                    twist_angle_degrees,
                    spin_peierls_coupling_constant,
                    dzyaloshinskii_moriya_interaction_mev,
                    heisenberg_exchange_coupling_j_mev,
                    surface_acoustic_wave_frequency_ghz,
                    cryogenic_temperature_mk,
                    inter_skyrmion_pitch_nm,
                    braiding_path_length_um,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<MoireSkyrmionAnyonBraidingMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = MoireSkyrmionAnyonBraidingSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut max_gap = f64::MIN;

        let mut sum_stability = 0.0;
        let mut min_stability = f64::MAX;
        let mut max_stability = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_dephasing = 0.0;
        let mut min_dephasing = f64::MAX;
        let mut max_dephasing = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.anyonic_braiding_phase_fidelity;
            min_fidelity = min_fidelity.min(m.anyonic_braiding_phase_fidelity);
            max_fidelity = max_fidelity.max(m.anyonic_braiding_phase_fidelity);

            sum_gap += m.topological_protection_gap_mhz;
            min_gap = min_gap.min(m.topological_protection_gap_mhz);
            max_gap = max_gap.max(m.topological_protection_gap_mhz);

            sum_stability += m.skyrmion_topological_stability_fraction;
            min_stability = min_stability.min(m.skyrmion_topological_stability_fraction);
            max_stability = max_stability.max(m.skyrmion_topological_stability_fraction);

            sum_isolation += m.inter_skyrmion_crosstalk_isolation_db;
            min_isolation = min_isolation.min(m.inter_skyrmion_crosstalk_isolation_db);
            max_isolation = max_isolation.max(m.inter_skyrmion_crosstalk_isolation_db);

            sum_dephasing += m.topological_mode_dephasing_rate_hz;
            min_dephasing = min_dephasing.min(m.topological_mode_dephasing_rate_hz);
            max_dephasing = max_dephasing.max(m.topological_mode_dephasing_rate_hz);

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;

        MoireSkyrmionBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_anyonic_braiding_phase_fidelity: sum_fidelity / n,
            min_anyonic_braiding_phase_fidelity: min_fidelity,
            max_anyonic_braiding_phase_fidelity: max_fidelity,
            mean_topological_protection_gap_mhz: sum_gap / n,
            min_topological_protection_gap_mhz: min_gap,
            max_topological_protection_gap_mhz: max_gap,
            mean_skyrmion_topological_stability_fraction: sum_stability / n,
            min_skyrmion_topological_stability_fraction: min_stability,
            max_skyrmion_topological_stability_fraction: max_stability,
            mean_inter_skyrmion_crosstalk_isolation_db: sum_isolation / n,
            min_inter_skyrmion_crosstalk_isolation_db: min_isolation,
            max_inter_skyrmion_crosstalk_isolation_db: max_isolation,
            mean_topological_mode_dephasing_rate_hz: sum_dephasing / n,
            min_topological_mode_dephasing_rate_hz: min_dephasing,
            max_topological_mode_dephasing_rate_hz: max_dephasing,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
