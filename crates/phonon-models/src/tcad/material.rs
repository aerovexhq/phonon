//! Semiconductor material physical parameters, bandgap, carrier mobility, and lifetimes.
//!
//! Standards: SI units (meters, seconds, Volts, Kelvin, Amperes, Farads).

use phonon_core::constants::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, EPSILON_0, T_REF};

/// Physical semiconductor material properties.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialProperties {
    pub name: String,
    pub relative_permittivity: f64,
    pub bandgap_300k: f64,
    pub intrinsic_carrier_conc_300k: f64, // m^-3
    pub electron_mobility_300k: f64,      // m^2 / (V * s)
    pub hole_mobility_300k: f64,          // m^2 / (V * s)
    pub electron_lifetime: f64,           // s
    pub hole_lifetime: f64,               // s
}

impl MaterialProperties {
    /// Silicon at 300K: eps_r = 11.7, Eg = 1.12 eV, ni = 1.0e16 m^-3 (1e10 cm^-3).
    pub fn silicon() -> Self {
        Self {
            name: "Silicon".to_string(),
            relative_permittivity: 11.7,
            bandgap_300k: 1.12,
            intrinsic_carrier_conc_300k: 1.0e16,
            electron_mobility_300k: 0.140, // 1400 cm^2 / (V*s)
            hole_mobility_300k: 0.045,     // 450 cm^2 / (V*s)
            electron_lifetime: 1.0e-6,
            hole_lifetime: 1.0e-6,
        }
    }

    /// Germanium at 300K: eps_r = 16.0, Eg = 0.66 eV, ni = 2.4e19 m^-3.
    pub fn germanium() -> Self {
        Self {
            name: "Germanium".to_string(),
            relative_permittivity: 16.0,
            bandgap_300k: 0.66,
            intrinsic_carrier_conc_300k: 2.4e19,
            electron_mobility_300k: 0.390,
            hole_mobility_300k: 0.190,
            electron_lifetime: 1.0e-5,
            hole_lifetime: 1.0e-5,
        }
    }

    /// Gallium Arsenide (GaAs) at 300K: eps_r = 12.9, Eg = 1.42 eV, ni = 1.8e12 m^-3.
    pub fn gallium_arsenide() -> Self {
        Self {
            name: "GalliumArsenide".to_string(),
            relative_permittivity: 12.9,
            bandgap_300k: 1.42,
            intrinsic_carrier_conc_300k: 1.8e12,
            electron_mobility_300k: 0.850,
            hole_mobility_300k: 0.040,
            electron_lifetime: 1.0e-8,
            hole_lifetime: 1.0e-8,
        }
    }

    /// Absolute permittivity epsilon = eps_r * eps_0 (F/m).
    #[inline]
    pub fn permittivity(&self) -> f64 {
        self.relative_permittivity * EPSILON_0
    }

    /// Thermal voltage V_t = k * T / q (V).
    #[inline]
    pub fn thermal_voltage(temp_k: f64) -> f64 {
        BOLTZMANN_CONSTANT * temp_k / ELEMENTARY_CHARGE
    }

    /// Temperature-dependent intrinsic carrier concentration n_i(T) (m^-3).
    pub fn intrinsic_carrier_concentration(&self, temp_k: f64) -> f64 {
        if temp_k <= 0.0 {
            return self.intrinsic_carrier_conc_300k;
        }
        let t_ratio = temp_k / T_REF;
        // ni(T) = ni(300) * (T/300)^1.5 * exp(-Eg/(2*k*T) * (1 - T/300))
        let exponent = -(self.bandgap_300k * ELEMENTARY_CHARGE)
            / (2.0 * BOLTZMANN_CONSTANT * temp_k)
            * (1.0 - t_ratio);
        let safe_exp = exponent.clamp(-80.0, 80.0).exp();
        self.intrinsic_carrier_conc_300k * t_ratio.powf(1.5) * safe_exp
    }

    /// Temperature-dependent electron mobility mu_n(T) (m^2 / (V * s)).
    pub fn electron_mobility(&self, temp_k: f64) -> f64 {
        if temp_k <= 0.0 {
            return self.electron_mobility_300k;
        }
        self.electron_mobility_300k * (temp_k / T_REF).powf(-1.5)
    }

    /// Temperature-dependent hole mobility mu_p(T) (m^2 / (V * s)).
    pub fn hole_mobility(&self, temp_k: f64) -> f64 {
        if temp_k <= 0.0 {
            return self.hole_mobility_300k;
        }
        self.hole_mobility_300k * (temp_k / T_REF).powf(-1.5)
    }

    /// Electron diffusion coefficient D_n = mu_n * V_t (m^2 / s).
    pub fn electron_diffusion_coeff(&self, temp_k: f64) -> f64 {
        self.electron_mobility(temp_k) * Self::thermal_voltage(temp_k)
    }

    /// Hole diffusion coefficient D_p = mu_p * V_t (m^2 / s).
    pub fn hole_diffusion_coeff(&self, temp_k: f64) -> f64 {
        self.hole_mobility(temp_k) * Self::thermal_voltage(temp_k)
    }
}

/// Standard semiconductor material selection enum.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum SemiconductorMaterial {
    #[default]
    Silicon,
    Germanium,
    GalliumArsenide,
    Chemical(Box<crate::chemistry::ChemicalMaterial>),
    Custom(MaterialProperties),
}

impl From<&crate::chemistry::ChemicalMaterial> for MaterialProperties {
    fn from(c: &crate::chemistry::ChemicalMaterial) -> Self {
        let (mu_n, mu_p) = c.low_field_mobilities(T_REF);
        Self {
            name: c.name.clone(),
            relative_permittivity: c.relative_permittivity,
            bandgap_300k: c.bandstructure.bandgap_ev(T_REF),
            intrinsic_carrier_conc_300k: c.bandstructure.intrinsic_carrier_density(T_REF),
            electron_mobility_300k: mu_n,
            hole_mobility_300k: mu_p,
            electron_lifetime: 1.0e-6,
            hole_lifetime: 1.0e-6,
        }
    }
}

impl SemiconductorMaterial {
    pub fn properties(&self) -> MaterialProperties {
        match self {
            Self::Silicon => MaterialProperties::silicon(),
            Self::Germanium => MaterialProperties::germanium(),
            Self::GalliumArsenide => MaterialProperties::gallium_arsenide(),
            Self::Chemical(chem) => MaterialProperties::from(chem.as_ref()),
            Self::Custom(props) => props.clone(),
        }
    }
}
