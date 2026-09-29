//! Parameters and metrics for quantum acoustoelectric moiré superlattices,
//! flat phonon bands, and correlated electron-phonon pairing.

/// Parameters for acoustoelectric moiré heterostructures and dynamic strain potentials.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustoelectricMoireParams {
    /// Twist angle $\theta$ in degrees (nominal $0.8^\circ - 2.5^\circ$).
    pub twist_angle_deg: f64,
    /// Interlayer moiré potential amplitude $V_0$ in $\text{meV}$ (nominal $10.0 - 80.0\text{ meV}$).
    pub moire_potential_amplitude_mev: f64,
    /// Acoustic SAW central drive frequency $f_0$ in $\text{GHz}$ (nominal $1.0 - 6.0\text{ GHz}$).
    pub saw_frequency_ghz: f64,
    /// Dynamic piezoelectric strain amplitude $\epsilon_0$ in units of $10^{-4}$ (nominal $0.5 - 5.0$).
    pub dynamic_strain_amplitude_1e4: f64,
    /// Acoustic deformation potential $\Xi$ in $\text{eV}$ (nominal $2.0 - 10.0\text{ eV}$).
    pub deformation_potential_ev: f64,
    /// Unquenched bare phonon bandwidth $W_0$ in $\text{meV}$ (nominal $15.0 - 50.0\text{ meV}$).
    pub bare_phonon_bandwidth_mev: f64,
    /// On-site Coulomb correlation energy $U$ in $\text{meV}$ (nominal $25.0 - 90.0\text{ meV}$).
    pub coulomb_correlation_u_mev: f64,
    /// Effective electronic dielectric constant $\epsilon_r$ (nominal $4.0 - 15.0$).
    pub dielectric_constant: f64,
}

impl Default for AcoustoelectricMoireParams {
    fn default() -> Self {
        Self {
            twist_angle_deg: 1.10, // near magic angle ~ 1.08 deg
            moire_potential_amplitude_mev: 35.0,
            saw_frequency_ghz: 2.8,
            dynamic_strain_amplitude_1e4: 2.0,
            deformation_potential_ev: 5.5,
            bare_phonon_bandwidth_mev: 28.0,
            coulomb_correlation_u_mev: 45.0,
            dielectric_constant: 6.5,
        }
    }
}

/// Evaluated metrics for acoustoelectric moiré superlattices and correlated flat bands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustoelectricMoireMetrics {
    /// Phonon kinetic bandwidth quenching ratio $W_0 / W_{\mathrm{ph}}$ ($\ge 10.0\times$).
    pub bandwidth_quenching_ratio: f64,
    /// Correlated electron-phonon pairing enhancement ratio $\lambda_{\mathrm{eff}} / \lambda_0$ ($\ge 3.0\times$).
    pub electron_phonon_pairing_ratio: f64,
    /// Strong correlation ratio $U / W_{\mathrm{ph}}$ ($\ge 3.0$).
    pub correlation_ratio_u_over_w: f64,
    /// Acoustic moiré topological minigap $\Delta_M$ in $\text{meV}$ ($\ge 5.0\text{ meV}$).
    pub moire_minigap_mev: f64,
    /// Programmable quantum simulator array gate fidelity in percent ($\ge 98.0\%$).
    pub quantum_simulation_fidelity_pct: f64,
    /// Acoustic Wigner crystallization melting temperature in Kelvin ($\ge 20.0\text{ K}$).
    pub wigner_crystal_melting_temp_k: f64,
}

impl AcoustoelectricMoireParams {
    /// Creates a new parameter set for acoustoelectric moiré superlattices.
    pub fn new(
        theta_deg: f64,
        v0_mev: f64,
        f0_ghz: f64,
        strain_1e4: f64,
        w0_mev: f64,
        u_mev: f64,
    ) -> Self {
        Self {
            twist_angle_deg: theta_deg.clamp(0.2, 10.0),
            moire_potential_amplitude_mev: v0_mev.clamp(1.0, 300.0),
            saw_frequency_ghz: f0_ghz.clamp(0.2, 20.0),
            dynamic_strain_amplitude_1e4: strain_1e4.clamp(0.1, 20.0),
            bare_phonon_bandwidth_mev: w0_mev.clamp(5.0, 100.0),
            coulomb_correlation_u_mev: u_mev.clamp(5.0, 200.0),
            ..Default::default()
        }
    }
}
