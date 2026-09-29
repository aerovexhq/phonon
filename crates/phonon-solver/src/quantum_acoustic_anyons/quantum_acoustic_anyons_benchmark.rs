//! Multi-threaded Rayon benchmark runner for quantum acoustic anyon braiding networks
//! across 10,000 parameter sweeps.

use super::quantum_acoustic_anyons_solver::QuantumAcousticAnyonSolver;
use phonon_models::quantum_acoustic_anyons::QuantumAcousticAnyonParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for quantum acoustic anyons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumAcousticAnyonSweepPoint {
    pub fidelity_pct: f64,
    pub leakage: f64,
    pub gap_mhz: f64,
    pub phase_error_rad: f64,
    pub readout_snr_db: f64,
    pub coherence_time_us: f64,
}

/// Comprehensive benchmark report for quantum acoustic anyons.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumAcousticAnyonBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean braiding gate fidelity in percent ($\ge 99.90\%$ required).
    pub mean_fidelity_pct: f64,
    /// Minimum braiding gate fidelity in percent.
    pub min_fidelity_pct: f64,
    /// Mean non-adiabatic leakage probability ($\le 1.0\times 10^{-5}$ required).
    pub mean_leakage: f64,
    /// Maximum non-adiabatic leakage probability.
    pub max_leakage: f64,
    /// Mean effective topological minigap in $\text{MHz}$ ($\ge 15.0\text{ MHz}$ required).
    pub mean_gap_mhz: f64,
    /// Minimum effective topological minigap in $\text{MHz}$.
    pub min_gap_mhz: f64,
    /// Mean braiding phase error in radians ($\le 0.005\text{ rad}$ required).
    pub mean_phase_error_rad: f64,
    /// Maximum braiding phase error in radians.
    pub max_phase_error_rad: f64,
    /// Mean parity readout SNR in decibels ($\ge 30.0\text{ dB}$ required).
    pub mean_readout_snr_db: f64,
    /// Minimum parity readout SNR in decibels.
    pub min_readout_snr_db: f64,
    /// Mean coherence time in microseconds ($\ge 50.0\,\mu\text{s}$ required).
    pub mean_coherence_time_us: f64,
    /// Minimum coherence time in microseconds.
    pub min_coherence_time_us: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for quantum acoustic anyon braiding.
#[derive(Debug, Clone)]
pub struct QuantumAcousticAnyonBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for QuantumAcousticAnyonBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl QuantumAcousticAnyonBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> QuantumAcousticAnyonBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<QuantumAcousticAnyonSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let gap0 = 25.0 + 45.0 * frac; // 25 to 70 MHz
                let f_saw = 1.5 + 4.0 * pseudo_hash; // 1.5 to 5.5 GHz
                let duration = 80.0 + 200.0 * frac; // 80 to 280 ns
                let pot = 10.0 + 25.0 * pseudo_hash; // 10 to 35 meV
                let l_arm = 1.5 + 2.5 * frac; // 1.5 to 4.0 um
                let alpha_so = 0.8 + 1.2 * pseudo_hash; // 0.8 to 2.0 eV*Ang
                let ez = 1.0 + 2.0 * frac; // 1.0 to 3.0 meV
                let t_mk = 15.0 + 35.0 * pseudo_hash; // 15 to 50 mK

                let params = QuantumAcousticAnyonParams {
                    topological_gap_mhz: gap0,
                    saw_frequency_ghz: f_saw,
                    braiding_duration_ns: duration,
                    saw_potential_amplitude_mev: pot,
                    junction_arm_length_um: l_arm,
                    spin_orbit_coupling_ev_ang: alpha_so,
                    zeeman_energy_mev: ez,
                    temperature_mk: t_mk,
                };

                let solver = QuantumAcousticAnyonSolver::new(params);
                let m = solver.solve();

                QuantumAcousticAnyonSweepPoint {
                    fidelity_pct: m.braiding_fidelity_pct,
                    leakage: m.non_adiabatic_leakage,
                    gap_mhz: m.effective_topological_gap_mhz,
                    phase_error_rad: m.braiding_phase_error_rad,
                    readout_snr_db: m.parity_readout_snr_db,
                    coherence_time_us: m.coherence_time_us,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;

        let mut sum_leak = 0.0;
        let mut max_leak = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;

        let mut sum_err = 0.0;
        let mut max_err = f64::MIN;

        let mut sum_snr = 0.0;
        let mut min_snr = f64::MAX;

        let mut sum_t2 = 0.0;
        let mut min_t2 = f64::MAX;

        let mut compliant_count = 0;

        for r in &results {
            sum_fid += r.fidelity_pct;
            if r.fidelity_pct < min_fid {
                min_fid = r.fidelity_pct;
            }

            sum_leak += r.leakage;
            if r.leakage > max_leak {
                max_leak = r.leakage;
            }

            sum_gap += r.gap_mhz;
            if r.gap_mhz < min_gap {
                min_gap = r.gap_mhz;
            }

            sum_err += r.phase_error_rad;
            if r.phase_error_rad > max_err {
                max_err = r.phase_error_rad;
            }

            sum_snr += r.readout_snr_db;
            if r.readout_snr_db < min_snr {
                min_snr = r.readout_snr_db;
            }

            sum_t2 += r.coherence_time_us;
            if r.coherence_time_us < min_t2 {
                min_t2 = r.coherence_time_us;
            }

            // Physical criteria validation:
            let is_compliant = r.fidelity_pct >= 99.90
                && r.leakage <= 1.0e-5
                && r.gap_mhz >= 15.0
                && r.phase_error_rad <= 0.005
                && r.readout_snr_db >= 30.0
                && r.coherence_time_us >= 50.0;

            if is_compliant {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        QuantumAcousticAnyonBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_fidelity_pct: sum_fid / n,
            min_fidelity_pct: min_fid,
            mean_leakage: sum_leak / n,
            max_leakage: max_leak,
            mean_gap_mhz: sum_gap / n,
            min_gap_mhz: min_gap,
            mean_phase_error_rad: sum_err / n,
            max_phase_error_rad: max_err,
            mean_readout_snr_db: sum_snr / n,
            min_readout_snr_db: min_snr,
            mean_coherence_time_us: sum_t2 / n,
            min_coherence_time_us: min_t2,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
