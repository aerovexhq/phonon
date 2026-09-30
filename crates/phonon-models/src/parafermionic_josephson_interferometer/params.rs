#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! topological chiral parafermionic Josephson junctions and non-Abelian readout interferometers.

/// Physical parameter configuration for quantum acoustic topological chiral parafermionic
/// Josephson junctions and non-Abelian readout interferometers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParafermionicJosephsonInterferometerParams {
    /// Parafermion statistical order m in Z_m clock models (clamp 2.0 to 6.0, default 3.0).
    pub parafermion_order_m: f64,
    /// Josephson coupling energy E_J in meV (clamp 2.0 to 45.0, default 20.0).
    pub josephson_coupling_energy_mev: f64,
    /// Superconducting pairing gap Delta_sc in meV (clamp 1.0 to 25.0, default 12.5).
    pub superconducting_pairing_gap_mev: f64,
    /// Acoustic resonator fundamental frequency in GHz (clamp 1.0 to 12.0, default 6.2).
    pub acoustic_resonator_frequency_ghz: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave readout probe power in micro-watts (clamp 0.5 to 30.0, default 5.0).
    pub microwave_readout_power_uw: f64,
    /// Chiral Josephson junction length in nanometers (clamp 50.0 to 800.0, default 220.0).
    pub junction_length_nm: f64,
    /// Tunnel barrier normal transparency D (clamp 0.40 to 0.98, default 0.85).
    pub barrier_transparency: f64,
}

impl Default for ParafermionicJosephsonInterferometerParams {
    fn default() -> Self {
        Self {
            parafermion_order_m: 3.0,
            josephson_coupling_energy_mev: 20.0,
            superconducting_pairing_gap_mev: 12.5,
            acoustic_resonator_frequency_ghz: 6.2,
            cryogenic_temperature_mk: 10.0,
            microwave_readout_power_uw: 5.0,
            junction_length_nm: 220.0,
            barrier_transparency: 0.85,
        }
    }
}

impl ParafermionicJosephsonInterferometerParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        parafermion_order_m: f64,
        josephson_coupling_energy_mev: f64,
        superconducting_pairing_gap_mev: f64,
        acoustic_resonator_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        microwave_readout_power_uw: f64,
        junction_length_nm: f64,
        barrier_transparency: f64,
    ) -> Self {
        Self {
            parafermion_order_m: parafermion_order_m.clamp(2.0, 6.0),
            josephson_coupling_energy_mev: josephson_coupling_energy_mev.clamp(2.0, 45.0),
            superconducting_pairing_gap_mev: superconducting_pairing_gap_mev.clamp(1.0, 25.0),
            acoustic_resonator_frequency_ghz: acoustic_resonator_frequency_ghz.clamp(1.0, 12.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_readout_power_uw: microwave_readout_power_uw.clamp(0.5, 30.0),
            junction_length_nm: junction_length_nm.clamp(50.0, 800.0),
            barrier_transparency: barrier_transparency.clamp(0.40, 0.98),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic topological chiral parafermionic
/// Josephson junctions and non-Abelian readout interferometers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParafermionicJosephsonInterferometerMetrics {
    /// Parafermionic non-Abelian state readout fidelity (target >= 0.9980).
    pub state_readout_fidelity: f64,
    /// Parafermionic quantum state retention fraction (target >= 0.9970).
    pub parafermionic_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-junction acoustic and electromagnetic crosstalk isolation in decibels (target >= 54.0 dB).
    pub inter_junction_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
