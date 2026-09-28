//! Shear Alfvén wave dynamics, magnetosonic dispersion, and Toroidal Alfvén Eigenmodes (TAE).

use super::constants::{PLASMA_ADIABATIC_INDEX, VACUUM_PERMEABILITY};

/// Fundamental magnetohydrodynamic wave velocities and dispersion relations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlfvenWaveProperties;

impl AlfvenWaveProperties {
    /// Shear Alfvén phase and group speed along magnetic field lines:
    /// $$v_A = \frac{B}{\sqrt{\mu_0 \rho}}$$
    pub fn alfven_speed(b_magnitude: f64, mass_density: f64) -> f64 {
        let safe_rho = mass_density.max(1e-25);
        b_magnitude / (VACUUM_PERMEABILITY * safe_rho).sqrt()
    }

    /// Ion acoustic sound speed:
    /// $$c_s = \sqrt{\frac{\gamma p}{\rho}}$$
    pub fn sound_speed(pressure_pa: f64, mass_density: f64) -> f64 {
        let safe_rho = mass_density.max(1e-25);
        ((PLASMA_ADIABATIC_INDEX * pressure_pa) / safe_rho).sqrt()
    }

    /// Compressional / Fast magnetosonic wave speed at propagation angle $\theta$ relative to $\mathbf{B}$:
    /// $$v_{fast}^2 = \frac{1}{2}\left( (v_A^2 + c_s^2) + \sqrt{(v_A^2 + c_s^2)^2 - 4 v_A^2 c_s^2 \cos^2\theta} \right)$$
    pub fn fast_magnetosonic_speed(v_a: f64, c_s: f64, theta_rad: f64) -> f64 {
        let va2 = v_a.powi(2);
        let cs2 = c_s.powi(2);
        let sum = va2 + cs2;
        let disc = (sum.powi(2) - 4.0 * va2 * cs2 * theta_rad.cos().powi(2)).max(0.0);
        (0.5 * (sum + disc.sqrt())).sqrt()
    }

    /// Slow magnetosonic wave speed at propagation angle $\theta$:
    /// $$v_{slow}^2 = \frac{1}{2}\left( (v_A^2 + c_s^2) - \sqrt{(v_A^2 + c_s^2)^2 - 4 v_A^2 c_s^2 \cos^2\theta} \right)$$
    pub fn slow_magnetosonic_speed(v_a: f64, c_s: f64, theta_rad: f64) -> f64 {
        let va2 = v_a.powi(2);
        let cs2 = c_s.powi(2);
        let sum = va2 + cs2;
        let disc = (sum.powi(2) - 4.0 * va2 * cs2 * theta_rad.cos().powi(2)).max(0.0);
        (0.5 * (sum - disc.sqrt()).max(0.0)).sqrt()
    }

    /// Parallel wavenumber $k_\parallel = \frac{m - n q(r)}{q(r) R_0}$ for mode $(m, n)$.
    pub fn k_parallel(m: usize, n: usize, q: f64, r0: f64) -> f64 {
        let safe_q = q.max(0.1);
        let safe_r0 = r0.max(0.1);
        ((m as f64) - (n as f64) * safe_q) / (safe_q * safe_r0)
    }

    /// Continuous shear Alfvén frequency $\omega_A(r) = |k_\parallel(r)| v_A(r)$.
    pub fn shear_alfven_frequency(m: usize, n: usize, q: f64, r0: f64, v_a: f64) -> f64 {
        let k_par = Self::k_parallel(m, n, q, r0);
        k_par.abs() * v_a
    }
}

/// Toroidal Alfvén Eigenmode (TAE) formed in the continuum gap created by toroidal coupling.
///
/// Toroidicity couples poloidal harmonics $m$ and $m+1$, opening a discrete eigenmode gap
/// at safety factor $q_{TAE} = \frac{m + 1/2}{n}$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToroidalAlfvenEigenmode {
    /// Poloidal mode number $m$.
    pub m: usize,
    /// Toroidal mode number $n$.
    pub n: usize,
}

impl ToroidalAlfvenEigenmode {
    /// Creates a TAE mode specification with indices $(m, n)$.
    pub fn new(m: usize, n: usize) -> Self {
        assert!(n > 0, "Toroidal mode number n must be positive for TAE");
        Self { m, n }
    }

    /// Resonant safety factor for TAE gap center $q_{TAE} = \frac{m + 1/2}{n}$.
    pub fn resonant_q(&self) -> f64 {
        ((self.m as f64) + 0.5) / (self.n as f64)
    }

    /// Center eigenfrequency of the TAE gap:
    /// $$\omega_{TAE} = \frac{v_A}{2 q_{TAE} R_0}$$
    pub fn eigenfrequency(&self, v_a: f64, r0: f64) -> f64 {
        let q_tae = self.resonant_q();
        v_a / (2.0 * q_tae * r0.max(0.1))
    }

    /// Width of the continuum gap $\Delta\omega_{gap} \approx \epsilon \cdot \omega_{TAE}$ where $\epsilon = r / R_0$.
    pub fn gap_width(&self, epsilon: f64, v_a: f64, r0: f64) -> f64 {
        let omega = self.eigenfrequency(v_a, r0);
        epsilon * omega
    }

    /// Ion Landau damping rate $\gamma_L / \omega_{TAE}$ estimate:
    /// Damping peaks when ion thermal speed $v_{th,i} \approx v_A / 3$.
    pub fn ion_landau_damping_ratio(&self, v_th_ion: f64, v_a: f64) -> f64 {
        let ratio = v_th_ion / (v_a / 3.0).max(1e-3);
        // Exponential resonance profile
        0.05 * ratio.powi(3) * (-ratio.powi(2)).exp()
    }
}
