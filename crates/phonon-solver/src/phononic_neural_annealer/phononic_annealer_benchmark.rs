//! Multi-threaded Rayon benchmark runner for quantum phononic neural annealers
//! and adiabatic acoustic Ising machines across 10,000 parameter sweeps.

use super::phononic_annealer_solver::PhononicAnnealerSolver;
use phonon_models::phononic_neural_annealer::PhononicAnnealerParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for phononic neural annealers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononicAnnealerSweepPoint {
    pub fidelity_pct: f64,
    pub speedup: f64,
    pub energy_fj: f64,
    pub approx_ratio: f64,
    pub solution_us: f64,
    pub contrast_db: f64,
}

/// Comprehensive benchmark report for quantum phononic neural annealers.
#[derive(Debug, Clone, PartialEq)]
pub struct PhononicAnnealerBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean ground-state convergence fidelity in percent ($\\ge 98.0\\%$ required).
    pub mean_fidelity_pct: f64,
    /// Minimum ground-state convergence fidelity in percent.
    pub min_fidelity_pct: f64,
    /// Mean computational speedup factor ($\\ge 100.0\\times$ required).
    pub mean_speedup: f64,
    /// Minimum computational speedup factor.
    pub min_speedup: f64,
    /// Mean energy per spin flip in femtojoules ($\\le 50.0\\text{ fJ}$ required).
    pub mean_energy_fj: f64,
    /// Maximum energy per spin flip in femtojoules.
    pub max_energy_fj: f64,
    /// Mean graph approximation ratio ($\\ge 0.95$ required).
    pub mean_approx_ratio: f64,
    /// Minimum graph approximation ratio.
    pub min_approx_ratio: f64,
    /// Mean solution time in microseconds ($\\le 10.0\\,\\mu\\text{s}$ required).
    pub mean_solution_us: f64,
    /// Maximum solution time in microseconds.
    pub max_solution_us: f64,
    /// Mean bifurcation contrast in dB ($\\ge 25.0\\text{ dB}$ required).
    pub mean_contrast_db: f64,
    /// Minimum bifurcation contrast in dB.
    pub min_contrast_db: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for quantum phononic neural annealers.
#[derive(Debug, Clone)]
pub struct PhononicAnnealerBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for PhononicAnnealerBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl PhononicAnnealerBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> PhononicAnnealerBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<PhononicAnnealerSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let spins = match idx % 4 {
                    0 => 64,
                    1 => 128,
                    2 => 256,
                    _ => 512,
                };
                let pump = 25.0 + 80.0 * frac; // 25 to 105 MHz
                let coupling = 1.5 + 6.5 * pseudo_hash; // 1.5 to 8.0 MHz
                let ramp = 1.2 + 5.0 * frac; // 1.2 to 6.2 us
                let noise = -105.0 + 35.0 * pseudo_hash; // -105 to -70 dBm

                let params = PhononicAnnealerParams {
                    network_spin_count: spins,
                    parametric_pump_rate_mhz: pump,
                    acoustic_coupling_strength_mhz: coupling,
                    non_linear_elastic_kerr: 1.2e-4,
                    annealing_ramp_time_us: ramp,
                    thermal_noise_level_dbm: noise,
                };

                let solver = PhononicAnnealerSolver::new(params);
                let m = solver.solve();

                PhononicAnnealerSweepPoint {
                    fidelity_pct: m.convergence_fidelity_pct,
                    speedup: m.speedup_factor,
                    energy_fj: m.energy_per_flip_fj,
                    approx_ratio: m.graph_approximation_ratio,
                    solution_us: m.solution_time_us,
                    contrast_db: m.bifurcation_contrast_db,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;

        let mut sum_sp = 0.0;
        let mut min_sp = f64::MAX;

        let mut sum_e = 0.0;
        let mut max_e = f64::MIN;

        let mut sum_app = 0.0;
        let mut min_app = f64::MAX;

        let mut sum_sol = 0.0;
        let mut max_sol = f64::MIN;

        let mut sum_cr = 0.0;
        let mut min_cr = f64::MAX;

        let mut compliant_count = 0;

        for r in &results {
            sum_fid += r.fidelity_pct;
            if r.fidelity_pct < min_fid {
                min_fid = r.fidelity_pct;
            }

            sum_sp += r.speedup;
            if r.speedup < min_sp {
                min_sp = r.speedup;
            }

            sum_e += r.energy_fj;
            if r.energy_fj > max_e {
                max_e = r.energy_fj;
            }

            sum_app += r.approx_ratio;
            if r.approx_ratio < min_app {
                min_app = r.approx_ratio;
            }

            sum_sol += r.solution_us;
            if r.solution_us > max_sol {
                max_sol = r.solution_us;
            }

            sum_cr += r.contrast_db;
            if r.contrast_db < min_cr {
                min_cr = r.contrast_db;
            }

            // Physical criteria validation:
            let is_compliant = r.fidelity_pct >= 98.0
                && r.speedup >= 100.0
                && r.energy_fj <= 50.0
                && r.approx_ratio >= 0.95
                && r.solution_us <= 10.0
                && r.contrast_db >= 25.0;

            if is_compliant {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        PhononicAnnealerBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_fidelity_pct: sum_fid / n,
            min_fidelity_pct: min_fid,
            mean_speedup: sum_sp / n,
            min_speedup: min_sp,
            mean_energy_fj: sum_e / n,
            max_energy_fj: max_e,
            mean_approx_ratio: sum_app / n,
            min_approx_ratio: min_app,
            mean_solution_us: sum_sol / n,
            max_solution_us: max_sol,
            mean_contrast_db: sum_cr / n,
            min_contrast_db: min_cr,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
