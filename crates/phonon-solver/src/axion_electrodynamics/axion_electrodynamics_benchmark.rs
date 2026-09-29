//! Multi-threaded Rayon benchmark runner for quantum axion electrodynamics,
//! haloscope SNR (>= 15.0 dB), and topological magnetoplasmon isolation (>= 25.0 dB) across 10,000 sweeps.

use super::axion_polariton_solver::AxionPolaritonSolver;
use super::magnetoplasmon_solver::TopologicalMagnetoplasmonSolver;
use phonon_models::axion_electrodynamics::{
    AxionElectrodynamicsParams, TopologicalMagnetoplasmonParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point in quantum axion electrodynamics and topological magnetoplasmons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionSweepPoint {
    pub conversion_power_dbm: f64,
    pub snr_db: f64,
    pub witten_conductance_siemens: f64,
    pub polariton_gap_ghz: f64,
    pub non_reciprocal_isolation_db: f64,
    pub forward_transmission_db: f64,
    pub backward_isolation_db: f64,
    pub soliton_charge: i32,
}

/// Comprehensive benchmark report for quantum axion electrodynamics and topological magnetoplasmons.
#[derive(Debug, Clone, PartialEq)]
pub struct AxionElectrodynamicsBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean haloscope signal-to-noise ratio in decibels ($\ge 15.0\text{ dB}$ required).
    pub mean_snr_db: f64,
    /// Minimum haloscope signal-to-noise ratio in decibels.
    pub min_snr_db: f64,
    /// Maximum haloscope signal-to-noise ratio in decibels.
    pub max_snr_db: f64,
    /// Mean non-reciprocal magnetoplasmon isolation contrast in decibels ($\ge 25.0\text{ dB}$ required).
    pub mean_isolation_db: f64,
    /// Minimum non-reciprocal isolation contrast in decibels.
    pub min_isolation_db: f64,
    /// Maximum non-reciprocal isolation contrast in decibels.
    pub max_isolation_db: f64,
    /// Mean converted signal power in $\text{dBm}$.
    pub mean_conversion_power_dbm: f64,
    /// Mean Witten anomalous Hall conductance in Siemens ($\text{S}$).
    pub mean_witten_conductance_siemens: f64,
    /// Mean axion-polariton anti-crossing gap in $\text{GHz}$.
    pub mean_polariton_gap_ghz: f64,
    /// Fraction of parameter sweeps satisfying all physical and performance bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for axion electrodynamics and magnetoplasmons.
#[derive(Debug, Clone)]
pub struct AxionElectrodynamicsBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for AxionElectrodynamicsBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl AxionElectrodynamicsBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> AxionElectrodynamicsBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<AxionSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Varied axion haloscope parameters
                let mass_uev = 12.0 + 36.0 * frac; // 12.0 to 48.0 ueV
                let b_field = 6.0 + 8.0 * pseudo_hash; // 6.0 to 14.0 T
                let q_factor = 40_000.0 + 160_000.0 * frac; // 40k to 200k
                let volume = 0.005 + 0.035 * pseudo_hash; // 0.005 to 0.040 m^3
                let t_sys = 0.08 + 0.85 * (1.0 - frac); // 0.08 to 0.93 K
                let tau = 0.4 + 3.6 * pseudo_hash; // 0.4 to 4.0 s
                let topo_enhance = 2_500.0 + 15_000.0 * frac; // 2,500 to 17,500

                let axion_params = AxionElectrodynamicsParams {
                    axion_mass_uev: mass_uev,
                    magnetic_field_tesla: b_field,
                    cavity_volume_m3: volume,
                    cavity_q_factor: q_factor,
                    system_noise_temp_k: t_sys,
                    integration_time_s: tau,
                    topological_polariton_enhancement: topo_enhance,
                    ..Default::default()
                };

                let axion_solver = AxionPolaritonSolver::new(axion_params);
                let axion_metrics = axion_solver.solve();

                // Varied topological magnetoplasmon parameters
                let dw_width = 8.0 + 16.0 * pseudo_hash; // 8.0 to 24.0 nm
                let length_um = 15.0 + 15.0 * frac; // 15.0 to 30.0 um
                let alpha_fwd = 0.02 + 0.04 * (1.0 - pseudo_hash); // 0.02 to 0.06 dB/um
                let alpha_bwd = 2.0 + 2.0 * frac; // 2.0 to 4.0 dB/um
                let dmi = 1.2 + 2.0 * pseudo_hash; // 1.2 to 3.2 mJ/m^2

                let mp_params = TopologicalMagnetoplasmonParams {
                    domain_wall_width_nm: dw_width,
                    waveguide_length_um: length_um,
                    forward_attenuation_db_per_um: alpha_fwd,
                    backward_attenuation_db_per_um: alpha_bwd,
                    chiral_dmi_energy_mj_m2: dmi,
                    ..Default::default()
                };

                let mp_solver = TopologicalMagnetoplasmonSolver::new(mp_params);
                let mp_metrics = mp_solver.solve();

                AxionSweepPoint {
                    conversion_power_dbm: axion_metrics.conversion_power_dbm,
                    snr_db: axion_metrics.snr_db,
                    witten_conductance_siemens: axion_metrics
                        .witten_anomalous_hall_conductance_siemens,
                    polariton_gap_ghz: axion_metrics.polariton_gap_ghz,
                    non_reciprocal_isolation_db: mp_metrics.non_reciprocal_isolation_db,
                    forward_transmission_db: mp_metrics.forward_transmission_db,
                    backward_isolation_db: mp_metrics.backward_isolation_db,
                    soliton_charge: mp_metrics.soliton_topological_charge,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1.0e-6);

        let mut sum_snr = 0.0;
        let mut min_snr = f64::MAX;
        let mut max_snr = f64::MIN;

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut max_iso = f64::MIN;

        let mut sum_power = 0.0;
        let mut sum_witten = 0.0;
        let mut sum_gap = 0.0;
        let mut compliant_count = 0usize;

        for pt in &results {
            sum_snr += pt.snr_db;
            if pt.snr_db < min_snr {
                min_snr = pt.snr_db;
            }
            if pt.snr_db > max_snr {
                max_snr = pt.snr_db;
            }

            sum_iso += pt.non_reciprocal_isolation_db;
            if pt.non_reciprocal_isolation_db < min_iso {
                min_iso = pt.non_reciprocal_isolation_db;
            }
            if pt.non_reciprocal_isolation_db > max_iso {
                max_iso = pt.non_reciprocal_isolation_db;
            }

            sum_power += pt.conversion_power_dbm;
            sum_witten += pt.witten_conductance_siemens;
            sum_gap += pt.polariton_gap_ghz;

            // Strict compliance criteria:
            // 1. SNR >= 15.0 dB
            // 2. Isolation contrast >= 25.0 dB
            // 3. Polariton gap > 0.0 GHz
            // 4. Witten Hall conductance > 0.0 S
            // 5. Soliton charge == 1
            if pt.snr_db >= 15.0
                && pt.non_reciprocal_isolation_db >= 25.0
                && pt.polariton_gap_ghz > 0.0
                && pt.witten_conductance_siemens > 0.0
                && pt.soliton_charge == 1
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        AxionElectrodynamicsBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_snr_db: sum_snr / n,
            min_snr_db: min_snr,
            max_snr_db: max_snr,
            mean_isolation_db: sum_iso / n,
            min_isolation_db: min_iso,
            max_isolation_db: max_iso,
            mean_conversion_power_dbm: sum_power / n,
            mean_witten_conductance_siemens: sum_witten / n,
            mean_polariton_gap_ghz: sum_gap / n,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
