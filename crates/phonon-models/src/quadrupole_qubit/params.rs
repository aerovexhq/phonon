#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Second-Order Topological Quadrupole Insulator &
//! Corner-State Qubit Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous second-order topological quadrupole insulator and corner-state qubit engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadrupoleQubitParams {
    /// Acoustic-quadrupole exchange coupling energy in meV (clamp 1.0 to 35.0, default 25.0).
    pub quadrupole_coupling_mev: f64,
    /// Topological corner excitation bandgap energy in meV (clamp 2.0 to 45.0, default 31.0).
    pub topological_corner_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 10.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Corner-state shuttle dispatch speed in m/s (clamp 200.0 to 3000.0, default 2200.0).
    pub corner_shuttle_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave readout probe power in microwatts (clamp 0.5 to 30.0, default 10.0).
    pub microwave_probe_power_uw: f64,
    /// Synthetic quadrupole unit cells multiplication factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_quadrupole_cells_factor: f64,
    /// Higher-order topological lattice cell pitch in micrometers (clamp 0.5 to 20.0, default 9.0).
    pub lattice_cell_pitch_um: f64,
}

impl Default for QuadrupoleQubitParams {
    fn default() -> Self {
        Self {
            quadrupole_coupling_mev: 25.0,
            topological_corner_gap_mev: 31.0,
            acoustic_drive_frequency_ghz: 10.0,
            corner_shuttle_dispatch_speed_m_per_s: 2200.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 10.0,
            synthetic_quadrupole_cells_factor: 4.0,
            lattice_cell_pitch_um: 9.0,
        }
    }
}

impl QuadrupoleQubitParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        quadrupole_coupling_mev: f64,
        topological_corner_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        corner_shuttle_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_quadrupole_cells_factor: f64,
        lattice_cell_pitch_um: f64,
    ) -> Self {
        Self {
            quadrupole_coupling_mev: quadrupole_coupling_mev.clamp(1.0, 35.0),
            topological_corner_gap_mev: topological_corner_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            corner_shuttle_dispatch_speed_m_per_s: corner_shuttle_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_quadrupole_cells_factor: synthetic_quadrupole_cells_factor.clamp(1.0, 8.0),
            lattice_cell_pitch_um: lattice_cell_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Second-Order Topological Quadrupole Insulator & Corner-State Qubit Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadrupoleQubitMetrics {
    /// 0D corner-state spatial localization fidelity (target >= 0.9980).
    pub corner_localization_fidelity: f64,
    /// Corner-mode qubit quantum state retention fraction (target >= 0.9970).
    pub qubit_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-corner crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_corner_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
