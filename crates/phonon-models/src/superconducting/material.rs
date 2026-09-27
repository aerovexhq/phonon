//! Superconducting materials, presets, and BCS gap dynamics.
//!
//! Formulates BCS (Bardeen-Cooper-Schrieffer) temperature-dependent gap $\Delta(T)$
//! and the Ambegaokar-Baratoff relation for the critical current product $I_c R_n$.

use phonon_core::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE};

/// Superconducting material parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct SuperconductorMaterial {
    /// Material name identifier (e.g. "Nb", "NbN", "Al", "YBCO").
    pub name: String,
    /// Critical transition temperature $T_c$ in Kelvin ($K$).
    pub critical_temp_k: f64,
    /// Energy gap at zero Kelvin $\Delta_0$ in electron-volts ($eV$).
    pub gap_zero_ev: f64,
    /// Normal-state resistivity $\rho_n$ in $\Omega \cdot m$.
    pub normal_resistivity: f64,
    /// London penetration depth $\lambda_L(0)$ in meters ($m$).
    pub penetration_depth: f64,
    /// BCS coherence length $\xi_0$ in meters ($m$).
    pub coherence_length: f64,
}

impl SuperconductorMaterial {
    /// Niobium (Nb) preset: standard material for RSFQ circuits ($T_c = 9.2 \text{ K}$).
    pub fn niobium() -> Self {
        Self {
            name: "Nb".to_string(),
            critical_temp_k: 9.2,
            gap_zero_ev: 1.5e-3, // 1.5 meV
            normal_resistivity: 1.5e-7,
            penetration_depth: 39e-9, // 39 nm
            coherence_length: 38e-9,  // 38 nm
        }
    }

    /// Niobium Nitride (NbN) preset: higher-temperature nitride ($T_c = 16.0 \text{ K}$).
    pub fn niobium_nitride() -> Self {
        Self {
            name: "NbN".to_string(),
            critical_temp_k: 16.0,
            gap_zero_ev: 2.8e-3, // 2.8 meV
            normal_resistivity: 1.5e-6,
            penetration_depth: 200e-9,
            coherence_length: 5e-9,
        }
    }

    /// Aluminum (Al) preset: standard for superconducting qubits / transmons ($T_c = 1.2 \text{ K}$).
    pub fn aluminum() -> Self {
        Self {
            name: "Al".to_string(),
            critical_temp_k: 1.2,
            gap_zero_ev: 0.18e-3, // 0.18 meV
            normal_resistivity: 3.0e-8,
            penetration_depth: 16e-9,
            coherence_length: 1600e-9,
        }
    }

    /// YBCO ($YBa_2Cu_3O_{7-\delta}$) high-$T_c$ cuprate preset ($T_c = 93.0 \text{ K}$).
    pub fn ybco() -> Self {
        Self {
            name: "YBCO".to_string(),
            critical_temp_k: 93.0,
            gap_zero_ev: 20.0e-3, // 20.0 meV
            normal_resistivity: 1.0e-5,
            penetration_depth: 150e-9,
            coherence_length: 1.5e-9,
        }
    }

    /// Computes the BCS superconducting energy gap $\Delta(T)$ in electron-volts ($eV$):
    /// $$\Delta(T) = \Delta_0 \tanh\left(1.74 \sqrt{\frac{T_c}{T} - 1}\right) \quad \text{for } T < T_c$$
    /// $$\Delta(T) = 0 \quad \text{for } T \ge T_c$$
    pub fn bcs_gap_ev(&self, temp_k: f64) -> f64 {
        if temp_k >= self.critical_temp_k {
            return 0.0;
        }
        let t = temp_k.max(1e-4);
        let arg = (self.critical_temp_k / t - 1.0).max(0.0).sqrt();
        self.gap_zero_ev * (1.74 * arg).tanh()
    }

    /// Computes the Ambegaokar-Baratoff $I_c R_n$ product in Volts ($V$) at temperature $T$:
    /// $$I_c R_n(T) = \frac{\pi \Delta(T)}{2 e} \tanh\left( \frac{\Delta(T)}{2 k_B T} \right)$$
    pub fn ambegaokar_baratoff_ic_rn(&self, temp_k: f64) -> f64 {
        let gap_ev = self.bcs_gap_ev(temp_k);
        if gap_ev <= 1e-12 {
            return 0.0;
        }

        let delta_joules = gap_ev * ELEMENTARY_CHARGE;
        let t = temp_k.max(1e-4);
        let kb_t = BOLTZMANN_CONSTANT * t;

        let tanh_term = (delta_joules / (2.0 * kb_t)).clamp(-50.0, 50.0).tanh();
        (std::f64::consts::PI * gap_ev / 2.0) * tanh_term
    }

    /// Temperature-dependent London penetration depth $\lambda_L(T)$:
    /// $$\lambda_L(T) = \frac{\lambda_L(0)}{\sqrt{1 - (T / T_c)^4}}$$
    pub fn penetration_depth_m(&self, temp_k: f64) -> f64 {
        if temp_k >= self.critical_temp_k {
            return f64::INFINITY;
        }
        let ratio = (temp_k / self.critical_temp_k).clamp(0.0, 0.9999);
        let den = (1.0 - ratio.powi(4)).sqrt();
        self.penetration_depth / den
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_niobium_bcs_gap_temperature_dependence() {
        let nb = SuperconductorMaterial::niobium();
        let gap_0 = nb.bcs_gap_ev(0.1);
        let gap_4_2 = nb.bcs_gap_ev(4.2);
        let gap_above = nb.bcs_gap_ev(10.0);

        // At T near 0K, gap should approach 1.5 meV
        assert!((gap_0 - 1.5e-3).abs() < 1e-4);
        // At 4.2K, gap is slightly reduced but still significant
        assert!(gap_4_2 > 1.2e-3 && gap_4_2 < 1.5e-3);
        // Above Tc (9.2K), gap is exactly zero (normal state)
        assert_eq!(gap_above, 0.0);
    }

    #[test]
    fn test_ambegaokar_baratoff_product() {
        let nb = SuperconductorMaterial::niobium();
        let ic_rn_4_2 = nb.ambegaokar_baratoff_ic_rn(4.2);
        // At 4.2K in Nb, Ic * Rn is typically ~ 1.5 - 2.1 mV
        assert!(ic_rn_4_2 > 1.5e-3 && ic_rn_4_2 < 2.5e-3);
    }
}
