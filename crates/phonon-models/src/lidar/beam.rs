//! Physically Rigorous Pulsed Time-of-Flight (ToF) Laser Beam Physics
//!
//! Formulates optical laser beam parameters, Gaussian beam spatial propagation,
//! surface bidirectional reflectance (BRDF/albedo), LiDAR range equation,
//! and multi-echo optical pulse return.

use crate::em::Vector3D;
use phonon_core::constants::SPEED_OF_LIGHT;

/// Standard eye-safe aerospace LiDAR wavelength: 1550 nm (InGaAs/InP).
pub const WAVELENGTH_1550_NM: f64 = 1550.0;

/// Standard automotive and industrial LiDAR wavelength: 905 nm (GaAs).
pub const WAVELENGTH_905_NM: f64 = 905.0;

/// Configuration for a pulsed laser beam emission and optical receiver.
#[derive(Debug, Clone, PartialEq)]
pub struct LaserPulseConfig {
    /// Optical carrier wavelength in nanometers (e.g. 905.0 or 1550.0 nm).
    pub wavelength_nm: f64,
    /// Laser pulse energy in Joules (e.g. 20 uJ = 2.0e-5 J).
    pub pulse_energy_joules: f64,
    /// Full-Width at Half-Maximum (FWHM) pulse duration in seconds (e.g. 5.0 ns = 5.0e-9 s).
    pub pulse_duration_s: f64,
    /// Beam waist radius w_0 at exit aperture in meters (e.g. 1.5 mm = 1.5e-3 m).
    pub beam_waist_radius_m: f64,
    /// Full beam divergence angle in radians (e.g. 1.2 mrad = 1.2e-3 rad).
    pub divergence_rad: f64,
    /// Optical receiver collection aperture diameter in meters (e.g. 25 mm = 0.025 m).
    pub receiver_aperture_diameter_m: f64,
    /// Optical transmission and bandpass filter efficiency in [0.0, 1.0].
    pub optical_efficiency: f64,
    /// Receiver noise equivalent power (NEP) in Watts.
    pub noise_equivalent_power_w: f64,
}

impl LaserPulseConfig {
    /// Constructs a standard 905 nm automotive LiDAR configuration.
    pub fn new_automotive_905nm() -> Self {
        Self {
            wavelength_nm: WAVELENGTH_905_NM,
            pulse_energy_joules: 15.0e-6,        // 15 uJ
            pulse_duration_s: 5.0e-9,            // 5 ns
            beam_waist_radius_m: 1.5e-3,         // 1.5 mm
            divergence_rad: 1.5e-3,              // 1.5 mrad
            receiver_aperture_diameter_m: 0.025, // 25 mm
            optical_efficiency: 0.80,
            noise_equivalent_power_w: 2.0e-9, // 2 nW
        }
    }

    /// Constructs a high-power 1550 nm eye-safe aerospace LiDAR configuration.
    pub fn new_aerospace_1550nm() -> Self {
        Self {
            wavelength_nm: WAVELENGTH_1550_NM,
            pulse_energy_joules: 80.0e-6, // 80 uJ (allowed due to ~100x higher eye safety limit)
            pulse_duration_s: 4.0e-9,     // 4 ns
            beam_waist_radius_m: 2.0e-3,  // 2.0 mm
            divergence_rad: 0.8e-3,       // 0.8 mrad (tighter collimation)
            receiver_aperture_diameter_m: 0.035, // 35 mm
            optical_efficiency: 0.85,
            noise_equivalent_power_w: 1.0e-9, // 1 nW (InGaAs APD)
        }
    }

    /// Peak optical pulse emission power in Watts: P_0 = E_pulse / tau_pulse.
    pub fn peak_power_watts(&self) -> f64 {
        self.pulse_energy_joules / self.pulse_duration_s.max(1e-12)
    }

    /// Optical receiver collection aperture area A_rx = pi * (D / 2)^2 in m^2.
    pub fn receiver_aperture_area_m2(&self) -> f64 {
        let r = self.receiver_aperture_diameter_m / 2.0;
        std::f64::consts::PI * r * r
    }

    /// Evaluates the transverse Gaussian beam spot radius w(R) at distance R in meters.
    ///
    /// Uses Gaussian TEM_00 beam expansion:
    /// w(R) = sqrt(w_0^2 + (R * tan(theta_div / 2))^2)
    pub fn beam_radius_at_range(&self, range_m: f64) -> f64 {
        let r = range_m.max(0.0);
        let theta_half = self.divergence_rad / 2.0;
        let expansion = r * theta_half.tan();
        (self.beam_waist_radius_m * self.beam_waist_radius_m + expansion * expansion).sqrt()
    }

    /// Evaluates illuminated beam spot area on a normal target surface at range R in m^2.
    pub fn beam_spot_area_at_range(&self, range_m: f64) -> f64 {
        let w = self.beam_radius_at_range(range_m);
        std::f64::consts::PI * w * w
    }

    /// Evaluates returned optical power P_rx(R) in Watts according to the classical LiDAR Range Equation:
    ///
    /// P_rx(R) = P_0 * [A_rx / (pi * R^2)] * rho * cos(theta_inc) * eta_opt * exp(-2 * alpha_ext * R)
    pub fn returned_optical_power(
        &self,
        range_m: f64,
        target_reflectivity: f64,
        incidence_angle_rad: f64,
        atmospheric_extinction_per_m: f64,
    ) -> f64 {
        if range_m < 0.1 {
            return 0.0;
        }

        let p_0 = self.peak_power_watts();
        let a_rx = self.receiver_aperture_area_m2();
        let rho = target_reflectivity.clamp(0.0, 1.0);
        let cos_theta = incidence_angle_rad.cos().max(0.0);

        // Geometric path loss for Lambertian target:
        let geometric_factor = a_rx / (std::f64::consts::PI * range_m * range_m);

        // Two-way Beer-Lambert atmospheric transmission: T_atm = exp(-2 * alpha * R)
        let t_atm = (-2.0 * atmospheric_extinction_per_m * range_m).exp();

        p_0 * geometric_factor * rho * cos_theta * self.optical_efficiency * t_atm
    }

    /// Evaluates round-trip Time of Flight (ToF) in seconds for range R:
    /// t_tof = 2 * R / c
    pub fn time_of_flight_seconds(range_m: f64) -> f64 {
        (2.0 * range_m.max(0.0)) / SPEED_OF_LIGHT
    }

    /// Converts round-trip Time of Flight (ToF) in seconds to one-way range R in meters:
    /// R = c * t_tof / 2
    pub fn range_from_time_of_flight(tof_seconds: f64) -> f64 {
        (SPEED_OF_LIGHT * tof_seconds.max(0.0)) / 2.0
    }
}

/// Individual optical echo return from a pulsed LiDAR measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EchoReturn {
    /// Target one-way range in meters.
    pub range_m: f64,
    /// Measured round-trip Time of Flight in seconds.
    pub tof_seconds: f64,
    /// Peak received optical power in Watts.
    pub received_power_w: f64,
    /// Signal-to-Noise Ratio (SNR) in decibels.
    pub snr_db: f64,
    /// Target surface albedo / reflectivity in [0.0, 1.0].
    pub target_albedo: f64,
    /// Echo rank index (0 for first echo, 1 for second, etc.).
    pub echo_index: usize,
    /// True if this is the last detectable echo in the return pulse train.
    pub is_last_echo: bool,
}

/// Laser Ray with origin and normalized propagation direction vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaserRay {
    pub origin: Vector3D,
    pub direction: Vector3D,
}

impl LaserRay {
    pub fn new(origin: Vector3D, direction: Vector3D) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }
}
