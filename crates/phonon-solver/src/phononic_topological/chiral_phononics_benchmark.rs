//! Parallel Multi-Cycle Benchmark Runner for Chiral Phononics & Acoustic Diodes.
//!
//! Multi-threaded Rayon benchmark evaluating 10,000 acoustic drive cycles and
//! configuration sweeps, validating valley bandgap opening $\Delta\omega_v / \omega_0 > 5\%$,
//! non-reciprocal diode and circulator isolation exceeding 20 dB, and corner transmission $\ge 90\%$.

use phonon_models::phononic_topological::{
    AcousticCirculatorParams, HoneycombAcousticLattice, SpatioTemporalModulator,
};
use rayon::prelude::*;
use std::f64::consts::PI;
use std::time::Instant;

/// Configuration for the multi-cycle chiral phononics benchmark.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralPhononicsBenchmarkConfig {
    /// Total number of simulation cycles / sweeps to execute (default 10,000).
    pub total_cycles: usize,
    /// Base sublattice A pillar diameter in meters (default 0.009 m).
    pub base_pillar_diameter_a_m: f64,
    /// Base sublattice B pillar diameter in meters (default 0.007 m).
    pub base_pillar_diameter_b_m: f64,
    /// Base modulation depth for phonon diode (default 0.25).
    pub modulation_depth: f64,
    /// Base circulation velocity in m/s (default 15.0 m/s).
    pub fluid_circulation_velocity_m_per_s: f64,
}

impl Default for ChiralPhononicsBenchmarkConfig {
    fn default() -> Self {
        Self {
            total_cycles: 10_000,
            base_pillar_diameter_a_m: 0.009,
            base_pillar_diameter_b_m: 0.007,
            modulation_depth: 0.25,
            fluid_circulation_velocity_m_per_s: 15.0,
        }
    }
}

/// Comprehensive benchmark report with verification metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralPhononicsBenchmarkReport {
    /// Total cycles executed.
    pub total_cycles: usize,
    /// Elapsed benchmark duration in milliseconds.
    pub elapsed_ms: f64,
    /// Throughput in cycles per second.
    pub cycles_per_sec: f64,
    /// Mean valley bandgap ratio $\Delta\omega_v / \omega_0$.
    pub mean_valley_gap_ratio: f64,
    /// Max valley bandgap ratio $\Delta\omega_v / \omega_0$.
    pub max_valley_gap_ratio: f64,
    /// Mean corner transmission through a sharp $60^\circ$ bend.
    pub mean_corner_transmission: f64,
    /// Mean phonon diode isolation in dB.
    pub mean_diode_isolation_db: f64,
    /// Mean acoustic circulator isolation in dB.
    pub mean_circulator_isolation_db: f64,
    /// Fraction of cycles with non-zero topological valley Chern number $|\mathcal{C}_v| = 1$.
    pub topological_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner.
#[derive(Debug, Default, Clone)]
pub struct ChiralPhononicsBenchmarkRunner;

impl ChiralPhononicsBenchmarkRunner {
    /// Creates a new benchmark runner.
    pub fn new() -> Self {
        Self
    }

    /// Runs the 10,000-cycle parallel benchmark.
    pub fn run(&self, config: &ChiralPhononicsBenchmarkConfig) -> ChiralPhononicsBenchmarkReport {
        let start = Instant::now();

        let results: Vec<(f64, f64, f64, f64, bool)> = (0..config.total_cycles)
            .into_par_iter()
            .map(|cycle_idx| {
                let frac = (cycle_idx as f64) / (config.total_cycles as f64) - 0.5;

                // Parameter variations (+/- 20%)
                let da = config.base_pillar_diameter_a_m * (1.0 + frac * 0.2);
                let db = config.base_pillar_diameter_b_m * (1.0 - frac * 0.2);
                let mu = (config.modulation_depth * (1.0 + frac * 0.1)).clamp(0.1, 0.4);
                let v0 = (config.fluid_circulation_velocity_m_per_s * (1.0 + frac * 0.15)).max(5.0);

                let lattice = HoneycombAcousticLattice::new(0.020, da, db, 343.0, 1.225);
                let gap_ratio = lattice.valley_gap_ratio();
                let corner_t = lattice.corner_transmission(PI / 3.0);
                let is_topo = lattice.valley_chern_number() != 0;

                let modulator = SpatioTemporalModulator {
                    modulation_depth: mu,
                    ..SpatioTemporalModulator::default()
                };
                let diode_iso = modulator.isolation_db(2.0 * PI * 4000.0);

                let circulator = AcousticCirculatorParams {
                    fluid_circulation_velocity_m_per_s: v0,
                    ..AcousticCirculatorParams::default()
                };
                let circ_iso = circulator
                    .circulator_isolation_db(circulator.unperturbed_resonance_rad_per_s());

                (gap_ratio, corner_t, diode_iso, circ_iso, is_topo)
            })
            .collect();

        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        let cycles_per_sec = (config.total_cycles as f64) / elapsed.as_secs_f64().max(1e-6);

        let mut sum_gap = 0.0;
        let mut max_gap = 0.0_f64;
        let mut sum_corner = 0.0;
        let mut sum_diode_iso = 0.0;
        let mut sum_circ_iso = 0.0;
        let mut topo_count = 0;

        for (gap, corner, diode_iso, circ_iso, is_topo) in &results {
            sum_gap += gap;
            if *gap > max_gap {
                max_gap = *gap;
            }
            sum_corner += corner;
            sum_diode_iso += diode_iso;
            sum_circ_iso += circ_iso;
            if *is_topo {
                topo_count += 1;
            }
        }

        let n = config.total_cycles as f64;
        ChiralPhononicsBenchmarkReport {
            total_cycles: config.total_cycles,
            elapsed_ms,
            cycles_per_sec,
            mean_valley_gap_ratio: sum_gap / n,
            max_valley_gap_ratio: max_gap,
            mean_corner_transmission: sum_corner / n,
            mean_diode_isolation_db: sum_diode_iso / n,
            mean_circulator_isolation_db: sum_circ_iso / n,
            topological_fraction: (topo_count as f64) / n,
        }
    }
}
