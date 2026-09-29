//! Semiclassical Peierls-Boltzmann thermal Hall transport solver.
//!
//! # Physical Formalism
//! - Anomalous Wavepacket Velocity:
//!   $$\vec{v}_n(\vec{k}) = \frac{1}{\hbar} \nabla_{\vec{k}} \omega_n(\vec{k}) - \dot{\vec{k}} \times \vec{\Omega}_n(\vec{k})$$
//! - Longitudinal Phonon Thermal Conductivity:
//!   $$\kappa_{xx}(T) = \frac{1}{2 V} \sum_{n, \vec{k}} c_v(\omega_{n,\vec{k}}, T) v_{g, x}^2(\vec{k}) \tau_p$$
//!   where $c_v(\omega, T) = k_B \left(\frac{\hbar\omega}{k_B T}\right)^2 \frac{e^{\hbar\omega/k_B T}}{(e^{\hbar\omega/k_B T} - 1)^2}$.
//! - Transverse Phonon Thermal Hall Conductivity:
//!   $$\kappa_{xy}(T) = -\frac{k_B^2 T}{\hbar V} \sum_{n, \vec{k}} c_2(f_B(\omega_{n,\vec{k}}, T)) \Omega_{n,z}(\vec{k})$$
//!   where $c_2(x) = (1+x)\ln^2\left(\frac{1+x}{x}\right) - \ln^2(x) - 2 \mathrm{Li}_2(-x)$.
//! - Phonon Hall Angle:
//!   $$\theta_{\mathrm{TH}} = \arctan\left(\frac{\kappa_{xy}}{\kappa_{xx}}\right)$$

use phonon_models::chiral_phonon::{HoneycombChiralLattice, Wavevector2D};
use std::f64::consts::PI;

const HBAR: f64 = 1.054_571_817e-34; // J*s
const KB: f64 = 1.380_649e-23; // J/K

/// Result of Peierls-Boltzmann thermal Hall conductivity calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct ThermalHallResult {
    /// Temperature in Kelvin.
    pub temperature_k: f64,
    /// Longitudinal thermal conductivity $\kappa_{xx}$ in $\mathrm{W / (m \cdot K)}$.
    pub kappa_xx_w_per_m_k: f64,
    /// Transverse thermal Hall conductivity $\kappa_{xy}$ in $\mathrm{W / (m \cdot K)}$.
    pub kappa_xy_w_per_m_k: f64,
    /// Phonon Hall angle $\theta_{\mathrm{TH}} = \arctan(\kappa_{xy}/\kappa_{xx})$ in radians.
    pub thermal_hall_angle_rad: f64,
    /// Total volumetric lattice heat capacity $C_v$ in $\mathrm{J / (m^3 \cdot K)}$.
    pub heat_capacity_j_per_m3_k: f64,
    /// Topological Chern number $\mathcal{C}_{ph}$.
    pub acoustic_chern_number: i32,
}

/// Semiclassical Peierls-Boltzmann phonon transport solver.
#[derive(Debug, Clone, PartialEq)]
pub struct ThermalHallBoltzmannSolver {
    pub lattice: HoneycombChiralLattice,
    /// Phonon relaxation time $\tau_p$ in seconds (typically $10^{-11} - 10^{-10}\text{ s}$).
    pub relaxation_time_s: f64,
    /// Effective monolayer thickness in meters (e.g. 0.5 nm).
    pub layer_thickness_m: f64,
}

impl ThermalHallBoltzmannSolver {
    /// Creates a new Boltzmann thermal Hall solver for a chiral honeycomb lattice.
    pub fn new(lattice: HoneycombChiralLattice) -> Self {
        Self {
            lattice,
            relaxation_time_s: 2.0e-11, // 20 ps
            layer_thickness_m: 5.0e-10, // 0.5 nm
        }
    }

    /// Evaluates mode heat capacity $c_v(\omega, T)$:
    #[inline]
    pub fn mode_heat_capacity(omega: f64, t: f64) -> f64 {
        let t_safe = t.max(1e-3);
        let x = (HBAR * omega) / (KB * t_safe);
        if x > 50.0 {
            0.0
        } else if x < 1e-4 {
            KB
        } else {
            let exp_x = x.exp();
            KB * x.powi(2) * exp_x / (exp_x - 1.0).powi(2)
        }
    }

    /// Evaluates Bose-Einstein distribution function $f_B(\omega, T)$:
    #[inline]
    pub fn bose_einstein(omega: f64, t: f64) -> f64 {
        let t_safe = t.max(1e-3);
        let x = (HBAR * omega) / (KB * t_safe);
        if x > 50.0 {
            0.0
        } else if x < 1e-4 {
            1.0 / x
        } else {
            1.0 / (x.exp() - 1.0)
        }
    }

    /// Evaluates the Berry curvature weighting function $c_2(x)$:
    /// $$c_2(x) = (1+x)\ln^2\left(\frac{1+x}{x}\right) - \ln^2(x) - 2 \mathrm{Li}_2(-x)$$
    pub fn c2_weight(x: f64) -> f64 {
        if x <= 1e-6 {
            return 0.0;
        }
        if x > 100.0 {
            // Asymptotic expansion for classical limit x >> 1:
            // c2(x) ~ 2 + ln(x)
            return 2.0 + x.ln();
        }

        let term1 = (1.0 + x) * ((1.0 + x) / x).ln().powi(2);
        let term2 = x.ln().powi(2);

        // Approximate Dilogarithm -Li2(-x) = int_0^x ln(1+t)/t dt using 8-point Simpson rule
        let n = 8;
        let dt = x / (n as f64);
        let mut simpson_integral = 0.0;
        for i in 0..=n {
            let t = i as f64 * dt;
            let val = if t < 1e-7 {
                1.0 - 0.5 * t
            } else {
                (1.0 + t).ln() / t
            };
            let weight = if i == 0 || i == n {
                1.0
            } else if i % 2 == 1 {
                4.0
            } else {
                2.0
            };
            simpson_integral += weight * val;
        }
        let li2_neg_x = -(simpson_integral * dt / 3.0);

        (term1 - term2 - 2.0 * li2_neg_x).max(0.0)
    }

    /// Solves for $\kappa_{xx}$, $\kappa_{xy}$, and thermal Hall angle at temperature $T$.
    pub fn solve(&self, temperature_k: f64, grid_size: usize) -> ThermalHallResult {
        let a = self.lattice.params.lattice_constant_m;
        let k_max = 2.0 * PI / (a * 3.0f64.sqrt());
        let dk = (2.0 * k_max) / (grid_size as f64);
        let area_cell = (3.0f64.sqrt() / 2.0) * a.powi(2);
        let vol_eff = area_cell * self.layer_thickness_m;

        let mut sum_kxx = 0.0;
        let mut sum_kxy = 0.0;
        let mut sum_cv = 0.0;

        for i in 0..grid_size {
            let kx = -k_max + (i as f64 + 0.5) * dk;
            for j in 0..grid_size {
                let ky = -k_max + (j as f64 + 0.5) * dk;
                let k = Wavevector2D::new(kx, ky);
                let mode = self.lattice.evaluate_mode(&k);

                // For the split circular modes (omega_+ and omega_-)
                for (omega, sign) in [(mode.omega_plus_rad_s, 1.0), (mode.omega_minus_rad_s, -1.0)]
                {
                    let cv = Self::mode_heat_capacity(omega, temperature_k);
                    let f = Self::bose_einstein(omega, temperature_k);
                    let c2 = Self::c2_weight(f);

                    let vx = mode.group_velocity_m_s.0;
                    sum_kxx += cv * vx.powi(2) * self.relaxation_time_s;
                    sum_cv += cv;

                    // Transverse Berry flux contribution:
                    let omega_z = sign * mode.berry_curvature_m2;
                    sum_kxy += c2 * omega_z;
                }
            }
        }

        let num_k = (grid_size * grid_size) as f64;
        let kappa_xx = (sum_kxx / (num_k * vol_eff)).max(1e-4);
        let prefactor_xy = (KB.powi(2) * temperature_k) / (HBAR * vol_eff * num_k);
        let kappa_xy = prefactor_xy * sum_kxy;
        let thermal_hall_angle = (kappa_xy / kappa_xx).atan();
        let heat_capacity = sum_cv / (num_k * vol_eff);
        let chern = self.lattice.compute_chern_number(grid_size);

        ThermalHallResult {
            temperature_k,
            kappa_xx_w_per_m_k: kappa_xx,
            kappa_xy_w_per_m_k: kappa_xy,
            thermal_hall_angle_rad: thermal_hall_angle,
            heat_capacity_j_per_m3_k: heat_capacity,
            acoustic_chern_number: chern,
        }
    }
}
