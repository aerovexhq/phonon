#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for non-Abelian quantum acoustic Kitaev
//! spin-liquid anyon braiding and Majorana nanoresonator transceivers across multi-threaded Rayon workers.

use crate::kitaev_spin_liquid_braiding::KitaevSpinLiquidBraidingSolver;
use phonon_models::kitaev_spin_liquid_braiding::{
    KitaevSpinLiquidBraidingMetrics, KitaevSpinLiquidBraidingParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for Kitaev spin-liquid anyon braiding parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KitaevSpinLiquidBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_majorana_anyon_braiding_fidelity: f64,
    pub min_majorana_anyon_braiding_fidelity: f64,
    pub max_majorana_anyon_braiding_fidelity: f64,
    pub mean_topological_gap_protection_mhz: f64,
    pub min_topological_gap_protection_mhz: f64,
    pub max_topological_gap_protection_mhz: f64,
    pub mean_non_abelian_state_leakage: f64,
    pub min_non_abelian_state_leakage: f64,
    pub max_non_abelian_state_leakage: f64,
    pub mean_inter_qubit_crosstalk_isolation_db: f64,
    pub min_inter_qubit_crosstalk_isolation_db: f64,
    pub max_inter_qubit_crosstalk_isolation_db: f64,
    pub mean_chiral_edge_energy_flux_uw_per_m2: f64,
    pub min_chiral_edge_energy_flux_uw_per_m2: f64,
    pub max_chiral_edge_energy_flux_uw_per_m2: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct KitaevSpinLiquidBenchmarkRunner;

impl KitaevSpinLiquidBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> KitaevSpinLiquidBenchmarkResult {
        let sweep_params: Vec<KitaevSpinLiquidBraidingParams> = (0..cycles)
            .map(|i| {
                let kitaev_exchange_coupling_j_mev = 1.0 + 23.0 * (((i * 7) % 50) as f64 / 50.0);
                let strain_gauge_coupling_lambda = 0.15 + 0.75 * (((i * 13) % 45) as f64 / 45.0);
                let external_magnetic_field_tesla = 1.0 + 10.5 * (((i * 11) % 40) as f64 / 40.0);
                let braiding_operation_time_ns = 20.0 + 450.0 * (((i * 17) % 35) as f64 / 35.0);
                let nanoresonator_frequency_ghz = 1.5 + 13.0 * (((i * 19) % 45) as f64 / 45.0);
                let cryogenic_temperature_mk = 2.0 + 45.0 * (((i * 23) % 30) as f64 / 30.0);
                let inter_qubit_separation_um = 0.8 + 8.8 * (((i * 29) % 32) as f64 / 32.0);
                let non_abelian_quasiparticle_density_per_um2 =
                    0.02 + 0.95 * (((i * 31) % 25) as f64 / 25.0);

                KitaevSpinLiquidBraidingParams::new(
                    kitaev_exchange_coupling_j_mev,
                    strain_gauge_coupling_lambda,
                    external_magnetic_field_tesla,
                    braiding_operation_time_ns,
                    nanoresonator_frequency_ghz,
                    cryogenic_temperature_mk,
                    inter_qubit_separation_um,
                    non_abelian_quasiparticle_density_per_um2,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<KitaevSpinLiquidBraidingMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = KitaevSpinLiquidBraidingSolver::new(*p);
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

        let mut sum_leakage = 0.0;
        let mut min_leakage = f64::MAX;
        let mut max_leakage = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_flux = 0.0;
        let mut min_flux = f64::MAX;
        let mut max_flux = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.majorana_anyon_braiding_fidelity;
            if m.majorana_anyon_braiding_fidelity < min_fidelity {
                min_fidelity = m.majorana_anyon_braiding_fidelity;
            }
            if m.majorana_anyon_braiding_fidelity > max_fidelity {
                max_fidelity = m.majorana_anyon_braiding_fidelity;
            }

            sum_gap += m.topological_gap_protection_mhz;
            if m.topological_gap_protection_mhz < min_gap {
                min_gap = m.topological_gap_protection_mhz;
            }
            if m.topological_gap_protection_mhz > max_gap {
                max_gap = m.topological_gap_protection_mhz;
            }

            sum_leakage += m.non_abelian_state_leakage;
            if m.non_abelian_state_leakage < min_leakage {
                min_leakage = m.non_abelian_state_leakage;
            }
            if m.non_abelian_state_leakage > max_leakage {
                max_leakage = m.non_abelian_state_leakage;
            }

            sum_isolation += m.inter_qubit_crosstalk_isolation_db;
            if m.inter_qubit_crosstalk_isolation_db < min_isolation {
                min_isolation = m.inter_qubit_crosstalk_isolation_db;
            }
            if m.inter_qubit_crosstalk_isolation_db > max_isolation {
                max_isolation = m.inter_qubit_crosstalk_isolation_db;
            }

            sum_flux += m.chiral_edge_energy_flux_uw_per_m2;
            if m.chiral_edge_energy_flux_uw_per_m2 < min_flux {
                min_flux = m.chiral_edge_energy_flux_uw_per_m2;
            }
            if m.chiral_edge_energy_flux_uw_per_m2 > max_flux {
                max_flux = m.chiral_edge_energy_flux_uw_per_m2;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        let physical_compliance_fraction = (compliant_count as f64) / n;

        KitaevSpinLiquidBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_majorana_anyon_braiding_fidelity: sum_fidelity / n,
            min_majorana_anyon_braiding_fidelity: min_fidelity,
            max_majorana_anyon_braiding_fidelity: max_fidelity,
            mean_topological_gap_protection_mhz: sum_gap / n,
            min_topological_gap_protection_mhz: min_gap,
            max_topological_gap_protection_mhz: max_gap,
            mean_non_abelian_state_leakage: sum_leakage / n,
            min_non_abelian_state_leakage: min_leakage,
            max_non_abelian_state_leakage: max_leakage,
            mean_inter_qubit_crosstalk_isolation_db: sum_isolation / n,
            min_inter_qubit_crosstalk_isolation_db: min_isolation,
            max_inter_qubit_crosstalk_isolation_db: max_isolation,
            mean_chiral_edge_energy_flux_uw_per_m2: sum_flux / n,
            min_chiral_edge_energy_flux_uw_per_m2: min_flux,
            max_chiral_edge_energy_flux_uw_per_m2: max_flux,
            physical_compliance_fraction,
        }
    }
}
