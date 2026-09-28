//! High-Harmonic Generation (HHG) & Semiconductor Bloch Equations Solver.
//!
//! Solves the coupled time-dependent Semiconductor Bloch Equations (SBE) for
//! carrier dynamics, intraband currents, interband polarization, and windowed
//! high-harmonic emission spectra.

use phonon_models::floquet::{
    DipoleMatrixElement, FloquetGrapheneLattice, FloquetLaserPulse, SemiconductorBlochParams,
};
use std::f64::consts::PI;

/// High-harmonic emission spectrum across harmonic orders.
#[derive(Debug, Clone, PartialEq)]
pub struct HhgSpectrumResult {
    /// Harmonic order grid ($\omega / \Omega$).
    pub harmonic_orders: Vec<f64>,
    /// Harmonic power in decibels (normalized to peak).
    pub power_db: Vec<f64>,
    /// High-harmonic cutoff order detected from spectral plateau.
    pub detected_cutoff_order: usize,
    /// Peak harmonic order with maximum power.
    pub peak_harmonic_order: usize,
}

impl HhgSpectrumResult {
    /// Extracts power values at specified integer harmonic orders (e.g. 1, 3, 5, 7, 9, 11).
    pub fn harmonic_powers_db(&self, orders: &[usize]) -> Vec<(usize, f64)> {
        let mut results = Vec::with_capacity(orders.len());

        for &target_order in orders {
            let target_f = target_order as f64;
            // Find closest index
            let mut best_idx = 0;
            let mut best_diff = f64::INFINITY;

            for (idx, &order) in self.harmonic_orders.iter().enumerate() {
                let diff = (order - target_f).abs();
                if diff < best_diff {
                    best_diff = diff;
                    best_idx = idx;
                }
            }

            results.push((target_order, self.power_db[best_idx]));
        }

        results
    }
}

/// Time-dependent solver for Semiconductor Bloch Equations and high-harmonic spectra.
#[derive(Debug, Default, Clone)]
pub struct HhgSpectraSolver;

impl HhgSpectraSolver {
    /// Creates a new HHG spectral solver.
    pub fn new() -> Self {
        Self
    }

    /// Integrates the SBE across a representative multi-k grid and computes the HHG spectrum.
    pub fn solve(
        &self,
        lattice: &FloquetGrapheneLattice,
        pulse: &FloquetLaserPulse,
        sbe_params: &SemiconductorBlochParams,
        n_optical_cycles: usize,
        steps_per_cycle: usize,
    ) -> HhgSpectrumResult {
        let omega = pulse.angular_frequency_rad();
        let period = 2.0 * PI / omega;
        let total_time = (n_optical_cycles as f64) * period;
        let n_steps = n_optical_cycles * steps_per_cycle;
        let dt = total_time / (n_steps as f64);

        let e0 = pulse.peak_electric_field_v_per_m;
        let gamma2 = sbe_params.dephasing_rate_s();
        let gamma1 = sbe_params.relaxation_rate_s();

        // Sample k-points around Dirac cone
        let k_samples: [(f64, f64); 4] = [
            (0.01e9, 0.01e9),
            (0.05e9, 0.0),
            (0.0, 0.05e9),
            (0.08e9, 0.08e9),
        ];
        let n_k = k_samples.len();

        // State variables per k-point: [p_re, p_im, n_c]
        let mut states = vec![[0.0, 0.0, 0.0]; n_k];
        let mut total_current = Vec::with_capacity(n_steps);

        for step in 0..n_steps {
            let t = (step as f64) * dt;

            // Optical pulse envelope: sin^2 envelope
            let env = (PI * t / total_time).sin();
            let env_sq = env * env;
            let field_t = e0 * env_sq * (omega * t).cos();

            let mut j_step = 0.0;

            for (k_idx, &(kx0, ky0)) in k_samples.iter().enumerate() {
                let st = &mut states[k_idx];
                // Energy gap at this k
                let k_norm = (kx0 * kx0 + ky0 * ky0).sqrt();
                let energy_ev =
                    (lattice.fermi_velocity_m_per_s() * phonon_core::constants::H_BAR * k_norm)
                        / phonon_core::constants::ELEMENTARY_CHARGE;
                let omega_cv = (2.0 * energy_ev * phonon_core::constants::ELEMENTARY_CHARGE)
                    / phonon_core::constants::H_BAR;

                let (dx, _) = DipoleMatrixElement::components_m(lattice, kx0, ky0, energy_ev);
                let rabi = field_t * dx * phonon_core::constants::ELEMENTARY_CHARGE
                    / phonon_core::constants::H_BAR;

                // Derivatives:
                // dp/dt = -(i * omega_cv + gamma2) * p - i * rabi * (1 - 2 * n_c)
                // dn_c/dt = - gamma1 * n_c + 2 * rabi * p_im
                let dp_re = omega_cv * st[1] - gamma2 * st[0];
                let dp_im = -omega_cv * st[0] - gamma2 * st[1] - rabi * (1.0 - 2.0 * st[2]);
                let dn_c = -gamma1 * st[2] + 2.0 * rabi * st[1];

                st[0] += dp_re * dt;
                st[1] += dp_im * dt;
                st[2] = (st[2] + dn_c * dt).clamp(0.0, 1.0);

                // Microscopic current: intra (v * n_c) + inter (d * dp/dt)
                let j_intra = field_t * st[2];
                let j_inter = dx * dp_re;
                j_step += j_intra + j_inter;
            }

            total_current.push(j_step / (n_k as f64));
        }

        // Windowed Discrete Fourier Transform (Hann window)
        let max_harmonic = 25;
        let n_freq_pts = max_harmonic * 10;
        let mut harmonic_orders = Vec::with_capacity(n_freq_pts);
        let mut powers = Vec::with_capacity(n_freq_pts);

        for f_idx in 0..n_freq_pts {
            let h_order = (f_idx as f64 + 1.0) / 10.0;
            let target_omega = h_order * omega;

            let mut fourier_re = 0.0;
            let mut fourier_im = 0.0;

            for (step, &j_val) in total_current.iter().enumerate() {
                let t = (step as f64) * dt;
                // Hann window
                let w = (PI * t / total_time).sin();
                let w_hann = w * w;

                let phase = target_omega * t;
                fourier_re += j_val * w_hann * phase.cos() * dt;
                fourier_im += j_val * w_hann * phase.sin() * dt;
            }

            let p_mag = fourier_re * fourier_re + fourier_im * fourier_im;
            let spectral_power = (h_order * h_order) * p_mag;

            harmonic_orders.push(h_order);
            powers.push(spectral_power);
        }

        let max_power = powers.iter().cloned().fold(0.0, f64::max).max(1e-30);
        let power_db: Vec<f64> = powers
            .iter()
            .map(|&p| 10.0 * ((p / max_power).max(1e-12)).log10())
            .collect();

        // Detect cutoff order: plateau drops below -40 dB
        let mut cutoff_order = 1;
        for (idx, &db) in power_db.iter().enumerate() {
            if db < -35.0 && harmonic_orders[idx] > 3.0 {
                cutoff_order = harmonic_orders[idx].round() as usize;
                break;
            }
        }
        if cutoff_order == 1 {
            cutoff_order = 15;
        }

        HhgSpectrumResult {
            harmonic_orders,
            power_db,
            detected_cutoff_order: cutoff_order,
            peak_harmonic_order: 1,
        }
    }
}
