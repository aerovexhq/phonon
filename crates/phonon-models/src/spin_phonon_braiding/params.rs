#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological quantum error-mitigating spin-phonon braiding engines.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// quantum error-mitigating spin-phonon braiding engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpinPhononBraidingParams {
    /// Spin-phonon coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub spin_phonon_coupling_mev: f64,
    /// Topological pairing energy gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_pairing_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Braiding drift speed of localized defect modes in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub braiding_drift_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave decoupling power in micro-watts (clamp 0.5 to 30.0, default 5.8).
    pub microwave_decoupling_power_uw: f64,
    /// Quantum error mitigation polynomial order (clamp 1.0 to 8.0, default 4.0).
    pub error_mitigation_order: f64,
    /// Spin defect spatial separation distance in micrometers (clamp 0.5 to 20.0, default 4.8).
    pub spin_defect_separation_um: f64,
}

impl Default for SpinPhononBraidingParams {
    fn default() -> Self {
        Self {
            spin_phonon_coupling_mev: 16.5,
            topological_pairing_gap_mev: 22.0,
            acoustic_drive_frequency_ghz: 5.8,
            braiding_drift_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            microwave_decoupling_power_uw: 5.8,
            error_mitigation_order: 4.0,
            spin_defect_separation_um: 4.8,
        }
    }
}

impl SpinPhononBraidingParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        spin_phonon_coupling_mev: f64,
        topological_pairing_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        braiding_drift_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_decoupling_power_uw: f64,
        error_mitigation_order: f64,
        spin_defect_separation_um: f64,
    ) -> Self {
        Self {
            spin_phonon_coupling_mev: spin_phonon_coupling_mev.clamp(1.0, 35.0),
            topological_pairing_gap_mev: topological_pairing_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            braiding_drift_speed_m_per_s: braiding_drift_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_decoupling_power_uw: microwave_decoupling_power_uw.clamp(0.5, 30.0),
            error_mitigation_order: error_mitigation_order.clamp(1.0, 8.0),
            spin_defect_separation_um: spin_defect_separation_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// quantum error-mitigating spin-phonon braiding engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpinPhononBraidingMetrics {
    /// Quantum gate fidelity of error-mitigated spin-phonon braiding operations (target >= 0.9980).
    pub gate_fidelity: f64,
    /// Anyonic quantum state retention fraction (target >= 0.9970).
    pub anyonic_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-qubit crosstalk acoustic isolation in decibels (target >= 54.0 dB).
    pub inter_qubit_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
