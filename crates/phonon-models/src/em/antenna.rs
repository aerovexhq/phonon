//! Physical Antenna Transducer Models
//!
//! Provides first-principles models for electromagnetic antennas, including:
//! - Half-wave dipoles and quarter-wave monopoles (Balanis analytical formulation)
//! - Microstrip patch antennas (cavity model with dielectric substrate)
//! - Pyramidal horn antennas and parabolic dish reflectors
//! - 2D Phased Array beamformers with electronic steering
//! - Radiation resistance, high-frequency skin effect ohmic loss, and radiation efficiency
//! - Fraunhofer far-field / Fresnel / reactive near-field boundary criteria

use crate::em::vector_wave::Polarization;
use phonon_core::constants::SPEED_OF_LIGHT;
use std::f64::consts::PI;

/// Permeability of free space $\mu_0$ in H/m.
pub const VACUUM_PERMEABILITY: f64 = 4.0 * PI * 1.0e-7;

/// Characteristic impedance of free space $\eta_0 \approx 376.73\,\Omega$.
pub const VACUUM_IMPEDANCE: f64 = 376.730_313_668;

/// Antenna element geometry and radiation archetype.
#[derive(Debug, Clone, PartialEq)]
pub enum AntennaGeometry {
    /// Ideal isotropic radiator (0 dBi directivity across all angles).
    Isotropic,
    /// Center-fed wire dipole antenna of physical length $L$ and wire radius $a$.
    Dipole {
        /// Total wire length in meters (typically $\lambda / 2$).
        length_m: f64,
        /// Conductor wire radius in meters.
        wire_radius_m: f64,
        /// Conductor bulk conductivity in S/m (e.g. Copper: 5.8e7).
        conductivity: f64,
    },
    /// Quarter-wave vertical monopole of height $h$ mounted over an infinite conductive ground plane.
    Monopole {
        /// Height of vertical whip in meters (typically $\lambda / 4$).
        height_m: f64,
        /// Conductor wire radius in meters.
        wire_radius_m: f64,
        /// Conductor bulk conductivity in S/m.
        conductivity: f64,
    },
    /// Rectangular microstrip patch antenna over a dielectric substrate with ground plane.
    MicrostripPatch {
        /// Resonant patch length in meters.
        length_m: f64,
        /// Patch width in meters.
        width_m: f64,
        /// Relative permittivity $\epsilon_r$ of dielectric substrate.
        substrate_er: f64,
        /// Substrate height/thickness in meters.
        substrate_height_m: f64,
        /// Conductor conductivity in S/m.
        conductivity: f64,
    },
    /// Pyramidal aperture horn antenna.
    PyramidalHorn {
        /// Aperture width (H-plane) in meters.
        aperture_a_m: f64,
        /// Aperture height (E-plane) in meters.
        aperture_b_m: f64,
        /// Axial horn flare length in meters.
        axial_length_m: f64,
        /// Aperture illumination efficiency $\eta_{ap} \in [0.4, 0.8]$ (typically ~0.51 for standard gain horns).
        aperture_efficiency: f64,
    },
    /// Parabolic reflector dish antenna.
    ParabolicDish {
        /// Circular dish aperture diameter in meters.
        diameter_m: f64,
        /// Aperture efficiency $\eta_{ap} \in [0.5, 0.75]$ (typically ~0.60).
        aperture_efficiency: f64,
    },
    /// 2D Planar Phased Array with electronic beam steering.
    PhasedArray2D {
        /// Number of antenna elements along X axis.
        num_elements_x: usize,
        /// Number of antenna elements along Y axis.
        num_elements_y: usize,
        /// Element spacing along X axis in meters (typically $\lambda / 2$).
        spacing_x_m: f64,
        /// Element spacing along Y axis in meters (typically $\lambda / 2$).
        spacing_y_m: f64,
        /// Beam steering polar elevation angle $\theta_0$ in radians ($0 = \text{broadside / +Z}$).
        steer_theta_rad: f64,
        /// Beam steering azimuth angle $\phi_0$ in radians.
        steer_phi_rad: f64,
    },
}

/// Comprehensive physical antenna model with impedance, efficiency, and 3D directivity.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalAntenna {
    /// Descriptive name or model number.
    pub name: String,
    /// Operating center carrier frequency in Hz.
    pub carrier_frequency_hz: f64,
    /// Antenna radiating geometry and aperture structure.
    pub geometry: AntennaGeometry,
    /// Transmitted / received polarization state.
    pub polarization: Polarization,
}

impl PhysicalAntenna {
    /// Creates a new physical antenna tuned for the specified carrier frequency.
    pub fn new(
        name: impl Into<String>,
        carrier_frequency_hz: f64,
        geometry: AntennaGeometry,
        polarization: Polarization,
    ) -> Self {
        Self {
            name: name.into(),
            carrier_frequency_hz,
            geometry,
            polarization,
        }
    }

    /// Creates a resonant half-wave dipole tuned for the given frequency using copper wire.
    pub fn half_wave_dipole(name: impl Into<String>, frequency_hz: f64) -> Self {
        let lambda = SPEED_OF_LIGHT / frequency_hz;
        let length_m = 0.5 * lambda * 0.95; // 0.95 velocity factor for finite thickness wire
        let wire_radius_m = (length_m * 0.001_f64).max(0.0005);
        Self {
            name: name.into(),
            carrier_frequency_hz: frequency_hz,
            geometry: AntennaGeometry::Dipole {
                length_m,
                wire_radius_m,
                conductivity: 5.8e7, // Copper
            },
            polarization: Polarization::LinearVertical,
        }
    }

    /// Creates a resonant quarter-wave monopole over ground plane.
    pub fn quarter_wave_monopole(name: impl Into<String>, frequency_hz: f64) -> Self {
        let lambda = SPEED_OF_LIGHT / frequency_hz;
        let height_m = 0.25 * lambda * 0.95;
        let wire_radius_m = (height_m * 0.002_f64).max(0.0005);
        Self {
            name: name.into(),
            carrier_frequency_hz: frequency_hz,
            geometry: AntennaGeometry::Monopole {
                height_m,
                wire_radius_m,
                conductivity: 5.8e7,
            },
            polarization: Polarization::LinearVertical,
        }
    }

    /// Creates a standard Wi-Fi / SDR microstrip patch antenna on FR4 substrate ($\epsilon_r = 4.4$, $h = 1.6\text{ mm}$).
    pub fn microstrip_patch(
        name: impl Into<String>,
        frequency_hz: f64,
        substrate_er: f64,
        substrate_height_m: f64,
    ) -> Self {
        let _lambda = SPEED_OF_LIGHT / frequency_hz;
        // Width for efficient radiation
        let width_m = (SPEED_OF_LIGHT / (2.0 * frequency_hz)) * (2.0 / (substrate_er + 1.0)).sqrt();
        // Effective dielectric constant
        let ee = (substrate_er + 1.0) / 2.0
            + ((substrate_er - 1.0) / 2.0)
                * (1.0 + 12.0 * (substrate_height_m / width_m)).powf(-0.5);
        // Fringing extension
        let dl = substrate_height_m * 0.412 * ((ee + 0.3) * (width_m / substrate_height_m + 0.264))
            / ((ee - 0.258) * (width_m / substrate_height_m + 0.8));
        let length_m = (SPEED_OF_LIGHT / (2.0 * frequency_hz * ee.sqrt())) - 2.0 * dl;

        Self {
            name: name.into(),
            carrier_frequency_hz: frequency_hz,
            geometry: AntennaGeometry::MicrostripPatch {
                length_m: length_m.max(0.001),
                width_m: width_m.max(0.001),
                substrate_er,
                substrate_height_m,
                conductivity: 5.8e7,
            },
            polarization: Polarization::LinearHorizontal,
        }
    }

    /// Creates a 2D phased array beamformer with $M \times N$ elements.
    pub fn phased_array(
        name: impl Into<String>,
        frequency_hz: f64,
        elements_x: usize,
        elements_y: usize,
        steer_theta_rad: f64,
        steer_phi_rad: f64,
    ) -> Self {
        let lambda = SPEED_OF_LIGHT / frequency_hz;
        let spacing = 0.5 * lambda;
        Self {
            name: name.into(),
            carrier_frequency_hz: frequency_hz,
            geometry: AntennaGeometry::PhasedArray2D {
                num_elements_x: elements_x.max(1),
                num_elements_y: elements_y.max(1),
                spacing_x_m: spacing,
                spacing_y_m: spacing,
                steer_theta_rad,
                steer_phi_rad,
            },
            polarization: Polarization::LinearVertical,
        }
    }

    /// Operating free-space wavelength in meters ($\lambda = c / f$).
    #[inline]
    pub fn wavelength_m(&self) -> f64 {
        SPEED_OF_LIGHT / self.carrier_frequency_hz
    }

    /// Wavenumber $k = \frac{2\pi}{\lambda}$ in rad/m.
    #[inline]
    pub fn wavenumber(&self) -> f64 {
        2.0 * PI * self.carrier_frequency_hz / SPEED_OF_LIGHT
    }

    /// Maximum physical dimension (largest aperture or wire length) in meters.
    pub fn largest_dimension_m(&self) -> f64 {
        match &self.geometry {
            AntennaGeometry::Isotropic => 0.0,
            AntennaGeometry::Dipole { length_m, .. } => *length_m,
            AntennaGeometry::Monopole { height_m, .. } => *height_m * 2.0, // Image source equivalent
            AntennaGeometry::MicrostripPatch {
                length_m, width_m, ..
            } => length_m.hypot(*width_m),
            AntennaGeometry::PyramidalHorn {
                aperture_a_m,
                aperture_b_m,
                axial_length_m,
                ..
            } => aperture_a_m.hypot(*aperture_b_m).max(*axial_length_m),
            AntennaGeometry::ParabolicDish { diameter_m, .. } => *diameter_m,
            AntennaGeometry::PhasedArray2D {
                num_elements_x,
                num_elements_y,
                spacing_x_m,
                spacing_y_m,
                ..
            } => {
                let dx = (*num_elements_x as f64) * spacing_x_m;
                let dy = (*num_elements_y as f64) * spacing_y_m;
                dx.hypot(dy)
            }
        }
    }

    /// Fraunhofer far-field boundary distance $R_{ff} = \frac{2 D^2}{\lambda}$ in meters.
    ///
    /// At distances $R > R_{ff}$, spherical wave wavefront curvature is less than $\pi/8$ radians,
    /// permitting standard far-field Friis transmission calculations.
    pub fn far_field_distance_m(&self) -> f64 {
        let d = self.largest_dimension_m();
        let lambda = self.wavelength_m();
        if d <= 0.0 {
            0.0
        } else {
            (2.0 * d * d / lambda).max(lambda / (2.0 * PI))
        }
    }

    /// Reactive near-field boundary distance $R_{nf} = 0.62 \sqrt{D^3 / \lambda}$ or $\frac{\lambda}{2\pi}$.
    pub fn reactive_near_field_distance_m(&self) -> f64 {
        let d = self.largest_dimension_m();
        let lambda = self.wavelength_m();
        if d <= 0.0 {
            0.0
        } else if d > lambda {
            0.62 * (d * d * d / lambda).sqrt()
        } else {
            lambda / (2.0 * PI)
        }
    }

    /// Antenna radiation resistance $R_{rad}$ in Ohms ($\Omega$).
    pub fn radiation_resistance_ohms(&self) -> f64 {
        let lambda = self.wavelength_m();
        match &self.geometry {
            AntennaGeometry::Isotropic => VACUUM_IMPEDANCE / (4.0 * PI), // ~30 Ohms
            AntennaGeometry::Dipole { length_m, .. } => {
                let l_over_lambda = length_m / lambda;
                if (l_over_lambda - 0.5).abs() < 0.1 {
                    // Resonant half-wave dipole: R_rad = 73.13 Ohms
                    73.13
                } else if l_over_lambda < 0.1 {
                    // Short electric dipole: R_rad = 20 * pi^2 * (l/lambda)^2
                    20.0 * PI * PI * l_over_lambda.powi(2)
                } else {
                    // Approximate sinusoidal current distribution integration
                    let kl = 2.0 * PI * l_over_lambda;
                    73.13 * (kl / PI).powi(2)
                }
            }
            AntennaGeometry::Monopole { height_m, .. } => {
                let h_over_lambda = height_m / lambda;
                if (h_over_lambda - 0.25).abs() < 0.05 {
                    // Resonant quarter-wave monopole: R_rad = 36.56 Ohms
                    36.56
                } else {
                    10.0 * PI * PI * (2.0 * h_over_lambda).powi(2)
                }
            }
            AntennaGeometry::MicrostripPatch { width_m, .. } => {
                // Approximate radiation resistance via slot conductance G_1
                let k0 = self.wavenumber();
                let g1 = if k0 * width_m > 0.1 {
                    width_m / (120.0 * lambda) * (1.0 - (k0 * width_m).powi(2) / 24.0)
                } else {
                    (k0 * width_m).powi(2) / (120.0 * PI * PI)
                };
                let r_in = 1.0 / (2.0 * g1.max(1e-6));
                r_in.clamp(50.0, 300.0)
            }
            AntennaGeometry::PyramidalHorn { .. } => VACUUM_IMPEDANCE, // Matched aperture ~377 Ohms
            AntennaGeometry::ParabolicDish { .. } => VACUUM_IMPEDANCE,
            AntennaGeometry::PhasedArray2D {
                num_elements_x,
                num_elements_y,
                ..
            } => {
                // Active input impedance per port typically matched to 50 Ohms
                50.0 / (*num_elements_x * *num_elements_y) as f64
            }
        }
    }

    /// Ohmic surface dissipation loss resistance $R_{loss}$ in Ohms ($\Omega$) caused by skin effect.
    pub fn loss_resistance_ohms(&self) -> f64 {
        let f = self.carrier_frequency_hz;
        match &self.geometry {
            AntennaGeometry::Isotropic => 0.0,
            AntennaGeometry::Dipole {
                length_m,
                wire_radius_m,
                conductivity,
            } => {
                // High-frequency skin surface resistance: R_s = sqrt(pi * f * mu / sigma)
                let r_s = (PI * f * VACUUM_PERMEABILITY / conductivity).sqrt();
                // Loss resistance R_loss = R_s * l / (2 * pi * a)
                (r_s * length_m / (2.0 * PI * wire_radius_m)).max(0.001)
            }
            AntennaGeometry::Monopole {
                height_m,
                wire_radius_m,
                conductivity,
            } => {
                let r_s = (PI * f * VACUUM_PERMEABILITY / conductivity).sqrt();
                (r_s * height_m / (2.0 * PI * wire_radius_m)).max(0.0005)
            }
            AntennaGeometry::MicrostripPatch {
                length_m,
                width_m,
                conductivity,
                ..
            } => {
                let r_s = (PI * f * VACUUM_PERMEABILITY / conductivity).sqrt();
                r_s * (length_m / width_m)
            }
            AntennaGeometry::PyramidalHorn { .. } => 0.05, // Negligible metallic flare losses
            AntennaGeometry::ParabolicDish { .. } => 0.02,
            AntennaGeometry::PhasedArray2D { .. } => 0.1,
        }
    }

    /// Antenna radiation efficiency $\eta_{rad} = \frac{R_{rad}}{R_{rad} + R_{loss}} \in [0.0, 1.0]$.
    #[inline]
    pub fn radiation_efficiency(&self) -> f64 {
        let r_rad = self.radiation_resistance_ohms();
        let r_loss = self.loss_resistance_ohms();
        let total = r_rad + r_loss;
        if total <= 0.0 {
            1.0
        } else {
            (r_rad / total).clamp(0.0, 1.0)
        }
    }

    /// Beam solid angle $\Omega_A = \iint_{4\pi} F(\theta, \phi) d\Omega$ in steradians.
    pub fn beam_solid_angle(&self) -> f64 {
        match &self.geometry {
            AntennaGeometry::Isotropic => 4.0 * PI,
            AntennaGeometry::Dipole { .. } => 7.658_114,
            AntennaGeometry::Monopole { .. } => 3.829_057,
            _ => {
                let n_th = 36;
                let n_ph = 72;
                let d_th = PI / (n_th as f64);
                let d_ph = (2.0 * PI) / (n_ph as f64);
                let mut omega = 0.0;
                for i in 0..n_th {
                    let th = (i as f64 + 0.5) * d_th;
                    let sin_th = th.sin();
                    for j in 0..n_ph {
                        let ph = (j as f64) * d_ph;
                        let f = self.normalized_power_pattern(th, ph);
                        omega += f * sin_th * d_th * d_ph;
                    }
                }
                omega.max(1e-6)
            }
        }
    }

    /// Peak maximum directivity $D_{max} = \frac{4\pi}{\Omega_A}$ (dimensionless linear scale).
    pub fn max_directivity_linear(&self) -> f64 {
        match &self.geometry {
            AntennaGeometry::Isotropic => 1.0,
            AntennaGeometry::Dipole { .. } => 1.640_922, // 2.151 dBi
            AntennaGeometry::Monopole { .. } => 3.281_845, // 5.161 dBi
            _ => {
                let omega = self.beam_solid_angle();
                (4.0 * PI / omega).max(1.0)
            }
        }
    }

    /// Peak maximum directivity in dBi ($10 \log_{10}(D_{max})$).
    #[inline]
    pub fn max_directivity_dbi(&self) -> f64 {
        10.0 * self.max_directivity_linear().max(1e-12).log10()
    }

    /// Peak maximum power gain $G_{max} = \eta_{rad} D_{max}$ (linear scale).
    #[inline]
    pub fn max_gain_linear(&self) -> f64 {
        self.radiation_efficiency() * self.max_directivity_linear()
    }

    /// Peak maximum power gain in dBi.
    #[inline]
    pub fn max_gain_dbi(&self) -> f64 {
        10.0 * self.max_gain_linear().max(1e-12).log10()
    }

    /// Normalized radiation power pattern $F(\theta, \phi) \in [0.0, 1.0]$.
    ///
    /// - $\theta$: polar elevation angle from zenith $+Z$ in radians ($[0, \pi]$).
    /// - $\phi$: azimuth angle from $+X$ towards $+Y$ in radians ($[0, 2\pi]$).
    pub fn normalized_power_pattern(&self, theta_rad: f64, phi_rad: f64) -> f64 {
        let theta = theta_rad.clamp(0.0, PI);
        match &self.geometry {
            AntennaGeometry::Isotropic => 1.0,
            AntennaGeometry::Dipole { .. } => {
                // Dipole oriented along Z axis: F(theta) = [cos(pi/2 * cos theta) / sin theta]^2
                let sin_th = theta.sin();
                if sin_th.abs() < 1e-6 {
                    0.0
                } else {
                    let num = ((PI * 0.5) * theta.cos()).cos();
                    (num / sin_th).powi(2).clamp(0.0, 1.0)
                }
            }
            AntennaGeometry::Monopole { .. } => {
                // Monopole on infinite ground plane (Z >= 0)
                if theta > PI * 0.5 {
                    0.0 // Ground plane shadowing
                } else {
                    let sin_th = theta.sin();
                    if sin_th.abs() < 1e-6 {
                        0.0
                    } else {
                        let num = ((PI * 0.5) * theta.cos()).cos();
                        (num / sin_th).powi(2).clamp(0.0, 1.0)
                    }
                }
            }
            AntennaGeometry::MicrostripPatch {
                length_m, width_m, ..
            } => {
                // Microstrip patch on XY plane radiating into +Z (theta <= pi/2)
                if theta > PI * 0.5 {
                    0.0
                } else {
                    let k = self.wavenumber();
                    let cos_th = theta.cos();
                    let sin_th = theta.sin();
                    let cos_phi = phi_rad.cos();
                    let sin_phi = phi_rad.sin();

                    let psi_x = k * length_m * 0.5 * sin_th * cos_phi;
                    let psi_y = k * width_m * 0.5 * sin_th * sin_phi;

                    let fx = psi_x.cos();
                    let fy = if psi_y.abs() < 1e-5 {
                        1.0
                    } else {
                        psi_y.sin() / psi_y
                    };

                    (fx * fy * cos_th).powi(2).clamp(0.0, 1.0)
                }
            }
            AntennaGeometry::PyramidalHorn {
                aperture_a_m,
                aperture_b_m,
                ..
            } => {
                // Aperture radiating into +Z direction (theta = 0)
                if theta > PI * 0.5 {
                    0.0
                } else {
                    let k = self.wavenumber();
                    let sin_th = theta.sin();
                    let cos_phi = phi_rad.cos();
                    let sin_phi = phi_rad.sin();

                    let u = k * aperture_a_m * 0.5 * sin_th * cos_phi;
                    let v = k * aperture_b_m * 0.5 * sin_th * sin_phi;

                    let fu = if (u.abs() - PI * 0.5).abs() < 1e-4 {
                        PI * 0.25
                    } else {
                        (PI * 0.5 * u.cos()) / (PI * PI * 0.25 - u * u)
                    };

                    let fv = if v.abs() < 1e-5 { 1.0 } else { v.sin() / v };

                    ((fu * fv).powi(2) * (1.0 + theta.cos()) * 0.5).clamp(0.0, 1.0)
                }
            }
            AntennaGeometry::ParabolicDish { diameter_m, .. } => {
                // Circular aperture Airy disk pattern: F(theta) = [2 * J1(u) / u]^2
                if theta > PI * 0.5 {
                    0.0
                } else {
                    let k = self.wavenumber();
                    let radius = diameter_m * 0.5;
                    let u = k * radius * theta.sin();
                    if u.abs() < 1e-5 {
                        1.0
                    } else {
                        // First-order Bessel function J1(u) approximation
                        let j1 = bessel_j1(u);
                        (2.0 * j1 / u).powi(2).clamp(0.0, 1.0)
                    }
                }
            }
            AntennaGeometry::PhasedArray2D {
                num_elements_x,
                num_elements_y,
                spacing_x_m,
                spacing_y_m,
                steer_theta_rad,
                steer_phi_rad,
            } => {
                let k = self.wavenumber();
                let sin_th = theta.sin();
                let sin_th0 = steer_theta_rad.sin();

                // Phase differences along X and Y axes
                let psi_x =
                    k * spacing_x_m * (sin_th * phi_rad.cos() - sin_th0 * steer_phi_rad.cos());
                let psi_y =
                    k * spacing_y_m * (sin_th * phi_rad.sin() - sin_th0 * steer_phi_rad.sin());

                let af_x = array_factor_1d(*num_elements_x, psi_x);
                let af_y = array_factor_1d(*num_elements_y, psi_y);

                let array_factor_norm = (af_x * af_y).powi(2);
                // Broadside element pattern (e.g. cosine)
                let elem_pat = theta.cos().max(0.0);
                (array_factor_norm * elem_pat).clamp(0.0, 1.0)
            }
        }
    }

    /// 3D Directivity $D(\theta, \phi)$ in linear scale.
    #[inline]
    pub fn directivity_linear(&self, theta_rad: f64, phi_rad: f64) -> f64 {
        self.max_directivity_linear() * self.normalized_power_pattern(theta_rad, phi_rad)
    }

    /// 3D Directivity in dBi.
    #[inline]
    pub fn directivity_dbi(&self, theta_rad: f64, phi_rad: f64) -> f64 {
        10.0 * self
            .directivity_linear(theta_rad, phi_rad)
            .max(1e-12)
            .log10()
    }

    /// 3D Antenna Power Gain $G(\theta, \phi) = \eta_{rad} D(\theta, \phi)$ in linear scale.
    #[inline]
    pub fn gain_linear(&self, theta_rad: f64, phi_rad: f64) -> f64 {
        self.radiation_efficiency() * self.directivity_linear(theta_rad, phi_rad)
    }

    /// 3D Antenna Power Gain in dBi.
    #[inline]
    pub fn gain_dbi(&self, theta_rad: f64, phi_rad: f64) -> f64 {
        10.0 * self.gain_linear(theta_rad, phi_rad).max(1e-12).log10()
    }

    /// Maximum Effective Aperture Area $A_e = \frac{\lambda^2}{4\pi} G_{max}$ in $m^2$.
    #[inline]
    pub fn effective_aperture_m2(&self) -> f64 {
        let lambda = self.wavelength_m();
        (lambda * lambda / (4.0 * PI)) * self.max_gain_linear()
    }

    /// Peak effective antenna height / length $h_{eff}$ in meters along maximum gain boresight.
    ///
    /// Relates incident electric field to induced open-circuit terminal voltage:
    /// $V_{oc} = |\mathbf{E}| \cdot h_{eff}$.
    pub fn effective_length_m(&self) -> f64 {
        let lambda = self.wavelength_m();
        let r_ant = self.radiation_resistance_ohms() + self.loss_resistance_ohms();
        let g_max = self.max_gain_linear();
        (r_ant * g_max * lambda * lambda / (PI * VACUUM_IMPEDANCE)).sqrt()
    }

    /// Directed effective antenna length $h_{eff}(\theta, \phi)$ in meters along incident radiation angles.
    pub fn effective_length_directed_m(&self, theta_rad: f64, phi_rad: f64) -> f64 {
        let lambda = self.wavelength_m();
        let r_ant = self.radiation_resistance_ohms() + self.loss_resistance_ohms();
        let g = self.gain_linear(theta_rad, phi_rad);
        (r_ant * g * lambda * lambda / (PI * VACUUM_IMPEDANCE)).sqrt()
    }
}

/// Computes normalized 1D array factor $\frac{\sin(N \psi / 2)}{N \sin(\psi / 2)}$.
#[inline]
fn array_factor_1d(n: usize, psi: f64) -> f64 {
    if n <= 1 {
        return 1.0;
    }
    let denom = (psi * 0.5).sin();
    if denom.abs() < 1e-6 {
        1.0
    } else {
        let num = ((n as f64) * psi * 0.5).sin();
        (num / ((n as f64) * denom)).abs()
    }
}

/// Bessel function of the first kind of order 1 $J_1(x)$.
fn bessel_j1(x: f64) -> f64 {
    let ax = x.abs();
    if ax < 3.75 {
        let y = (x / 3.75).powi(2);
        x * (0.5
            + y * (-0.56249985
                + y * (0.21093573 + y * (-0.03954289 + y * (0.00443319 - y * 0.00031761)))))
    } else {
        let y = 3.75 / ax;
        let f0 = 0.79788456
            + y * (0.00000156
                + y * (0.01659667 + y * (0.00017105 + y * (-0.00249511 + y * 0.00113653))));
        let theta0 = ax - 2.35619449
            + y * (0.04166397
                + y * (-0.00003954 + y * (-0.00262573 + y * (-0.00054125 + y * (-0.00029333)))));
        let ans = (1.0 / ax.sqrt()) * f0 * theta0.cos();
        if x < 0.0 {
            -ans
        } else {
            ans
        }
    }
}
