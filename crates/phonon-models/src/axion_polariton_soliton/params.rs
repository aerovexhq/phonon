#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological axion-polariton quantum simulators and non-linear anyonic soliton engines.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// axion-polariton quantum simulators and non-linear anyonic soliton engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionPolaritonParams {
    /// Axion coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub axion_coupling_energy_mev: f64,
    /// Topological polariton energy gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_polariton_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Anyonic soliton propagation speed in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub soliton_propagation_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Optical parametric pump power in micro-watts (clamp 0.5 to 30.0, default 5.8).
    pub optical_parametric_pump_power_uw: f64,
    /// Non-linear Kerr coefficient in pm^2/V^2 (clamp 0.1 to 5.0, default 1.6).
    pub non_linear_kerr_coefficient_pm2_per_v2: f64,
    /// Polariton waveguide spatial pitch in micrometers (clamp 0.5 to 20.0, default 4.8).
    pub polariton_waveguide_pitch_um: f64,
}

impl Default for AxionPolaritonParams {
    fn default() -> Self {
        Self {
            axion_coupling_energy_mev: 16.5,
            topological_polariton_gap_mev: 22.0,
            acoustic_drive_frequency_ghz: 5.8,
            soliton_propagation_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            optical_parametric_pump_power_uw: 5.8,
            non_linear_kerr_coefficient_pm2_per_v2: 1.6,
            polariton_waveguide_pitch_um: 4.8,
        }
    }
}

impl AxionPolaritonParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        axion_coupling_energy_mev: f64,
        topological_polariton_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        soliton_propagation_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        optical_parametric_pump_power_uw: f64,
        non_linear_kerr_coefficient_pm2_per_v2: f64,
        polariton_waveguide_pitch_um: f64,
    ) -> Self {
        Self {
            axion_coupling_energy_mev: axion_coupling_energy_mev.clamp(1.0, 35.0),
            topological_polariton_gap_mev: topological_polariton_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            soliton_propagation_speed_m_per_s: soliton_propagation_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            optical_parametric_pump_power_uw: optical_parametric_pump_power_uw.clamp(0.5, 30.0),
            non_linear_kerr_coefficient_pm2_per_v2: non_linear_kerr_coefficient_pm2_per_v2.clamp(0.1, 5.0),
            polariton_waveguide_pitch_um: polariton_waveguide_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// axion-polariton quantum simulators and non-linear anyonic soliton engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionPolaritonMetrics {
    /// Quantum simulation fidelity across topological axion-polariton modes (target >= 0.9980).
    pub simulation_fidelity: f64,
    /// Anyonic soliton quantum state retention fraction (target >= 0.9970).
    pub soliton_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
