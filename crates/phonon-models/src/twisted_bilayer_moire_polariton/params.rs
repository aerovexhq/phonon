#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! twisted bilayer moire polariton superlattices and flat-band phonon superconductors.

/// Physical parameter configuration for quantum acoustic twisted bilayer moire polaritons
/// and flat-band phonon-mediated superconducting superlattices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwistedBilayerMoirePolaritonParams {
    /// Relative twist angle between bilayer graphene sheets in degrees (clamp 0.80 to 1.40, default 1.08).
    pub twist_angle_degrees: f64,
    /// Interlayer tunneling energy w in meV (clamp 50.0 to 150.0, default 110.0).
    pub interlayer_tunneling_energy_mev: f64,
    /// Acoustic phonon deformation potential D in eV (clamp 2.0 to 15.0, default 7.5).
    pub acoustic_deformation_potential_ev: f64,
    /// Moiré acoustic phonon driving frequency in GHz (clamp 0.5 to 10.0, default 3.6).
    pub moire_acoustic_frequency_ghz: f64,
    /// Operating cryogenic temperature in milli-Kelvin (clamp 1.0 to 50.0, default 18.0).
    pub cryogenic_temperature_mk: f64,
    /// Dimensionless electron-phonon Eliashberg coupling constant lambda (clamp 0.20 to 2.50, default 1.15).
    pub electron_phonon_coupling_lambda: f64,
    /// Inter-valley coherence length in nanometers (clamp 20.0 to 300.0, default 120.0).
    pub inter_valley_coherence_length_nm: f64,
    /// Superconducting channel transport length in micrometers (clamp 1.0 to 20.0, default 5.5).
    pub superconducting_channel_length_um: f64,
}

impl Default for TwistedBilayerMoirePolaritonParams {
    fn default() -> Self {
        Self {
            twist_angle_degrees: 1.08,
            interlayer_tunneling_energy_mev: 110.0,
            acoustic_deformation_potential_ev: 7.5,
            moire_acoustic_frequency_ghz: 3.6,
            cryogenic_temperature_mk: 18.0,
            electron_phonon_coupling_lambda: 1.15,
            inter_valley_coherence_length_nm: 120.0,
            superconducting_channel_length_um: 5.5,
        }
    }
}

impl TwistedBilayerMoirePolaritonParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        twist_angle_degrees: f64,
        interlayer_tunneling_energy_mev: f64,
        acoustic_deformation_potential_ev: f64,
        moire_acoustic_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        electron_phonon_coupling_lambda: f64,
        inter_valley_coherence_length_nm: f64,
        superconducting_channel_length_um: f64,
    ) -> Self {
        Self {
            twist_angle_degrees: twist_angle_degrees.clamp(0.80, 1.40),
            interlayer_tunneling_energy_mev: interlayer_tunneling_energy_mev.clamp(50.0, 150.0),
            acoustic_deformation_potential_ev: acoustic_deformation_potential_ev.clamp(2.0, 15.0),
            moire_acoustic_frequency_ghz: moire_acoustic_frequency_ghz.clamp(0.5, 10.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            electron_phonon_coupling_lambda: electron_phonon_coupling_lambda.clamp(0.20, 2.50),
            inter_valley_coherence_length_nm: inter_valley_coherence_length_nm.clamp(20.0, 300.0),
            superconducting_channel_length_um: superconducting_channel_length_um.clamp(1.0, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic twisted bilayer moire polariton
/// superlattices and flat-band phonon superconductors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwistedBilayerMoirePolaritonMetrics {
    /// Polariton superconducting state fidelity (target >= 0.9970).
    pub polariton_superconducting_fidelity: f64,
    /// Flat-band acoustic polariton group velocity in m/s (target <= 150.0 m/s).
    pub flat_band_group_velocity_mps: f64,
    /// Critical transition temperature Tc enhancement factor (target >= 4.50).
    pub tc_enhancement_factor: f64,
    /// Inter-valley crosstalk isolation in dB (target >= 50.0 dB).
    pub inter_valley_crosstalk_isolation_db: f64,
    /// Magic-angle alignment tolerance fraction (target >= 0.9980).
    pub magic_angle_alignment_tolerance_fraction: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
