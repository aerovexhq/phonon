#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for non-Abelian
//! quantum acoustic twisted bilayer topological superfluidity and chiral Majorana
//! vortex networks.

/// Physical parameter configuration for non-Abelian quantum acoustic twisted bilayer
/// topological superfluidity and chiral Majorana vortex networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwistedBilayerTopologicalSuperfluidParams {
    /// Moiré superlattice twist angle in degrees (clamp 0.80 to 1.40, default 1.12).
    pub twist_angle_degrees: f64,
    /// Interlayer Josephson acoustic tunneling coupling in meV (clamp 5.0 to 50.0, default 22.0).
    pub interlayer_josephson_coupling_mev: f64,
    /// Chiral p-wave topological pairing amplitude in meV (clamp 2.0 to 30.0, default 14.5).
    pub p_wave_pairing_amplitude_mev: f64,
    /// Acoustic vortex core resonance frequency in GHz (clamp 1.0 to 15.0, default 4.8).
    pub acoustic_vortex_frequency_ghz: f64,
    /// Operating cryogenic temperature in milli-Kelvin (clamp 1.0 to 50.0, default 12.0).
    pub cryogenic_temperature_mk: f64,
    /// Acoustic vortex core radius in nanometers (clamp 10.0 to 150.0, default 45.0).
    pub vortex_core_radius_nm: f64,
    /// Inter-vortex center-to-center separation distance in micrometers (clamp 0.5 to 10.0, default 2.8).
    pub inter_vortex_separation_um: f64,
    /// Microscopic pinning potential barrier energy in meV (clamp 1.0 to 25.0, default 9.2).
    pub pinning_potential_barrier_mev: f64,
}

impl Default for TwistedBilayerTopologicalSuperfluidParams {
    fn default() -> Self {
        Self {
            twist_angle_degrees: 1.12,
            interlayer_josephson_coupling_mev: 22.0,
            p_wave_pairing_amplitude_mev: 14.5,
            acoustic_vortex_frequency_ghz: 4.8,
            cryogenic_temperature_mk: 12.0,
            vortex_core_radius_nm: 45.0,
            inter_vortex_separation_um: 2.8,
            pinning_potential_barrier_mev: 9.2,
        }
    }
}

impl TwistedBilayerTopologicalSuperfluidParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        twist_angle_degrees: f64,
        interlayer_josephson_coupling_mev: f64,
        p_wave_pairing_amplitude_mev: f64,
        acoustic_vortex_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        vortex_core_radius_nm: f64,
        inter_vortex_separation_um: f64,
        pinning_potential_barrier_mev: f64,
    ) -> Self {
        Self {
            twist_angle_degrees: twist_angle_degrees.clamp(0.80, 1.40),
            interlayer_josephson_coupling_mev: interlayer_josephson_coupling_mev.clamp(5.0, 50.0),
            p_wave_pairing_amplitude_mev: p_wave_pairing_amplitude_mev.clamp(2.0, 30.0),
            acoustic_vortex_frequency_ghz: acoustic_vortex_frequency_ghz.clamp(1.0, 15.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            vortex_core_radius_nm: vortex_core_radius_nm.clamp(10.0, 150.0),
            inter_vortex_separation_um: inter_vortex_separation_um.clamp(0.5, 10.0),
            pinning_potential_barrier_mev: pinning_potential_barrier_mev.clamp(1.0, 25.0),
        }
    }
}

/// Multi-physics evaluation metrics for non-Abelian quantum acoustic twisted bilayer
/// topological superfluidity and chiral Majorana vortex networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwistedBilayerTopologicalSuperfluidMetrics {
    /// Acoustic vortex quantum state fidelity (target >= 0.9980).
    pub vortex_state_fidelity: f64,
    /// Topological vortex pinning energy protection gap in MHz (target >= 40.0 MHz).
    pub topological_vortex_pinning_gap_mhz: f64,
    /// Inter-vortex crosstalk acoustic isolation in decibels (target >= 52.0 dB).
    pub inter_vortex_crosstalk_isolation_db: f64,
    /// Topological vortex mode dephasing rate in Hz (target <= 18.0 Hz).
    pub topological_vortex_dephasing_rate_hz: f64,
    /// Chiral Majorana zero mode wavefunction purity (target >= 0.992).
    pub chiral_majorana_mode_purity: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
