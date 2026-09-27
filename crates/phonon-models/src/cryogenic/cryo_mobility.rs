//! Cryogenic carrier mobility models down to 4 Kelvin.
//!
//! Formulates multi-mechanism scattering via Matthiessen's rule:
//! - Brooks-Herring ionized impurity scattering $\mu_{ii}(T) \propto T^{3/2} / N_i$
//! - Erginsoy neutral impurity scattering $\mu_{ni} \propto 1 / N_n$ (temperature-independent, dominant at 4K freeze-out)
//! - Acoustic phonon scattering $\mu_{ph}(T) \propto T^{-\alpha}$
//! - Surface roughness scattering $\mu_{sr}(\mathcal{E}_{eff}) \propto \mathcal{E}_{eff}^{-\beta}$

use phonon_core::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, EPSILON_0, EPSILON_R_SI, H_BAR};

/// Free electron rest mass in kg.
const ELECTRON_MASS: f64 = 9.109_383_701_5e-31;

/// Multi-mechanism cryogenic mobility model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CryogenicMobilityModel {
    /// Effective mass ratio $m^* / m_0$ (e.g. 0.26 for electrons in Si conduction band).
    pub effective_mass_ratio: f64,
    /// Relative dielectric permittivity $\epsilon_r$ of the semiconductor host.
    pub epsilon_r: f64,
    /// Acoustic phonon mobility at 300K in $m^2 / (V \cdot s)$ (e.g. $0.14 \text{ m}^2/(V\cdot s)$ = $1400 \text{ cm}^2/(V\cdot s)$).
    pub mu_acoustic_300k: f64,
    /// Acoustic phonon temperature exponent $\alpha_{ph}$ ($\approx 1.5 - 1.75$).
    pub acoustic_alpha: f64,
    /// Surface roughness scattering prefactor $\delta_{sr}$ in SI units.
    pub surface_roughness_delta: f64,
    /// Surface roughness field exponent $\beta_{sr}$ ($\approx 2.0$).
    pub surface_roughness_beta: f64,
}

impl Default for CryogenicMobilityModel {
    fn default() -> Self {
        Self {
            effective_mass_ratio: 0.26, // Silicon longitudinal/transverse average
            epsilon_r: EPSILON_R_SI,
            mu_acoustic_300k: 0.140, // 1400 cm^2 / (V * s)
            acoustic_alpha: 1.5,
            surface_roughness_delta: 1e17,
            surface_roughness_beta: 2.0,
        }
    }
}

impl CryogenicMobilityModel {
    /// Effective semiconductor mass $m^* = (m^* / m_0) m_0$ in kg.
    #[inline]
    pub fn effective_mass_kg(&self) -> f64 {
        self.effective_mass_ratio * ELECTRON_MASS
    }

    /// Dielectric permittivity $\epsilon_s = \epsilon_r \epsilon_0$ in F/m.
    #[inline]
    pub fn permittivity_si(&self) -> f64 {
        self.epsilon_r * EPSILON_0
    }

    /// Acoustic phonon mobility in $m^2 / (V \cdot s)$:
    /// $$\mu_{ph}(T) = \mu_0 \left( \frac{T}{300\text{ K}} \right)^{-\alpha_{ph}}$$
    pub fn acoustic_phonon_mobility(&self, temp_k: f64) -> f64 {
        let t = temp_k.max(1.0);
        let ratio = t / 300.0;
        self.mu_acoustic_300k * ratio.powf(-self.acoustic_alpha)
    }

    /// Brooks-Herring ionized impurity scattering mobility in $m^2 / (V \cdot s)$:
    /// $$\mu_{ii}(T) = \frac{2^{7/2} (4\pi \epsilon_s)^2 (k_B T)^{3/2}}{\pi^{3/2} q^3 m^{*1/2} N_i} \cdot \left[ \ln(1 + \gamma) - \frac{\gamma}{1+\gamma} \right]^{-1}$$
    /// where the screening parameter is:
    /// $$\gamma = \frac{24 m^* \epsilon_s (k_B T)^2}{q^2 \hbar^2 n}$$
    pub fn ionized_impurity_mobility(
        &self,
        temp_k: f64,
        n_ionized: f64,
        carrier_density: f64,
    ) -> f64 {
        if n_ionized <= 1e10 {
            return 1e4; // Infinitely high mobility when impurities are un-ionized
        }

        let t = temp_k.max(1.0);
        let kb_t = BOLTZMANN_CONSTANT * t;
        let q = ELEMENTARY_CHARGE;
        let eps_s = self.permittivity_si();
        let m_star = self.effective_mass_kg();
        let n = carrier_density.max(1e15);

        let hbar = H_BAR;
        let gamma_num = 24.0 * m_star * eps_s * kb_t * kb_t;
        let gamma_den = q * q * hbar * hbar * n;
        let gamma = (gamma_num / gamma_den).clamp(1e-6, 1e8);

        let screening_f = (1.0 + gamma).ln() - (gamma / (1.0 + gamma));
        let screening_f = screening_f.max(1e-6);

        let num = 2.0f64.powf(3.5) * (4.0 * std::f64::consts::PI * eps_s).powi(2) * kb_t.powf(1.5);
        let den = std::f64::consts::PI.powf(1.5) * q.powi(3) * m_star.sqrt() * n_ionized;

        (num / (den * screening_f)).clamp(1e-6, 1e4)
    }

    /// Erginsoy neutral impurity scattering mobility in $m^2 / (V \cdot s)$:
    /// $$\mu_{ni} = \frac{m^* q^3}{20 \epsilon_s \hbar^3 N_n}$$
    /// This mechanism is strictly temperature-independent and dominates at 4K where impurities freeze out into neutral states ($N_n = N_D - N_D^+$).
    pub fn neutral_impurity_mobility(&self, n_neutral: f64) -> f64 {
        if n_neutral <= 1e10 {
            return 1e4;
        }

        let q = ELEMENTARY_CHARGE;
        let eps_s = self.permittivity_si();
        let m_star = self.effective_mass_kg();
        let hbar = H_BAR;

        let num = m_star * q.powi(3);
        let den = 20.0 * eps_s * hbar.powi(3) * n_neutral;

        (num / den).clamp(1e-6, 1e4)
    }

    /// Surface roughness scattering in the inversion layer under transverse electric field $\mathcal{E}_{eff}$:
    /// $$\mu_{sr} = \delta_{sr} \mathcal{E}_{eff}^{-\beta_{sr}}$$
    pub fn surface_roughness_mobility(&self, effective_field_v_per_m: f64) -> f64 {
        let field = effective_field_v_per_m.abs();
        if field < 1e4 {
            return 1e4;
        }
        (self.surface_roughness_delta / field.powf(self.surface_roughness_beta)).clamp(1e-6, 1e4)
    }

    /// Total effective mobility $\mu_{eff}$ via Matthiessen's summation:
    /// $$\frac{1}{\mu_{eff}} = \frac{1}{\mu_{ph}} + \frac{1}{\mu_{ii}} + \frac{1}{\mu_{ni}} + \frac{1}{\mu_{sr}}$$
    pub fn effective_mobility(
        &self,
        temp_k: f64,
        n_ionized: f64,
        n_neutral: f64,
        carrier_density: f64,
        effective_field: f64,
    ) -> f64 {
        let mu_ph = self.acoustic_phonon_mobility(temp_k);
        let mu_ii = self.ionized_impurity_mobility(temp_k, n_ionized, carrier_density);
        let mu_ni = self.neutral_impurity_mobility(n_neutral);
        let mu_sr = self.surface_roughness_mobility(effective_field);

        let inv_mu = (1.0 / mu_ph) + (1.0 / mu_ii) + (1.0 / mu_ni) + (1.0 / mu_sr);
        (1.0 / inv_mu).clamp(1e-6, 1e4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acoustic_phonon_freezing_out() {
        let model = CryogenicMobilityModel::default();
        let mu_300 = model.acoustic_phonon_mobility(300.0);
        let mu_77 = model.acoustic_phonon_mobility(77.0);
        let mu_4 = model.acoustic_phonon_mobility(4.0);

        // Acoustic phonon scattering decreases drastically at cryogenic temperatures,
        // increasing phonon-limited mobility
        assert!((mu_300 - 0.14).abs() < 1e-4);
        assert!(mu_77 > mu_300 * 5.0);
        assert!(mu_4 > mu_77 * 10.0);
    }

    #[test]
    fn test_neutral_impurity_dominance_at_4k() {
        let model = CryogenicMobilityModel::default();
        // At 4K, N_neutral is high (~1e22 m^-3 = 1e16 cm^-3)
        let mu_ni = model.neutral_impurity_mobility(1e22);
        assert!(mu_ni > 0.0 && mu_ni.is_finite());

        // Effective mobility at 4K should be constrained by neutral impurity scattering
        let mu_eff = model.effective_mobility(4.0, 1e18, 1e22, 1e18, 1e4);
        assert!(mu_eff <= mu_ni);
        assert!(mu_eff > 0.0);
    }
}
