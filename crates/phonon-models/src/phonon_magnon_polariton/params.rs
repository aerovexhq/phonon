#![deny(unsafe_code)]

//! Physical parameters and metrics configuration for coherent quantum
//! phonon-magnon-polariton transducers and chiral spin-acoustic interfaces.

/// Physical parameter configuration for phonon-magnon-polariton transducers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononMagnonPolaritonParams {
    /// Spin-wave magnon resonance frequency in GHz (clamp 2.0 to 20.0, default 8.5).
    pub spin_wave_frequency_ghz: f64,
    /// Phononic acoustic crystal resonance frequency in GHz (clamp 2.0 to 20.0, default 8.5).
    pub acoustic_frequency_ghz: f64,
    /// Magnetoelastic coupling rate g_me in MHz (clamp 10.0 to 150.0, default 65.0).
    pub magnetoelastic_coupling_mhz: f64,
    /// Yttrium iron garnet (YIG) thin-film thickness in nm (clamp 10.0 to 500.0, default 100.0).
    pub yig_film_thickness_nm: f64,
    /// Piezoelectric acoustic cavity dissipation loss rate kappa_a in MHz (clamp 0.05 to 5.0, default 0.45).
    pub piezo_acoustic_loss_mhz: f64,
    /// Gilbert damping parameter alpha for spin-wave magnons (clamp 1.0e-5 to 1.0e-3, default 1.5e-4).
    pub magnon_damping_alpha: f64,
    /// Chiral dipolar asymmetry factor eta_chiral (clamp 0.50 to 0.99, default 0.88).
    pub chiral_asymmetry_factor: f64,
    /// Cryogenic dilution refrigerator operating temperature in milli-Kelvin (clamp 1.0 to 100.0, default 20.0).
    pub operating_temp_m_k: f64,
}

impl Default for PhononMagnonPolaritonParams {
    fn default() -> Self {
        Self {
            spin_wave_frequency_ghz: 8.5,
            acoustic_frequency_ghz: 8.5,
            magnetoelastic_coupling_mhz: 65.0,
            yig_film_thickness_nm: 100.0,
            piezo_acoustic_loss_mhz: 0.45,
            magnon_damping_alpha: 1.5e-4,
            chiral_asymmetry_factor: 0.88,
            operating_temp_m_k: 20.0,
        }
    }
}

impl PhononMagnonPolaritonParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        spin_wave_frequency_ghz: f64,
        acoustic_frequency_ghz: f64,
        magnetoelastic_coupling_mhz: f64,
        yig_film_thickness_nm: f64,
        piezo_acoustic_loss_mhz: f64,
        magnon_damping_alpha: f64,
        chiral_asymmetry_factor: f64,
        operating_temp_m_k: f64,
    ) -> Self {
        Self {
            spin_wave_frequency_ghz: spin_wave_frequency_ghz.clamp(2.0, 20.0),
            acoustic_frequency_ghz: acoustic_frequency_ghz.clamp(2.0, 20.0),
            magnetoelastic_coupling_mhz: magnetoelastic_coupling_mhz.clamp(10.0, 150.0),
            yig_film_thickness_nm: yig_film_thickness_nm.clamp(10.0, 500.0),
            piezo_acoustic_loss_mhz: piezo_acoustic_loss_mhz.clamp(0.05, 5.0),
            magnon_damping_alpha: magnon_damping_alpha.clamp(1.0e-5, 1.0e-3),
            chiral_asymmetry_factor: chiral_asymmetry_factor.clamp(0.50, 0.99),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 100.0),
        }
    }
}

/// Multi-physics evaluation metrics for coherent quantum phonon-magnon-polariton transducers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononMagnonPolaritonMetrics {
    /// Polariton cooperativity C (target >= 50.0).
    pub polariton_cooperativity: f64,
    /// Bidirectional phonon-magnon transduction efficiency (target >= 0.850).
    pub bidirectional_transduction_efficiency: f64,
    /// Spin-wave dephasing dissipation rate in MHz (target <= 1.00).
    pub spin_wave_dephasing_rate_mhz: f64,
    /// Non-reciprocal chiral magnon-phonon isolation in dB (target >= 30.0).
    pub chiral_isolation_db: f64,
    /// Single-quantum acoustic magnon conversion fidelity (target >= 0.990).
    pub single_quantum_conversion_fidelity: f64,
    /// Overall physical compliance flag across all polariton roadmap targets.
    pub is_physically_compliant: bool,
}
