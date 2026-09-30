#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian topological defect Majorana-Kramers pair network processors and
//! time-reversal-symmetric phononic braiding engines.

/// Physical parameter configuration for quantum acoustic non-Abelian topological defect
/// Majorana-Kramers pair network processors and time-reversal-symmetric phononic braiding engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaKramersNetworkParams {
    /// Spin-orbit phononic coupling energy in meV (clamp 2.0 to 40.0, default 18.5).
    pub spin_orbit_phononic_coupling_mev: f64,
    /// Time-reversal pairing gap Delta_TR in meV (clamp 1.5 to 30.0, default 14.0).
    pub time_reversal_pairing_gap_mev: f64,
    /// Acoustic drive fundamental frequency in GHz (clamp 1.0 to 12.0, default 5.5).
    pub acoustic_drive_frequency_ghz: f64,
    /// Defect shuttling velocity in m/s (clamp 200.0 to 3000.0, default 1350.0).
    pub shuttling_velocity_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave control pulse power in micro-watts (clamp 0.5 to 30.0, default 5.5).
    pub microwave_control_power_uw: f64,
    /// Topological defect separation distance in micrometers (clamp 0.5 to 15.0, default 3.5).
    pub defect_separation_distance_um: f64,
    /// Substrate electromechanical piezoelectric coupling coefficient (clamp 0.10 to 0.95, default 0.65).
    pub substrate_piezoelectric_coupling: f64,
}

impl Default for MajoranaKramersNetworkParams {
    fn default() -> Self {
        Self {
            spin_orbit_phononic_coupling_mev: 18.5,
            time_reversal_pairing_gap_mev: 14.0,
            acoustic_drive_frequency_ghz: 5.5,
            shuttling_velocity_m_per_s: 1350.0,
            cryogenic_temperature_mk: 10.0,
            microwave_control_power_uw: 5.5,
            defect_separation_distance_um: 3.5,
            substrate_piezoelectric_coupling: 0.65,
        }
    }
}

impl MajoranaKramersNetworkParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        spin_orbit_phononic_coupling_mev: f64,
        time_reversal_pairing_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        shuttling_velocity_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_control_power_uw: f64,
        defect_separation_distance_um: f64,
        substrate_piezoelectric_coupling: f64,
    ) -> Self {
        Self {
            spin_orbit_phononic_coupling_mev: spin_orbit_phononic_coupling_mev.clamp(2.0, 40.0),
            time_reversal_pairing_gap_mev: time_reversal_pairing_gap_mev.clamp(1.5, 30.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            shuttling_velocity_m_per_s: shuttling_velocity_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_control_power_uw: microwave_control_power_uw.clamp(0.5, 30.0),
            defect_separation_distance_um: defect_separation_distance_um.clamp(0.5, 15.0),
            substrate_piezoelectric_coupling: substrate_piezoelectric_coupling.clamp(0.10, 0.95),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian topological defect
/// Majorana-Kramers pair network processors and time-reversal-symmetric phononic braiding engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaKramersNetworkMetrics {
    /// Non-Abelian phononic braiding gate fidelity (target >= 0.9980).
    pub braiding_fidelity: f64,
    /// Time-reversal-protected Kramers pair quantum state retention fraction (target >= 0.9970).
    pub kramers_pair_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 46.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-defect acoustic and topological crosstalk isolation in decibels (target >= 54.0 dB).
    pub inter_defect_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
