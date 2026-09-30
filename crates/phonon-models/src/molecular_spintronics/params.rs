#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Molecular Spintronic Qubit Interface &
//! Diamond NV-Center Acoustic Transducer Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous molecular spintronic qubit interface and diamond NV-center acoustic transducer engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MolecularSpintronicsParams {
    /// Molecular spin-strain and spintronic exchange coupling energy in meV (clamp 1.0 to 35.0, default 22.0).
    pub spintronic_coupling_mev: f64,
    /// Topological spintronic protection bandgap energy in meV (clamp 2.0 to 45.0, default 28.0).
    pub topological_spintronic_gap_mev: f64,
    /// Surface acoustic wave drive carrier frequency in GHz (clamp 1.0 to 12.0, default 8.5).
    pub acoustic_drive_frequency_ghz: f64,
    /// Spin-acoustic wave propagation and dispatch speed in m/s (clamp 200.0 to 3000.0, default 1900.0).
    pub spin_acoustic_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Dispersive microwave readout probe power in microwatts (clamp 0.5 to 30.0, default 8.5).
    pub microwave_probe_power_uw: f64,
    /// Synthetic diamond NV-center acoustic cavity Purcell enhancement factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_nv_cavity_factor: f64,
    /// Molecular spintronic qubit unit cell routing pitch in micrometers (clamp 0.5 to 20.0, default 7.5).
    pub spintronic_cell_pitch_um: f64,
}

impl Default for MolecularSpintronicsParams {
    fn default() -> Self {
        Self {
            spintronic_coupling_mev: 22.0,
            topological_spintronic_gap_mev: 28.0,
            acoustic_drive_frequency_ghz: 8.5,
            spin_acoustic_dispatch_speed_m_per_s: 1900.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 8.5,
            synthetic_nv_cavity_factor: 4.0,
            spintronic_cell_pitch_um: 7.5,
        }
    }
}

impl MolecularSpintronicsParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        spintronic_coupling_mev: f64,
        topological_spintronic_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        spin_acoustic_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_nv_cavity_factor: f64,
        spintronic_cell_pitch_um: f64,
    ) -> Self {
        Self {
            spintronic_coupling_mev: spintronic_coupling_mev.clamp(1.0, 35.0),
            topological_spintronic_gap_mev: topological_spintronic_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            spin_acoustic_dispatch_speed_m_per_s: spin_acoustic_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_nv_cavity_factor: synthetic_nv_cavity_factor.clamp(1.0, 8.0),
            spintronic_cell_pitch_um: spintronic_cell_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Molecular Spintronic Qubit Interface & Diamond NV-Center Acoustic Transducer Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MolecularSpintronicsMetrics {
    /// Transduction fidelity across diamond NV-center acoustic transducer interfaces (target >= 0.9980).
    pub transduction_fidelity: f64,
    /// Quantum spin state retention fraction across molecular spintronic interfaces (target >= 0.9970).
    pub spin_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-qubit crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_qubit_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
