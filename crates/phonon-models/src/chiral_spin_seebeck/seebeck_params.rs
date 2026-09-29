//! Chiral phonon-magnon spin Seebeck cascades, angular momentum transfer,
//! and cryogenic phononic thermocells.

/// Physical constants for spin Seebeck and caloritronic transport.
pub mod seebeck_constants {
    /// Reduced Planck constant $\hbar$ in $\text{J}\cdot\text{s}$.
    pub const HBAR: f64 = 1.054_571_817e-34;
    /// Elementary electric charge $e$ in Coulombs.
    pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19;
    /// Boltzmann constant $k_B$ in $\text{J/K}$.
    pub const K_BOLTZMANN: f64 = 1.380_649e-23;
    /// Bohr magneton $\mu_B$ in $\text{J/T}$.
    pub const BOHR_MAGNETON: f64 = 9.274_010_078_3e-24;
    /// Electron mass $m_e$ in $\text{kg}$.
    pub const ELECTRON_MASS: f64 = 9.109_383_701_5e-31;
}

/// Parameters for chiral phonon-magnon spin Seebeck thermoelectric cascades.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralSpinSeebeckParams {
    /// Hot reservoir temperature $T_{\mathrm{hot}}$ in Kelvin (nominal $0.3 - 4.5\text{ K}$).
    pub temp_hot_k: f64,
    /// Cold reservoir temperature $T_{\mathrm{cold}}$ in Kelvin (nominal $0.05 - 1.5\text{ K}$).
    pub temp_cold_k: f64,
    /// Interfacial real spin-mixing conductance $g_r^{\uparrow\downarrow}$ in $\text{m}^{-2}$ (nominal $10^{18} - 5\times 10^{19}\text{ m}^{-2}$).
    pub spin_mixing_conductance_m2: f64,
    /// Heavy metal spin Hall angle $\theta_{\mathrm{SH}}$ (nominal $0.08 - 0.25$).
    pub spin_hall_angle: f64,
    /// Detector stripe length $L_{\mathrm{det}}$ in micrometers ($\mu\text{m}$) (nominal $5.0 - 50.0\,\mu\text{m}$).
    pub detector_length_um: f64,
    /// Heavy metal electrical resistivity $\rho_{\mathrm{HM}}$ in $\Omega\cdot\text{m}$ (nominal $1.0\times 10^{-7} - 5.0\times 10^{-7}\,\Omega\cdot\text{m}$).
    pub detector_resistivity_ohm_m: f64,
    /// Degree of circular acoustic phonon polarization $p_{\mathrm{circ}} \in [0.5, 1.0]$.
    pub chiral_phonon_polarization: f64,
    /// Effective spin-phonon coupling rate $g_{\mathrm{sp}}$ in $\text{GHz}$ (nominal $0.1 - 2.5\text{ GHz}$).
    pub spin_phonon_coupling_ghz: f64,
    /// Forward thermal conductance $\kappa_+$ in $\text{W/K}$ (nominal $1.0\times 10^{-6} - 1.0\times 10^{-5}\text{ W/K}$).
    pub forward_thermal_conductance_w_k: f64,
    /// Backward thermal conductance $\kappa_-$ in $\text{W/K}$ (nominal $0.5\times 10^{-7} - 6.0\times 10^{-7}\text{ W/K}$).
    pub backward_thermal_conductance_w_k: f64,
    /// Thermocell internal load resistance $R_{\mathrm{load}}$ in $\Omega$ (nominal $50 - 500\,\Omega$).
    pub thermocell_internal_resistance_ohm: f64,
}

impl Default for ChiralSpinSeebeckParams {
    fn default() -> Self {
        Self {
            temp_hot_k: 2.2,
            temp_cold_k: 0.4,
            spin_mixing_conductance_m2: 1.5e19,
            spin_hall_angle: 0.12,
            detector_length_um: 20.0,
            detector_resistivity_ohm_m: 2.5e-7,
            chiral_phonon_polarization: 0.85,
            spin_phonon_coupling_ghz: 1.2,
            forward_thermal_conductance_w_k: 3.5e-6,
            backward_thermal_conductance_w_k: 2.2e-7,
            thermocell_internal_resistance_ohm: 120.0,
        }
    }
}

/// Evaluated metrics for chiral phonon-magnon spin Seebeck cascades and thermocells.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralSpinSeebeckMetrics {
    /// Thermal rectification ratio $\mathcal{R}_{\mathrm{th}} = \kappa_+ / \kappa_- \ge 10.0\times$.
    pub thermal_rectification_ratio: f64,
    /// Inverse Spin Hall Effect (ISHE) generated voltage $V_{\mathrm{ISHE}}$ in microvolts ($\mu\text{V}$) ($\ge 5.0\,\mu\text{V}$).
    pub spin_seebeck_voltage_uv: f64,
    /// Forward heat current $J_{\mathrm{heat}, +}$ in microwatts ($\mu\text{W}$).
    pub forward_heat_current_uw: f64,
    /// Backward heat current $J_{\mathrm{heat}, -}$ in microwatts ($\mu\text{W}$).
    pub backward_heat_current_uw: f64,
    /// Thermocell maximum electrical power output $P_{\mathrm{gen}}$ in picowatts ($\text{pW}$).
    pub thermocell_power_output_pw: f64,
    /// Theoretical Carnot efficiency limit $\eta_{\mathrm{Carnot}} \times 100\%$.
    pub carnot_efficiency_percent: f64,
    /// Thermocell heat-to-electricity conversion efficiency $\eta_{\mathrm{th}} \times 100\%$.
    pub thermocell_efficiency_percent: f64,
    /// Injected interfacial spin current density $J_s$ in $\text{A/m}^2$.
    pub spin_current_density_a_m2: f64,
}

impl ChiralSpinSeebeckParams {
    /// Creates a new parameter set for chiral phonon-magnon spin Seebeck transport.
    pub fn new(
        temp_hot_k: f64,
        temp_cold_k: f64,
        forward_conductance_w_k: f64,
        backward_conductance_w_k: f64,
    ) -> Self {
        Self {
            temp_hot_k: temp_hot_k.max(0.1),
            temp_cold_k: temp_cold_k.max(0.01),
            forward_thermal_conductance_w_k: forward_conductance_w_k.max(1.0e-9),
            backward_thermal_conductance_w_k: backward_conductance_w_k.max(1.0e-11),
            ..Default::default()
        }
    }
}
