#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for cavity
//! quantum acoustomagnonic polariton condensation and chiral superfluid spin-phonon lasers.

/// Physical parameter configuration for cavity quantum acoustomagnonic polariton condensation
/// and chiral superfluid spin-phonon lasers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustomagnonicPolaritonLaserParams {
    /// Magnon Kittel mode resonance frequency in GHz (clamp 2.0 to 18.0, default 8.5 GHz).
    pub magnon_kittel_frequency_ghz: f64,
    /// High-Q acoustic resonator mode frequency in GHz (clamp 2.0 to 18.0, default 8.5 GHz).
    pub acoustic_resonator_frequency_ghz: f64,
    /// Magnon-phonon dispersive/piezo-magnetic coupling rate in MHz (clamp 5.0 to 100.0, default 38.0 MHz).
    pub magnon_phonon_coupling_mhz: f64,
    /// Optical or microwave pump drive power in microwatts (clamp 1.0 to 100.0, default 22.0 uW).
    pub optical_microwave_pump_power_uw: f64,
    /// Magnon intrinsic damping rate in MHz (clamp 0.5 to 15.0, default 2.2 MHz).
    pub magnon_damping_rate_mhz: f64,
    /// Acoustic mode intrinsic decay rate in kHz (clamp 10.0 to 500.0, default 85.0 kHz).
    pub acoustic_decay_rate_khz: f64,
    /// Cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0 mK).
    pub cryogenic_temperature_mk: f64,
    /// Non-linear Kerr self-interaction coefficient in Hz (clamp 1.0 to 100.0, default 12.0 Hz).
    pub non_linear_kerr_coefficient_hz: f64,
}

impl Default for AcoustomagnonicPolaritonLaserParams {
    fn default() -> Self {
        Self {
            magnon_kittel_frequency_ghz: 8.5,
            acoustic_resonator_frequency_ghz: 8.5,
            magnon_phonon_coupling_mhz: 38.0,
            optical_microwave_pump_power_uw: 22.0,
            magnon_damping_rate_mhz: 2.2,
            acoustic_decay_rate_khz: 85.0,
            cryogenic_temperature_mk: 15.0,
            non_linear_kerr_coefficient_hz: 12.0,
        }
    }
}

impl AcoustomagnonicPolaritonLaserParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        magnon_kittel_frequency_ghz: f64,
        acoustic_resonator_frequency_ghz: f64,
        magnon_phonon_coupling_mhz: f64,
        optical_microwave_pump_power_uw: f64,
        magnon_damping_rate_mhz: f64,
        acoustic_decay_rate_khz: f64,
        cryogenic_temperature_mk: f64,
        non_linear_kerr_coefficient_hz: f64,
    ) -> Self {
        Self {
            magnon_kittel_frequency_ghz: magnon_kittel_frequency_ghz.clamp(2.0, 18.0),
            acoustic_resonator_frequency_ghz: acoustic_resonator_frequency_ghz.clamp(2.0, 18.0),
            magnon_phonon_coupling_mhz: magnon_phonon_coupling_mhz.clamp(5.0, 100.0),
            optical_microwave_pump_power_uw: optical_microwave_pump_power_uw.clamp(1.0, 100.0),
            magnon_damping_rate_mhz: magnon_damping_rate_mhz.clamp(0.5, 15.0),
            acoustic_decay_rate_khz: acoustic_decay_rate_khz.clamp(10.0, 500.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            non_linear_kerr_coefficient_hz: non_linear_kerr_coefficient_hz.clamp(1.0, 100.0),
        }
    }
}

/// Multi-physics evaluation metrics for cavity quantum acoustomagnonic polariton condensation
/// and chiral superfluid spin-phonon lasers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustomagnonicPolaritonLaserMetrics {
    /// Polariton Bose-Einstein condensation threshold pump power in microwatts (target <= 15.0 uW).
    pub polariton_condensation_threshold_uw: f64,
    /// Condensate macroscopic phase coherence lifetime in microseconds (target >= 120.0 us).
    pub condensate_coherence_lifetime_us: f64,
    /// Side-mode suppression ratio of chiral spin-phonon lasing in dB (target >= 45.0 dB).
    pub side_mode_suppression_ratio_db: f64,
    /// Emission linewidth narrowing factor relative to bare cavity mode (target >= 80.0x).
    pub linewidth_narrowing_factor: f64,
    /// Polariton macroscopic superfluid fraction (target >= 0.850).
    pub polariton_superfluid_fraction: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
