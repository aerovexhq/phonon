//! Stochastic Differential Equation (SDE) Langevin integrator for cavity optomechanical trajectories.
//!
//! Formulates the stochastic Langevin equations of motion:
//! $$dx = \frac{p}{m_{eff}} dt$$
//! $$dp = \left( -m_{eff} \Omega_m^2 x - (\gamma_m + \Gamma_{opt}) p \right) dt + \sqrt{2 m_{eff} (\gamma_m + \Gamma_{opt}) k_B T_{eff}} dW_t$$
//!
//! Evaluates real-time phase-space trajectories, thermalization, dynamical backaction cooling,
//! steady-state position and momentum variances $\langle x^2 \rangle, \langle p^2 \rangle$,
//! and extracted mean phonon occupancy $\bar{n}_{sim} = \frac{\langle E \rangle - \frac{1}{2}\hbar\Omega_m}{\hbar\Omega_m}$.

use phonon_core::constants::{BOLTZMANN_CONSTANT, H_BAR};
use phonon_models::optomechanics::SidebandCoolingParams;

/// Result of an optomechanical stochastic trajectory simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct TrajectoryResult {
    /// Final displacement in meters.
    pub final_displacement_m: f64,
    /// Final momentum in kg*m/s.
    pub final_momentum_kg_m_s: f64,
    /// Time-averaged variance of displacement $\langle x^2 \rangle$ in $\text{m}^2$.
    pub displacement_variance_m2: f64,
    /// Time-averaged variance of momentum $\langle p^2 \rangle$ in $(\text{kg}\cdot\text{m/s})^2$.
    pub momentum_variance_kg2_m2_s2: f64,
    /// Simulated mean mechanical phonon occupancy $\bar{n}_m$.
    pub simulated_phonon_occupancy: f64,
    /// Analytical expected mean phonon occupancy.
    pub analytical_phonon_occupancy: f64,
    /// Relative cooling error $|\bar{n}_{sim} - \bar{n}_{ana}| / (\bar{n}_{ana} + 0.1)$.
    pub relative_error: f64,
    /// Number of integration steps executed.
    pub steps_integrated: usize,
}

/// Stochastic Langevin SDE integrator.
pub struct LangevinSdeSolver;

impl LangevinSdeSolver {
    /// Integrates a single stochastic Langevin trajectory over `num_steps` with time step `dt_s`.
    /// Uses a pseudo-random linear congruential generator (LCG) / Box-Muller transform for deterministic zero-allocation Gaussian noise.
    pub fn simulate_trajectory(
        params: &SidebandCoolingParams,
        num_steps: usize,
        dt_s: f64,
        seed: u64,
    ) -> TrajectoryResult {
        let m_eff = params.system.effective_mass_kg;
        let omega_m_rad = 2.0 * std::f64::consts::PI * params.system.mechanical_frequency_hz;
        let gamma_eff_rad =
            2.0 * std::f64::consts::PI * params.effective_mechanical_damping_hz().max(1.0);
        let t_eff = params.effective_temperature_k();

        // Diffusion coefficient: 2 * m_eff * gamma_eff * k_B * T_eff
        let diffusion_strength = 2.0 * m_eff * gamma_eff_rad * BOLTZMANN_CONSTANT * t_eff.max(1e-6);
        let noise_std = (diffusion_strength * dt_s).sqrt();

        let mut x = 0.0;
        let mut p = 0.0;
        let mut sum_x2 = 0.0;
        let mut sum_p2 = 0.0;
        let mut rng_state = seed.wrapping_add(0x9E3779B97F4A7C15);

        // Warm up for 200 steps to reach stationary distribution
        let warmup_steps = 200.min(num_steps / 5);
        let total_steps = num_steps.max(100);

        for step in 0..total_steps {
            // Box-Muller transform for standard normal random variable
            let (u1, u2) = Self::next_uniform_pair(&mut rng_state);
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();

            // Stratonovich-Heun predictor-corrector step
            let stochastic_kick = noise_std * z;

            // Predictor
            let dx_pred = (p / m_eff) * dt_s;
            let dp_pred = (-m_eff * omega_m_rad * omega_m_rad * x - gamma_eff_rad * p) * dt_s
                + stochastic_kick;

            let x_pred = x + dx_pred;
            let p_pred = p + dp_pred;

            // Corrector
            let dx_corr = (0.5 * (p + p_pred) / m_eff) * dt_s;
            let dp_corr = (-m_eff * omega_m_rad * omega_m_rad * (0.5 * (x + x_pred))
                - gamma_eff_rad * (0.5 * (p + p_pred)))
                * dt_s
                + stochastic_kick;

            x += dx_corr;
            p += dp_corr;

            if step >= warmup_steps {
                sum_x2 += x * x;
                sum_p2 += p * p;
            }
        }

        let sample_count = (total_steps - warmup_steps) as f64;
        let var_x = sum_x2 / sample_count.max(1.0);
        let var_p = sum_p2 / sample_count.max(1.0);

        // Total mechanical oscillator energy E = (1/2) m omega^2 <x^2> + <p^2> / (2m)
        let total_energy = 0.5 * m_eff * omega_m_rad * omega_m_rad * var_x + (0.5 / m_eff) * var_p;
        let hbar_omega = H_BAR * omega_m_rad;
        let n_sim = ((total_energy - 0.5 * hbar_omega) / hbar_omega.max(1e-35)).max(0.0);
        let n_ana = params.cooled_phonon_occupancy();

        let rel_error = (n_sim - n_ana).abs() / (n_ana + 0.1);

        TrajectoryResult {
            final_displacement_m: x,
            final_momentum_kg_m_s: p,
            displacement_variance_m2: var_x,
            momentum_variance_kg2_m2_s2: var_p,
            simulated_phonon_occupancy: n_sim,
            analytical_phonon_occupancy: n_ana,
            relative_error: rel_error,
            steps_integrated: total_steps,
        }
    }

    #[inline]
    fn next_uniform_pair(state: &mut u64) -> (f64, f64) {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let r1 = ((*state >> 11) as f64) / 9007199254740992.0;
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let r2 = ((*state >> 11) as f64) / 9007199254740992.0;
        (r1.clamp(1e-9, 1.0 - 1e-9), r2.clamp(1e-9, 1.0 - 1e-9))
    }
}
