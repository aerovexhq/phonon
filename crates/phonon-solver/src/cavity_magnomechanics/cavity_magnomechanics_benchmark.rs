//! Multi-threaded Rayon benchmark runner for cavity quantum magnomechanics,
//! continuous-variable entanglement, and macroscopic quantum state fidelity across 10,000 sweeps.

use super::lyapunov_covariance_solver::LyapunovCovarianceSolver;
use super::quantum_transducer_solver::QuantumTransducerSolver;
use phonon_models::cavity_magnomechanics::CavityMagnomechanicalParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point in cavity magnomechanics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CavityMagnomechanicsSweepPoint {
    pub magnon_phonon_log_negativity: f64,
    pub quantum_state_fidelity: f64,
    pub transduction_efficiency: f64,
    pub effective_phonon_occupation: f64,
    pub cooperativity: f64,
    pub quadrature_squeezing_db: f64,
}

/// Comprehensive benchmark report for cavity quantum magnomechanics.
#[derive(Debug, Clone, PartialEq)]
pub struct CavityMagnomechanicsBenchmarkReport {
    /// Total number of sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean magnon-phonon logarithmic negativity ($E_N > 0$ required).
    pub mean_log_negativity: f64,
    /// Minimum magnon-phonon logarithmic negativity.
    pub min_log_negativity: f64,
    /// Mean quantum state fidelity ($\\ge 0.90$ required).
    pub mean_quantum_fidelity: f64,
    /// Minimum quantum state fidelity.
    pub min_quantum_fidelity: f64,
    /// Mean quantum transduction efficiency ($\\ge 0.50$ required).
    pub mean_transduction_efficiency: f64,
    /// Minimum quantum transduction efficiency.
    pub min_transduction_efficiency: f64,
    /// Mean effective phonon occupation number ($< 1.0$ required).
    pub mean_phonon_occupation: f64,
    /// Maximum effective phonon occupation number.
    pub max_phonon_occupation: f64,
    /// Fraction of parameter sweeps satisfying all physical criteria.
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for cavity magnomechanics.
#[derive(Debug, Clone)]
pub struct CavityMagnomechanicsBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for CavityMagnomechanicsBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl CavityMagnomechanicsBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> CavityMagnomechanicsBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<CavityMagnomechanicsSweepPoint> = (0..total)
            .into_par_iter()
            .map(|i| {
                let u = i as f64 / total.max(1) as f64;

                // Parameter sweeping across physical ranges:
                // Bath temperature in [10.0, 40.0] mK
                let t_mk = 10.0 + 30.0 * u;
                // Phonon frequency in [15.0, 35.0] MHz
                let f_ph_mhz = 15.0 + 20.0 * ((i * 7) % total) as f64 / total as f64;
                // Photon-magnon coupling in [18.0, 32.0] MHz
                let g_ma_mhz = 18.0 + 14.0 * ((i * 11) % total) as f64 / total as f64;
                // Coherent magnon number in [2.0e11, 1.0e12]
                let n_m = (2.0 + 8.0 * ((i * 17) % total) as f64 / total as f64) * 1.0e11;
                // Phonon damping in [50.0, 150.0] Hz
                let gamma_b_hz = 50.0 + 100.0 * ((i * 23) % total) as f64 / total as f64;

                let params = CavityMagnomechanicalParams {
                    cavity_frequency_ghz: 9.0,
                    magnon_frequency_ghz: 9.0,
                    phonon_frequency_mhz: f_ph_mhz,
                    photon_damping_mhz: 2.5,
                    magnon_damping_mhz: 1.5,
                    phonon_damping_hz: gamma_b_hz,
                    photon_magnon_coupling_mhz: g_ma_mhz,
                    single_spin_magnetostriction_hz: 1.2,
                    coherent_magnon_number: n_m,
                    bath_temperature_mk: t_mk,
                };

                let lyapunov = LyapunovCovarianceSolver::new(params);
                let base_metrics = lyapunov.solve_magnomechanical_metrics();
                let ent_metrics = lyapunov.solve_entanglement_metrics();

                let transducer = QuantumTransducerSolver::new(params);
                let sq_db = transducer.solve_quadrature_squeezing_db();

                CavityMagnomechanicsSweepPoint {
                    magnon_phonon_log_negativity: ent_metrics.magnon_phonon_log_negativity,
                    quantum_state_fidelity: ent_metrics.quantum_state_fidelity,
                    transduction_efficiency: ent_metrics.transduction_efficiency_fraction,
                    effective_phonon_occupation: base_metrics.effective_phonon_occupation,
                    cooperativity: base_metrics.magnon_phonon_cooperativity,
                    quadrature_squeezing_db: sq_db,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_en = 0.0;
        let mut min_en = f64::INFINITY;
        let mut sum_fid = 0.0;
        let mut min_fid = f64::INFINITY;
        let mut sum_eta = 0.0;
        let mut min_eta = f64::INFINITY;
        let mut sum_neff = 0.0;
        let mut max_neff = f64::NEG_INFINITY;
        let mut compliant_count = 0usize;

        for p in &results {
            sum_en += p.magnon_phonon_log_negativity;
            if p.magnon_phonon_log_negativity < min_en {
                min_en = p.magnon_phonon_log_negativity;
            }

            sum_fid += p.quantum_state_fidelity;
            if p.quantum_state_fidelity < min_fid {
                min_fid = p.quantum_state_fidelity;
            }

            sum_eta += p.transduction_efficiency;
            if p.transduction_efficiency < min_eta {
                min_eta = p.transduction_efficiency;
            }

            sum_neff += p.effective_phonon_occupation;
            if p.effective_phonon_occupation > max_neff {
                max_neff = p.effective_phonon_occupation;
            }

            // Criteria:
            // 1. Logarithmic negativity E_N > 0.0
            // 2. Quantum state fidelity >= 0.90
            // 3. Transduction efficiency >= 0.50
            // 4. Effective phonon occupation < 1.0 (ground state cooling)
            if p.magnon_phonon_log_negativity > 0.0
                && p.quantum_state_fidelity >= 0.90
                && p.transduction_efficiency >= 0.50
                && p.effective_phonon_occupation < 1.0
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        CavityMagnomechanicsBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_log_negativity: sum_en / n,
            min_log_negativity: min_en,
            mean_quantum_fidelity: sum_fid / n,
            min_quantum_fidelity: min_fid,
            mean_transduction_efficiency: sum_eta / n,
            min_transduction_efficiency: min_eta,
            mean_phonon_occupation: sum_neff / n,
            max_phonon_occupation: max_neff,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
