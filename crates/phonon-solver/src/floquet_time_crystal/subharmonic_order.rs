#![deny(unsafe_code)]

//! Subharmonic Spectral Analysis, Perturbation Rigidity Plateau,
//! and Temporal Edwards-Anderson Spin-Glass Order Parameter
//! for Topological Acoustic Floquet Time Crystals.

use std::f64::consts::PI;
use super::drive_hamiltonian::{
    FloquetState, FloquetStateKind, FloquetTimeCrystalParams, FloquetUnitaryOperator,
    StroboscopicTrajectory,
};

/// Discrete Fourier Transform (DFT) and subharmonic spectral rigidity analysis
/// of the stroboscopic acoustic magnetization dynamics.
#[derive(Debug, Clone, PartialEq)]
pub struct SubharmonicSpectralAnalysis {
    /// Normalized frequency axis omega / Omega in [0.0, 1.0], where Omega = 2 * pi / T.
    pub frequencies: Vec<f64>,
    /// Power spectral density S(omega) across the frequency grid.
    pub power_spectrum: Vec<f64>,
    /// Identified dominant peak frequency omega_peak / Omega (expected 0.5 for DTC).
    pub subharmonic_peak_frequency: f64,
    /// Subharmonic peak fraction S(Omega/2) / sum_omega S(omega) in [0.0, 1.0].
    pub subharmonic_fraction: f64,
    /// Whether the response is rigidly subharmonically locked (fraction >= 0.70 and peak ~ 0.5).
    pub is_rigidly_locked: bool,
}

impl SubharmonicSpectralAnalysis {
    /// Analyzes the stroboscopic magnetization trajectory.
    pub fn from_trajectory(trajectory: &StroboscopicTrajectory) -> Self {
        Self::analyze(&trajectory.average_magnetization)
    }

    /// Evaluates DFT power spectrum and subharmonic locking from raw magnetization series.
    pub fn analyze(magnetization: &[f64]) -> Self {
        let k = magnetization.len();
        if k < 2 {
            return Self {
                frequencies: vec![0.5],
                power_spectrum: vec![1.0],
                subharmonic_peak_frequency: 0.5,
                subharmonic_fraction: 1.0,
                is_rigidly_locked: true,
            };
        }

        // Discrete frequency grid: 101 points from nu = 0.0 to 1.0
        let num_bins = 101;
        let mut frequencies = Vec::with_capacity(num_bins);
        let mut power_spectrum = Vec::with_capacity(num_bins);

        let mut max_power = 0.0_f64;
        let mut peak_freq = 0.0_f64;
        let mut total_power = 0.0_f64;
        let mut subharmonic_window_power = 0.0_f64;

        for bin in 0..num_bins {
            let nu = (bin as f64) / ((num_bins - 1) as f64);
            frequencies.push(nu);

            // DFT: sum_n M_z(n) * exp(-i * 2 * pi * nu * n)
            let mut sum_re = 0.0_f64;
            let mut sum_im = 0.0_f64;
            for (n, &m) in magnetization.iter().enumerate() {
                let angle = -2.0 * PI * nu * (n as f64);
                sum_re += m * angle.cos();
                sum_im += m * angle.sin();
            }

            let power = sum_re * sum_re + sum_im * sum_im;
            power_spectrum.push(power);
            total_power += power;

            if power > max_power {
                max_power = power;
                peak_freq = nu;
            }

            // Window around nu = 0.5 (|nu - 0.5| <= 0.05)
            if (nu - 0.5).abs() <= 0.05 {
                subharmonic_window_power += power;
            }
        }

        let subharmonic_fraction = (subharmonic_window_power / total_power.max(1e-15)).clamp(0.0, 1.0);
        let is_rigidly_locked = subharmonic_fraction >= 0.70 && (peak_freq - 0.5).abs() <= 0.04;

        Self {
            frequencies,
            power_spectrum,
            subharmonic_peak_frequency: peak_freq,
            subharmonic_fraction,
            is_rigidly_locked,
        }
    }
}

/// Perturbation Rigidity Phase Diagram scanning pulse error epsilon in [-0.2, +0.2].
///
/// Demonstrates the broad topological Discrete Time Crystal stability plateau where
/// the subharmonic intensity S_sub(epsilon) remains pinned above 0.60 despite drive perturbations.
#[derive(Debug, Clone, PartialEq)]
pub struct RigidityPhaseDiagram {
    /// Sampled pulse error values epsilon in [-0.2, +0.2].
    pub epsilons: Vec<f64>,
    /// Computed subharmonic peak intensity S_sub(epsilon) for each sample.
    pub subharmonic_intensities: Vec<f64>,
    /// Lower bound of the DTC plateau where S_sub >= 0.60.
    pub plateau_min_epsilon: f64,
    /// Upper bound of the DTC plateau where S_sub >= 0.60.
    pub plateau_max_epsilon: f64,
    /// Plateau stability width: Delta epsilon = max - min.
    pub plateau_width: f64,
    /// Whether a robust DTC phase is established (plateau width >= 0.15).
    pub is_dtc_phase: bool,
}

impl RigidityPhaseDiagram {
    /// Computes the rigidity phase diagram scanning epsilon across `num_samples`.
    pub fn compute(
        base_params: &FloquetTimeCrystalParams,
        num_samples: usize,
        cycles: usize,
    ) -> Self {
        let n_samples = num_samples.clamp(11, 61);
        let n_cycles = cycles.clamp(20, 100);

        let mut epsilons = Vec::with_capacity(n_samples);
        let mut subharmonic_intensities = Vec::with_capacity(n_samples);

        // Keep chain length manageable for fast interactive evaluation
        let eval_chain_length = base_params.chain_length.min(6);

        let mut plateau_min = 0.0_f64;
        let mut plateau_max = 0.0_f64;
        let mut found_plateau_start = false;

        for i in 0..n_samples {
            let frac = (i as f64) / ((n_samples - 1) as f64);
            let eps = -0.20 + 0.40 * frac;
            epsilons.push(eps);

            let mut params = *base_params;
            params.chain_length = eval_chain_length;
            params.pulse_error_epsilon = eps;

            let op = FloquetUnitaryOperator::new(&params);
            let state0 = FloquetState::from_kind(FloquetStateKind::AllUp, eval_chain_length, 42);
            let traj = op.evolve(&state0, n_cycles);
            let spectral = SubharmonicSpectralAnalysis::from_trajectory(&traj);

            let intensity = spectral.subharmonic_fraction;
            subharmonic_intensities.push(intensity);

            if intensity >= 0.60 {
                if !found_plateau_start {
                    plateau_min = eps;
                    found_plateau_start = true;
                }
                plateau_max = eps;
            }
        }

        let plateau_width = if found_plateau_start {
            (plateau_max - plateau_min).max(0.0)
        } else {
            0.0
        };

        let is_dtc_phase = plateau_width >= 0.15;

        Self {
            epsilons,
            subharmonic_intensities,
            plateau_min_epsilon: plateau_min,
            plateau_max_epsilon: plateau_max,
            plateau_width,
            is_dtc_phase,
        }
    }
}

/// Temporal Edwards-Anderson spin-glass / acoustic correlation order parameter:
/// q_EA(t) = (1 / N) * sum_i <sigma_z_i(0) * sigma_z_i(t)>.
///
/// Persistent non-zero q_EA indicates time-crystalline order vs decay to 0 in thermalizing regimes.
#[derive(Debug, Clone, PartialEq)]
pub struct EdwardsAndersonOrder {
    /// Temporal correlation values q_EA(n * T) for each cycle n.
    pub correlations: Vec<f64>,
    /// Late-time asymptotic order parameter magnitude (averaged over last 30% of cycles).
    pub asymptotic_order: f64,
    /// Whether long-range temporal order is preserved (asymptotic_order >= 0.20).
    pub is_ordered: bool,
}

impl EdwardsAndersonOrder {
    /// Computes Edwards-Anderson temporal order from stroboscopic site polarizations.
    pub fn compute(trajectory: &StroboscopicTrajectory) -> Self {
        let n_cycles = trajectory.site_polarizations.len();
        if n_cycles == 0 {
            return Self {
                correlations: vec![1.0],
                asymptotic_order: 1.0,
                is_ordered: true,
            };
        }

        let initial_m = &trajectory.site_polarizations[0];
        let num_sites = initial_m.len().max(1);

        let mut correlations = Vec::with_capacity(n_cycles);
        for cycle_m in &trajectory.site_polarizations {
            let mut sum_corr = 0.0_f64;
            for i in 0..num_sites {
                let m0 = if i < initial_m.len() { initial_m[i] } else { 0.0 };
                let mt = if i < cycle_m.len() { cycle_m[i] } else { 0.0 };
                sum_corr += m0 * mt;
            }
            correlations.push(sum_corr / (num_sites as f64));
        }

        // Asymptotic order magnitude over the final 30% of cycles
        let start_idx = (n_cycles as f64 * 0.70).floor() as usize;
        let late_window = &correlations[start_idx..];
        let asymptotic_order = if !late_window.is_empty() {
            let sum_abs: f64 = late_window.iter().map(|&q| q.abs()).sum();
            sum_abs / (late_window.len() as f64)
        } else {
            0.0
        };

        let is_ordered = asymptotic_order >= 0.20;

        Self {
            correlations,
            asymptotic_order,
            is_ordered,
        }
    }
}
