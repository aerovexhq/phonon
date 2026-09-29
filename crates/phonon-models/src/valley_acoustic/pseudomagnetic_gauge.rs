//! Strain-induced synthetic gauge vector potentials, valley pseudomagnetic fields,
//! and relativistic pseudo-Landau levels in 2D phononic crystal lattices.

use phonon_core::constants::{ELEMENTARY_CHARGE, H_BAR};

/// Valley index classification for 2D hexagonal/triangular acoustic lattices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValleyIndex {
    /// Valley $K$ with $\tau_z = +1$.
    ValleyK,
    /// Complementary valley $K'$ with $\tau_z = -1$.
    ValleyKPrime,
}

impl ValleyIndex {
    /// Returns the valley sign factor $\tau_z = \pm 1$.
    #[inline]
    pub fn tau_z(&self) -> f64 {
        match self {
            ValleyIndex::ValleyK => 1.0,
            ValleyIndex::ValleyKPrime => -1.0,
        }
    }
}

/// Parameters for strain-induced acoustic pseudomagnetic gauge fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrainGaugeParams {
    /// Acoustic lattice constant $a$ in meters (nominal $1.0\text{ nm}$ for nanoscale phononic crystals).
    pub lattice_constant_m: f64,
    /// Phonon sound velocity $v_s$ in meters per second (nominal $3000\text{ m/s}$).
    pub sound_velocity_m_s: f64,
    /// Dimensionless acoustic Grüneisen parameter $\beta = -\frac{\partial \ln t}{\partial \ln a}$ (nominal $3.5$).
    pub gruneisen_beta: f64,
    /// Triaxial strain curvature coefficient $C_s$ in $\text{m}^{-1}$.
    pub strain_coefficient_c_s: f64,
}

impl Default for StrainGaugeParams {
    fn default() -> Self {
        Self {
            lattice_constant_m: 1.0e-9,
            sound_velocity_m_s: 3000.0,
            gruneisen_beta: 3.5,
            strain_coefficient_c_s: 8.0e6,
        }
    }
}

impl StrainGaugeParams {
    /// Creates new strain gauge parameters.
    pub fn new(
        lattice_constant_m: f64,
        sound_velocity_m_s: f64,
        gruneisen_beta: f64,
        strain_coefficient_c_s: f64,
    ) -> Self {
        Self {
            lattice_constant_m: lattice_constant_m.max(1e-12),
            sound_velocity_m_s: sound_velocity_m_s.max(10.0),
            gruneisen_beta: gruneisen_beta.max(0.1),
            strain_coefficient_c_s: strain_coefficient_c_s.max(0.0),
        }
    }

    /// Computes the in-plane triaxial strain tensor $(u_{xx}, u_{yy}, u_{xy})$ at position $(x, y)$:
    /// $$u_{xx} = 2 C_s y, \quad u_{yy} = -2 C_s y, \quad u_{xy} = 2 C_s x$$
    #[inline]
    pub fn triaxial_strain_tensor(&self, x_m: f64, y_m: f64) -> (f64, f64, f64) {
        let u_xx = 2.0 * self.strain_coefficient_c_s * y_m;
        let u_yy = -2.0 * self.strain_coefficient_c_s * y_m;
        let u_xy = 2.0 * self.strain_coefficient_c_s * x_m;
        (u_xx, u_yy, u_xy)
    }

    /// Evaluates the gauge vector potential $\mathbf{A}_{\mathrm{ps}}(\mathbf{r})$ in $\text{T}\cdot\text{m}$:
    /// $$\mathbf{A}_{\mathrm{ps}} = \tau_z \frac{\hbar}{e} \frac{\beta}{a} (u_{xx} - u_{yy}, -2 u_{xy})$$
    #[inline]
    pub fn vector_potential_at(&self, valley: ValleyIndex, x_m: f64, y_m: f64) -> (f64, f64) {
        let hbar_over_e = H_BAR / ELEMENTARY_CHARGE;
        let factor = valley.tau_z() * hbar_over_e * (self.gruneisen_beta / self.lattice_constant_m);
        let (u_xx, u_yy, u_xy) = self.triaxial_strain_tensor(x_m, y_m);
        let ax = factor * (u_xx - u_yy);
        let ay = factor * (-2.0 * u_xy);
        (ax, ay)
    }

    /// Uniform valley pseudomagnetic field $B_{\mathrm{ps}} = (\nabla \times \mathbf{A}_{\mathrm{ps}})_z$ in Tesla ($T$):
    /// $$B_{\mathrm{ps}} = \tau_z \frac{\hbar}{e} \frac{8 \beta C_s}{a}$$
    #[inline]
    pub fn pseudomagnetic_field_tesla(&self, valley: ValleyIndex) -> f64 {
        let hbar_over_e = H_BAR / ELEMENTARY_CHARGE;
        valley.tau_z()
            * hbar_over_e
            * (8.0 * self.gruneisen_beta * self.strain_coefficient_c_s / self.lattice_constant_m)
    }

    /// Absolute pseudomagnetic field magnitude $|B_{\mathrm{ps}}|$ in Tesla ($T$).
    #[inline]
    pub fn pseudomagnetic_field_magnitude_tesla(&self) -> f64 {
        self.pseudomagnetic_field_tesla(ValleyIndex::ValleyK).abs()
    }

    /// Magnetic confinement length $\ell_B = \sqrt{\frac{\hbar}{e |B_{\mathrm{ps}}|}}$ in meters.
    #[inline]
    pub fn magnetic_length_m(&self) -> f64 {
        let b = self.pseudomagnetic_field_magnitude_tesla().max(1e-6);
        let hbar_over_e = H_BAR / ELEMENTARY_CHARGE;
        (hbar_over_e / b).sqrt()
    }

    /// Relativistic acoustic pseudo-Landau level angular frequency $\omega_n$ in $\text{rad/s}$:
    /// $$\omega_n = \operatorname{sgn}(n) \frac{v_s}{\ell_B} \sqrt{2 |n|}$$
    #[inline]
    pub fn landau_level_frequency_rad_s(&self, n: i32) -> f64 {
        let lb = self.magnetic_length_m();
        let sign = if n > 0 {
            1.0
        } else if n < 0 {
            -1.0
        } else {
            0.0
        };
        sign * (self.sound_velocity_m_s / lb) * (2.0 * (n.abs() as f64)).sqrt()
    }

    /// Energy spacing between $n = 0$ and $n = 1$ pseudo-Landau levels in $\text{meV}$.
    #[inline]
    pub fn landau_level_gap_mev(&self) -> f64 {
        let omega_1 = self.landau_level_frequency_rad_s(1);
        (H_BAR * omega_1 / ELEMENTARY_CHARGE) * 1000.0
    }
}
