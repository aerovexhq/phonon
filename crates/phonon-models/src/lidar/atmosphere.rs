//! Atmospheric Optical Extinction, Mie Scattering & Backscatter Clutter
//!
//! Formulates optical attenuation across adverse atmospheric conditions:
//! 1. Beer-Lambert optical extinction: alpha_ext = alpha_abs + alpha_scat.
//! 2. Kruse-Kim wavelength-dependent Mie scattering for fog and haze across 905 nm and 1550 nm.
//! 3. Marshall-Palmer rain droplet extinction and dust/smoke obscuration.
//! 4. Atmospheric volume backscatter clutter generating near-field false echoes.

use crate::lidar::beam::LaserPulseConfig;
use phonon_core::constants::SPEED_OF_LIGHT;

/// Type of optical fog condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FogType {
    /// Radiation fog: small droplet radii (~2-4 um), high optical extinction.
    RadiationFog,
    /// Advection fog: larger marine droplet radii (~10-20 um).
    AdvectionFog,
}

/// Adverse atmospheric condition governing optical transmission and scattering.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AtmosphericCondition {
    /// Clear standard atmosphere (visibility >= 25 km).
    #[default]
    ClearAir,
    /// Light haze (visibility ~ 3 - 5 km).
    Haze { visibility_m: f64 },
    /// Dense fog characterized by meteorological visibility in meters.
    Fog {
        visibility_m: f64,
        fog_type: FogType,
    },
    /// Rainfall characterized by precipitation rate in mm/hour.
    Rain { rate_mm_hr: f64 },
    /// Smoke and particulate obscuration in mg/m^3.
    SmokeAndDust { mass_concentration_mg_m3: f64 },
}

impl AtmosphericCondition {
    /// Evaluates the atmospheric optical extinction coefficient alpha_ext in m^-1
    /// for a given optical carrier wavelength in nanometers.
    ///
    /// Extinction follows the Kruse and Kim formulation:
    /// alpha(lambda, V) = (3.91 / V) * (lambda / 550.0)^(-q)
    pub fn extinction_coefficient_per_m(&self, wavelength_nm: f64) -> f64 {
        match self {
            Self::ClearAir => {
                // Clear air Rayleigh and background aerosol extinction (~0.2 dB/km):
                0.000046 // ~0.2 dB/km
            }
            Self::Haze { visibility_m } => {
                let v_km = (visibility_m / 1000.0).max(0.1);
                // Kim model q parameter for haze:
                let q = if v_km > 50.0 {
                    1.6
                } else if v_km > 6.0 {
                    1.3
                } else if v_km > 1.0 {
                    0.16 * v_km + 0.34
                } else {
                    v_km - 0.5
                }
                .max(0.0);

                let alpha_550 = 3.91 / (visibility_m.max(10.0));
                alpha_550 * (wavelength_nm / 550.0).powf(-q)
            }
            Self::Fog { visibility_m, .. } => {
                let v = visibility_m.max(10.0);
                // In dense fog (V < 500 m), droplet size (5 - 15 um) >> lambda (0.9 - 1.55 um),
                // so Mie scattering is in the geometrical limit with q approx 0:
                let q = if v < 500.0 {
                    0.0
                } else {
                    0.585 * (v / 1000.0).powf(1.0 / 3.0)
                };

                let alpha_550 = 3.91 / v;
                alpha_550 * (wavelength_nm / 550.0).powf(-q)
            }
            Self::Rain { rate_mm_hr } => {
                let r = rate_mm_hr.max(0.0);
                if r < 1e-4 {
                    return 0.000046;
                }
                // Optical rainfall attenuation: gamma = 0.28 * R^0.64 in dB/km
                // alpha = gamma * ln(10) / (10 * 1000)
                let gamma_db_per_km = 0.28 * r.powf(0.64);
                let alpha_rain = gamma_db_per_km * std::f64::consts::LN_10 / 10000.0;
                alpha_rain + 0.000046
            }
            Self::SmokeAndDust {
                mass_concentration_mg_m3,
            } => {
                let c = mass_concentration_mg_m3.max(0.0);
                // Specific mass extinction coefficient for aerosol smoke: ~4.5 m^2/g = 0.0045 m^2/mg
                let alpha_smoke = 0.0045 * c;
                alpha_smoke + 0.000046
            }
        }
    }

    /// Evaluates two-way atmospheric transmittance T_two_way = exp(-2 * alpha_ext * range_m).
    pub fn two_way_transmittance(&self, range_m: f64, wavelength_nm: f64) -> f64 {
        let alpha = self.extinction_coefficient_per_m(wavelength_nm);
        (-2.0 * alpha * range_m.max(0.0)).exp()
    }

    /// Evaluates atmospheric volume backscattering coefficient beta_pi in m^-1 * sr^-1.
    pub fn volume_backscatter_coefficient(&self, wavelength_nm: f64) -> f64 {
        match self {
            Self::ClearAir => 0.0,
            _ => {
                let alpha = self.extinction_coefficient_per_m(wavelength_nm);
                // Under Mie scattering in fog/smoke, backscatter phase function P_pi ~ 0.05 sr^-1:
                let phase_function_pi = 0.05;
                alpha * phase_function_pi
            }
        }
    }

    /// Evaluates parasitic atmospheric backscatter power P_backscatter(R) in Watts
    /// returning from a volume layer at distance R with pulse duration tau_p.
    ///
    /// P_bs(R) = P_0 * A_rx * [c * tau_p / 2] * [beta_pi / R^2] * exp(-2 * alpha * R)
    pub fn atmospheric_backscatter_power(&self, laser: &LaserPulseConfig, range_m: f64) -> f64 {
        if range_m < 0.2 {
            return 0.0;
        }

        let alpha = self.extinction_coefficient_per_m(laser.wavelength_nm);
        let beta_pi = self.volume_backscatter_coefficient(laser.wavelength_nm);
        let p_0 = laser.peak_power_watts();
        let a_rx = laser.receiver_aperture_area_m2();
        let delta_r = (SPEED_OF_LIGHT * laser.pulse_duration_s) / 2.0;

        let geometric = a_rx / (range_m * range_m);
        let two_way_atten = (-2.0 * alpha * range_m).exp();

        p_0 * geometric * delta_r * beta_pi * laser.optical_efficiency * two_way_atten
    }
}
