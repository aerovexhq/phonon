//! Spin Pumping & Inverse Spin Hall Effect (ISHE) Models.
//!
//! Formulates interfacial dynamic spin injection across ferromagnet/heavy-metal (YIG/Pt)
//! interfaces, spin-mixing conductance, Gilbert damping enhancement, and transverse ISHE
//! charge voltage generation.

use super::magnon_cavity::YigMaterial;
use phonon_core::constants::{ELEMENTARY_CHARGE, H_BAR};
use std::f64::consts::PI;

/// Bohr magneton $\mu_B$ in J/T.
pub const BOHR_MAGNETON: f64 = 9.274_010_078_3e-24;

/// Electron Landé g-factor.
pub const G_FACTOR_ELECTRON: f64 = 2.002_319_304_36;

/// Heterostructure interface between a ferromagnet (YIG) and a normal metal (Platinum, Pt).
#[derive(Debug, Clone, PartialEq)]
pub struct YigPtInterface {
    /// Real part of the interfacial spin mixing conductance $g_r^{\uparrow\downarrow}$ in $\text{m}^{-2}$.
    pub spin_mixing_conductance: f64,
    /// YIG layer effective thickness $d_{YIG}$ in meters (default 20 nm).
    pub yig_thickness_m: f64,
    /// Platinum layer thickness $d_{Pt}$ in meters (default 10 nm).
    pub pt_thickness_m: f64,
    /// Platinum strip length $L_{Pt}$ in meters (default 2 mm).
    pub pt_length_m: f64,
    /// Platinum strip width $w_{Pt}$ in meters (default 1 mm).
    pub pt_width_m: f64,
    /// Platinum electrical conductivity $\sigma_{Pt}$ in S/m (default 4.0e6 S/m).
    pub pt_conductivity: f64,
    /// Platinum spin Hall angle $\theta_{SH}$ (dimensionless, ~0.08).
    pub spin_hall_angle: f64,
    /// Platinum spin diffusion length $\lambda_{sd}$ in meters (default 3 nm).
    pub spin_diffusion_length_m: f64,
}

impl Default for YigPtInterface {
    fn default() -> Self {
        Self {
            spin_mixing_conductance: 4.0e18,
            yig_thickness_m: 20.0e-9,
            pt_thickness_m: 10.0e-9,
            pt_length_m: 2.0e-3,
            pt_width_m: 1.0e-3,
            pt_conductivity: 4.0e6,
            spin_hall_angle: 0.08,
            spin_diffusion_length_m: 3.0e-9,
        }
    }
}

impl YigPtInterface {
    /// Additional Gilbert damping enhancement $\Delta\alpha$ due to spin pumping into Pt:
    /// $$\Delta\alpha = \frac{g \mu_B g_r^{\uparrow\downarrow}}{4\pi M_s d_{YIG}}$$
    pub fn damping_enhancement(&self, yig: &YigMaterial) -> f64 {
        let num = G_FACTOR_ELECTRON * BOHR_MAGNETON * self.spin_mixing_conductance;
        let den = 4.0 * PI * yig.saturation_magnetization * self.yig_thickness_m;
        num / den
    }

    /// Total effective Gilbert damping $\alpha_{eff} = \alpha_0 + \Delta\alpha$.
    pub fn total_damping(&self, yig: &YigMaterial) -> f64 {
        yig.intrinsic_damping + self.damping_enhancement(yig)
    }

    /// Ohmic resistance of the Platinum strip along its length $L_{Pt}$ in Ohms:
    /// $$R_{Pt} = \frac{L_{Pt}}{\sigma_{Pt} w_{Pt} d_{Pt}}$$
    pub fn pt_resistance_ohms(&self) -> f64 {
        self.pt_length_m / (self.pt_conductivity * self.pt_width_m * self.pt_thickness_m)
    }

    /// DC spin current density $J_s^{dc}$ injected across the interface in $\text{J/m}^2$:
    /// $$J_s^{dc} = \frac{\hbar g_r^{\uparrow\downarrow}}{4\pi} \omega \sin^2\theta_c$$
    pub fn spin_current_density_dc(&self, omega_rad: f64, cone_angle_rad: f64) -> f64 {
        let sin_theta = cone_angle_rad.sin();
        (H_BAR * self.spin_mixing_conductance / (4.0 * PI)) * omega_rad * (sin_theta * sin_theta)
    }

    /// Equivalent DC charge current density $j_s^{dc} = \frac{2e}{\hbar} J_s^{dc}$ in $\text{A/m}^2$.
    pub fn charge_current_density_equivalent(&self, js_dc: f64) -> f64 {
        (2.0 * ELEMENTARY_CHARGE / H_BAR) * js_dc
    }

    /// Total integrated transverse ISHE charge current $I_c$ in Amperes:
    /// $$I_c = w_{Pt} \theta_{SH} \lambda_{sd} \tanh\left(\frac{d_{Pt}}{2\lambda_{sd}}\right) j_s^{dc}$$
    pub fn ishe_charge_current(&self, omega_rad: f64, cone_angle_rad: f64) -> f64 {
        let js_dc = self.spin_current_density_dc(omega_rad, cone_angle_rad);
        let js_charge = self.charge_current_density_equivalent(js_dc);
        let tanh_term = (self.pt_thickness_m / (2.0 * self.spin_diffusion_length_m)).tanh();
        self.pt_width_m
            * self.spin_hall_angle
            * self.spin_diffusion_length_m
            * tanh_term
            * js_charge
    }

    /// Open-circuit transverse DC voltage $V_{ISHE}$ measured across the Platinum strip:
    /// $$V_{ISHE} = R_{Pt} I_c$$
    pub fn ishe_voltage_volts(&self, omega_rad: f64, cone_angle_rad: f64) -> f64 {
        let r_pt = self.pt_resistance_ohms();
        let i_c = self.ishe_charge_current(omega_rad, cone_angle_rad);
        r_pt * i_c
    }
}
