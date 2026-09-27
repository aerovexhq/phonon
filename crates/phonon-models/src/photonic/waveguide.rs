//! Integrated optical waveguide model for Silicon and Silicon Nitride photonics.
//!
//! Formulates:
//! - Waveguide modal propagation with power attenuation $\alpha_{loss}$ in dB/m.
//! - Effective refractive index $n_{eff}(\lambda, T)$ with thermo-optic coefficient $dn/dT$.
//! - Group index $n_g$, group delay $\tau_g = n_g L / c$, and chromatic dispersion.
//! - Phase accumulation $\beta(\lambda, T) L$ and complex field envelope transformation.

use phonon_core::{OpticalSignal, SPEED_OF_LIGHT, T_REF};

/// Physical specification of an integrated optical waveguide.
#[derive(Debug, Clone, PartialEq)]
pub struct OpticalWaveguideModel {
    /// Physical propagation length $L$ in meters ($m$).
    pub length_m: f64,
    /// Propagation loss in decibels per meter ($\text{dB}/m$) (e.g. $200\text{ dB}/m \approx 2\text{ dB}/cm$).
    pub loss_db_per_m: f64,
    /// Effective refractive index $n_{eff,0}$ at reference temperature $T_0 = 300\text{ K}$ and $\lambda_0 = 1550\text{ nm}$.
    pub n_eff_0: f64,
    /// Group index $n_g = n_{eff} - \lambda_0 \frac{dn_{eff}}{d\lambda}$.
    pub group_index_ng: f64,
    /// Thermo-optic coefficient $\frac{dn_{eff}}{dT}$ in $K^{-1}$ (e.g. $+1.86 \times 10^{-4}\text{ K}^{-1}$ for Si).
    pub dn_dt: f64,
    /// Chromatic dispersion parameter $D$ in $\text{s} / m^2$ (typically $\sim 1000\text{ ps}/(\text{nm}\cdot\text{km})$).
    pub dispersion_s_m2: f64,
    /// Nominal center design wavelength $\lambda_0$ in meters ($m$) ($1.55\,\mu\text{m}$).
    pub lambda_0_m: f64,
}

impl OpticalWaveguideModel {
    /// Silicon-on-Insulator (SOI) standard strip waveguide preset ($450\text{ nm} \times 220\text{ nm}$ at $1550\text{ nm}$).
    pub fn silicon_strip(length_m: f64) -> Self {
        Self {
            length_m,
            loss_db_per_m: 200.0, // 2 dB/cm
            n_eff_0: 2.45,
            group_index_ng: 4.20,
            dn_dt: 1.86e-4, // +1.86e-4 / K
            dispersion_s_m2: -1.0e-3,
            lambda_0_m: 1.55e-6,
        }
    }

    /// Silicon Nitride ($\text{Si}_3\text{N}_4$) low-loss waveguide preset ($800\text{ nm} \times 400\text{ nm}$).
    pub fn silicon_nitride(length_m: f64) -> Self {
        Self {
            length_m,
            loss_db_per_m: 20.0, // 0.2 dB/cm
            n_eff_0: 1.75,
            group_index_ng: 2.10,
            dn_dt: 2.50e-5, // +2.5e-5 / K (much lower thermo-optic drift than Si)
            dispersion_s_m2: 2.0e-4,
            lambda_0_m: 1.55e-6,
        }
    }

    /// Evaluates dynamic effective index $n_{eff}(\lambda, T)$ taking into account
    /// wavelength dispersion and thermo-optic temperature drift:
    /// $$n_{eff}(\lambda, T) = n_{eff,0} + \frac{dn}{dT}(T - T_0) - \frac{n_g - n_{eff,0}}{\lambda_0}(\lambda - \lambda_0)$$
    pub fn effective_index(&self, wavelength_m: f64, temp_k: f64) -> f64 {
        let delta_t = temp_k - T_REF;
        let delta_lambda = wavelength_m - self.lambda_0_m;
        let dispersion_slope = (self.group_index_ng - self.n_eff_0) / self.lambda_0_m;

        self.n_eff_0 + self.dn_dt * delta_t - dispersion_slope * delta_lambda
    }

    /// Evaluates modal propagation constant $\beta(\lambda, T) = \frac{2\pi n_{eff}(\lambda, T)}{\lambda}$ in $m^{-1}$.
    pub fn propagation_constant(&self, wavelength_m: f64, temp_k: f64) -> f64 {
        let n_eff = self.effective_index(wavelength_m, temp_k);
        (2.0 * std::f64::consts::PI * n_eff) / wavelength_m.max(1e-12)
    }

    /// Physical group delay $\tau_g = \frac{n_g L}{c}$ in seconds ($s$).
    pub fn group_delay_seconds(&self) -> f64 {
        (self.group_index_ng * self.length_m) / SPEED_OF_LIGHT
    }

    /// Linear optical power transmission factor:
    /// $$\mathcal{T} = 10^{-\frac{\alpha L}{10}}$$
    pub fn power_transmission_factor(&self) -> f64 {
        let total_loss_db = self.loss_db_per_m * self.length_m;
        10.0_f64.powf(-total_loss_db / 10.0)
    }

    /// Propagates an input optical signal through the waveguide at temperature $T$,
    /// returning the attenuated and phase-shifted output signal:
    pub fn propagate(&self, input: &OpticalSignal, temp_k: f64) -> OpticalSignal {
        let trans = self.power_transmission_factor();
        let p_out = input.power_watts * trans;

        let beta = self.propagation_constant(input.wavelength_m, temp_k);
        let phase_accum = beta * self.length_m;
        let total_phase = (input.phase_rad + phase_accum).rem_euclid(2.0 * std::f64::consts::PI);

        OpticalSignal {
            wavelength_m: input.wavelength_m,
            power_watts: p_out,
            phase_rad: total_phase,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silicon_strip_waveguide_transmission_and_delay() {
        let wg = OpticalWaveguideModel::silicon_strip(0.01); // 1 cm
                                                             // 2 dB/cm -> transmission ~ 10^(-0.2) ~ 0.6309
        let trans = wg.power_transmission_factor();
        assert!((trans - 0.630957).abs() < 1e-4);

        // Group delay for 1 cm silicon waveguide:
        // tau_g = 4.2 * 0.01 / 3e8 ~ 1.401e-10 s = 140.1 ps
        let tau = wg.group_delay_seconds();
        assert!((tau - 1.401e-10).abs() < 1e-12);

        let input = OpticalSignal::new(1.55e-6, 1e-3, 0.0); // 1 mW (0 dBm)
        let output = wg.propagate(&input, 300.0);
        assert!((output.power_watts - 0.630957e-3).abs() < 1e-6);
        assert_eq!(output.wavelength_m, 1.55e-6);
    }

    #[test]
    fn test_thermo_optic_effective_index_shift() {
        let wg = OpticalWaveguideModel::silicon_strip(1e-3);
        let n_300k = wg.effective_index(1.55e-6, 300.0);
        let n_350k = wg.effective_index(1.55e-6, 350.0);

        // Delta n = 1.86e-4 * 50 = +0.0093
        assert!((n_350k - n_300k - 0.0093).abs() < 1e-5);
    }
}
