#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for topological
//! acoustic parafermionic fractional Josephson interconnects and non-Abelian quantum logic.

use std::f64::consts::PI;

/// Physical parameter configuration for topological acoustic parafermionic fractional Josephson interconnects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FractionalJosephsonParafermionParams {
    /// Superconducting phase difference across the fractional junction in radians (clamp 0.0 to 6.0 * PI, default 2.0 * PI / 3.0).
    pub superconducting_phase_difference_rad: f64,
    /// Fractional quantum Hall filling factor nu (clamp 0.10 to 1.0, default 0.333333).
    pub fractional_filling_factor_nu: f64,
    /// Proximity-induced superconducting pairing gap in MHz (clamp 5.0 to 80.0, default 28.0 MHz).
    pub induced_pairing_gap_mhz: f64,
    /// Acoustic wavepacket center frequency driving braiding dynamics in GHz (clamp 1.0 to 12.0, default 4.2 GHz).
    pub acoustic_wavepacket_frequency_ghz: f64,
    /// Junction barrier transparency parameter (clamp 0.50 to 0.99, default 0.93).
    pub junction_barrier_transparency: f64,
    /// Parafermion acoustic braiding velocity in m/s (clamp 200.0 to 2500.0, default 1150.0 m/s).
    pub parafermion_braiding_velocity_mps: f64,
    /// Dilution cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 12.0 mK).
    pub cryogenic_temperature_mk: f64,
    /// Heterostructure channel length in micrometers (clamp 0.5 to 10.0, default 3.5 um).
    pub heterostructure_length_um: f64,
}

impl Default for FractionalJosephsonParafermionParams {
    fn default() -> Self {
        Self {
            superconducting_phase_difference_rad: 2.0 * PI / 3.0,
            fractional_filling_factor_nu: 0.333333,
            induced_pairing_gap_mhz: 28.0,
            acoustic_wavepacket_frequency_ghz: 4.2,
            junction_barrier_transparency: 0.93,
            parafermion_braiding_velocity_mps: 1150.0,
            cryogenic_temperature_mk: 12.0,
            heterostructure_length_um: 3.5,
        }
    }
}

impl FractionalJosephsonParafermionParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        superconducting_phase_difference_rad: f64,
        fractional_filling_factor_nu: f64,
        induced_pairing_gap_mhz: f64,
        acoustic_wavepacket_frequency_ghz: f64,
        junction_barrier_transparency: f64,
        parafermion_braiding_velocity_mps: f64,
        cryogenic_temperature_mk: f64,
        heterostructure_length_um: f64,
    ) -> Self {
        Self {
            superconducting_phase_difference_rad: superconducting_phase_difference_rad.clamp(0.0, 6.0 * PI),
            fractional_filling_factor_nu: fractional_filling_factor_nu.clamp(0.10, 1.0),
            induced_pairing_gap_mhz: induced_pairing_gap_mhz.clamp(5.0, 80.0),
            acoustic_wavepacket_frequency_ghz: acoustic_wavepacket_frequency_ghz.clamp(1.0, 12.0),
            junction_barrier_transparency: junction_barrier_transparency.clamp(0.50, 0.99),
            parafermion_braiding_velocity_mps: parafermion_braiding_velocity_mps.clamp(200.0, 2500.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            heterostructure_length_um: heterostructure_length_um.clamp(0.5, 10.0),
        }
    }
}

/// Multi-physics evaluation metrics for topological acoustic parafermionic fractional Josephson interconnects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FractionalJosephsonParafermionMetrics {
    /// Non-Abelian fractional braiding phase fidelity (target >= 0.9970).
    pub fractional_braiding_phase_fidelity: f64,
    /// Fractional Josephson supercurrent phase coherence lifetime in ms (target >= 10.0 ms).
    pub fractional_josephson_coherence_ms: f64,
    /// Non-adiabatic excitation leakage probability into continuum states (target <= 1.0e-5).
    pub non_adiabatic_excitation_leakage: f64,
    /// Quasiparticle parity poisoning immunity ratio in dB (target >= 40.0 dB).
    pub quasiparticle_parity_poisoning_immunity_db: f64,
    /// Fractional conductance quantization error in units of e^2/h (target <= 0.0030).
    pub fractional_conductance_quantization_error: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
