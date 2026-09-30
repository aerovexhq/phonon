#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for topological acoustic fracton dynamics
//! and sub-system symmetry-protected phononic multipole routers across multi-threaded Rayon workers.

use crate::topological_acoustic_fracton::TopologicalAcousticFractonSolver;
use phonon_models::topological_acoustic_fracton::{
    TopologicalAcousticFractonMetrics, TopologicalAcousticFractonParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for topological acoustic fracton parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalAcousticFractonBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_fracton_confinement_fidelity: f64,
    pub min_fracton_confinement_fidelity: f64,
    pub max_fracton_confinement_fidelity: f64,
    pub mean_sub_dimensional_edge_channel_isolation_db: f64,
    pub min_sub_dimensional_edge_channel_isolation_db: f64,
    pub max_sub_dimensional_edge_channel_isolation_db: f64,
    pub mean_multipole_charge_conservation_error: f64,
    pub min_multipole_charge_conservation_error: f64,
    pub max_multipole_charge_conservation_error: f64,
    pub mean_fracton_diffusion_dephasing_rate_hz: f64,
    pub min_fracton_diffusion_dephasing_rate_hz: f64,
    pub max_fracton_diffusion_dephasing_rate_hz: f64,
    pub mean_sub_system_boundary_mode_purity: f64,
    pub min_sub_system_boundary_mode_purity: f64,
    pub max_sub_system_boundary_mode_purity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct TopologicalAcousticFractonBenchmarkRunner;

impl TopologicalAcousticFractonBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> TopologicalAcousticFractonBenchmarkResult {
        let sweep_params: Vec<TopologicalAcousticFractonParams> = (0..cycles)
            .map(|i| {
                let higher_rank_gauge_coupling_g =
                    0.20 + 4.70 * (((i * 7) % 50) as f64 / 50.0);
                let sub_dimensional_lattice_constant_nm =
                    60.0 + 420.0 * (((i * 13) % 45) as f64 / 45.0);
                let acoustic_phonon_frequency_ghz =
                    1.2 + 13.5 * (((i * 11) % 40) as f64 / 40.0);
                let multipole_moment_order =
                    1.0 + 3.0 * (((i * 17) % 35) as f64 / 35.0);
                let cryogenic_temperature_mk =
                    2.0 + 45.0 * (((i * 19) % 45) as f64 / 45.0);
                let sub_system_layer_count =
                    6.0 + 56.0 * (((i * 23) % 30) as f64 / 30.0);
                let fracton_pinning_potential_mev =
                    0.8 + 18.5 * (((i * 29) % 32) as f64 / 32.0);
                let inter_router_separation_um =
                    0.8 + 10.8 * (((i * 31) % 25) as f64 / 25.0);

                TopologicalAcousticFractonParams::new(
                    higher_rank_gauge_coupling_g,
                    sub_dimensional_lattice_constant_nm,
                    acoustic_phonon_frequency_ghz,
                    multipole_moment_order,
                    cryogenic_temperature_mk,
                    sub_system_layer_count,
                    fracton_pinning_potential_mev,
                    inter_router_separation_um,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<TopologicalAcousticFractonMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = TopologicalAcousticFractonSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_error = 0.0;
        let mut min_error = f64::MAX;
        let mut max_error = f64::MIN;

        let mut sum_rate = 0.0;
        let mut min_rate = f64::MAX;
        let mut max_rate = f64::MIN;

        let mut sum_purity = 0.0;
        let mut min_purity = f64::MAX;
        let mut max_purity = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.fracton_confinement_fidelity;
            if m.fracton_confinement_fidelity < min_fidelity {
                min_fidelity = m.fracton_confinement_fidelity;
            }
            if m.fracton_confinement_fidelity > max_fidelity {
                max_fidelity = m.fracton_confinement_fidelity;
            }

            sum_isolation += m.sub_dimensional_edge_channel_isolation_db;
            if m.sub_dimensional_edge_channel_isolation_db < min_isolation {
                min_isolation = m.sub_dimensional_edge_channel_isolation_db;
            }
            if m.sub_dimensional_edge_channel_isolation_db > max_isolation {
                max_isolation = m.sub_dimensional_edge_channel_isolation_db;
            }

            sum_error += m.multipole_charge_conservation_error;
            if m.multipole_charge_conservation_error < min_error {
                min_error = m.multipole_charge_conservation_error;
            }
            if m.multipole_charge_conservation_error > max_error {
                max_error = m.multipole_charge_conservation_error;
            }

            sum_rate += m.fracton_diffusion_dephasing_rate_hz;
            if m.fracton_diffusion_dephasing_rate_hz < min_rate {
                min_rate = m.fracton_diffusion_dephasing_rate_hz;
            }
            if m.fracton_diffusion_dephasing_rate_hz > max_rate {
                max_rate = m.fracton_diffusion_dephasing_rate_hz;
            }

            sum_purity += m.sub_system_boundary_mode_purity;
            if m.sub_system_boundary_mode_purity < min_purity {
                min_purity = m.sub_system_boundary_mode_purity;
            }
            if m.sub_system_boundary_mode_purity > max_purity {
                max_purity = m.sub_system_boundary_mode_purity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        TopologicalAcousticFractonBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_fracton_confinement_fidelity: sum_fidelity / n,
            min_fracton_confinement_fidelity: min_fidelity,
            max_fracton_confinement_fidelity: max_fidelity,
            mean_sub_dimensional_edge_channel_isolation_db: sum_isolation / n,
            min_sub_dimensional_edge_channel_isolation_db: min_isolation,
            max_sub_dimensional_edge_channel_isolation_db: max_isolation,
            mean_multipole_charge_conservation_error: sum_error / n,
            min_multipole_charge_conservation_error: min_error,
            max_multipole_charge_conservation_error: max_error,
            mean_fracton_diffusion_dephasing_rate_hz: sum_rate / n,
            min_fracton_diffusion_dephasing_rate_hz: min_rate,
            max_fracton_diffusion_dephasing_rate_hz: max_rate,
            mean_sub_system_boundary_mode_purity: sum_purity / n,
            min_sub_system_boundary_mode_purity: min_purity,
            max_sub_system_boundary_mode_purity: max_purity,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
