//! Microscopic CMOS Active Pixel Sensor (APS) Photodiode Physics
//!
//! Formulates first-principles physical models for 4T CMOS Active Pixel Sensors:
//! 1. Silicon Pinned Photodiode (PPD) quantum efficiency across 350 - 1100 nm.
//! 2. Depletion capacitance and Full-Well Capacity (FWC) in electrons (e-).
//! 3. Thermal dark current generation: I_dark(T) = I_0 * (T/T_0)^3 * exp(-E_g / (2 * k_B * T)).
//! 4. Comprehensive noise mechanisms: Poisson photon shot noise, dark current shot noise,
//!    thermal Johnson-Nyquist read noise, Correlated Double Sampling (CDS) reset suppression,
//!    and Analog-to-Digital Converter (ADC) quantization.
//! 5. Shutter timing: Global Shutter vs Rolling Shutter row integration delay.

use crate::em::ChannelRng;
use phonon_core::constants::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE};

/// Silicon bandgap energy at 300 K in Joules: approx 1.124 eV.
pub const SILICON_BANDGAP_JOULES: f64 = 1.124 * ELEMENTARY_CHARGE;

/// Cut-off wavelength for crystalline silicon bandgap: lambda_gap approx 1107 nm.
pub const SILICON_CUTOFF_WAVELENGTH_NM: f64 = 1107.0;

/// Shutter Readout Architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShutterType {
    /// Global Shutter: all pixels integrate and freeze charge simultaneously.
    GlobalShutter,
    /// Rolling Shutter: sequential row-by-row reset and readout introducing line skew.
    RollingShutter,
}

/// Comprehensive Physical Configuration for a CMOS Active Pixel Sensor Cell.
#[derive(Debug, Clone, PartialEq)]
pub struct CmosPixelConfig {
    /// Physical pixel pitch in meters (e.g. 3.0 um = 3.0e-6 m).
    pub pixel_pitch_m: f64,
    /// Photodiode optical fill factor in (0.0, 1.0] (active photosensitive area fraction).
    pub fill_factor: f64,
    /// Full-Well Capacity (FWC) in electrons (e-, e.g. 32,000 e-).
    pub full_well_capacity: f64,
    /// Floating diffusion node capacitance C_fd in Farads (e.g. 2.0 fF = 2.0e-15 F).
    pub floating_diffusion_capacitance_f: f64,
    /// Dark current density at reference temperature (20 C) in A/m^2.
    pub dark_current_density_ref: f64,
    /// Readout noise floor sigma_read in electrons RMS (e-).
    pub read_noise_electrons: f64,
    /// Analog-to-Digital Converter (ADC) bit depth (e.g. 12 bits => 4096 DN).
    pub adc_bit_depth: u32,
    /// Maximum voltage swing at ADC input in Volts (e.g. 1.0 V).
    pub adc_voltage_range_v: f64,
    /// Shutter readout timing mode.
    pub shutter_type: ShutterType,
}

impl Default for CmosPixelConfig {
    fn default() -> Self {
        Self::new_standard_industrial()
    }
}

impl CmosPixelConfig {
    /// Creates a standard 3.0 micrometer industrial 4T CMOS Active Pixel Sensor.
    pub fn new_standard_industrial() -> Self {
        Self {
            pixel_pitch_m: 3.0e-6,
            fill_factor: 0.72,
            full_well_capacity: 32_000.0,
            floating_diffusion_capacitance_f: 2.0e-15, // 2 fF
            dark_current_density_ref: 1.0e-6,          // ~10 pA/cm^2 = 1.0e-6 A/m^2 at 20 C
            read_noise_electrons: 2.2,                 // 2.2 e- RMS (with CDS)
            adc_bit_depth: 12,                         // 12-bit ADC (0..4095)
            adc_voltage_range_v: 1.0,
            shutter_type: ShutterType::GlobalShutter,
        }
    }

    /// Photodiode photosensitive active area A_pd = p_pix^2 * FF in m^2.
    pub fn active_area_m2(&self) -> f64 {
        self.pixel_pitch_m * self.pixel_pitch_m * self.fill_factor
    }

    /// Charge-to-Voltage Conversion Gain G_c = q / C_fd in Volts per electron.
    pub fn conversion_gain_v_per_e(&self) -> f64 {
        ELEMENTARY_CHARGE / self.floating_diffusion_capacitance_f
    }

    /// Dynamic Range (DR) of the pixel in decibels:
    /// DR = 20 * log10(FWC / sigma_read)
    pub fn dynamic_range_db(&self) -> f64 {
        if self.read_noise_electrons > 1e-6 {
            20.0 * (self.full_well_capacity / self.read_noise_electrons).log10()
        } else {
            120.0
        }
    }

    /// Maximum Digital Number (DN_max = 2^{N_bits} - 1).
    pub fn max_digital_number(&self) -> u32 {
        (1u32 << self.adc_bit_depth) - 1
    }

    /// Evaluates dark current generated in electrons for a given temperature (T in Kelvin)
    /// and exposure time (t_exp in seconds) using the Arrhenius law:
    /// I_dark(T) = I_0 * (T / T_0)^3 * exp(-E_g / (2 * k_B * T))
    pub fn dark_current_electrons(&self, temp_kelvin: f64, exposure_time_s: f64) -> f64 {
        let t = temp_kelvin.max(10.0);
        let t0 = 293.15; // 20 C
        let eg = SILICON_BANDGAP_JOULES;
        let kb = BOLTZMANN_CONSTANT;

        // Arrhenius thermal scaling ratio:
        let t_ratio = t / t0;
        let exp_factor = (-eg / (2.0 * kb * t)).exp() / (-eg / (2.0 * kb * t0)).exp();
        let i_dark =
            self.dark_current_density_ref * self.active_area_m2() * t_ratio.powi(3) * exp_factor;

        // Convert current (Amperes = Coulombs/sec) to electrons:
        let electrons_sec = i_dark / ELEMENTARY_CHARGE;
        (electrons_sec * exposure_time_s).max(0.0)
    }
}

/// Evaluates silicon photodiode quantum efficiency eta_QE(lambda) in [0.0, 1.0]
/// across wavelength lambda in nanometers (300 nm - 1100 nm).
///
/// Models anti-reflective surface coating, silicon absorption depth, and bandgap cut-off.
pub fn silicon_quantum_efficiency(wavelength_nm: f64) -> f64 {
    if wavelength_nm < 320.0 || wavelength_nm >= SILICON_CUTOFF_WAVELENGTH_NM {
        return 0.0;
    }

    let lambda = wavelength_nm;
    if lambda < 530.0 {
        let t = (lambda - 320.0) / (530.0 - 320.0);
        0.15 + 0.67 * t.powf(0.8)
    } else {
        let t = (SILICON_CUTOFF_WAVELENGTH_NM - lambda) / (SILICON_CUTOFF_WAVELENGTH_NM - 530.0);
        0.82 * t.powf(1.4)
    }
    .clamp(0.0, 0.90)
}

/// Transduced pixel measurement containing physical electron counts, analog voltages, and quantized digital number.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelOutput {
    /// Photoelectrons generated by incident photons: N_photo = eta_QE * N_photons.
    pub photoelectrons: f64,
    /// Dark current electrons accumulated during integration.
    pub dark_electrons: f64,
    /// Total integrated electrons before saturation and noise.
    pub raw_signal_electrons: f64,
    /// Synthesized total electrons including Poisson shot noise and read noise.
    pub noisy_electrons: f64,
    /// Final collected electrons clipped to Full-Well Capacity: N_e = min(N_noisy, FWC).
    pub collected_electrons: f64,
    /// Analog voltage before ADC in Volts: V_sig = N_e * G_c.
    pub analog_voltage_v: f64,
    /// Quantized Digital Number (DN) output from ADC (0 .. 2^{N_bits}-1).
    pub digital_number: u32,
    /// Signal-to-Noise Ratio (SNR) in decibels.
    pub snr_db: f64,
    /// Saturation flag: true if pixel exceeded Full-Well Capacity.
    pub is_saturated: bool,
}

/// Transduces incident photon flux on a CMOS pixel cell into physical electrons and quantized ADC output.
pub fn transduce_cmos_pixel(
    config: &CmosPixelConfig,
    incident_photons: f64,
    wavelength_nm: f64,
    exposure_time_s: f64,
    temp_kelvin: f64,
    rng: &mut ChannelRng,
) -> PixelOutput {
    let qe = silicon_quantum_efficiency(wavelength_nm);
    let n_photo_mean = incident_photons.max(0.0) * qe;
    let n_dark_mean = config.dark_current_electrons(temp_kelvin, exposure_time_s);

    // Photon Shot Noise: Poisson variance = mean
    let shot_std = (n_photo_mean + n_dark_mean).sqrt();
    let (g1, _) = rng.next_gaussian();
    let shot_noise = g1 * shot_std;

    // Readout Noise: Gaussian thermal Johnson noise in source follower (CDS removes kTC)
    let (g2, _) = rng.next_gaussian();
    let read_noise = g2 * config.read_noise_electrons;

    // Total synthesized electrons:
    let n_total_noisy = (n_photo_mean + n_dark_mean + shot_noise + read_noise).max(0.0);

    // Full-Well Capacity clipping:
    let fwc = config.full_well_capacity;
    let is_saturated = n_total_noisy >= fwc;
    let collected_e = n_total_noisy.min(fwc);

    // Conversion gain & analog voltage:
    let gc = config.conversion_gain_v_per_e();
    let v_analog = (collected_e * gc).min(config.adc_voltage_range_v);

    // ADC Quantization:
    let max_dn = config.max_digital_number() as f64;
    let dn_float = (v_analog / config.adc_voltage_range_v) * max_dn;
    let digital_number = (dn_float.round() as u32).min(config.max_digital_number());

    // Signal-to-Noise Ratio (SNR):
    let total_noise_var = n_photo_mean + n_dark_mean + config.read_noise_electrons.powi(2);
    let snr_db = if total_noise_var > 1e-9 && n_photo_mean > 1e-9 {
        20.0 * (n_photo_mean / total_noise_var.sqrt()).log10()
    } else {
        0.0
    };

    PixelOutput {
        photoelectrons: n_photo_mean,
        dark_electrons: n_dark_mean,
        raw_signal_electrons: n_photo_mean + n_dark_mean,
        noisy_electrons: n_total_noisy,
        collected_electrons: collected_e,
        analog_voltage_v: v_analog,
        digital_number,
        snr_db,
        is_saturated,
    }
}
