#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Quantum Metamaterial
//! Polariton Laser & Coherent Soliton Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven quantum metamaterial polariton laser
/// and coherent soliton engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumMetamaterialPolaritonLaserParams {
    /// Laser coupling energy in meV (clamp 1.0 to 35.0, default 35.0).
    pub laser_coupling_mev: f64,
    /// Topological polariton gap energy in meV (clamp 2.0 to 45.0, default 45.0).
    pub topological_polariton_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Soliton dispatch speed in m/s (clamp 200.0 to 3000.0, default 3000.0).
    pub soliton_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Optical pump power in microwatts (clamp 0.5 to 30.0, default 23.5).
    pub optical_pump_power_uw: f64,
    /// Synthetic metamaterial cavities factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_metamaterial_cavities_factor: f64,
    /// Metamaterial lattice pitch in micrometers (clamp 0.5 to 25.0, default 22.5).
    pub metamaterial_lattice_pitch_um: f64,
}

impl Default for QuantumMetamaterialPolaritonLaserParams {
    fn default() -> Self {
        Self {
            laser_coupling_mev: 35.0,
            topological_polariton_gap_mev: 45.0,
            acoustic_drive_frequency_ghz: 12.0,
            soliton_dispatch_speed_m_per_s: 3000.0,
            cryogenic_temperature_mk: 10.0,
            optical_pump_power_uw: 23.5,
            synthetic_metamaterial_cavities_factor: 4.0,
            metamaterial_lattice_pitch_um: 22.5,
        }
    }
}

impl QuantumMetamaterialPolaritonLaserParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        laser_coupling_mev: f64,
        topological_polariton_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        soliton_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        optical_pump_power_uw: f64,
        synthetic_metamaterial_cavities_factor: f64,
        metamaterial_lattice_pitch_um: f64,
    ) -> Self {
        Self {
            laser_coupling_mev: laser_coupling_mev.clamp(1.0, 35.0),
            topological_polariton_gap_mev: topological_polariton_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            soliton_dispatch_speed_m_per_s: soliton_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            optical_pump_power_uw: optical_pump_power_uw.clamp(0.5, 30.0),
            synthetic_metamaterial_cavities_factor: synthetic_metamaterial_cavities_factor.clamp(1.0, 8.0),
            metamaterial_lattice_pitch_um: metamaterial_lattice_pitch_um.clamp(0.5, 25.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Quantum Metamaterial Polariton Laser & Coherent Soliton Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumMetamaterialPolaritonLaserMetrics {
    /// Polariton laser fidelity (target >= 0.9980).
    pub polariton_laser_fidelity: f64,
    /// Coherent soliton retention fraction (target >= 0.9970).
    pub coherent_soliton_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-cavity crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_cavity_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
