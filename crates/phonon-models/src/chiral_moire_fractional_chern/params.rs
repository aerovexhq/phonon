#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for chiral acoustic
//! moire fractional Chern insulators and anyonic interferometric braiding networks.

/// Physical parameter configuration for chiral acoustic moire fractional Chern insulators
/// and anyonic interferometric braiding networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralMoireFractionalChernParams {
    /// Acoustic superlattice twist angle in degrees (clamp 0.50 to 10.0, default 1.08 deg).
    pub twist_angle_deg: f64,
    /// Moire interlayer acoustic potential well depth in meV (clamp 2.0 to 50.0, default 18.0 meV).
    pub moire_potential_depth_mev: f64,
    /// Fractional filling factor nu of the topological flatband (clamp 0.10 to 1.0, default 0.333333).
    pub fractional_filling_factor: f64,
    /// Number of arms in the acoustic anyon interferometer network (clamp 2 to 8, default 4).
    pub acoustic_interferometer_arms: usize,
    /// Bandwidth of the isolated topological flatband in kHz (clamp 10.0 to 500.0, default 85.0 kHz).
    pub topological_flatband_width_khz: f64,
    /// Coherent anyonic braiding microwave drive frequency in GHz (clamp 1.0 to 15.0, default 5.2 GHz).
    pub braiding_drive_frequency_ghz: f64,
    /// Cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 12.0 mK).
    pub cryogenic_temperature_mk: f64,
    /// Chiral phonon acoustic damping rate in Hz (clamp 1.0 to 100.0, default 15.0 Hz).
    pub chiral_damping_rate_hz: f64,
}

impl Default for ChiralMoireFractionalChernParams {
    fn default() -> Self {
        Self {
            twist_angle_deg: 1.08,
            moire_potential_depth_mev: 18.0,
            fractional_filling_factor: 0.333333,
            acoustic_interferometer_arms: 4,
            topological_flatband_width_khz: 85.0,
            braiding_drive_frequency_ghz: 5.2,
            cryogenic_temperature_mk: 12.0,
            chiral_damping_rate_hz: 15.0,
        }
    }
}

impl ChiralMoireFractionalChernParams {
    /// Creates a new parameter configuration with rigorous physical boundary clamping.
    pub fn new(
        twist_angle_deg: f64,
        moire_potential_depth_mev: f64,
        fractional_filling_factor: f64,
        acoustic_interferometer_arms: usize,
        topological_flatband_width_khz: f64,
        braiding_drive_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        chiral_damping_rate_hz: f64,
    ) -> Self {
        Self {
            twist_angle_deg: twist_angle_deg.clamp(0.50, 10.0),
            moire_potential_depth_mev: moire_potential_depth_mev.clamp(2.0, 50.0),
            fractional_filling_factor: fractional_filling_factor.clamp(0.10, 1.0),
            acoustic_interferometer_arms: acoustic_interferometer_arms.clamp(2, 8),
            topological_flatband_width_khz: topological_flatband_width_khz.clamp(10.0, 500.0),
            braiding_drive_frequency_ghz: braiding_drive_frequency_ghz.clamp(1.0, 15.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            chiral_damping_rate_hz: chiral_damping_rate_hz.clamp(1.0, 100.0),
        }
    }
}

/// Multi-physics evaluation metrics for chiral acoustic moire fractional Chern insulators
/// and anyonic interferometric braiding networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralMoireFractionalChernMetrics {
    /// Fault-tolerant anyonic braiding phase fidelity (target >= 0.9980).
    pub anyonic_braiding_phase_fidelity: f64,
    /// Moire flatband anyon coherence lifetime in milliseconds (target >= 15.0 ms).
    pub moire_flatband_coherence_ms: f64,
    /// Non-adiabatic transition leakage probability during braiding (target <= 1.0e-5).
    pub non_adiabatic_braiding_leakage: f64,
    /// Quasiparticle parity poisoning immunity ratio in dB (target >= 42.0 dB).
    pub quasiparticle_parity_poisoning_immunity_db: f64,
    /// Braiding geometric phase stability error in radians (target <= 0.0020 rad).
    pub braiding_phase_stability_error_rad: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
