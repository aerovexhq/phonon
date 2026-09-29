//! Multi-threaded Rayon benchmark runner for chiral skyrmion-phonon drag,
//! topological Hall acoustics, and racetrack logic across 10,000 parameter sweeps.

use super::thiele_skyrmion_solver::SkyrmionPhononSolver;
use phonon_models::skyrmion_phonon_drag::SkyrmionPhononParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for skyrmion dynamics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionSweepPoint {
    pub drift_velocity_m_s: f64,
    pub hall_angle_deg: f64,
    pub drag_force_pn: f64,
    pub logic_contrast_db: f64,
    pub circulator_isolation_db: f64,
    pub energy_bit_fj: f64,
    pub stability_factor: f64,
}

/// Comprehensive benchmark report for chiral skyrmion-phonon drag systems.
#[derive(Debug, Clone, PartialEq)]
pub struct SkyrmionPhononBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean skyrmion drift velocity in $\text{m/s}$ ($> 100.0\text{ m/s}$ required).
    pub mean_drift_velocity_m_s: f64,
    /// Minimum skyrmion drift velocity in $\text{m/s}$.
    pub min_drift_velocity_m_s: f64,
    /// Maximum skyrmion drift velocity in $\text{m/s}$.
    pub max_drift_velocity_m_s: f64,
    /// Mean topological Hall deflection angle in degrees ($10.0^\circ - 70.0^\circ$).
    pub mean_hall_angle_deg: f64,
    /// Minimum topological Hall angle in degrees.
    pub min_hall_angle_deg: f64,
    /// Maximum topological Hall angle in degrees.
    pub max_hall_angle_deg: f64,
    /// Mean acoustic drag force in piconewtons.
    pub mean_drag_force_pn: f64,
    /// Mean logic switching contrast in decibels ($\ge 25.0\text{ dB}$ required).
    pub mean_logic_contrast_db: f64,
    /// Minimum logic switching contrast in decibels.
    pub min_logic_contrast_db: f64,
    /// Mean circulator isolation in decibels ($\ge 20.0\text{ dB}$ required).
    pub mean_circulator_isolation_db: f64,
    /// Minimum circulator isolation in decibels.
    pub min_circulator_isolation_db: f64,
    /// Mean energy dissipation per shift bit in femtojoules ($< 1.0\text{ fJ}$ required).
    pub mean_energy_bit_fj: f64,
    /// Maximum energy dissipation per shift bit in femtojoules.
    pub max_energy_bit_fj: f64,
    /// Mean topological thermal stability factor ($\ge 1.0$ required).
    pub mean_stability_factor: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for skyrmion-phonon drag.
#[derive(Debug, Clone)]
pub struct SkyrmionPhononBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for SkyrmionPhononBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl SkyrmionPhononBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> SkyrmionPhononBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<SkyrmionSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let radius = 10.0 + 18.0 * frac; // 10 to 28 nm
                let alpha = 0.012 + 0.028 * pseudo_hash; // 0.012 to 0.040
                let b_mpa = 6.0 + 12.0 * frac; // 6 to 18 MPa
                let strain_ppm = 300.0 + 600.0 * pseudo_hash; // 300 to 900 ppm
                let freq = 1.2 + 2.8 * frac; // 1.2 to 4.0 GHz
                let v_s = 3400.0 + 600.0 * pseudo_hash; // 3400 to 4000 m/s
                let temp = 50.0 + 270.0 * frac; // 50 to 320 K
                let eta_conf = 0.90 + 0.08 * pseudo_hash; // 0.90 to 0.98

                let params = SkyrmionPhononParams {
                    skyrmion_radius_nm: radius,
                    topological_charge_q: 1,
                    gilbert_damping_alpha: alpha,
                    saturation_magnetization_ma_m: 0.85,
                    film_thickness_nm: 1.5,
                    magnetoelastic_coupling_b_mpa: b_mpa,
                    saw_strain_amplitude_ppm: strain_ppm,
                    saw_frequency_ghz: freq,
                    sound_velocity_m_s: v_s,
                    temperature_k: temp,
                    racetrack_confinement_factor: eta_conf,
                };

                let solver = SkyrmionPhononSolver::new(params);
                let m = solver.solve();

                SkyrmionSweepPoint {
                    drift_velocity_m_s: m.skyrmion_drift_velocity_m_s,
                    hall_angle_deg: m.topological_hall_angle_deg,
                    drag_force_pn: m.acoustic_drag_force_pn,
                    logic_contrast_db: m.logic_switching_contrast_db,
                    circulator_isolation_db: m.circulator_isolation_db,
                    energy_bit_fj: m.energy_dissipation_per_bit_fj,
                    stability_factor: m.topological_stability_factor,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1.0e-6);

        let mut sum_v = 0.0;
        let mut min_v = f64::MAX;
        let mut max_v = f64::MIN;

        let mut sum_hall = 0.0;
        let mut min_hall = f64::MAX;
        let mut max_hall = f64::MIN;

        let mut sum_force = 0.0;

        let mut sum_logic = 0.0;
        let mut min_logic = f64::MAX;

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;

        let mut sum_energy = 0.0;
        let mut max_energy = f64::MIN;

        let mut sum_stab = 0.0;
        let mut compliant_count = 0usize;

        for pt in &results {
            sum_v += pt.drift_velocity_m_s;
            if pt.drift_velocity_m_s < min_v {
                min_v = pt.drift_velocity_m_s;
            }
            if pt.drift_velocity_m_s > max_v {
                max_v = pt.drift_velocity_m_s;
            }

            sum_hall += pt.hall_angle_deg;
            if pt.hall_angle_deg < min_hall {
                min_hall = pt.hall_angle_deg;
            }
            if pt.hall_angle_deg > max_hall {
                max_hall = pt.hall_angle_deg;
            }

            sum_force += pt.drag_force_pn;

            sum_logic += pt.logic_contrast_db;
            if pt.logic_contrast_db < min_logic {
                min_logic = pt.logic_contrast_db;
            }

            sum_iso += pt.circulator_isolation_db;
            if pt.circulator_isolation_db < min_iso {
                min_iso = pt.circulator_isolation_db;
            }

            sum_energy += pt.energy_bit_fj;
            if pt.energy_bit_fj > max_energy {
                max_energy = pt.energy_bit_fj;
            }

            sum_stab += pt.stability_factor;

            // Strict compliance conditions:
            // 1. Drift velocity > 100.0 m/s
            // 2. Hall angle between 10.0 and 70.0 deg
            // 3. Logic contrast >= 25.0 dB
            // 4. Circulator isolation >= 20.0 dB
            // 5. Energy per bit < 1.0 fJ
            // 6. Stability factor >= 1.0
            if pt.drift_velocity_m_s > 100.0
                && pt.hall_angle_deg >= 10.0
                && pt.hall_angle_deg <= 70.0
                && pt.logic_contrast_db >= 25.0
                && pt.circulator_isolation_db >= 20.0
                && pt.energy_bit_fj < 1.0
                && pt.stability_factor >= 1.0
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        SkyrmionPhononBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_drift_velocity_m_s: sum_v / n,
            min_drift_velocity_m_s: min_v,
            max_drift_velocity_m_s: max_v,
            mean_hall_angle_deg: sum_hall / n,
            min_hall_angle_deg: min_hall,
            max_hall_angle_deg: max_hall,
            mean_drag_force_pn: sum_force / n,
            mean_logic_contrast_db: sum_logic / n,
            min_logic_contrast_db: min_logic,
            mean_circulator_isolation_db: sum_iso / n,
            min_circulator_isolation_db: min_iso,
            mean_energy_bit_fj: sum_energy / n,
            max_energy_bit_fj: max_energy,
            mean_stability_factor: sum_stab / n,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
