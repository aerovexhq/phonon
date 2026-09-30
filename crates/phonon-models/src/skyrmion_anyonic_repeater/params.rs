#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological skyrmion-lattice anyonic quantum repeaters and
//! entanglement distillation nodes.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// skyrmion-lattice anyonic quantum repeaters and entanglement distillation nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionAnyonicRepeaterParams {
    /// Interfacial Dzyaloshinskii-Moriya interaction energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub dzyaloshinskii_moriya_energy_mev: f64,
    /// Superconducting pairing energy gap in meV (clamp 2.0 to 45.0, default 21.0).
    pub superconducting_pairing_gap_mev: f64,
    /// Acoustic carrier frequency in GHz (clamp 1.0 to 12.0, default 5.6).
    pub acoustic_carrier_frequency_ghz: f64,
    /// Entanglement distillation shuttling speed in m/s (clamp 200.0 to 3000.0, default 1350.0).
    pub distillation_shuttling_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave pump power in micro-watts (clamp 0.5 to 30.0, default 5.5).
    pub microwave_pump_power_uw: f64,
    /// Skyrmion lattice constant in nanometers (clamp 30.0 to 250.0, default 90.0).
    pub skyrmion_lattice_constant_nm: f64,
    /// Node separation distance in micrometers (clamp 0.5 to 20.0, default 5.0).
    pub node_separation_distance_um: f64,
}

impl Default for SkyrmionAnyonicRepeaterParams {
    fn default() -> Self {
        Self {
            dzyaloshinskii_moriya_energy_mev: 16.5,
            superconducting_pairing_gap_mev: 21.0,
            acoustic_carrier_frequency_ghz: 5.6,
            distillation_shuttling_speed_m_per_s: 1350.0,
            cryogenic_temperature_mk: 10.0,
            microwave_pump_power_uw: 5.5,
            skyrmion_lattice_constant_nm: 90.0,
            node_separation_distance_um: 5.0,
        }
    }
}

impl SkyrmionAnyonicRepeaterParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        dzyaloshinskii_moriya_energy_mev: f64,
        superconducting_pairing_gap_mev: f64,
        acoustic_carrier_frequency_ghz: f64,
        distillation_shuttling_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_pump_power_uw: f64,
        skyrmion_lattice_constant_nm: f64,
        node_separation_distance_um: f64,
    ) -> Self {
        Self {
            dzyaloshinskii_moriya_energy_mev: dzyaloshinskii_moriya_energy_mev.clamp(1.0, 35.0),
            superconducting_pairing_gap_mev: superconducting_pairing_gap_mev.clamp(2.0, 45.0),
            acoustic_carrier_frequency_ghz: acoustic_carrier_frequency_ghz.clamp(1.0, 12.0),
            distillation_shuttling_speed_m_per_s: distillation_shuttling_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_pump_power_uw: microwave_pump_power_uw.clamp(0.5, 30.0),
            skyrmion_lattice_constant_nm: skyrmion_lattice_constant_nm.clamp(30.0, 250.0),
            node_separation_distance_um: node_separation_distance_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// skyrmion-lattice anyonic quantum repeaters and entanglement distillation nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionAnyonicRepeaterMetrics {
    /// Quantum acoustic repeater end-to-end fidelity (target >= 0.9980).
    pub repeater_fidelity: f64,
    /// Anyon bound state retention fraction (target >= 0.9970).
    pub anyon_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-node crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_node_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
