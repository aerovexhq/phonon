//! 3D Vector Electromagnetic Waves & Maxwell Poynting Radiation.
//!
//! Provides mathematically rigorous 3D vector electromagnetic fields,
//! polarization models, complex field vectors, and Poynting energy flux
//! calculations according to Maxwell equations.

use phonon_core::constants::SPEED_OF_LIGHT;
use std::f64::consts::PI;

/// Characteristic intrinsic impedance of free space: $\\eta_0 = \\sqrt{\\mu_0 / \\epsilon_0} \\approx 376.73\\,\\Omega$.
pub const INTRINSIC_IMPEDANCE_VACUUM: f64 = 376.730_313_668;

/// 3D Cartesian Vector with vector algebra operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3D {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[inline]
    pub fn norm_squared(&self) -> f64 {
        self.dot(self)
    }

    #[inline]
    pub fn norm(&self) -> f64 {
        self.norm_squared().sqrt()
    }

    #[inline]
    pub fn normalize(&self) -> Self {
        let n = self.norm();
        if n > 1e-15 {
            Self {
                x: self.x / n,
                y: self.y / n,
                z: self.z / n,
            }
        } else {
            Self::ZERO
        }
    }

    #[inline]
    pub fn distance(&self, other: &Self) -> f64 {
        (*self - *other).norm()
    }

    #[inline]
    pub fn scale(&self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    #[inline]
    pub fn magnitude(&self) -> f64 {
        self.norm()
    }

    #[inline]
    pub fn angle_to(&self, other: &Self) -> f64 {
        let denom = self.norm() * other.norm();
        if denom < 1e-15 {
            0.0
        } else {
            let cos_theta = (self.dot(other) / denom).clamp(-1.0, 1.0);
            cos_theta.acos()
        }
    }
}

impl std::ops::Add for Vector3D {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub for Vector3D {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Neg for Vector3D {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl std::ops::Mul<f64> for Vector3D {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl std::ops::Mul<Vector3D> for f64 {
    type Output = Vector3D;
    #[inline]
    fn mul(self, rhs: Vector3D) -> Vector3D {
        Vector3D {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl std::ops::Div<f64> for Vector3D {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

/// Complex 3D Electromagnetic Field Vector: $[E_x, E_y, E_z]$ where each component has real and imaginary parts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComplexField3D {
    pub x: (f64, f64),
    pub y: (f64, f64),
    pub z: (f64, f64),
}

impl ComplexField3D {
    pub const ZERO: Self = Self {
        x: (0.0, 0.0),
        y: (0.0, 0.0),
        z: (0.0, 0.0),
    };

    #[inline]
    pub const fn new(x: (f64, f64), y: (f64, f64), z: (f64, f64)) -> Self {
        Self { x, y, z }
    }

    /// Computes the magnitude (RMS amplitude) of the field vector: $\\sqrt{|E_x|^2 + |E_y|^2 + |E_z|^2}$.
    #[inline]
    pub fn magnitude(&self) -> f64 {
        let mag_sq = (self.x.0 * self.x.0 + self.x.1 * self.x.1)
            + (self.y.0 * self.y.0 + self.y.1 * self.y.1)
            + (self.z.0 * self.z.0 + self.z.1 * self.z.1);
        mag_sq.sqrt()
    }

    /// Scales all components by a complex factor $(a_r, a_i)$.
    #[inline]
    pub fn scale_complex(&self, factor: (f64, f64)) -> Self {
        let mul = |c: (f64, f64)| -> (f64, f64) {
            (
                c.0 * factor.0 - c.1 * factor.1,
                c.0 * factor.1 + c.1 * factor.0,
            )
        };
        Self {
            x: mul(self.x),
            y: mul(self.y),
            z: mul(self.z),
        }
    }

    /// Scales all components by a real scalar.
    #[inline]
    pub fn scale_real(&self, s: f64) -> Self {
        Self {
            x: (self.x.0 * s, self.x.1 * s),
            y: (self.y.0 * s, self.y.1 * s),
            z: (self.z.0 * s, self.z.1 * s),
        }
    }

    /// Vector addition of two complex fields.
    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: (self.x.0 + other.x.0, self.x.1 + other.x.1),
            y: (self.y.0 + other.y.0, self.y.1 + other.y.1),
            z: (self.z.0 + other.z.0, self.z.1 + other.z.1),
        }
    }
}

/// Electromagnetic Wave Polarization Mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Polarization {
    /// Linear Horizontal (parallel to X-Y ground plane).
    LinearHorizontal,
    /// Linear Vertical (parallel to Z axis).
    LinearVertical,
    /// Linear polarization at an arbitrary angle $\\psi$ (in radians) relative to horizontal.
    LinearSlant(f64),
    /// Right-Hand Circular Polarization (RHCP).
    CircularRight,
    /// Left-Hand Circular Polarization (LHCP).
    CircularLeft,
}

impl Polarization {
    /// Computes the polarization mismatch factor $L_{pol} \\in [0.0, 1.0]$ between TX and RX antennas.
    pub fn mismatch_factor(&self, other: &Self) -> f64 {
        match (self, other) {
            (Self::LinearHorizontal, Self::LinearHorizontal) => 1.0,
            (Self::LinearVertical, Self::LinearVertical) => 1.0,
            (Self::LinearSlant(a), Self::LinearSlant(b)) => (a - b).cos().powi(2),

            (Self::LinearHorizontal, Self::LinearVertical)
            | (Self::LinearVertical, Self::LinearHorizontal) => 0.0,

            (Self::LinearHorizontal, Self::LinearSlant(psi))
            | (Self::LinearSlant(psi), Self::LinearHorizontal) => psi.cos().powi(2),
            (Self::LinearVertical, Self::LinearSlant(psi))
            | (Self::LinearSlant(psi), Self::LinearVertical) => psi.sin().powi(2),

            (Self::CircularRight, Self::CircularRight) => 1.0,
            (Self::CircularLeft, Self::CircularLeft) => 1.0,

            (Self::CircularRight, Self::CircularLeft)
            | (Self::CircularLeft, Self::CircularRight) => 0.0,

            (Self::CircularRight, Self::LinearHorizontal)
            | (Self::CircularRight, Self::LinearVertical)
            | (Self::CircularRight, Self::LinearSlant(_))
            | (Self::CircularLeft, Self::LinearHorizontal)
            | (Self::CircularLeft, Self::LinearVertical)
            | (Self::CircularLeft, Self::LinearSlant(_))
            | (Self::LinearHorizontal, Self::CircularRight)
            | (Self::LinearVertical, Self::CircularRight)
            | (Self::LinearSlant(_), Self::CircularRight)
            | (Self::LinearHorizontal, Self::CircularLeft)
            | (Self::LinearVertical, Self::CircularLeft)
            | (Self::LinearSlant(_), Self::CircularLeft) => 0.5,
        }
    }
}

/// 3D Vector Electromagnetic Wave Source (Transmitter).
#[derive(Debug, Clone, PartialEq)]
pub struct EmWaveSource {
    /// Carrier frequency in Hertz ($Hz$).
    pub frequency_hz: f64,
    /// Transmitted RF power in Watts ($W$).
    pub tx_power_watts: f64,
    /// Transmit antenna gain (dimensionless linear ratio, $G_t \\ge 1.0$).
    pub tx_gain_linear: f64,
    /// Antenna polarization mode.
    pub polarization: Polarization,
    /// 3D position in Cartesian space (meters).
    pub position: Vector3D,
    /// Radiation boresight direction unit vector.
    pub boresight: Vector3D,
}

impl EmWaveSource {
    /// Creates a new electromagnetic wave source.
    pub fn new(
        frequency_hz: f64,
        tx_power_watts: f64,
        tx_gain_linear: f64,
        polarization: Polarization,
        position: Vector3D,
        boresight: Vector3D,
    ) -> Self {
        Self {
            frequency_hz: frequency_hz.max(1.0),
            tx_power_watts: tx_power_watts.max(0.0),
            tx_gain_linear: tx_gain_linear.max(0.01),
            polarization,
            position,
            boresight: boresight.normalize(),
        }
    }

    /// Wavelength in vacuum $\\lambda = c / f$ (meters).
    #[inline]
    pub fn wavelength(&self) -> f64 {
        SPEED_OF_LIGHT / self.frequency_hz
    }

    /// Wavenumber in vacuum $k = 2\\pi / \\lambda$ (rad/m).
    #[inline]
    pub fn wavenumber(&self) -> f64 {
        2.0 * PI / self.wavelength()
    }

    /// Angular frequency $\\omega = 2\\pi f$ (rad/s).
    #[inline]
    pub fn angular_frequency(&self) -> f64 {
        2.0 * PI * self.frequency_hz
    }

    /// Equivalent Isotropically Radiated Power (EIRP) in Watts: $EIRP = P_t \\cdot G_t$.
    #[inline]
    pub fn eirp_watts(&self) -> f64 {
        self.tx_power_watts * self.tx_gain_linear
    }

    /// EIRP in decibel-milliwatts ($\\text{dBm}$).
    #[inline]
    pub fn eirp_dbm(&self) -> f64 {
        10.0 * (self.eirp_watts() * 1000.0).log10()
    }

    /// Time-averaged Poynting vector power density $S(d) = \\frac{P_t G_t}{4\\pi d^2}\\text{ (W/m}^2\\text{)}$ at distance $d$.
    pub fn poynting_flux_at_distance(&self, distance_m: f64) -> f64 {
        let d = distance_m.max(1e-3);
        self.eirp_watts() / (4.0 * PI * d * d)
    }

    /// Evaluates the complex electric field vector $\\mathbf{E}(\\mathbf{r})$ at target position $\\mathbf{r}$.
    pub fn electric_field_at(&self, target_pos: &Vector3D) -> ComplexField3D {
        let disp = *target_pos - self.position;
        let d = disp.norm().max(1e-3);
        let k = self.wavenumber();
        let phase = -k * d;

        let e_amp = (INTRINSIC_IMPEDANCE_VACUUM * self.poynting_flux_at_distance(d)).sqrt();

        let cos_p = phase.cos();
        let sin_p = phase.sin();

        let (ex_factor, ey_factor, ez_factor) = match self.polarization {
            Polarization::LinearHorizontal => (1.0, 0.0, 0.0),
            Polarization::LinearVertical => (0.0, 0.0, 1.0),
            Polarization::LinearSlant(psi) => (psi.cos(), 0.0, psi.sin()),
            Polarization::CircularRight => {
                let frac = 1.0 / std::f64::consts::SQRT_2;
                return ComplexField3D::new(
                    (e_amp * frac * cos_p, e_amp * frac * sin_p),
                    (e_amp * frac * sin_p, -e_amp * frac * cos_p),
                    (0.0, 0.0),
                );
            }
            Polarization::CircularLeft => {
                let frac = 1.0 / std::f64::consts::SQRT_2;
                return ComplexField3D::new(
                    (e_amp * frac * cos_p, e_amp * frac * sin_p),
                    (-e_amp * frac * sin_p, e_amp * frac * cos_p),
                    (0.0, 0.0),
                );
            }
        };

        ComplexField3D::new(
            (e_amp * ex_factor * cos_p, e_amp * ex_factor * sin_p),
            (e_amp * ey_factor * cos_p, e_amp * ey_factor * sin_p),
            (e_amp * ez_factor * cos_p, e_amp * ez_factor * sin_p),
        )
    }

    /// Free-space path loss (FSPL) in decibels at distance $d$.
    pub fn free_space_path_loss_db(&self, distance_m: f64) -> f64 {
        let d = distance_m.max(1e-3);
        let lambda = self.wavelength();
        20.0 * (4.0 * PI * d / lambda).log10()
    }
}
