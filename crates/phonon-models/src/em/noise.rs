//! Multi-Source Physical RF Noise, Atmospheric Extinction & Solar Radiation Models.
//!
//! Provides Johnson-Nyquist thermal noise, antenna noise temperature, solar radio flux
//! bursts ($F_{10.7}$), Cosmic Microwave Background ($2.725\text{ K}$), ITU-R P.676
//! gaseous absorption, and ITU-R P.838 rain attenuation.

use phonon_core::constants::BOLTZMANN_CONSTANT;

/// Standard noise reference temperature $T_0 = 290.0\text{ K}$ (IEEE standard).
pub const STANDARD_NOISE_TEMP_KELVIN: f64 = 290.0;

/// Cosmic Microwave Background (CMB) temperature in Kelvin ($2.725\text{ K}$).
pub const COSMIC_MICROWAVE_BACKGROUND_KELVIN: f64 = 2.725;

/// Apparent angular diameter of the Sun viewed from Earth (degrees).
pub const SOLAR_DISK_DIAMETER_DEG: f64 = 0.533;

/// Comprehensive RF Noise and Environmental Channel Parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct RfNoiseModel {
    /// Channel bandwidth in Hertz ($B$).
    pub bandwidth_hz: f64,
    /// Receiver noise figure in decibels ($NF$).
    pub noise_figure_db: f64,
    /// Ambient environmental temperature in Kelvin ($T_{env}$).
    pub ambient_temp_k: f64,
    /// 10.7 cm Solar Radio Flux Index in solar flux units ($\text{sfu} = 10^{-22}\text{ W/m}^2/\text{Hz}$).
    ///
    /// Quiet Sun: $70\text{ sfu}$; Active Sun: $150 - 250\text{ sfu}$; Solar Bursts: $> 1000\text{ sfu}$.
    pub solar_flux_index_sfu: f64,
    /// Angular offset between antenna boresight and Sun center (degrees).
    pub sun_offset_angle_deg: f64,
    /// Antenna 3 dB half-power beamwidth in degrees ($\theta_{3dB}$).
    pub antenna_beamwidth_deg: f64,
    /// Rain precipitation rate in millimeters per hour ($R_{rain}$).
    pub rain_rate_mm_hr: f64,
    /// Atmospheric path length through precipitation in kilometers ($d_{rain}$).
    pub rain_path_length_km: f64,
}

impl Default for RfNoiseModel {
    fn default() -> Self {
        Self {
            bandwidth_hz: 20.0e6,        // 20 MHz channel (e.g. Wi-Fi / LTE)
            noise_figure_db: 4.0,        // 4 dB receiver LNA noise figure
            ambient_temp_k: 290.0,       // 290 K standard
            solar_flux_index_sfu: 120.0, // Moderate solar activity
            sun_offset_angle_deg: 45.0,  // Sun not directly in main-lobe
            antenna_beamwidth_deg: 30.0,
            rain_rate_mm_hr: 0.0, // Clear weather
            rain_path_length_km: 0.0,
        }
    }
}

impl RfNoiseModel {
    pub fn new(bandwidth_hz: f64, noise_figure_db: f64) -> Self {
        Self {
            bandwidth_hz: bandwidth_hz.max(1.0),
            noise_figure_db: noise_figure_db.max(0.0),
            ..Default::default()
        }
    }

    /// Effective receiver noise temperature in Kelvin:
    ///
    /// $$T_{rx} = T_0 \left(10^{NF/10} - 1\right)$$
    pub fn receiver_noise_temperature(&self) -> f64 {
        let f_lin = 10.0_f64.powf(self.noise_figure_db / 10.0);
        STANDARD_NOISE_TEMP_KELVIN * (f_lin - 1.0)
    }

    /// Solar brightness temperature in Kelvin at carrier frequency $f$:
    ///
    /// $$T_{sun}(f) \approx 675 \cdot \frac{F_{10.7}}{f_{\text{GHz}}}$$
    pub fn solar_brightness_temperature(&self, freq_hz: f64) -> f64 {
        let f_ghz = (freq_hz / 1e9).max(0.1);
        (675.0 * self.solar_flux_index_sfu) / f_ghz
    }

    /// Antenna noise temperature contribution from solar radiation:
    ///
    /// Evaluates beam-filling factor and Gaussian antenna pattern gain reduction with offset:
    ///
    /// $$T_{ant, sun} = T_{sun} \cdot \left(\frac{\theta_{sun}}{\theta_{3dB}}\right)^2 \cdot e^{-4\ln(2) (\Delta\theta / \theta_{3dB})^2}$$
    pub fn solar_noise_temperature_contribution(&self, freq_hz: f64) -> f64 {
        let t_sun = self.solar_brightness_temperature(freq_hz);
        let bw = self.antenna_beamwidth_deg.max(0.1);

        // Beam coupling ratio (capped at 1.0 if solar disk fills the entire main beam)
        let beam_coupling = (SOLAR_DISK_DIAMETER_DEG / bw).powi(2).min(1.0);

        // Gaussian radiation pattern drop-off
        let ln2_4 = 4.0 * std::f64::consts::LN_2;
        let pattern_factor = (-ln2_4 * (self.sun_offset_angle_deg / bw).powi(2)).exp();

        t_sun * beam_coupling * pattern_factor
    }

    /// Sky and atmospheric background noise temperature in Kelvin.
    pub fn sky_background_temperature(&self, elevation_deg: f64, freq_hz: f64) -> f64 {
        let f_ghz = (freq_hz / 1e9).max(0.1);
        let el = elevation_deg.clamp(1.0, 90.0);

        // Baseline zenith sky noise: CMB + atmospheric emission
        let zenith_temp = COSMIC_MICROWAVE_BACKGROUND_KELVIN
            + if f_ghz < 10.0 {
                3.0 + 0.5 * f_ghz
            } else if f_ghz < 40.0 {
                8.0 + 0.8 * (f_ghz - 10.0)
            } else {
                30.0 + 2.0 * (f_ghz - 40.0)
            };

        // Geometric secant law for atmospheric path through thin spherical shell
        let cosec_el = 1.0 / el.to_radians().sin();
        zenith_temp * cosec_el.min(10.0)
    }

    /// Total system noise temperature:
    ///
    /// $$T_{sys} = T_{rx} + T_{ant} = T_{rx} + T_{sky} + T_{sun}$$
    pub fn total_system_noise_temperature(&self, elevation_deg: f64, freq_hz: f64) -> f64 {
        let t_rx = self.receiver_noise_temperature();
        let t_sky = self.sky_background_temperature(elevation_deg, freq_hz);
        let t_sun = self.solar_noise_temperature_contribution(freq_hz);

        t_rx + t_sky + t_sun
    }

    /// Thermal Johnson-Nyquist noise power in Watts:
    ///
    /// $$P_n = k_B T_{sys} B$$
    pub fn thermal_noise_power_watts(&self, elevation_deg: f64, freq_hz: f64) -> f64 {
        let t_sys = self.total_system_noise_temperature(elevation_deg, freq_hz);
        BOLTZMANN_CONSTANT * t_sys * self.bandwidth_hz
    }

    /// Thermal noise power floor in decibel-milliwatts ($\text{dBm}$).
    pub fn thermal_noise_power_dbm(&self, elevation_deg: f64, freq_hz: f64) -> f64 {
        let p_w = self.thermal_noise_power_watts(elevation_deg, freq_hz);
        10.0 * (p_w.max(1e-25) * 1000.0).log10()
    }

    /// ITU-R P.676 atmospheric gaseous absorption loss in decibels ($L_a\text{ dB}$).
    ///
    /// Models oxygen resonance absorption near $60\text{ GHz}$ and water vapor line at $22.235\text{ GHz}$.
    pub fn itu_r_p676_gaseous_attenuation_db(freq_hz: f64, path_length_km: f64) -> f64 {
        let f_ghz = freq_hz / 1e9;
        let d_km = path_length_km.max(0.0);

        if f_ghz < 1.0 {
            return 0.005 * d_km;
        }

        // Specific attenuation gamma in dB/km
        // Water vapor resonance component (22.2 GHz)
        let gamma_w = 0.05 * (1.0 / ((f_ghz - 22.235).powi(2) + 4.0)) + 0.0002 * f_ghz;

        // Oxygen resonance complex band (57 - 63 GHz)
        let gamma_o = if (50.0..=70.0).contains(&f_ghz) {
            15.0 * (-((f_ghz - 60.0) / 4.0).powi(2)).exp() + 0.2
        } else {
            0.007 * (f_ghz / 10.0).powi(2)
        };

        (gamma_w + gamma_o) * d_km
    }

    /// ITU-R P.838 rain attenuation in decibels ($L_R\text{ dB}$).
    ///
    /// Formulates specific attenuation power law:
    ///
    /// $$\gamma_R = k \cdot R_{rain}^\alpha \text{ (dB/km)}$$
    pub fn itu_r_p838_rain_attenuation_db(&self, freq_hz: f64) -> f64 {
        if self.rain_rate_mm_hr <= 0.01 || self.rain_path_length_km <= 0.001 {
            return 0.0;
        }

        let f_ghz = (freq_hz / 1e9).clamp(1.0, 100.0);
        let r = self.rain_rate_mm_hr;

        // Simplified frequency-dependent power law coefficients
        let k = 0.0001 * f_ghz.powf(2.2);
        let alpha = 1.35 - 0.15 * (f_ghz / 30.0).min(1.0);

        let gamma_r = k * r.powf(alpha);
        gamma_r * self.rain_path_length_km
    }

    /// Signal-to-Noise Ratio (SNR) in decibels given received power in Watts:
    ///
    /// $$\text{SNR}_{\text{dB}} = 10\log_{10}\left(\frac{P_{rx}}{P_n}\right)$$
    pub fn snr_db(&self, rx_power_watts: f64, elevation_deg: f64, freq_hz: f64) -> f64 {
        let p_n = self.thermal_noise_power_watts(elevation_deg, freq_hz);
        if rx_power_watts > 1e-25 && p_n > 1e-25 {
            10.0 * (rx_power_watts / p_n).log10()
        } else {
            -100.0
        }
    }
}
