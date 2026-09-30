#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological fractional quantum Hall acoustic interferometers and
//! anyonic phase modulators.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// fractional quantum Hall acoustic interferometers and anyonic phase modulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FQHInterferometerParams {
    /// Fractional coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub fractional_coupling_energy_mev: f64,
    /// Bulk topological quantum Hall gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_hall_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Interferometer drift speed in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub interferometer_drift_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Gate modulation voltage in mV (clamp 0.5 to 30.0, default 5.8).
    pub gate_modulation_voltage_mv: f64,
    /// Fractional charge fraction in units of elementary charge e (clamp 0.1 to 5.0, default 1.6).
    pub fractional_charge_fraction_e: f64,
    /// Interferometer arm length in micrometers (clamp 0.5 to 20.0, default 4.8).
    pub interferometer_arm_length_um: f64,
}

impl Default for FQHInterferometerParams {
    fn default() -> Self {
        Self {
            fractional_coupling_energy_mev: 16.5,
            topological_hall_gap_mev: 22.0,
            acoustic_drive_frequency_ghz: 5.8,
            interferometer_drift_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            gate_modulation_voltage_mv: 5.8,
            fractional_charge_fraction_e: 1.6,
            interferometer_arm_length_um: 4.8,
        }
    }
}

impl FQHInterferometerParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        fractional_coupling_energy_mev: f64,
        topological_hall_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        interferometer_drift_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        gate_modulation_voltage_mv: f64,
        fractional_charge_fraction_e: f64,
        interferometer_arm_length_um: f64,
    ) -> Self {
        Self {
            fractional_coupling_energy_mev: fractional_coupling_energy_mev.clamp(1.0, 35.0),
            topological_hall_gap_mev: topological_hall_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            interferometer_drift_speed_m_per_s: interferometer_drift_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            gate_modulation_voltage_mv: gate_modulation_voltage_mv.clamp(0.5, 30.0),
            fractional_charge_fraction_e: fractional_charge_fraction_e.clamp(0.1, 5.0),
            interferometer_arm_length_um: interferometer_arm_length_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// fractional quantum Hall acoustic interferometers and anyonic phase modulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FQHInterferometerMetrics {
    /// Modulation fidelity across topological interferometer paths (target >= 0.9980).
    pub modulation_fidelity: f64,
    /// Anyon quantum state retention fraction (target >= 0.9970).
    pub anyon_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
