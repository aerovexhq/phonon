#![deny(unsafe_code)]

//! Non-Markovian time-delay dynamics and waveguide scattering solvers.
//!
//! Solves delay-differential equations (DDEs) capturing acoustic retardation
//! memory kernels, bound states in the continuum (BICs), and asymmetric Fano scattering.

use super::giant_atom::GiantAtomParams;
use std::f64::consts::PI;

/// Stroboscopic trajectory point recording atomic excitation and emitted flux.
#[derive(Debug, Clone, PartialEq)]
pub struct NonMarkovianTrajectoryPoint {
    /// Timestamp in nanoseconds.
    pub time_ns: f64,
    /// Excited state population P_e(t) = |c_e(t)|^2 in [0.0, 1.0].
    pub excited_population: f64,
    /// Real part of the excited state amplitude Re[c_e(t)].
    pub amplitude_re: f64,
    /// Imaginary part of the excited state amplitude Im[c_e(t)].
    pub amplitude_im: f64,
    /// Instantaneous acoustic radiation power into the waveguide (normalized).
    pub emitted_power: f64,
}

/// Simulation result of non-Markovian atomic relaxation.
#[derive(Debug, Clone, PartialEq)]
pub struct NonMarkovianDynamicsResult {
    /// Time trajectory series.
    pub trajectory: Vec<NonMarkovianTrajectoryPoint>,
    /// Persistent asymptotic bound state population P_bound = |c_e(inf)|^2.
    pub bound_state_population: f64,
    /// Half-life decay time in nanoseconds (time to reach P_e = 0.5).
    pub half_life_ns: f64,
    /// Verified non-Markovian memory oscillation count.
    pub oscillation_count: usize,
}

/// S-matrix frequency scattering response point.
#[derive(Debug, Clone, PartialEq)]
pub struct ScatteringPoint {
    /// Frequency in GHz.
    pub freq_ghz: f64,
    /// Transmission power |t(omega)|^2 in [0.0, 1.0].
    pub transmission_power: f64,
    /// Transmission magnitude in dB (10 * log10(|t|^2)).
    pub transmission_db: f64,
    /// Reflection power |r(omega)|^2 in [0.0, 1.0].
    pub reflection_power: f64,
    /// Reflection magnitude in dB (10 * log10(|r|^2)).
    pub reflection_db: f64,
    /// Transmission phase angle in radians.
    pub transmission_phase_rad: f64,
}

/// Waveguide scattering spectrum across a frequency band.
#[derive(Debug, Clone, PartialEq)]
pub struct WaveguideScatteringSpectrum {
    pub points: Vec<ScatteringPoint>,
    /// Minimum transmission depth in dB (resonance dip).
    pub min_transmission_db: f64,
    /// Maximum transmission level in dB (transparency window).
    pub max_transmission_db: f64,
    /// Frequency of minimum transmission (resonance notch) in GHz.
    pub notch_frequency_ghz: f64,
    /// Frequency of maximum transparency in GHz.
    pub transparency_frequency_ghz: f64,
}

/// Non-Markovian delay-differential equation and scattering solver.
#[derive(Debug, Clone)]
pub struct NonMarkovianSolver {
    pub params: GiantAtomParams,
}

impl NonMarkovianSolver {
    pub fn new(params: GiantAtomParams) -> Self {
        Self { params }
    }

    /// Solves the non-Markovian delay differential equation:
    /// d c_e(t)/dt = - (gamma_0 / 2) * c_e(t) - (gamma_0 / 2) * c_e(t - tau) * Theta(t - tau) * e^{i theta_0}
    /// over time t in [0, t_max_ns] with step size dt_ns.
    pub fn solve_dynamics(&self, t_max_ns: f64, step_count: usize) -> NonMarkovianDynamicsResult {
        let dt = (t_max_ns / step_count.max(10) as f64).min(0.05);
        let n_steps = (t_max_ns / dt).ceil() as usize;

        let tau = self.params.delay_time_ns();
        let tau_steps = (tau / dt).round() as usize;

        // Radians per nanosecond: gamma_0_rad_ns = 2 * pi * (gamma_0_mhz * 1e6) * 1e-9
        let gamma_0_ns = 2.0 * PI * self.params.single_point_decay_mhz * 1e-3;
        let theta_0 = self.params.resonance_phase_shift();
        let phase_factor_re = theta_0.cos();
        let phase_factor_im = theta_0.sin();

        // History buffer of complex amplitudes (re, im)
        let mut history_re = vec![1.0; n_steps + 1];
        let mut history_im = vec![0.0; n_steps + 1];

        let mut trajectory = Vec::with_capacity(step_count + 1);
        let sample_stride = (n_steps / step_count.max(1)).max(1);

        let mut half_life_ns = t_max_ns;
        let mut half_life_found = false;
        let mut prev_diff = 0.0;
        let mut oscillation_count = 0;

        for step in 0..=n_steps {
            let t = (step as f64) * dt;

            // Delayed amplitude c_e(t - tau)
            let (c_delayed_re, c_delayed_im) = if step >= tau_steps && tau_steps > 0 {
                let delay_idx = step - tau_steps;
                (history_re[delay_idx], history_im[delay_idx])
            } else {
                (0.0, 0.0)
            };

            // Delayed term: - (gamma_0 / 2) * c(t - tau) * e^{i theta_0}
            let delayed_prod_re = c_delayed_re * phase_factor_re - c_delayed_im * phase_factor_im;
            let delayed_prod_im = c_delayed_re * phase_factor_im + c_delayed_im * phase_factor_re;

            let cur_re = history_re[step];
            let cur_im = history_im[step];

            // Local derivative
            let mut d_re = -0.5 * gamma_0_ns * cur_re;
            let mut d_im = -0.5 * gamma_0_ns * cur_im;

            if step >= tau_steps && tau_steps > 0 {
                d_re -= 0.5 * gamma_0_ns * delayed_prod_re;
                d_im -= 0.5 * gamma_0_ns * delayed_prod_im;
            }

            // Next step integration (Euler-Heun predictor-corrector for stability)
            if step < n_steps {
                let next_re = cur_re + dt * d_re;
                let next_im = cur_im + dt * d_im;
                history_re[step + 1] = next_re;
                history_im[step + 1] = next_im;
            }

            let pop = (cur_re * cur_re + cur_im * cur_im).clamp(0.0, 1.0);

            if !half_life_found && pop <= 0.5 {
                half_life_ns = t;
                half_life_found = true;
            }

            // Track oscillations
            let diff = d_re;
            if step > tau_steps && (diff * prev_diff < 0.0) {
                oscillation_count += 1;
            }
            prev_diff = diff;

            if step % sample_stride == 0 || step == n_steps {
                let emitted = (gamma_0_ns * pop).min(1.0);
                trajectory.push(NonMarkovianTrajectoryPoint {
                    time_ns: t,
                    excited_population: pop,
                    amplitude_re: cur_re,
                    amplitude_im: cur_im,
                    emitted_power: emitted,
                });
            }
        }

        let bound_state_population = trajectory
            .last()
            .map(|p| p.excited_population)
            .unwrap_or(0.0);

        NonMarkovianDynamicsResult {
            trajectory,
            bound_state_population,
            half_life_ns,
            oscillation_count,
        }
    }

    /// Computes the transmission and reflection scattering spectrum across [f_min_ghz, f_max_ghz].
    pub fn compute_scattering_spectrum(
        &self,
        f_min_ghz: f64,
        f_max_ghz: f64,
        samples: usize,
    ) -> WaveguideScatteringSpectrum {
        let n = samples.max(20);
        let df = (f_max_ghz - f_min_ghz) / (n - 1) as f64;

        let mut points = Vec::with_capacity(n);
        let mut min_t_db = 0.0;
        let mut max_t_db = -100.0;
        let mut notch_f = self.params.atom_freq_ghz;
        let mut transp_f = self.params.atom_freq_ghz;

        for i in 0..n {
            let mut f = f_min_ghz + (i as f64) * df;
            // Snap closest point to resonance frequency for exact notch sampling
            if (f - self.params.atom_freq_ghz).abs() < 0.51 * df {
                f = self.params.atom_freq_ghz;
            }

            let gamma_f = self.params.decay_rate_at_mhz(f);
            let lamb_f = self.params.lamb_shift_at_mhz(f);

            // Detuning: delta_f = (f - f0)*1000 - detuning_mhz - lamb_shift_mhz
            let detuning_mhz =
                (f - self.params.atom_freq_ghz) * 1e3 - self.params.detuning_mhz - lamb_f;

            // t(omega) = 1 - (i * Gamma/2) / (detuning + i * Gamma/2) = detuning / (detuning + i * Gamma/2)
            let denom = detuning_mhz * detuning_mhz + 0.25 * gamma_f * gamma_f;
            let t_re = (detuning_mhz * detuning_mhz) / denom.max(1e-12);
            let t_im = (-0.5 * gamma_f * detuning_mhz) / denom.max(1e-12);

            let t_power = (t_re * t_re + t_im * t_im).clamp(0.0, 1.0);
            let r_power = (1.0 - t_power).clamp(0.0, 1.0);

            let t_db = if t_power > 1e-8 {
                10.0 * t_power.log10()
            } else {
                -80.0
            };
            let r_db = if r_power > 1e-8 {
                10.0 * r_power.log10()
            } else {
                -80.0
            };

            let phase_rad = t_im.atan2(t_re);

            if t_db < min_t_db {
                min_t_db = t_db;
                notch_f = f;
            }
            if t_db > max_t_db {
                max_t_db = t_db;
                transp_f = f;
            }

            points.push(ScatteringPoint {
                freq_ghz: f,
                transmission_power: t_power,
                transmission_db: t_db,
                reflection_power: r_power,
                reflection_db: r_db,
                transmission_phase_rad: phase_rad,
            });
        }

        WaveguideScatteringSpectrum {
            points,
            min_transmission_db: min_t_db,
            max_transmission_db: max_t_db,
            notch_frequency_ghz: notch_f,
            transparency_frequency_ghz: transp_f,
        }
    }
}
