//! Non-Equilibrium Green's Function (NEGF) transport through molecular magnetic junctions:
//! Kondo resonance splittings, spin-dependent transmission, and molecular TMR.

use crate::stno::giant_spin::{BOHR_MAGNETON, BOLTZMANN_K};

/// Parameters for a magnetic molecule / adatom coupled to source and drain electrodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NegfMolecularJunctionParams {
    /// Molecular orbital single-particle energy level epsilon_0 in eV.
    pub molecular_level_ev: f64,
    /// On-site Coulomb charging energy U in eV.
    pub on_site_coulomb_u_ev: f64,
    /// Left lead electronic coupling broadening Gamma_L in eV.
    pub gamma_left_ev: f64,
    /// Right lead electronic coupling broadening Gamma_R in eV.
    pub gamma_right_ev: f64,
    /// Lead spin polarization P_lead in [0, 1).
    pub lead_spin_polarization: f64,
    /// Electron temperature in Kelvin.
    pub temperature_k: f64,
}

impl NegfMolecularJunctionParams {
    /// Constructs molecular junction parameters.
    pub fn new(
        molecular_level_ev: f64,
        on_site_coulomb_u_ev: f64,
        gamma_left_ev: f64,
        gamma_right_ev: f64,
        lead_spin_polarization: f64,
        temperature_k: f64,
    ) -> Self {
        Self {
            molecular_level_ev,
            on_site_coulomb_u_ev,
            gamma_left_ev,
            gamma_right_ev,
            lead_spin_polarization,
            temperature_k,
        }
    }

    /// Baseline single C60-transition-metal adatom junction (e.g. Co/Au with T_K ~ 27 K).
    pub fn standard_magnetic_adatom() -> Self {
        Self::new(-0.18, 0.8, 0.04, 0.04, 0.4, 4.2)
    }

    /// Total lead coupling Gamma = Gamma_L + Gamma_R in eV.
    pub fn total_gamma_ev(&self) -> f64 {
        self.gamma_left_ev + self.gamma_right_ev
    }

    /// Kondo temperature T_K in Kelvin:
    /// $$k_B T_K pprox \sqrt{\frac{U \Gamma}{2}} \exp\left( -\frac{\pi |\epsilon_0| (|\epsilon_0| + U)}{2 U \Gamma} \right)$$
    pub fn kondo_temperature_k(&self) -> f64 {
        let u = self.on_site_coulomb_u_ev;
        let gamma = self.total_gamma_ev();
        let eps = self.molecular_level_ev.abs();

        let prefactor_ev = (0.5 * u * gamma).sqrt();
        let exponent = -(std::f64::consts::PI * eps * (eps + u)) / (2.0 * u * gamma.max(1e-6));
        let tk_ev = prefactor_ev * exponent.exp();

        // Convert eV to Kelvin: E_ev * 1.602176634e-19 / k_B
        let ev_to_joules = 1.602_176_634e-19;
        (tk_ev * ev_to_joules) / BOLTZMANN_K
    }

    /// Kondo resonance energy peak splitting under magnetic field B (in Tesla):
    /// $$\Delta \epsilon_K = 2 g \mu_B B$$ in eV.
    pub fn kondo_zeeman_splitting_ev(&self, b_field_tesla: f64, g_factor: f64) -> f64 {
        let delta_joules = 2.0 * g_factor * BOHR_MAGNETON * b_field_tesla;
        delta_joules / 1.602_176_634e-19
    }

    /// Tunneling Magnetoresistance (Julliere formula) TMR = 2 P_L P_R / (1 - P_L P_R):
    pub fn tunnel_magnetoresistance_ratio(&self) -> f64 {
        let p = self.lead_spin_polarization;
        (2.0 * p * p) / (1.0 - p * p).max(1e-6)
    }

    /// Spin-resolved lead coupling Gamma_{L, sigma} = Gamma_L * (1 +- P):
    pub fn spin_lead_couplings(&self) -> (f64, f64) {
        let p = self.lead_spin_polarization;
        let gamma_up = self.gamma_left_ev * (1.0 + p);
        let gamma_dn = self.gamma_left_ev * (1.0 - p);
        (gamma_up, gamma_dn)
    }
}
