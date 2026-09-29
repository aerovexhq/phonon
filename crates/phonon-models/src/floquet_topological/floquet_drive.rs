//! Ultrafast laser drive parameters and time-dependent vector potentials
//! for Floquet topological engineering.

use phonon_core::constants::{ELEMENTARY_CHARGE, H_BAR, SPEED_OF_LIGHT};
use std::f64::consts::PI;

/// Shape of the optical laser pulse envelope.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloquetPulseShape {
    /// Continuous-wave (CW) illumination with constant unit envelope.
    ContinuousWave,
    /// Gaussian pulse envelope: $f(t) = \exp(-4 \ln(2) t^2 / \tau^2)$.
    Gaussian {
        /// Full-width at half-maximum (FWHM) pulse duration in seconds.
        fwhm_s: f64,
    },
    /// Flat-top trapezoidal pulse with smooth cosine-tapered edges.
    FlatTop {
        /// Rise and fall time in seconds.
        ramp_s: f64,
        /// Plateau duration in seconds.
        plateau_s: f64,
    },
}

/// Parameters for ultrafast chiral laser driving in 2D Dirac materials.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetDriveParams {
    /// Laser photon energy $\hbar \Omega$ in electron-volts ($\text{eV}$).
    pub photon_energy_ev: f64,
    /// Peak electric field amplitude $E_0$ in volts per meter ($\text{V/m}$).
    pub peak_electric_field_v_m: f64,
    /// Light chirality / helicity $\sigma \in [-1.0, 1.0]$.
    /// $\sigma = +1.0$ is right-circularly polarized (RCP),
    /// $\sigma = -1.0$ is left-circularly polarized (LCP),
    /// $\sigma = 0.0$ is linearly polarized (LP).
    pub chirality: f64,
    /// Pulse envelope profile.
    pub pulse_shape: FloquetPulseShape,
}

impl FloquetDriveParams {
    /// Creates a new continuous-wave Floquet drive.
    pub fn new_cw(photon_energy_ev: f64, peak_electric_field_v_m: f64, chirality: f64) -> Self {
        Self {
            photon_energy_ev: photon_energy_ev.max(1e-3),
            peak_electric_field_v_m: peak_electric_field_v_m.max(0.0),
            chirality: chirality.clamp(-1.0, 1.0),
            pulse_shape: FloquetPulseShape::ContinuousWave,
        }
    }

    /// Creates a pulsed Floquet drive with a Gaussian envelope.
    pub fn new_gaussian(
        photon_energy_ev: f64,
        peak_electric_field_v_m: f64,
        chirality: f64,
        fwhm_s: f64,
    ) -> Self {
        Self {
            photon_energy_ev: photon_energy_ev.max(1e-3),
            peak_electric_field_v_m: peak_electric_field_v_m.max(0.0),
            chirality: chirality.clamp(-1.0, 1.0),
            pulse_shape: FloquetPulseShape::Gaussian {
                fwhm_s: fwhm_s.max(1e-18),
            },
        }
    }

    /// Photon energy in Joules: $E = \hbar \Omega$.
    #[inline]
    pub fn photon_energy_joules(&self) -> f64 {
        self.photon_energy_ev * ELEMENTARY_CHARGE
    }

    /// Optical angular frequency $\Omega = E / \hbar$ in radians per second.
    #[inline]
    pub fn angular_frequency_rad_s(&self) -> f64 {
        self.photon_energy_joules() / H_BAR
    }

    /// Optical period $T = 2\pi / \Omega$ in seconds.
    #[inline]
    pub fn optical_period_s(&self) -> f64 {
        2.0 * PI / self.angular_frequency_rad_s()
    }

    /// Vector potential amplitude $A_0 = E_0 / \Omega$ in $\text{V}\cdot\text{s/m}$ (or $\text{T}\cdot\text{m}$).
    #[inline]
    pub fn vector_potential_amplitude(&self) -> f64 {
        self.peak_electric_field_v_m / self.angular_frequency_rad_s()
    }

    /// Laser wavelength $\lambda = c / \nu$ in meters.
    #[inline]
    pub fn wavelength_m(&self) -> f64 {
        (SPEED_OF_LIGHT * 2.0 * PI) / self.angular_frequency_rad_s()
    }

    /// Dimensionless drive parameter $z = \frac{e v_F A_0}{\hbar \Omega} = \frac{e v_F E_0}{\hbar \Omega^2}$.
    #[inline]
    pub fn dimensionless_drive_z(&self, fermi_velocity_m_s: f64) -> f64 {
        let e_v_a0 = ELEMENTARY_CHARGE * fermi_velocity_m_s * self.vector_potential_amplitude();
        e_v_a0 / self.photon_energy_joules()
    }

    /// Pulse envelope amplitude factor $f(t) \in [0.0, 1.0]$.
    pub fn envelope(&self, t_s: f64) -> f64 {
        match self.pulse_shape {
            FloquetPulseShape::ContinuousWave => 1.0,
            FloquetPulseShape::Gaussian { fwhm_s } => {
                let sigma = fwhm_s / (2.0 * (2.0 * 2.0f64.ln()).sqrt());
                (-0.5 * (t_s / sigma).powi(2)).exp()
            }
            FloquetPulseShape::FlatTop { ramp_s, plateau_s } => {
                let half_plat = plateau_s * 0.5;
                let t_abs = t_s.abs();
                if t_abs <= half_plat {
                    1.0
                } else if t_abs <= half_plat + ramp_s {
                    let tau = (t_abs - half_plat) / ramp_s;
                    0.5 * (1.0 + (PI * tau).cos())
                } else {
                    0.0
                }
            }
        }
    }

    /// Evaluates the 2D vector potential $\mathbf{A}(t) = (A_x(t), A_y(t))$ in $\text{T}\cdot\text{m}$.
    #[inline]
    pub fn vector_potential_at(&self, t_s: f64) -> (f64, f64) {
        let a0 = self.vector_potential_amplitude() * self.envelope(t_s);
        let omega = self.angular_frequency_rad_s();
        let phase = omega * t_s;
        let ax = a0 * phase.cos();
        let ay = self.chirality * a0 * phase.sin();
        (ax, ay)
    }

    /// Evaluates the instantaneous electric field $\mathbf{E}(t) = -\partial \mathbf{A} / \partial t$ in $\text{V/m}$.
    #[inline]
    pub fn electric_field_at(&self, t_s: f64) -> (f64, f64) {
        let e0 = self.peak_electric_field_v_m * self.envelope(t_s);
        let omega = self.angular_frequency_rad_s();
        let phase = omega * t_s;
        let ex = e0 * phase.sin();
        let ey = -self.chirality * e0 * phase.cos();
        (ex, ey)
    }
}
