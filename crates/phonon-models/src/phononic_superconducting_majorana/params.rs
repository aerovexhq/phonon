#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for non-Abelian
//! chiral Majorana bound states in topological phononic superconducting junctions.

/// Physical parameter configuration for semiconductor-superconductor phononic junctions
/// supporting topologically protected chiral Majorana zero modes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononicSuperconductingMajoranaParams {
    /// Proximity-induced s-wave superconducting pairing gap in MHz (clamp 15.0 to 120.0, default 45.0 MHz).
    pub superconducting_gap_mhz: f64,
    /// Rashba spin-orbit coupling strength in meV*nm (clamp 10.0 to 150.0, default 65.0 meV*nm).
    pub spin_orbit_coupling_mev_nm: f64,
    /// Zeeman splitting energy from external magnetic field in MHz (clamp 20.0 to 200.0, default 80.0 MHz).
    pub zeeman_splitting_mhz: f64,
    /// Surface acoustic wave driving frequency in GHz (clamp 0.5 to 10.0, default 3.4 GHz).
    pub acoustic_driving_frequency_ghz: f64,
    /// Acoustic strain modulation amplitude in parts per million (clamp 10.0 to 500.0, default 125.0 ppm).
    pub acoustic_strain_amplitude_ppm: f64,
    /// Dilution refrigerator cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 12.0 mK).
    pub cryogenic_temperature_mk: f64,
    /// Interface barrier junction transparency parameter (clamp 0.50 to 0.99, default 0.92).
    pub junction_transparency: f64,
    /// Semiconductor nanowire junction length in micrometers (clamp 0.5 to 10.0, default 2.8 um).
    pub nanowire_length_um: f64,
}

impl Default for PhononicSuperconductingMajoranaParams {
    fn default() -> Self {
        Self {
            superconducting_gap_mhz: 45.0,
            spin_orbit_coupling_mev_nm: 65.0,
            zeeman_splitting_mhz: 80.0,
            acoustic_driving_frequency_ghz: 3.4,
            acoustic_strain_amplitude_ppm: 125.0,
            cryogenic_temperature_mk: 12.0,
            junction_transparency: 0.92,
            nanowire_length_um: 2.8,
        }
    }
}

impl PhononicSuperconductingMajoranaParams {
    /// Creates a new parameter configuration with rigorous physical bounds clamping.
    pub fn new(
        superconducting_gap_mhz: f64,
        spin_orbit_coupling_mev_nm: f64,
        zeeman_splitting_mhz: f64,
        acoustic_driving_frequency_ghz: f64,
        acoustic_strain_amplitude_ppm: f64,
        cryogenic_temperature_mk: f64,
        junction_transparency: f64,
        nanowire_length_um: f64,
    ) -> Self {
        Self {
            superconducting_gap_mhz: superconducting_gap_mhz.clamp(15.0, 120.0),
            spin_orbit_coupling_mev_nm: spin_orbit_coupling_mev_nm.clamp(10.0, 150.0),
            zeeman_splitting_mhz: zeeman_splitting_mhz.clamp(20.0, 200.0),
            acoustic_driving_frequency_ghz: acoustic_driving_frequency_ghz.clamp(0.5, 10.0),
            acoustic_strain_amplitude_ppm: acoustic_strain_amplitude_ppm.clamp(10.0, 500.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            junction_transparency: junction_transparency.clamp(0.50, 0.99),
            nanowire_length_um: nanowire_length_um.clamp(0.5, 10.0),
        }
    }
}

/// Multi-physics evaluation metrics for topological phononic superconducting Majorana junctions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononicSuperconductingMajoranaMetrics {
    /// Non-Abelian Majorana braiding phase fidelity (target >= 0.9980).
    pub braiding_phase_fidelity: f64,
    /// Topological protection minigap isolating Majorana zero modes in MHz (target >= 22.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Non-adiabatic transition leakage probability into excited continuum states (target <= 1.0e-5).
    pub non_adiabatic_leakage_probability: f64,
    /// Quasiparticle poisoning immunity of the superconducting junction in dB (target >= 38.0 dB).
    pub quasiparticle_poisoning_immunity_db: f64,
    /// Quantized zero-bias conductance peak error in units of G_0 = 2e^2/h (target <= 0.0020 G_0).
    pub zero_bias_conductance_error_g0: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
