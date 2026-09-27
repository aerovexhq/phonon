//! Displacement Damage Dose (DDD), Non-Ionizing Energy Loss (NIEL),
//! lattice Frenkel defects, carrier lifetime degradation, and dark current spikes.
//!
//! Formulates:
//! - Non-Ionizing Energy Loss (NIEL) and total displacement damage dose:
//!   $$D_{ddd} = S_{NIEL} \cdot \Phi \quad [\text{MeV/g}]$$
//! - Messenger-Spratt minority carrier lifetime reduction:
//!   $$\frac{1}{\tau(\Phi)} = \frac{1}{\tau_0} + K_{damage} \cdot \Phi \implies \tau(\Phi) = \frac{\tau_0}{1 + K_{damage} \tau_0 \Phi}$$
//! - Minority carrier diffusion length degradation:
//!   $$L_d(\Phi) = \sqrt{D \cdot \tau(\Phi)} = \frac{L_{d0}}{\sqrt{1 + K_{damage} \tau_0 \Phi}}$$
//! - Photodiode / image sensor depletion dark current generation spike:
//!   $$\Delta I_{dark} = \frac{q \cdot n_i \cdot V_{active}}{2} \left( \frac{1}{\tau(\Phi)} - \frac{1}{\tau_0} \right) = \frac{q \cdot n_i \cdot V_{active}}{2} K_{damage} \Phi$$
//! - BJT common-emitter current gain degradation:
//!   $$\frac{1}{\beta(\Phi)} = \frac{1}{\beta_0} + K_\beta \cdot \Phi \implies \beta(\Phi) = \frac{\beta_0}{1 + \beta_0 K_\beta \Phi}$$

use phonon_core::ELEMENTARY_CHARGE;

/// Conversion factor: 1 rad(Si) = 100 erg/g = 6.241509e7 MeV/g.
pub const RAD_SI_TO_MEV_PER_G: f64 = 6.241_509e7;

/// Physical parameters for Displacement Damage Dose (DDD) degradation in silicon devices.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplacementDamageModel {
    /// Non-Ionizing Energy Loss (NIEL) stopping power in $\text{MeV}\cdot\text{cm}^2/\text{g}$
    /// (e.g. $3.5\times 10^{-3}\text{ MeV}\cdot\text{cm}^2/\text{g}$ for $10\text{ MeV}$ protons).
    pub s_niel_mev_cm2_per_g: f64,
    /// Un-irradiated minority carrier lifetime $\tau_0$ in seconds ($s$) (typically $1\text{ }\mu\text{s} - 100\text{ }\mu\text{s}$).
    pub tau_0_s: f64,
    /// Carrier lifetime damage factor $K_{damage}$ in $\text{cm}^2/\text{s}$ (typically $10^{-7} - 10^{-5}\text{ cm}^2/\text{s}$).
    pub k_damage_cm2_per_s: f64,
    /// Un-irradiated BJT common-emitter current gain $\beta_0 = h_{FE0}$.
    pub beta_0: f64,
    /// BJT damage factor $K_\beta$ in $\text{cm}^2$ (typically $10^{-16} - 10^{-14}\text{ cm}^2/\text{particle}$).
    pub k_beta_cm2: f64,
    /// Carrier diffusion coefficient $D$ in $\text{m}^2/\text{s}$ (e.g. $3.5\times 10^{-3}\text{ m}^2/\text{s}$ for electrons).
    pub diffusion_coeff_m2_per_s: f64,
    /// Active depletion volume for photodiode dark current in cubic meters ($m^3$) (e.g. $10^{-16}\text{ m}^3$).
    pub active_volume_m3: f64,
    /// Intrinsic carrier concentration $n_i$ in $\text{m}^{-3}$ at 300 K ($\approx 1.5\times 10^{16}\text{ m}^{-3}$).
    pub ni_per_m3: f64,
}

impl DisplacementDamageModel {
    /// Presets a typical silicon bipolar & optoelectronic device in a space proton environment (10 MeV protons).
    pub fn space_proton_environment(beta_0: f64) -> Self {
        Self {
            s_niel_mev_cm2_per_g: 3.5e-3, // 3.5e-3 MeV*cm^2/g for 10 MeV protons
            tau_0_s: 10.0e-6,             // 10 microseconds pristine lifetime
            k_damage_cm2_per_s: 2.0e-6,   // 2e-6 cm^2/s for protons in Si
            beta_0: beta_0.max(1.0),
            k_beta_cm2: 5.0e-15,              // 5e-15 cm^2/proton
            diffusion_coeff_m2_per_s: 3.5e-3, // 35 cm^2/s = 3.5e-3 m^2/s
            active_volume_m3: 1.0e-16,        // 100 um^2 x 1 um depth = 1e-16 m^3
            ni_per_m3: 1.5e16,                // 1.5e10 cm^-3 = 1.5e16 m^-3
        }
    }

    /// Presets for 1 MeV equivalent neutron damage in power electronics and sensor detectors.
    pub fn neutron_1mev_environment(beta_0: f64) -> Self {
        Self {
            s_niel_mev_cm2_per_g: 2.0e-3,
            tau_0_s: 20.0e-6,
            k_damage_cm2_per_s: 1.0e-6,
            beta_0: beta_0.max(1.0),
            k_beta_cm2: 3.0e-15,
            diffusion_coeff_m2_per_s: 3.5e-3,
            active_volume_m3: 5.0e-16,
            ni_per_m3: 1.5e16,
        }
    }

    /// Computes the Displacement Damage Dose (DDD) in $\text{MeV/g}$ for a given particle fluence $\Phi$ ($\text{cm}^{-2}$).
    #[inline]
    pub fn displacement_dose_mev_per_g(&self, fluence_cm2: f64) -> f64 {
        self.s_niel_mev_cm2_per_g * fluence_cm2.max(0.0)
    }

    /// Computes the equivalent displacement damage dose in $\text{rad}(\text{Si})$.
    #[inline]
    pub fn displacement_dose_rad_si(&self, fluence_cm2: f64) -> f64 {
        self.displacement_dose_mev_per_g(fluence_cm2) / RAD_SI_TO_MEV_PER_G
    }

    /// Evaluates the degraded minority carrier lifetime $\tau(\Phi)$ in seconds ($s$)
    /// using the Messenger-Spratt relation:
    ///
    /// $$\tau(\Phi) = \frac{\tau_0}{1 + K_{damage} \cdot \tau_0 \cdot \Phi}$$
    #[inline]
    pub fn carrier_lifetime_s(&self, fluence_cm2: f64) -> f64 {
        let phi = fluence_cm2.max(0.0);
        let factor = 1.0 + self.k_damage_cm2_per_s * self.tau_0_s * phi;
        self.tau_0_s / factor
    }

    /// Evaluates degraded carrier diffusion length $L_d(\Phi) = \sqrt{D \cdot \tau(\Phi)}$ in meters ($m$).
    #[inline]
    pub fn diffusion_length_m(&self, fluence_cm2: f64) -> f64 {
        let tau = self.carrier_lifetime_s(fluence_cm2);
        (self.diffusion_coeff_m2_per_s * tau).max(0.0).sqrt()
    }

    /// Computes the dark current generation spike $\Delta I_{dark}$ in Amperes ($A$)
    /// in the sensitive active volume caused by radiation-induced mid-gap generation centers:
    ///
    /// $$\Delta I_{dark} = \frac{q \cdot n_i \cdot V_{active}}{2} K_{damage} \Phi$$
    #[inline]
    pub fn dark_current_spike_a(&self, fluence_cm2: f64) -> f64 {
        let phi = fluence_cm2.max(0.0);
        // Note: k_damage is in cm^2/s, need consistent units
        // Generation rate increase Delta G = (1/tau - 1/tau0)/2 = (k_damage * phi) / 2 [s^-1]
        let delta_g = 0.5 * self.k_damage_cm2_per_s * phi;
        ELEMENTARY_CHARGE * self.ni_per_m3 * self.active_volume_m3 * delta_g
    }

    /// Total dark current $I_{dark}(\Phi) = I_{dark, 0} + \Delta I_{dark}(\Phi)$ in Amperes ($A$).
    #[inline]
    pub fn total_dark_current_a(&self, i_dark_0_a: f64, fluence_cm2: f64) -> f64 {
        i_dark_0_a.max(0.0) + self.dark_current_spike_a(fluence_cm2)
    }

    /// Evaluates the degraded BJT common-emitter current gain $\beta(\Phi) = h_{FE}(\Phi)$
    /// using Messenger-Spratt gain degradation:
    ///
    /// $$\frac{1}{\beta(\Phi)} = \frac{1}{\beta_0} + K_\beta \cdot \Phi \implies \beta(\Phi) = \frac{\beta_0}{1 + \beta_0 K_\beta \Phi}$$
    #[inline]
    pub fn degraded_beta(&self, fluence_cm2: f64) -> f64 {
        let phi = fluence_cm2.max(0.0);
        let factor = 1.0 + self.beta_0 * self.k_beta_cm2 * phi;
        self.beta_0 / factor
    }

    /// Evaluates solar cell / photodiode collection efficiency degradation ratio $L(\Phi) / L_0$.
    #[inline]
    pub fn relative_collection_efficiency(&self, fluence_cm2: f64) -> f64 {
        let l_phi = self.diffusion_length_m(fluence_cm2);
        let l_0 = (self.diffusion_coeff_m2_per_s * self.tau_0_s).sqrt();
        if l_0 > 1e-15 {
            (l_phi / l_0).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pristine_ddd_values() {
        let model = DisplacementDamageModel::space_proton_environment(100.0);
        assert!((model.carrier_lifetime_s(0.0) - 10.0e-6).abs() < 1e-12);
        assert!((model.degraded_beta(0.0) - 100.0).abs() < 1e-9);
        assert!(model.dark_current_spike_a(0.0).abs() < 1e-25);
        assert!((model.relative_collection_efficiency(0.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_messenger_spratt_lifetime_reduction() {
        let model = DisplacementDamageModel::space_proton_environment(100.0);
        // At phi = 1e11 protons/cm^2:
        // k_damage * tau_0 * phi = 2e-6 * 10e-6 * 1e11 = 2e-11 * 1e11 = 2.0
        // factor = 1 + 2.0 = 3.0 -> tau = 10 us / 3.0 = 3.333 us
        let tau_degraded = model.carrier_lifetime_s(1.0e11);
        assert!((tau_degraded - 10.0e-6 / 3.0).abs() < 1e-10);
        assert!(tau_degraded < model.tau_0_s);
    }

    #[test]
    fn test_bjt_gain_degradation() {
        let model = DisplacementDamageModel::space_proton_environment(150.0);
        // phi = 2e12 cm^-2
        // beta_0 * k_beta * phi = 150 * 5e-15 * 2e12 = 1.5
        // factor = 1 + 1.5 = 2.5 -> beta = 150 / 2.5 = 60.0
        let beta_degraded = model.degraded_beta(2.0e12);
        assert!((beta_degraded - 60.0).abs() < 1e-9);
    }

    #[test]
    fn test_dark_current_proportional_to_fluence() {
        let model = DisplacementDamageModel::space_proton_environment(100.0);
        let spike1 = model.dark_current_spike_a(1.0e11);
        let spike2 = model.dark_current_spike_a(2.0e11);
        assert!(spike1 > 0.0);
        assert!((spike2 - 2.0 * spike1).abs() < 1e-20);
    }

    #[test]
    fn test_diffusion_length_and_solar_cell_efficiency() {
        let model = DisplacementDamageModel::space_proton_environment(100.0);
        let eff_half = model.relative_collection_efficiency(1.5e11);
        assert!(eff_half < 1.0);
        assert!(eff_half > 0.0);
    }
}
