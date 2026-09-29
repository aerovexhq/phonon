#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for topological
//! moire acoustic polaritonic lattices and flat-band phonon superfluidity.

/// Physical parameter configuration for twisted bilayer phononic moire superlattices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalMoirePolaritonParams {
    /// Relative twist angle between acoustic layers in degrees (clamp 0.5 to 5.0, default 1.08).
    pub twist_angle_deg: f64,
    /// Center acoustic polariton frequency in GHz (clamp 1.0 to 15.0, default 4.2).
    pub acoustic_center_freq_ghz: f64,
    /// Interlayer acoustic tunneling amplitude in MHz (clamp 10.0 to 150.0, default 55.0).
    pub interlayer_tunneling_mhz: f64,
    /// Moire superlattice spatial period in nanometers (clamp 50.0 to 500.0, default 180.0).
    pub moire_period_nm: f64,
    /// Non-linear polariton-polariton contact interaction in ueV*um^2 (clamp 0.5 to 20.0, default 5.5).
    pub non_linear_polariton_interaction_uev_um2: f64,
    /// Operating cryostat bath temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub operating_temp_m_k: f64,
    /// Polariton lifetime in picoseconds (clamp 50.0 to 1000.0, default 350.0).
    pub polariton_lifetime_ps: f64,
    /// Intrinsic acoustic quality factor Q (clamp 1.0e6 to 1.0e8, default 2.5e7).
    pub acoustic_quality_factor: f64,
}

impl Default for TopologicalMoirePolaritonParams {
    fn default() -> Self {
        Self {
            twist_angle_deg: 1.08,
            acoustic_center_freq_ghz: 4.2,
            interlayer_tunneling_mhz: 55.0,
            moire_period_nm: 180.0,
            non_linear_polariton_interaction_uev_um2: 5.5,
            operating_temp_m_k: 15.0,
            polariton_lifetime_ps: 350.0,
            acoustic_quality_factor: 2.5e7,
        }
    }
}

impl TopologicalMoirePolaritonParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        twist_angle_deg: f64,
        acoustic_center_freq_ghz: f64,
        interlayer_tunneling_mhz: f64,
        moire_period_nm: f64,
        non_linear_polariton_interaction_uev_um2: f64,
        operating_temp_m_k: f64,
        polariton_lifetime_ps: f64,
        acoustic_quality_factor: f64,
    ) -> Self {
        Self {
            twist_angle_deg: twist_angle_deg.clamp(0.5, 5.0),
            acoustic_center_freq_ghz: acoustic_center_freq_ghz.clamp(1.0, 15.0),
            interlayer_tunneling_mhz: interlayer_tunneling_mhz.clamp(10.0, 150.0),
            moire_period_nm: moire_period_nm.clamp(50.0, 500.0),
            non_linear_polariton_interaction_uev_um2: non_linear_polariton_interaction_uev_um2
                .clamp(0.5, 20.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            polariton_lifetime_ps: polariton_lifetime_ps.clamp(50.0, 1000.0),
            acoustic_quality_factor: acoustic_quality_factor.clamp(1.0e6, 1.0e8),
        }
    }
}

/// Multi-physics performance evaluation metrics for topological moire acoustic polariton lattices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalMoirePolaritonMetrics {
    /// Landau critical phonon superfluid velocity in m/s (target >= 2500.0).
    pub superfluid_velocity_m_per_s: f64,
    /// Quantum sound dissipationless propagation loss in dB/cm (target <= 0.020).
    pub propagation_loss_db_per_cm: f64,
    /// Polariton Bose-Einstein condensation threshold acoustic density in m^-2 (target <= 5.0e12).
    pub condensation_threshold_density: f64,
    /// Quantized topological Chern invariant number C (target == 1).
    pub chern_number: i32,
    /// Flat-band acoustic polariton bandwidth in MHz (target <= 2.0).
    pub flat_band_bandwidth_mhz: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
