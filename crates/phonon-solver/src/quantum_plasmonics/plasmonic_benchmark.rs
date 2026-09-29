#![deny(unsafe_code)]

//! High-Performance Parallel Rayon Benchmark for Quantum Plasmonic Nanocircuits.

use phonon_models::quantum_plasmonics::{
    NobleMetal, PlasmonicSlotWaveguide, QuantumEmitter, SinglePhotonTransistor,
    SppHydrodynamicModel,
};
use rayon::prelude::*;
use std::time::Instant;

/// Individual outcome of a quantum plasmonic parameter sweep.
#[derive(Debug, Clone, Copy)]
pub struct PlasmonicSweepResult {
    /// Slot gap width in nanometers.
    pub gap_width_nm: f64,
    /// Normalized sub-diffraction mode volume $V_{eff} / \lambda_0^3$.
    pub normalized_mode_volume: f64,
    /// Purcell enhancement factor $F_P$.
    pub purcell_factor: f64,
    /// Coupling beta factor $\beta_{spp} \in [0, 1]$.
    pub beta_factor: f64,
    /// Optical switching contrast in decibels $C_{dB}$.
    pub switching_contrast_db: f64,
    /// Non-local hydrodynamic resonance blueshift in GHz.
    pub blueshift_ghz: f64,
    /// SPP propagation length in micrometers.
    pub propagation_length_um: f64,
}

/// Comprehensive Statistical Report of the Quantum Plasmonic Benchmark.
#[derive(Debug, Clone, Copy)]
pub struct QuantumPlasmonicBenchmarkReport {
    /// Total number of sweeps executed.
    pub total_sweeps: usize,
    /// Elapsed wall-clock time in seconds.
    pub elapsed_seconds: f64,
    /// Processing throughput in sweeps per second.
    pub throughput_sweeps_per_sec: f64,
    /// Mean optical switching contrast in decibels.
    pub mean_contrast_db: f64,
    /// Minimum optical switching contrast in decibels.
    pub min_contrast_db: f64,
    /// Fraction of configurations achieving switching contrast $\ge 20\text{ dB}$.
    pub high_contrast_compliance_fraction: f64,
    /// Mean Purcell enhancement factor.
    pub mean_purcell_factor: f64,
    /// Mean single-mode coupling beta factor $\beta_{spp}$.
    pub mean_beta_factor: f64,
    /// Mean normalized mode volume $V_{eff} / \lambda_0^3$.
    pub mean_normalized_mode_volume: f64,
    /// Fraction of configurations with deep sub-diffraction confinement ($V_{eff} / \lambda_0^3 < 10^{-3}$).
    pub subdiffraction_compliance_fraction: f64,
    /// Mean non-local hydrodynamic blueshift in GHz.
    pub mean_blueshift_ghz: f64,
}

/// Runs the 10,000-sweep parallel Rayon benchmark across quantum plasmonic parameter space.
pub fn run_quantum_plasmonic_benchmark(num_sweeps: usize) -> QuantumPlasmonicBenchmarkReport {
    let start_time = Instant::now();

    let results: Vec<PlasmonicSweepResult> = (0..num_sweeps)
        .into_par_iter()
        .map(|idx| {
            // Material cycle: Silver, Gold, Copper, Aluminum
            let metal = match idx % 4 {
                0 => NobleMetal::Silver,
                1 => NobleMetal::Gold,
                2 => NobleMetal::Copper,
                _ => NobleMetal::Aluminum,
            };

            // Slot width in 2 - 12 nm range
            let gap_nm = 2.0 + ((idx % 21) as f64) * 0.5; // 2.0 to 12.0 nm
            let gap_m = gap_nm * 1e-9;
            let slot_height_m = 40.0e-9;
            let length_m = 1.0e-6;

            let eps_d = 2.25; // Silica
            let wavelength_m = (700.0 + ((idx % 30) as f64) * 5.0) * 1e-9; // 700 - 845 nm
            let omega =
                (2.0 * std::f64::consts::PI * phonon_models::quantum_plasmonics::SPEED_OF_LIGHT)
                    / wavelength_m;

            // Hydrodynamic model
            let hydro = SppHydrodynamicModel::new(metal, eps_d, gap_m, 0.2e-9);
            let blueshift_ghz = hydro.resonance_blueshift() / (2.0 * std::f64::consts::PI * 1e9);
            let prop_len_um = hydro.propagation_length(omega) * 1e6;

            // Waveguide & emitter
            let waveguide =
                PlasmonicSlotWaveguide::new(metal, eps_d, gap_m, slot_height_m, length_m);
            let norm_vol = waveguide.normalized_mode_volume(wavelength_m);

            // Dipole moment ~ 25 Debye
            let dipole = 25.0 * 3.336e-30;
            let emitter = QuantumEmitter::new(wavelength_m, dipole, 1.0e7, 1.0e8);

            let q_factor = 25.0 + ((idx % 15) as f64);
            let transistor = SinglePhotonTransistor::new(waveguide, emitter, q_factor);

            let fp = transistor.purcell_factor();
            let beta = transistor.beta_factor();
            let contrast_db = transistor.switching_contrast_db();

            PlasmonicSweepResult {
                gap_width_nm: gap_nm,
                normalized_mode_volume: norm_vol,
                purcell_factor: fp,
                beta_factor: beta,
                switching_contrast_db: contrast_db,
                blueshift_ghz,
                propagation_length_um: prop_len_um,
            }
        })
        .collect();

    let elapsed = start_time.elapsed().as_secs_f64();
    let total = results.len();
    let throughput = if elapsed > 0.0 {
        total as f64 / elapsed
    } else {
        0.0
    };

    let mut sum_contrast = 0.0;
    let mut min_contrast = 100.0f64;
    let mut high_contrast_count = 0;
    let mut sum_fp = 0.0;
    let mut sum_beta = 0.0;
    let mut sum_norm_vol = 0.0;
    let mut subdiff_count = 0;
    let mut sum_blueshift = 0.0;

    for r in &results {
        sum_contrast += r.switching_contrast_db;
        if r.switching_contrast_db < min_contrast {
            min_contrast = r.switching_contrast_db;
        }
        if r.switching_contrast_db >= 20.0 {
            high_contrast_count += 1;
        }

        sum_fp += r.purcell_factor;
        sum_beta += r.beta_factor;
        sum_norm_vol += r.normalized_mode_volume;
        if r.normalized_mode_volume < 1e-3 {
            subdiff_count += 1;
        }
        sum_blueshift += r.blueshift_ghz;
    }

    let n = total as f64;
    QuantumPlasmonicBenchmarkReport {
        total_sweeps: total,
        elapsed_seconds: elapsed,
        throughput_sweeps_per_sec: throughput,
        mean_contrast_db: sum_contrast / n,
        min_contrast_db: min_contrast,
        high_contrast_compliance_fraction: (high_contrast_count as f64) / n,
        mean_purcell_factor: sum_fp / n,
        mean_beta_factor: sum_beta / n,
        mean_normalized_mode_volume: sum_norm_vol / n,
        subdiffraction_compliance_fraction: (subdiff_count as f64) / n,
        mean_blueshift_ghz: sum_blueshift / n,
    }
}
