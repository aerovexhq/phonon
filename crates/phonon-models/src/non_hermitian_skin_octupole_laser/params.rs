#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Hermitian higher-order topological skin sensors and chiral octupole phonon lasers.

/// Physical parameter configuration for quantum acoustic non-Hermitian higher-order
/// topological skin sensors and chiral octupole phonon lasers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianSkinOctupoleLaserParams {
    /// Non-Hermitian hopping asymmetry factor (clamp 1.05 to 3.0, default 1.65).
    pub non_hermitian_asymmetry_factor: f64,
    /// Higher-order octupole hopping coupling in meV (clamp 5.0 to 45.0, default 24.0).
    pub octupole_hopping_coupling_mev: f64,
    /// Gain medium saturation intensity in micro-watts (clamp 1.0 to 50.0, default 15.0).
    pub gain_saturation_intensity_uw: f64,
    /// Normalized acoustic optical/microwave pump rate P / P_th (clamp 1.10 to 5.0, default 2.2).
    pub pump_rate_normalized: f64,
    /// Acoustic octupole resonance frequency in GHz (clamp 1.0 to 15.0, default 5.8).
    pub acoustic_octupole_frequency_ghz: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// 3D phononic crystal lattice cell count per axis (clamp 4.0 to 24.0, default 10.0).
    pub lattice_cell_count_3d: f64,
    /// Non-Hermitian skin mode localization decay length in nanometers (clamp 10.0 to 120.0, default 35.0).
    pub skin_localization_decay_length_nm: f64,
}

impl Default for NonHermitianSkinOctupoleLaserParams {
    fn default() -> Self {
        Self {
            non_hermitian_asymmetry_factor: 1.65,
            octupole_hopping_coupling_mev: 24.0,
            gain_saturation_intensity_uw: 15.0,
            pump_rate_normalized: 2.2,
            acoustic_octupole_frequency_ghz: 5.8,
            cryogenic_temperature_mk: 10.0,
            lattice_cell_count_3d: 10.0,
            skin_localization_decay_length_nm: 35.0,
        }
    }
}

impl NonHermitianSkinOctupoleLaserParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        non_hermitian_asymmetry_factor: f64,
        octupole_hopping_coupling_mev: f64,
        gain_saturation_intensity_uw: f64,
        pump_rate_normalized: f64,
        acoustic_octupole_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        lattice_cell_count_3d: f64,
        skin_localization_decay_length_nm: f64,
    ) -> Self {
        Self {
            non_hermitian_asymmetry_factor: non_hermitian_asymmetry_factor.clamp(1.05, 3.0),
            octupole_hopping_coupling_mev: octupole_hopping_coupling_mev.clamp(5.0, 45.0),
            gain_saturation_intensity_uw: gain_saturation_intensity_uw.clamp(1.0, 50.0),
            pump_rate_normalized: pump_rate_normalized.clamp(1.10, 5.0),
            acoustic_octupole_frequency_ghz: acoustic_octupole_frequency_ghz.clamp(1.0, 15.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            lattice_cell_count_3d: lattice_cell_count_3d.clamp(4.0, 24.0),
            skin_localization_decay_length_nm: skin_localization_decay_length_nm.clamp(10.0, 120.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Hermitian higher-order
/// topological skin sensors and chiral octupole phonon lasers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianSkinOctupoleLaserMetrics {
    /// Corner lasing mode spectral purity (target >= 0.9980).
    pub corner_lasing_mode_purity: f64,
    /// Non-Hermitian skin displacement sensitivity factor (target >= 95.0).
    pub skin_sensitivity_factor: f64,
    /// Higher-order skin topological protection gap in MHz (target >= 48.0 MHz).
    pub higher_order_skin_topological_gap_mhz: f64,
    /// Corner-to-bulk crosstalk isolation in decibels (target >= 55.0 dB).
    pub corner_to_bulk_crosstalk_isolation_db: f64,
    /// Topological corner mode dephasing rate in Hz (target <= 13.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
