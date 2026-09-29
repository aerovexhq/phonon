//! Topological magnetoplasmons, chiral magnetic domain walls,
//! and non-reciprocal topological isolation waveguides.

/// Parameters for chiral magnetic domain walls and topological magnetoplasmons (TMP).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalMagnetoplasmonParams {
    /// Magnetic domain wall width $\Delta_{\mathrm{DW}}$ in nanometers (nominal $5.0 - 25.0\text{ nm}$).
    pub domain_wall_width_nm: f64,
    /// Interfacial Dzyaloshinskii-Moriya interaction (DMI) energy $D$ in $\text{mJ/m}^2$ (nominal $0.8 - 3.5\text{ mJ/m}^2$).
    pub chiral_dmi_energy_mj_m2: f64,
    /// Saturation magnetization $M_s$ in $\text{kA/m}$ (nominal $250 - 850\text{ kA/m}$).
    pub saturation_magnetization_ka_m: f64,
    /// Magnetoplasmon operating frequency $\nu$ in $\text{GHz}$ (nominal $15.0 - 60.0\text{ GHz}$).
    pub magnetoplasmon_frequency_ghz: f64,
    /// Waveguide propagation length $L$ in micrometers ($\mu\text{m}$) (nominal $2.0 - 30.0\,\mu\text{m}$).
    pub waveguide_length_um: f64,
    /// Forward propagation attenuation rate $\alpha_+$ in $\text{dB}/\mu\text{m}$ (nominal $0.02 - 0.15\text{ dB}/\mu\text{m}$).
    pub forward_attenuation_db_per_um: f64,
    /// Backward propagation attenuation rate $\alpha_-$ in $\text{dB}/\mu\text{m}$ (nominal $1.2 - 4.5\text{ dB}/\mu\text{m}$).
    pub backward_attenuation_db_per_um: f64,
    /// Topological magnetoplasmon group velocity $v_{\mathrm{TMP}}$ in $\text{m/s}$ (nominal $1.5\times 10^5 - 6.0\times 10^5\text{ m/s}$).
    pub plasmon_velocity_m_s: f64,
}

impl Default for TopologicalMagnetoplasmonParams {
    fn default() -> Self {
        Self {
            domain_wall_width_nm: 12.0,
            chiral_dmi_energy_mj_m2: 2.0,
            saturation_magnetization_ka_m: 550.0,
            magnetoplasmon_frequency_ghz: 30.0,
            waveguide_length_um: 15.0,
            forward_attenuation_db_per_um: 0.05,
            backward_attenuation_db_per_um: 2.20,
            plasmon_velocity_m_s: 3.2e5,
        }
    }
}

/// Evaluated metrics for topological magnetoplasmons and non-reciprocal isolation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnetoplasmonMetrics {
    /// Non-reciprocal forward-to-backward isolation contrast in decibels ($\ge 25.0\text{ dB}$).
    pub non_reciprocal_isolation_db: f64,
    /// Forward insertion loss in decibels.
    pub forward_transmission_db: f64,
    /// Backward isolation attenuation in decibels.
    pub backward_isolation_db: f64,
    /// Chiral domain wall topological soliton charge $Q_{\mathrm{topo}} = \pm 1$.
    pub soliton_topological_charge: i32,
    /// Chiral edge magnetoplasmon group velocity in $\text{m/s}$.
    pub chiral_edge_velocity_m_s: f64,
    /// Transverse magnetoplasmon confinement skin depth in nanometers ($\text{nm}$).
    pub plasmon_confinement_depth_nm: f64,
    /// DMI chiral handedness index ($+1 =$ right-handed, $-1 =$ left-handed).
    pub dmi_chirality_handedness: i32,
}

impl TopologicalMagnetoplasmonParams {
    /// Creates a new parameter set for topological magnetoplasmons.
    pub fn new(
        domain_wall_width_nm: f64,
        chiral_dmi_energy_mj_m2: f64,
        waveguide_length_um: f64,
    ) -> Self {
        Self {
            domain_wall_width_nm: domain_wall_width_nm.max(1.0),
            chiral_dmi_energy_mj_m2: chiral_dmi_energy_mj_m2.max(0.1),
            waveguide_length_um: waveguide_length_um.max(0.5),
            ..Default::default()
        }
    }
}
