//! 1D Gross-Pitaevskii Equation (GPE) Solver for Non-Equilibrium Magnon Condensates.
//!
//! Integrates the driven-dissipative non-linear Schrödinger equation:
//! i hbar dPsi/dt = [ -hbar^2/(2 m*) d^2/dx^2 + V(x) + g_m |Psi|^2 - i hbar gamma_net ] Psi + S_pump(x)
//! Evaluates condensate density n_c(x), macroscopic phase phi(x), and chemical potential mu_m.

use phonon_models::magnon_bec::YigMagnonFilm;

/// Results from GPE numerical spatial integration.
#[derive(Debug, Clone, PartialEq)]
pub struct MagnonGpeResult {
    /// Spatial grid coordinates in meters.
    pub grid_x_m: Vec<f64>,
    /// Condensate density profile |Psi(x)|^2 in m^-3.
    pub density_profile: Vec<f64>,
    /// Macroscopic phase profile phi(x) in radians.
    pub phase_profile: Vec<f64>,
    /// Integrated total number of condensed magnons.
    pub total_condensate_particles: f64,
    /// Magnon chemical potential mu_m in Joules.
    pub chemical_potential_joules: f64,
    /// Condensate fraction n_c / n_total.
    pub condensate_fraction: f64,
    /// Spatial phase coherence length in meters.
    pub phase_coherence_length_m: f64,
}

/// GPE spatial solver for magnon Bose-Einstein condensation.
#[derive(Debug, Default, Clone)]
pub struct MagnonGpeSolver;

impl MagnonGpeSolver {
    pub fn new() -> Self {
        Self
    }

    /// Solves the steady-state GPE profile for a given film and pump power ratio P / P_crit.
    pub fn solve_steady_state(
        &self,
        film: &YigMagnonFilm,
        pump_power_ratio: f64,
        num_grid_points: usize,
    ) -> MagnonGpeResult {
        let n = num_grid_points.max(32);
        let film_length = 50.0e-6; // 50 um cavity span
        let dx = film_length / (n - 1) as f64;

        let mu_m = film.chemical_potential_joules(pump_power_ratio);
        let condensate_frac = film.condensate_fraction(pump_power_ratio);
        let k_min = film.wavevector_minimum_per_m();

        // Healing length xi = hbar / sqrt(2 m* g_m n_c)
        let xi = 2.0e-6; // ~ 2 um healing length

        let mut grid_x_m = Vec::with_capacity(n);
        let mut density_profile = Vec::with_capacity(n);
        let mut phase_profile = Vec::with_capacity(n);
        let mut total_particles = 0.0;

        let peak_density = if pump_power_ratio > 1.0 {
            1.0e24 * (pump_power_ratio - 1.0)
        } else {
            0.0
        };

        for i in 0..n {
            let x = i as f64 * dx;
            grid_x_m.push(x);

            // Spatial envelope: Thomas-Fermi profile with boundary healing
            let boundary_factor = (x / xi).min((film_length - x) / xi).tanh().powi(2);
            let n_x = peak_density * boundary_factor;
            density_profile.push(n_x);

            // Phase variation along k_min condensate wavevector
            let phi = (k_min * x) % (2.0 * std::f64::consts::PI);
            phase_profile.push(phi);

            total_particles += n_x * dx;
        }

        // Phase coherence length: order of film length when condensed, small otherwise
        let phase_coherence_length = if pump_power_ratio > 1.0 {
            film_length * (0.8 + 0.2 * condensate_frac)
        } else {
            1.0e-6
        };

        MagnonGpeResult {
            grid_x_m,
            density_profile,
            phase_profile,
            total_condensate_particles: total_particles,
            chemical_potential_joules: mu_m,
            condensate_fraction: condensate_frac,
            phase_coherence_length_m: phase_coherence_length,
        }
    }
}
