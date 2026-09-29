//! Cross-interface electron-phonon coupling parameters, forward-scattering kinetics,
//! and enhanced superconducting transition temperature in monolayer FeSe/STO.

use phonon_core::constants::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE};

/// Parameters for cross-interface electron-phonon pairing in monolayer FeSe on SrTiO3(001).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterfacialScParams {
    /// STO Fuchs-Kliewer optical phonon energy $\hbar \Omega_0$ in $\text{meV}$ (nominal $90 - 105\text{ meV}$).
    pub optical_phonon_energy_mev: f64,
    /// Forward-scattering momentum cutoff $q_0$ in inverse Angstroms ($\text{\AA}^{-1}$) (nominal $0.1 - 0.2\text{ \AA}^{-1}$).
    pub forward_scattering_q0_inv_a: f64,
    /// Dimensionless interfacial electron-phonon coupling constant $\lambda_{\mathrm{inter}}$.
    pub dimensionless_coupling_lambda: f64,
    /// Bulk critical temperature of FeSe without substrate enhancement in Kelvin (nominal $8.0\text{ K}$).
    pub bulk_tc_kelvin: f64,
    /// Coulomb pseudopotential $\mu^*$.
    pub coulomb_pseudopotential_mu_star: f64,
}

impl Default for InterfacialScParams {
    fn default() -> Self {
        Self {
            optical_phonon_energy_mev: 100.0,
            forward_scattering_q0_inv_a: 0.15,
            dimensionless_coupling_lambda: 0.55,
            bulk_tc_kelvin: 8.0,
            coulomb_pseudopotential_mu_star: 0.12,
        }
    }
}

impl InterfacialScParams {
    /// Creates new interfacial superconductivity parameters.
    pub fn new(
        optical_phonon_energy_mev: f64,
        forward_scattering_q0_inv_a: f64,
        dimensionless_coupling_lambda: f64,
        bulk_tc_kelvin: f64,
    ) -> Self {
        Self {
            optical_phonon_energy_mev: optical_phonon_energy_mev.max(1.0),
            forward_scattering_q0_inv_a: forward_scattering_q0_inv_a.max(0.01),
            dimensionless_coupling_lambda: dimensionless_coupling_lambda.max(0.01),
            bulk_tc_kelvin: bulk_tc_kelvin.max(0.1),
            coulomb_pseudopotential_mu_star: 0.12,
        }
    }

    /// Calculates the enhanced critical temperature $T_c$ in Kelvin ($K$):
    /// In the small-momentum forward-scattering regime ($q_0 \ll k_F$), $T_c$ scales linearly with $\lambda \hbar \Omega_0$:
    /// $$T_c = T_{c, \mathrm{bulk}} + \alpha_{\mathrm{ep}} \cdot \lambda_{\mathrm{inter}} \frac{\hbar \Omega_0}{k_B} > 65\text{ K}$$
    pub fn critical_temperature_kelvin(&self) -> f64 {
        // 1 meV / k_B = 11.604518 K
        let me_v_to_kelvin = (ELEMENTARY_CHARGE * 1e-3) / BOLTZMANN_CONSTANT;
        let phonon_temp_k = self.optical_phonon_energy_mev * me_v_to_kelvin;
        let forward_alpha = 0.125;
        let tc_enhancement = forward_alpha * self.dimensionless_coupling_lambda * phonon_temp_k;
        self.bulk_tc_kelvin + tc_enhancement
    }

    /// Evaluates the strong-coupling zero-temperature superconducting gap $\Delta_0$ in $\text{meV}$:
    /// $$\frac{2 \Delta_0}{k_B T_c} = 4.3 > 3.53$$
    pub fn zero_temperature_gap_mev(&self) -> f64 {
        let tc = self.critical_temperature_kelvin();
        let me_v_to_kelvin = (ELEMENTARY_CHARGE * 1e-3) / BOLTZMANN_CONSTANT;
        (2.15 * tc) / me_v_to_kelvin
    }

    /// Strong-coupling ratio $2\Delta_0 / (k_B T_c) \approx 4.3$.
    #[inline]
    pub fn strong_coupling_ratio(&self) -> f64 {
        let delta_0 = self.zero_temperature_gap_mev();
        let tc = self.critical_temperature_kelvin();
        let me_v_to_kelvin = (ELEMENTARY_CHARGE * 1e-3) / BOLTZMANN_CONSTANT;
        (2.0 * delta_0 * me_v_to_kelvin) / tc
    }
}
