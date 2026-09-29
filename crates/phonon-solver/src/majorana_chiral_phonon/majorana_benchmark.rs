//! Multi-threaded Rayon benchmark runner for chiral acoustic Majorana braiding across 10,000 parameter sweeps.

use super::majorana_braiding_solver::MajoranaBraidingSolver;
use phonon_models::majorana_chiral_phonon::MajoranaBraidingParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for Majorana braiding dynamics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaSweepPoint {
    pub fidelity_pct: f64,
    pub phase_rad: f64,
    pub phase_error_rad: f64,
    pub leakage: f64,
    pub readout_snr_db: f64,
    pub dephasing_time_us: f64,
}

/// Comprehensive benchmark report for chiral acoustic Majorana braiding.
#[derive(Debug, Clone, PartialEq)]
pub struct MajoranaBraidingBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean adiabatic braiding fidelity in percent ($\ge 99.0\%$ required).
    pub mean_braiding_fidelity_pct: f64,
    /// Minimum adiabatic braiding fidelity in percent.
    pub min_braiding_fidelity_pct: f64,
    /// Maximum adiabatic braiding fidelity in percent.
    pub max_braiding_fidelity_pct: f64,
    /// Mean non-Abelian Berry geometric phase in radians ($\approx \pi/2$).
    pub mean_non_abelian_phase_rad: f64,
    /// Maximum phase error from ideal $\pi/2$ in radians ($|\phi - \pi/2| \le 0.05\text{ rad}$).
    pub max_phase_error_rad: f64,
    /// Mean Landau-Zener transition leakage ($P_{\mathrm{LZ}} \le 1.0\times 10^{-3}$).
    pub mean_landau_zener_leakage: f64,
    /// Maximum Landau-Zener transition leakage.
    pub max_landau_zener_leakage: f64,
    /// Mean topological fermion parity readout SNR in decibels ($\ge 20.0\text{ dB}$).
    pub mean_parity_readout_snr_db: f64,
    /// Minimum topological fermion parity readout SNR in decibels.
    pub min_parity_readout_snr_db: f64,
    /// Mean topological qubit dephasing time in microseconds ($\ge 10.0\,\mu\text{s}$).
    pub mean_dephasing_time_us: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for acoustic Majorana braiding.
#[derive(Debug, Clone)]
pub struct MajoranaBraidingBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for MajoranaBraidingBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl MajoranaBraidingBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> MajoranaBraidingBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<MajoranaSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let length = 1.8 + 2.4 * frac; // 1.8 to 4.2 um
                let gap = 120.0 + 200.0 * pseudo_hash; // 120 to 320 ueV
                let duration = 30.0 + 70.0 * frac; // 30 to 100 ns
                let power = 50.0 + 300.0 * pseudo_hash; // 50 to 350 uW
                let temp = 15.0 + 45.0 * frac; // 15 to 60 mK
                let coupling = 8.0 + 20.0 * pseudo_hash; // 8 to 28 MHz

                let params = MajoranaBraidingParams {
                    nanowire_length_um: length,
                    topological_gap_muev: gap,
                    spin_orbit_coupling_mev_nm: 25.0,
                    saw_frequency_ghz: 1.8,
                    saw_acoustic_power_uw: power,
                    braiding_duration_ns: duration,
                    ambient_temperature_mk: temp,
                    readout_coupling_mhz: coupling,
                };

                let solver = MajoranaBraidingSolver::new(params);
                let m = solver.solve();

                MajoranaSweepPoint {
                    fidelity_pct: m.braiding_fidelity_pct,
                    phase_rad: m.non_abelian_phase_rad,
                    phase_error_rad: m.phase_error_rad,
                    leakage: m.landau_zener_leakage,
                    readout_snr_db: m.parity_readout_snr_db,
                    dephasing_time_us: m.topological_dephasing_time_us,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1.0e-6);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut max_fid = f64::MIN;

        let mut sum_phase = 0.0;
        let mut max_err = f64::MIN;

        let mut sum_lz = 0.0;
        let mut max_lz = f64::MIN;

        let mut sum_snr = 0.0;
        let mut min_snr = f64::MAX;

        let mut sum_t2 = 0.0;
        let mut compliant_count = 0usize;

        for pt in &results {
            sum_fid += pt.fidelity_pct;
            if pt.fidelity_pct < min_fid {
                min_fid = pt.fidelity_pct;
            }
            if pt.fidelity_pct > max_fid {
                max_fid = pt.fidelity_pct;
            }

            sum_phase += pt.phase_rad;
            if pt.phase_error_rad > max_err {
                max_err = pt.phase_error_rad;
            }

            sum_lz += pt.leakage;
            if pt.leakage > max_lz {
                max_lz = pt.leakage;
            }

            sum_snr += pt.readout_snr_db;
            if pt.readout_snr_db < min_snr {
                min_snr = pt.readout_snr_db;
            }

            sum_t2 += pt.dephasing_time_us;

            // Strict compliance conditions:
            // 1. Braiding fidelity >= 99.0%
            // 2. Phase error <= 0.05 rad
            // 3. Landau-Zener leakage <= 1.0e-3
            // 4. Parity readout SNR >= 20.0 dB
            // 5. Dephasing time >= 10.0 us
            if pt.fidelity_pct >= 99.0
                && pt.phase_error_rad <= 0.05
                && pt.leakage <= 1.0e-3
                && pt.readout_snr_db >= 20.0
                && pt.dephasing_time_us >= 10.0
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        MajoranaBraidingBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_braiding_fidelity_pct: sum_fid / n,
            min_braiding_fidelity_pct: min_fid,
            max_braiding_fidelity_pct: max_fid,
            mean_non_abelian_phase_rad: sum_phase / n,
            max_phase_error_rad: max_err,
            mean_landau_zener_leakage: sum_lz / n,
            max_landau_zener_leakage: max_lz,
            mean_parity_readout_snr_db: sum_snr / n,
            min_parity_readout_snr_db: min_snr,
            mean_dephasing_time_us: sum_t2 / n,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
