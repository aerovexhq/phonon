#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for topological
//! acoustic chiral skyrmion-lattice transducers and non-reciprocal magnon-polaron interconnects.

/// Physical parameter configuration for topological acoustic chiral skyrmion-lattice
/// transducers and non-reciprocal magnon-polaron interconnects in interfacial DMI heterostructures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralSkyrmionMagnonPolaronParams {
    /// Interfacial Dzyaloshinskii-Moriya exchange interaction strength in mJ/m^2 (clamp 0.50 to 6.0, default 2.6).
    pub dmi_exchange_strength_mj_m2: f64,
    /// Coherent surface acoustic strain drive amplitude in parts-per-million (ppm) (clamp 20.0 to 600.0, default 150.0).
    pub acoustic_strain_drive_amplitude_ppm: f64,
    /// Coherent magnon-polaron magneto-elastic coupling rate in MHz (clamp 5.0 to 100.0, default 35.0).
    pub magnon_polaron_coupling_mhz: f64,
    /// Chiral skyrmion triangular crystal lattice pitch / constant in nm (clamp 30.0 to 250.0, default 80.0).
    pub skyrmion_lattice_constant_nm: f64,
    /// Dimensionless magnetic Gilbert damping parameter alpha (clamp 0.001 to 0.05, default 0.012).
    pub gilbert_damping_alpha: f64,
    /// Coherent acoustic phonon driving frequency in GHz (clamp 1.0 to 15.0, default 5.5).
    pub acoustic_frequency_ghz: f64,
    /// Cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub cryogenic_temperature_mk: f64,
    /// Magnetic heterostructure thin-film thickness in nm (clamp 2.0 to 50.0, default 12.0).
    pub heterostructure_thickness_nm: f64,
}

impl Default for ChiralSkyrmionMagnonPolaronParams {
    fn default() -> Self {
        Self {
            dmi_exchange_strength_mj_m2: 2.6,
            acoustic_strain_drive_amplitude_ppm: 150.0,
            magnon_polaron_coupling_mhz: 35.0,
            skyrmion_lattice_constant_nm: 80.0,
            gilbert_damping_alpha: 0.012,
            acoustic_frequency_ghz: 5.5,
            cryogenic_temperature_mk: 15.0,
            heterostructure_thickness_nm: 12.0,
        }
    }
}

impl ChiralSkyrmionMagnonPolaronParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        dmi_exchange_strength_mj_m2: f64,
        acoustic_strain_drive_amplitude_ppm: f64,
        magnon_polaron_coupling_mhz: f64,
        skyrmion_lattice_constant_nm: f64,
        gilbert_damping_alpha: f64,
        acoustic_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        heterostructure_thickness_nm: f64,
    ) -> Self {
        Self {
            dmi_exchange_strength_mj_m2: dmi_exchange_strength_mj_m2.clamp(0.50, 6.0),
            acoustic_strain_drive_amplitude_ppm: acoustic_strain_drive_amplitude_ppm.clamp(20.0, 600.0),
            magnon_polaron_coupling_mhz: magnon_polaron_coupling_mhz.clamp(5.0, 100.0),
            skyrmion_lattice_constant_nm: skyrmion_lattice_constant_nm.clamp(30.0, 250.0),
            gilbert_damping_alpha: gilbert_damping_alpha.clamp(0.001, 0.05),
            acoustic_frequency_ghz: acoustic_frequency_ghz.clamp(1.0, 15.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            heterostructure_thickness_nm: heterostructure_thickness_nm.clamp(2.0, 50.0),
        }
    }
}

/// Multi-physics evaluation metrics for topological acoustic chiral skyrmion-lattice
/// transducers and non-reciprocal magnon-polaron interconnects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralSkyrmionMagnonPolaronMetrics {
    /// Topological Hall deflection angle theta_TH in degrees (target >= 18.0 deg).
    pub topological_hall_angle_deg: f64,
    /// Coherent magnon-polaron state transfer fidelity (target >= 0.9970).
    pub magnon_polaron_transfer_fidelity: f64,
    /// Non-reciprocal acoustic isolation ratio S21/S12 in dB (target >= 48.0 dB).
    pub non_reciprocal_acoustic_isolation_db: f64,
    /// Steady-state skyrmion lattice drift velocity v_d in m/s (target >= 180.0 m/s).
    pub skyrmion_drift_velocity_mps: f64,
    /// Topological charge conservation stability ratio Q/Q0 (target >= 0.990).
    pub topological_charge_stability_ratio: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
