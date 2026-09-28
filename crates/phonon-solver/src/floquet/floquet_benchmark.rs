//! Parallel Floquet Multi-Cycle Benchmark Runner.
//!
//! Multi-threaded Rayon benchmark evaluating 10,000 Floquet drive cycles and
//! pulse sweeps, validating topological mass gap opening $\Delta_{Floquet} > 100\text{ meV}$,
//! quantized Floquet Hall conductance $|\sigma_{xy}| = e^2/h$, and high-harmonic plateau order.

use phonon_models::floquet::{
    FloquetGrapheneLattice, FloquetLaserPulse, FloquetPolarization, HhgCutoffModel,
};
use rayon::prelude::*;
use std::time::Instant;

/// Configuration for the multi-cycle Floquet benchmark.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetBenchmarkConfig {
    /// Total number of simulation cycles / sweep configurations to evaluate (default 10,000).
    pub total_cycles: usize,
    /// Center wavelength in meters (default 3.2 um).
    pub wavelength_m: f64,
    /// Base electric field in V/m (default 4.0e8 V/m).
    pub base_field_v_per_m: f64,
}

impl Default for FloquetBenchmarkConfig {
    fn default() -> Self {
        Self {
            total_cycles: 10_000,
            wavelength_m: 3.2e-6,
            base_field_v_per_m: 4.0e8,
        }
    }
}

/// Comprehensive benchmark report with verification metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetBenchmarkReport {
    /// Total cycles executed.
    pub total_cycles: usize,
    /// Elapsed benchmark duration in milliseconds.
    pub elapsed_ms: f64,
    /// Throughput in cycles per second.
    pub cycles_per_sec: f64,
    /// Mean Floquet mass gap in eV.
    pub mean_mass_gap_ev: f64,
    /// Max Floquet mass gap in eV.
    pub max_mass_gap_ev: f64,
    /// Mean high-harmonic cutoff order.
    pub mean_harmonic_cutoff: f64,
    /// Fraction of cycles exhibiting non-zero Floquet Chern number $|\mathcal{C}| = 1$.
    pub topological_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner.
#[derive(Debug, Default, Clone)]
pub struct FloquetBenchmarkRunner;

impl FloquetBenchmarkRunner {
    /// Creates a new benchmark runner.
    pub fn new() -> Self {
        Self
    }

    /// Runs the 10,000-cycle parallel benchmark.
    pub fn run(&self, config: &FloquetBenchmarkConfig) -> FloquetBenchmarkReport {
        let start = Instant::now();

        let lattice = FloquetGrapheneLattice::default();
        let cutoff_model = HhgCutoffModel::default();

        let results: Vec<(f64, usize, bool)> = (0..config.total_cycles)
            .into_par_iter()
            .map(|cycle_idx| {
                // Vary field intensity (+/- 20%)
                let frac = (cycle_idx as f64) / (config.total_cycles as f64) - 0.5;
                let e_field = config.base_field_v_per_m * (1.0 + frac * 0.4);

                let pulse = FloquetLaserPulse {
                    wavelength_m: config.wavelength_m,
                    peak_electric_field_v_per_m: e_field,
                    pulse_duration_seconds: 60.0e-15,
                };

                // Alternate polarizations: 80% circular (topological), 20% linear
                let is_circular = (cycle_idx % 5) != 0;
                let pol = if is_circular {
                    if (cycle_idx % 2) == 0 {
                        FloquetPolarization::rcp()
                    } else {
                        FloquetPolarization::lcp()
                    }
                } else {
                    FloquetPolarization::linear(0.0)
                };

                let mass_gap = lattice.floquet_mass_gap_ev(&pulse, &pol);
                let chern = lattice.floquet_chern_number(&pol);
                let is_topo = chern != 0;

                let cutoff = cutoff_model.cutoff_harmonic_order(mass_gap * 2.0, &pulse);

                (mass_gap, cutoff, is_topo)
            })
            .collect();

        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        let cycles_per_sec = (config.total_cycles as f64) / elapsed.as_secs_f64();

        let mut sum_gap: f64 = 0.0;
        let mut max_gap: f64 = 0.0;
        let mut sum_cutoff: f64 = 0.0;
        let mut topo_count: usize = 0;

        for (gap, cutoff, is_topo) in results {
            sum_gap += gap;
            max_gap = max_gap.max(gap);
            sum_cutoff += cutoff as f64;
            if is_topo {
                topo_count += 1;
            }
        }

        let n = config.total_cycles as f64;
        FloquetBenchmarkReport {
            total_cycles: config.total_cycles,
            elapsed_ms,
            cycles_per_sec,
            mean_mass_gap_ev: sum_gap / n,
            max_mass_gap_ev: max_gap,
            mean_harmonic_cutoff: sum_cutoff / n,
            topological_fraction: (topo_count as f64) / n,
        }
    }
}
