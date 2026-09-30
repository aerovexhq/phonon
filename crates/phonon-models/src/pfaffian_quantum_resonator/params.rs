#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological Pfaffian superconducting qubit resonators and
//! parity-protected anyonic gate engines.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// Pfaffian superconducting qubit resonators and parity-protected anyonic gate engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PfaffianQuantumResonatorParams {
    /// Chiral Pfaffian topological pairing gap in meV (clamp 2.0 to 45.0, default 21.5).
    pub pfaffian_pairing_gap_mev: f64,
    /// Superconducting charging energy E_C in GHz (clamp 0.1 to 2.5, default 0.85).
    pub superconducting_charging_energy_ghz: f64,
    /// Acoustic resonator fundamental frequency in GHz (clamp 1.0 to 12.0, default 5.5).
    pub acoustic_resonator_frequency_ghz: f64,
    /// Piezoelectric electromechanical coupling strength in percent (clamp 0.5 to 15.0, default 6.2).
    pub piezoelectric_coupling_strength_percent: f64,
    /// External magnetic flux bias in units of magnetic flux quantum Phi_0 (clamp 0.05 to 0.95, default 0.45).
    pub magnetic_flux_bias_phi0: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave control drive power in micro-watts (clamp 0.5 to 30.0, default 5.8).
    pub microwave_drive_power_uw: f64,
    /// Acoustic resonator quality factor Q in units of 1,000 (clamp 10.0 to 500.0, default 180.0).
    pub resonator_quality_factor_k: f64,
}

impl Default for PfaffianQuantumResonatorParams {
    fn default() -> Self {
        Self {
            pfaffian_pairing_gap_mev: 21.5,
            superconducting_charging_energy_ghz: 0.85,
            acoustic_resonator_frequency_ghz: 5.5,
            piezoelectric_coupling_strength_percent: 6.2,
            magnetic_flux_bias_phi0: 0.45,
            cryogenic_temperature_mk: 10.0,
            microwave_drive_power_uw: 5.8,
            resonator_quality_factor_k: 180.0,
        }
    }
}

impl PfaffianQuantumResonatorParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        pfaffian_pairing_gap_mev: f64,
        superconducting_charging_energy_ghz: f64,
        acoustic_resonator_frequency_ghz: f64,
        piezoelectric_coupling_strength_percent: f64,
        magnetic_flux_bias_phi0: f64,
        cryogenic_temperature_mk: f64,
        microwave_drive_power_uw: f64,
        resonator_quality_factor_k: f64,
    ) -> Self {
        Self {
            pfaffian_pairing_gap_mev: pfaffian_pairing_gap_mev.clamp(2.0, 45.0),
            superconducting_charging_energy_ghz: superconducting_charging_energy_ghz.clamp(0.1, 2.5),
            acoustic_resonator_frequency_ghz: acoustic_resonator_frequency_ghz.clamp(1.0, 12.0),
            piezoelectric_coupling_strength_percent: piezoelectric_coupling_strength_percent.clamp(0.5, 15.0),
            magnetic_flux_bias_phi0: magnetic_flux_bias_phi0.clamp(0.05, 0.95),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_drive_power_uw: microwave_drive_power_uw.clamp(0.5, 30.0),
            resonator_quality_factor_k: resonator_quality_factor_k.clamp(10.0, 500.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// Pfaffian superconducting qubit resonators and parity-protected anyonic gate engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PfaffianQuantumResonatorMetrics {
    /// Parity-protected anyonic quantum gate fidelity (target >= 0.9980).
    pub gate_fidelity: f64,
    /// Pfaffian topological state retention fraction (target >= 0.9970).
    pub pfaffian_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-resonator crosstalk acoustic isolation in decibels (target >= 54.0 dB).
    pub inter_resonator_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
