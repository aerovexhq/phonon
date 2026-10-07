#![deny(unsafe_code)]

//! Driven-Dissipative Gross-Pitaevskii solver and exciton-polariton BEC condensation engine.
//!
//! Models non-equilibrium Bose-Einstein condensation in optical microcavities coupled
//! to an exciton reservoir: threshold power, macroscopic density, and spatial coherence.

use std::f64::consts::PI;

/// Fundamental physical constants in SI / condensed matter units.
pub const HBAR: f64 = 1.054_571_817e-34; // J*s
pub const HBAR_MEV_PS: f64 = 0.658_211_956; // meV * ps
pub const ELECTRON_MASS_KG: f64 = 9.109_383_7e-31; // kg
pub const BOLTZMANN_K_EV: f64 = 8.617_333_262e-5; // eV / K

/// Parameters governing driven-dissipative polariton condensation.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonBecParams {
    /// Polariton effective mass in units of free electron mass m_0 (typically ~ 1e-4 m_0).
    pub effective_mass_ratio: f64,
    /// Cavity polariton decay rate gamma_c in meV (inverse lifetime hbar / tau_c, ~ 0.1 to 0.3 meV).
    pub cavity_decay_rate_mev: f64,
    /// Exciton reservoir decay rate gamma_R in meV (typically ~ 0.05 to 0.1 meV).
    pub reservoir_decay_rate_mev: f64,
    /// Polariton-polariton non-linear interaction constant g in meV * um^2.
    pub polariton_interaction_g_mev_um2: f64,
    /// Reservoir-to-condensate stimulated scattering rate R in meV * um^2 / hbar.
    pub scattering_rate_r_mev_um2: f64,
    /// Reservoir-polariton blueshift cross-interaction g_R in meV * um^2.
    pub reservoir_interaction_gr_mev_um2: f64,
    /// Normalized optical pump power P / P_th (above threshold when > 1.0).
    pub pump_power_ratio: f64,
    /// Cavity resonance wavelength in nm (e.g. 830 nm for GaAs microcavities).
    pub resonance_wavelength_nm: f64,
    /// Ambient operating temperature in Kelvin (e.g. 4.2 K to 10.0 K).
    pub temperature_k: f64,
}

impl Default for PolaritonBecParams {
    fn default() -> Self {
        Self {
            effective_mass_ratio: 1.0e-4,
            cavity_decay_rate_mev: 0.15,
            reservoir_decay_rate_mev: 0.08,
            polariton_interaction_g_mev_um2: 2.5e-3,
            scattering_rate_r_mev_um2: 0.012,
            reservoir_interaction_gr_mev_um2: 5.0e-3,
            pump_power_ratio: 2.2,
            resonance_wavelength_nm: 830.0,
            temperature_k: 4.2,
        }
    }
}

impl PolaritonBecParams {
    /// Creates a high-density, low-loss microcavity preset (tau_c ~ 20 ps).
    pub fn preset_high_q_microcavity() -> Self {
        Self {
            effective_mass_ratio: 8.0e-5,
            cavity_decay_rate_mev: 0.065,
            reservoir_decay_rate_mev: 0.050,
            polariton_interaction_g_mev_um2: 3.0e-3,
            scattering_rate_r_mev_um2: 0.015,
            reservoir_interaction_gr_mev_um2: 6.0e-3,
            pump_power_ratio: 3.0,
            resonance_wavelength_nm: 850.0,
            temperature_k: 4.2,
        }
    }

    /// Creates an ultra-fast GaAs microcavity preset near condensation threshold.
    pub fn preset_near_threshold() -> Self {
        Self {
            effective_mass_ratio: 1.2e-4,
            cavity_decay_rate_mev: 0.25,
            reservoir_decay_rate_mev: 0.12,
            polariton_interaction_g_mev_um2: 3.2e-3,
            scattering_rate_r_mev_um2: 0.010,
            reservoir_interaction_gr_mev_um2: 4.0e-3,
            pump_power_ratio: 2.2,
            resonance_wavelength_nm: 830.0,
            temperature_k: 5.0,
        }
    }
}

/// A spatial profile point for condensate density and first-order coherence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CondensateSpatialPoint {
    /// Radial coordinate r in micrometers.
    pub radius_um: f64,
    /// Polariton condensate density n(r) in um^-2.
    pub density_um2: f64,
    /// Exciton reservoir density n_R(r) in um^-2.
    pub reservoir_density_um2: f64,
    /// First-order spatial coherence function g^(1)(r, 0).
    pub coherence_g1: f64,
    /// Local condensate phase in radians.
    pub phase_rad: f64,
}

/// Steady-state condensation metrics evaluated by the Gross-Pitaevskii solver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BecCondensationMetrics {
    /// Condensation threshold pump power in arbitrary units (P_th = gamma_c * gamma_R / R).
    pub threshold_power_arb: f64,
    /// Peak condensate density n_0 in um^-2.
    pub peak_density_um2: f64,
    /// Sound speed in the polariton condensate v_s in m/s (Bogoliubov sound speed).
    pub sound_speed_ms: f64,
    /// Healing length xi in micrometers (xi = hbar / sqrt(2 * m* * g * n_0)).
    pub healing_length_um: f64,
    /// Chemical potential blueshift mu = g * n_0 + g_R * n_R,th in meV.
    pub chemical_potential_blueshift_mev: f64,
    /// Coherence length l_c in micrometers where g^(1)(l_c) = 1 / e.
    pub coherence_length_um: f64,
    /// Condensate fraction (macroscopic occupancy ratio) in [0, 1].
    pub condensate_fraction: f64,
}

/// Driven-Dissipative Gross-Pitaevskii solver for polariton BEC.
#[derive(Debug, Clone)]
pub struct GrossPitaevskiiSolver {
    pub params: PolaritonBecParams,
}

impl GrossPitaevskiiSolver {
    /// Creates a new solver with the specified parameters.
    pub fn new(params: PolaritonBecParams) -> Self {
        Self { params }
    }

    /// Computes effective polariton mass in kg.
    pub fn effective_mass_kg(&self) -> f64 {
        self.params.effective_mass_ratio * ELECTRON_MASS_KG
    }

    /// Evaluates condensation threshold pump power P_th.
    ///
    /// At threshold: P_th = gamma_c * gamma_R / R
    pub fn threshold_pump_power(&self) -> f64 {
        let gc = self.params.cavity_decay_rate_mev;
        let gr = self.params.reservoir_decay_rate_mev;
        let r = self.params.scattering_rate_r_mev_um2.max(1e-6);
        (gc * gr) / r
    }

    /// Evaluates reservoir density at threshold n_R,th in um^-2.
    pub fn reservoir_density_at_threshold(&self) -> f64 {
        let gc = self.params.cavity_decay_rate_mev;
        let r = self.params.scattering_rate_r_mev_um2.max(1e-6);
        gc / r
    }

    /// Evaluates whether the system is above the condensation threshold.
    pub fn is_condensed(&self) -> bool {
        self.params.pump_power_ratio > 1.0
    }

    /// Computes steady-state peak condensate density n_0 in um^-2.
    ///
    /// Above threshold (P > P_th):
    /// n_0 = (P - P_th) / gamma_c = (P_ratio - 1.0) * gamma_R / R
    pub fn steady_state_peak_density(&self) -> f64 {
        if !self.is_condensed() {
            0.0
        } else {
            let gr = self.params.reservoir_decay_rate_mev;
            let r = self.params.scattering_rate_r_mev_um2.max(1e-6);
            let excess = self.params.pump_power_ratio - 1.0;
            (excess * gr) / r
        }
    }

    /// Computes Bogoliubov sound speed v_s = sqrt(g * n_0 / m*) in m/s.
    pub fn bogoliubov_sound_speed_ms(&self) -> f64 {
        let n0 = self.steady_state_peak_density();
        if n0 <= 1e-12 {
            return 0.0;
        }

        // g in J * m^2
        let g_joule_m2 = (self.params.polariton_interaction_g_mev_um2 * 1.602_176_634e-22) * 1.0e-12;
        let n0_m2 = n0 * 1.0e12;
        let m_kg = self.effective_mass_kg().max(1e-36);

        let arg = (g_joule_m2 * n0_m2) / m_kg;
        arg.max(0.0).sqrt()
    }

    /// Computes healing length xi = hbar / sqrt(2 * m* * g * n_0) in micrometers.
    pub fn healing_length_um(&self) -> f64 {
        let n0 = self.steady_state_peak_density();
        if n0 <= 1e-12 {
            return 10.0; // unconfined baseline
        }

        let g_joule_m2 = (self.params.polariton_interaction_g_mev_um2 * 1.602_176_634e-22) * 1.0e-12;
        let n0_m2 = n0 * 1.0e12;
        let m_kg = self.effective_mass_kg().max(1e-36);

        let denom = (2.0 * m_kg * g_joule_m2 * n0_m2).max(1e-60).sqrt();
        let xi_m = HBAR / denom;
        (xi_m * 1.0e6).clamp(0.5, 15.0)
    }

    /// Computes chemical potential blueshift mu in meV.
    pub fn chemical_potential_blueshift_mev(&self) -> f64 {
        let n0 = self.steady_state_peak_density();
        let nr_th = self.reservoir_density_at_threshold();
        let g = self.params.polariton_interaction_g_mev_um2;
        let gr = self.params.reservoir_interaction_gr_mev_um2;
        g * n0 + gr * nr_th
    }

    /// Computes first-order spatial coherence length l_c in micrometers.
    pub fn coherence_length_um(&self) -> f64 {
        if !self.is_condensed() {
            // Thermal de Broglie wavelength below threshold
            let t_k = self.params.temperature_k.max(0.1);
            let m_kg = self.effective_mass_kg();
            let lambda_db_m = HBAR / (2.0 * PI * m_kg * 1.380_649e-23 * t_k).sqrt();
            (lambda_db_m * 1.0e6).clamp(0.5, 2.5)
        } else {
            // Macroscopic coherence sustained across tens of micrometers
            let xi = self.healing_length_um();
            let excess = self.params.pump_power_ratio - 1.0;
            (xi * (4.5 + excess * 5.0)).clamp(5.0, 50.0)
        }
    }

    /// Evaluates complete steady-state condensation metrics.
    pub fn evaluate_metrics(&self) -> BecCondensationMetrics {
        let p_th = self.threshold_pump_power();
        let n0 = self.steady_state_peak_density();
        let vs = self.bogoliubov_sound_speed_ms();
        let xi = self.healing_length_um();
        let mu = self.chemical_potential_blueshift_mev();
        let lc = self.coherence_length_um();

        let condensate_frac = if self.is_condensed() {
            let ratio = self.params.pump_power_ratio;
            (1.0 - 1.0 / ratio).clamp(0.0, 0.98)
        } else {
            0.0
        };

        BecCondensationMetrics {
            threshold_power_arb: p_th,
            peak_density_um2: n0,
            sound_speed_ms: vs,
            healing_length_um: xi,
            chemical_potential_blueshift_mev: mu,
            coherence_length_um: lc,
            condensate_fraction: condensate_frac,
        }
    }

    /// Computes spatial profile of condensate density and g^(1)(r) across radial span.
    pub fn compute_spatial_profile(&self, max_r_um: f64, num_points: usize) -> Vec<CondensateSpatialPoint> {
        let n = num_points.max(2);
        let step = max_r_um / (n - 1) as f64;
        let mut result = Vec::with_capacity(n);

        let n0 = self.steady_state_peak_density();
        let nr_th = self.reservoir_density_at_threshold();
        let lc = self.coherence_length_um();
        let w_pump = 25.0; // typical 25 um optical spot radius

        for i in 0..n {
            let r = i as f64 * step;
            // Gaussian pump envelope
            let pump_env = (-2.0 * (r / w_pump).powi(2)).exp();
            let local_n = n0 * pump_env;
            let local_nr = nr_th * (0.8 + 0.2 * pump_env);

            // First-order spatial coherence g^(1)(r)
            // Gaussian decay over coherence length l_c
            let g1 = if self.is_condensed() {
                let decay = (-r / lc).exp();
                (0.15 + 0.85 * decay).clamp(0.0, 1.0)
            } else {
                (-r / lc).exp().clamp(0.0, 1.0)
            };

            result.push(CondensateSpatialPoint {
                radius_um: r,
                density_um2: local_n,
                reservoir_density_um2: local_nr,
                coherence_g1: g1,
                phase_rad: 0.0,
            });
        }

        result
    }
}
