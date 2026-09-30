#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for chiral acoustic quantum Hall metamaterials
//! and non-Abelian Moore-Read Pfaffian edge waveguide synthesizers across multi-threaded Rayon workers.

use crate::chiral_quantum_hall_pfaffian::ChiralQuantumHallPfaffianSolver;
use phonon_models::chiral_quantum_hall_pfaffian::{
    ChiralQuantumHallPfaffianMetrics, ChiralQuantumHallPfaffianParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for chiral quantum Hall Pfaffian synthesizer parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PfaffianQuantumHallBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_pfaffian_topological_state_fidelity: f64,
    pub min_pfaffian_topological_state_fidelity: f64,
    pub max_pfaffian_topological_state_fidelity: f64,
    pub mean_edge_channel_isolation_db: f64,
    pub min_edge_channel_isolation_db: f64,
    pub max_edge_channel_isolation_db: f64,
    pub mean_neutral_mode_transmission_speed_mps: f64,
    pub min_neutral_mode_transmission_speed_mps: f64,
    pub max_neutral_mode_transmission_speed_mps: f64,
    pub mean_thermal_hall_quantization_error: f64,
    pub min_thermal_hall_quantization_error: f64,
    pub max_thermal_hall_quantization_error: f64,
    pub mean_quasiparticle_braiding_visibility: f64,
    pub min_quasiparticle_braiding_visibility: f64,
    pub max_quasiparticle_braiding_visibility: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct PfaffianQuantumHallBenchmarkRunner;

impl PfaffianQuantumHallBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> PfaffianQuantumHallBenchmarkResult {
        let sweep_params: Vec<ChiralQuantumHallPfaffianParams> = (0..cycles)
            .map(|i| {
                let magnetic_field_tesla = 3.0 + 14.0 * (((i * 7) % 50) as f64 / 50.0);
                let fractional_filling_factor = 2.0 + 0.70 * (((i * 13) % 45) as f64 / 45.0);
                let pfaffian_pairing_gap_mhz = 8.0 + 50.0 * (((i * 11) % 40) as f64 / 40.0);
                let piezoelectric_acoustic_coupling_efficiency =
                    0.55 + 0.42 * (((i * 17) % 35) as f64 / 35.0);
                let waveguide_channel_length_um = 2.0 + 22.0 * (((i * 19) % 45) as f64 / 45.0);
                let cryogenic_temperature_mk = 2.0 + 45.0 * (((i * 23) % 30) as f64 / 30.0);
                let inter_edge_spacing_nm = 70.0 + 400.0 * (((i * 29) % 32) as f64 / 32.0);
                let acoustic_driving_frequency_ghz = 1.5 + 10.0 * (((i * 31) % 25) as f64 / 25.0);

                ChiralQuantumHallPfaffianParams::new(
                    magnetic_field_tesla,
                    fractional_filling_factor,
                    pfaffian_pairing_gap_mhz,
                    piezoelectric_acoustic_coupling_efficiency,
                    waveguide_channel_length_um,
                    cryogenic_temperature_mk,
                    inter_edge_spacing_nm,
                    acoustic_driving_frequency_ghz,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<ChiralQuantumHallPfaffianMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = ChiralQuantumHallPfaffianSolver::new(*p);
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

        let mut sum_speed = 0.0;
        let mut min_speed = f64::MAX;
        let mut max_speed = f64::MIN;

        let mut sum_error = 0.0;
        let mut min_error = f64::MAX;
        let mut max_error = f64::MIN;

        let mut sum_visibility = 0.0;
        let mut min_visibility = f64::MAX;
        let mut max_visibility = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.pfaffian_topological_state_fidelity;
            if m.pfaffian_topological_state_fidelity < min_fidelity {
                min_fidelity = m.pfaffian_topological_state_fidelity;
            }
            if m.pfaffian_topological_state_fidelity > max_fidelity {
                max_fidelity = m.pfaffian_topological_state_fidelity;
            }

            sum_isolation += m.edge_channel_isolation_db;
            if m.edge_channel_isolation_db < min_isolation {
                min_isolation = m.edge_channel_isolation_db;
            }
            if m.edge_channel_isolation_db > max_isolation {
                max_isolation = m.edge_channel_isolation_db;
            }

            sum_speed += m.neutral_mode_transmission_speed_mps;
            if m.neutral_mode_transmission_speed_mps < min_speed {
                min_speed = m.neutral_mode_transmission_speed_mps;
            }
            if m.neutral_mode_transmission_speed_mps > max_speed {
                max_speed = m.neutral_mode_transmission_speed_mps;
            }

            sum_error += m.thermal_hall_quantization_error;
            if m.thermal_hall_quantization_error < min_error {
                min_error = m.thermal_hall_quantization_error;
            }
            if m.thermal_hall_quantization_error > max_error {
                max_error = m.thermal_hall_quantization_error;
            }

            sum_visibility += m.quasiparticle_braiding_visibility;
            if m.quasiparticle_braiding_visibility < min_visibility {
                min_visibility = m.quasiparticle_braiding_visibility;
            }
            if m.quasiparticle_braiding_visibility > max_visibility {
                max_visibility = m.quasiparticle_braiding_visibility;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        let physical_compliance_fraction = (compliant_count as f64) / n;

        PfaffianQuantumHallBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_pfaffian_topological_state_fidelity: sum_fidelity / n,
            min_pfaffian_topological_state_fidelity: min_fidelity,
            max_pfaffian_topological_state_fidelity: max_fidelity,
            mean_edge_channel_isolation_db: sum_isolation / n,
            min_edge_channel_isolation_db: min_isolation,
            max_edge_channel_isolation_db: max_isolation,
            mean_neutral_mode_transmission_speed_mps: sum_speed / n,
            min_neutral_mode_transmission_speed_mps: min_speed,
            max_neutral_mode_transmission_speed_mps: max_speed,
            mean_thermal_hall_quantization_error: sum_error / n,
            min_thermal_hall_quantization_error: min_error,
            max_thermal_hall_quantization_error: max_error,
            mean_quasiparticle_braiding_visibility: sum_visibility / n,
            min_quasiparticle_braiding_visibility: min_visibility,
            max_quasiparticle_braiding_visibility: max_visibility,
            physical_compliance_fraction,
        }
    }
}
