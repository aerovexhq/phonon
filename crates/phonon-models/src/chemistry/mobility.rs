//! Doping-dependent carrier mobility, velocity saturation, and impact ionization physics.

use phonon_core::constants::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, T_REF};

/// Caughey-Thomas carrier mobility parameters for electrons or holes.
#[derive(Debug, Clone, PartialEq)]
pub struct CarrierMobilityParams {
    /// Maximum lattice mobility at 300K in $\text{m}^2 / (\text{V} \cdot \text{s})$.
    pub mu_max_300k: f64,
    /// Minimum impurity scattering mobility in $\text{m}^2 / (\text{V} \cdot \text{s})$.
    pub mu_min: f64,
    /// Reference doping concentration $N_{ref}$ in $\text{m}^{-3}$.
    pub n_ref: f64,
    /// Doping degradation exponent $\alpha$.
    pub alpha: f64,
    /// Temperature degradation exponent $\gamma$ ($\mu \propto T^{-\gamma}$).
    pub temp_exponent_gamma: f64,
    /// Saturation velocity at 300K in meters per second ($m/s$).
    pub v_sat_300k: f64,
    /// Velocity saturation exponent $\beta$ (Canali model).
    pub beta_v_sat: f64,
    /// Impact ionization prefactor $A$ in $\text{m}^{-1}$.
    pub impact_ion_a: f64,
    /// Impact ionization critical field $B$ in $V/m$.
    pub impact_ion_b: f64,
}

impl CarrierMobilityParams {
    /// Computes the low-field mobility $\mu_0(N, T)$ in $\text{m}^2 / (\text{V} \cdot \text{s})$
    /// via the Caughey-Thomas formulation:
    /// $$\mu_0(N, T) = \mu_{min} + \frac{\mu_{max}(T) - \mu_{min}}{1 + (N_{total} / N_{ref})^\alpha}$$
    pub fn low_field_mobility(&self, total_doping_m3: f64, temp_k: f64) -> f64 {
        let t = temp_k.max(10.0);
        let temp_factor = (t / T_REF).powf(-self.temp_exponent_gamma);
        let mu_max_t = self.mu_max_300k * temp_factor;

        let doping_ratio = (total_doping_m3.max(0.0) / self.n_ref).powf(self.alpha);
        let mu = self.mu_min + (mu_max_t - self.mu_min) / (1.0 + doping_ratio);
        mu.max(1e-6)
    }

    /// Computes the temperature-dependent saturation velocity $v_{sat}(T)$ in $m/s$:
    /// $$v_{sat}(T) = v_{sat, 300} \cdot (T / 300)^{-0.87}$$
    pub fn saturation_velocity(&self, temp_k: f64) -> f64 {
        let t = temp_k.max(10.0);
        self.v_sat_300k * (t / T_REF).powf(-0.87)
    }

    /// Computes the field-dependent drift velocity $v(E)$ in $m/s$ via the Canali model:
    /// $$v(E) = \frac{\mu_0 E}{\left[ 1 + (\mu_0 E / v_{sat})^\beta \right]^{1/\beta}}$$
    pub fn drift_velocity(
        &self,
        electric_field_v_m: f64,
        total_doping_m3: f64,
        temp_k: f64,
    ) -> f64 {
        let mu_0 = self.low_field_mobility(total_doping_m3, temp_k);
        let v_sat = self.saturation_velocity(temp_k);
        let e = electric_field_v_m.abs();
        let beta = self.beta_v_sat;

        let mu_e = mu_0 * e;
        let denom = (1.0 + (mu_e / v_sat).powf(beta)).powf(1.0 / beta);
        let v_mag = mu_e / denom;
        if electric_field_v_m < 0.0 {
            -v_mag
        } else {
            v_mag
        }
    }

    /// Computes the field-dependent effective mobility $\mu(E)$ in $\text{m}^2 / (\text{V} \cdot \text{s})$:
    pub fn field_dependent_mobility(
        &self,
        electric_field_v_m: f64,
        total_doping_m3: f64,
        temp_k: f64,
    ) -> f64 {
        let e = electric_field_v_m.abs();
        if e < 1.0 {
            self.low_field_mobility(total_doping_m3, temp_k)
        } else {
            let v = self.drift_velocity(e, total_doping_m3, temp_k);
            v / e
        }
    }

    /// Computes the Einstein carrier diffusion coefficient $D = \mu \cdot \frac{k_B T}{q}$ in $\text{m}^2/s$.
    pub fn diffusion_coefficient(&self, total_doping_m3: f64, temp_k: f64) -> f64 {
        let mu = self.low_field_mobility(total_doping_m3, temp_k);
        let vt = (BOLTZMANN_CONSTANT * temp_k) / ELEMENTARY_CHARGE;
        mu * vt
    }

    /// Computes the impact ionization coefficient $\alpha(E)$ in $\text{m}^{-1}$
    /// via the Chynoweth law: $\alpha = A \exp(-B / |E|)$.
    pub fn impact_ionization_coeff(&self, electric_field_v_m: f64) -> f64 {
        let e = electric_field_v_m.abs();
        if e < 1.0e5 {
            return 0.0;
        }
        let exp_arg = -self.impact_ion_b / e;
        self.impact_ion_a * exp_arg.clamp(-80.0, 0.0).exp()
    }
}

/// Consolidated chemical mobility model for both electrons and holes.
#[derive(Debug, Clone, PartialEq)]
pub struct ChemicalMobility {
    pub electron: CarrierMobilityParams,
    pub hole: CarrierMobilityParams,
}

impl ChemicalMobility {
    // =========================================================================
    // Standard Material Mobility Presets
    // =========================================================================

    /// Silicon ($Si$): $\mu_{n,max} = 0.1417 \text{ m}^2/(V\cdot s)$, $\mu_{p,max} = 0.0470 \text{ m}^2/(V\cdot s)$.
    pub fn silicon() -> Self {
        Self {
            electron: CarrierMobilityParams {
                mu_max_300k: 0.1417,
                mu_min: 0.00685,
                n_ref: 9.68e22, // 9.68e16 cm^-3 in m^-3
                alpha: 0.711,
                temp_exponent_gamma: 2.2,
                v_sat_300k: 1.07e5,
                beta_v_sat: 1.109,
                impact_ion_a: 7.03e7,
                impact_ion_b: 1.23e8,
            },
            hole: CarrierMobilityParams {
                mu_max_300k: 0.04705,
                mu_min: 0.00449,
                n_ref: 2.23e23, // 2.23e17 cm^-3 in m^-3
                alpha: 0.719,
                temp_exponent_gamma: 2.18,
                v_sat_300k: 8.37e4,
                beta_v_sat: 1.213,
                impact_ion_a: 1.58e8,
                impact_ion_b: 2.04e8,
            },
        }
    }

    /// Germanium ($Ge$): $\mu_{n,max} = 0.3900 \text{ m}^2/(V\cdot s)$, $\mu_{p,max} = 0.1900 \text{ m}^2/(V\cdot s)$.
    pub fn germanium() -> Self {
        Self {
            electron: CarrierMobilityParams {
                mu_max_300k: 0.3900,
                mu_min: 0.0200,
                n_ref: 5.0e22,
                alpha: 0.65,
                temp_exponent_gamma: 1.66,
                v_sat_300k: 6.0e4,
                beta_v_sat: 1.5,
                impact_ion_a: 5.0e7,
                impact_ion_b: 1.0e8,
            },
            hole: CarrierMobilityParams {
                mu_max_300k: 0.1900,
                mu_min: 0.0100,
                n_ref: 8.0e22,
                alpha: 0.65,
                temp_exponent_gamma: 2.33,
                v_sat_300k: 6.0e4,
                beta_v_sat: 1.5,
                impact_ion_a: 8.0e7,
                impact_ion_b: 1.5e8,
            },
        }
    }

    /// Gallium Arsenide ($GaAs$): Ultra-high electron mobility $\mu_{n,max} = 0.8500 \text{ m}^2/(V\cdot s)$.
    pub fn gallium_arsenide() -> Self {
        Self {
            electron: CarrierMobilityParams {
                mu_max_300k: 0.8500,
                mu_min: 0.0500,
                n_ref: 1.0e23,
                alpha: 0.55,
                temp_exponent_gamma: 1.5,
                v_sat_300k: 1.2e5,
                beta_v_sat: 2.0,
                impact_ion_a: 3.5e7,
                impact_ion_b: 6.85e7,
            },
            hole: CarrierMobilityParams {
                mu_max_300k: 0.0400,
                mu_min: 0.0020,
                n_ref: 2.0e23,
                alpha: 0.50,
                temp_exponent_gamma: 1.5,
                v_sat_300k: 8.0e4,
                beta_v_sat: 1.5,
                impact_ion_a: 4.0e7,
                impact_ion_b: 7.5e7,
            },
        }
    }

    /// Gallium Nitride ($GaN$): High saturation velocity $v_{sat} = 2.5 \times 10^5 \text{ m/s}$.
    pub fn gallium_nitride() -> Self {
        Self {
            electron: CarrierMobilityParams {
                mu_max_300k: 0.1500,
                mu_min: 0.0100,
                n_ref: 1.0e23,
                alpha: 0.70,
                temp_exponent_gamma: 2.0,
                v_sat_300k: 2.5e5,
                beta_v_sat: 2.0,
                impact_ion_a: 2.5e8,
                impact_ion_b: 3.4e8,
            },
            hole: CarrierMobilityParams {
                mu_max_300k: 0.0030,
                mu_min: 0.0005,
                n_ref: 1.0e24,
                alpha: 0.70,
                temp_exponent_gamma: 2.0,
                v_sat_300k: 1.0e5,
                beta_v_sat: 1.5,
                impact_ion_a: 3.0e8,
                impact_ion_b: 4.0e8,
            },
        }
    }

    /// Silicon Carbide ($4H-SiC$): High breakdown field and velocity saturation.
    pub fn silicon_carbide_4h() -> Self {
        Self {
            electron: CarrierMobilityParams {
                mu_max_300k: 0.0950,
                mu_min: 0.0040,
                n_ref: 2.0e23,
                alpha: 0.65,
                temp_exponent_gamma: 2.15,
                v_sat_300k: 2.0e5,
                beta_v_sat: 1.5,
                impact_ion_a: 1.6e8,
                impact_ion_b: 2.5e8,
            },
            hole: CarrierMobilityParams {
                mu_max_300k: 0.0120,
                mu_min: 0.0010,
                n_ref: 3.0e23,
                alpha: 0.65,
                temp_exponent_gamma: 2.15,
                v_sat_300k: 1.2e5,
                beta_v_sat: 1.5,
                impact_ion_a: 2.0e8,
                impact_ion_b: 3.0e8,
            },
        }
    }
}
