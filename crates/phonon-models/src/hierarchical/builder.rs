//! Hierarchical device synthesis API providing unified programmatic access to both
//! microscopic structural TCAD physics and high-speed analytical compact models.

use crate::tcad::{
    extract_diode_model, extract_mosfet_model, MaterialProperties, TcadDevice, TcadDeviceBuilder,
};
use crate::{DiodeModel, MosfetModel};
use phonon_core::constants::{ELEMENTARY_CHARGE, EPSILON_0, T_REF};

/// Unified builder for synthesizing diodes at both low-level TCAD and high-level compact tiers.
#[derive(Debug, Clone)]
pub struct HierarchicalDiodeBuilder {
    pub name: String,
    pub length: f64,
    pub area: f64,
    pub p_doping: f64, // m^-3
    pub n_doping: f64, // m^-3
}

impl HierarchicalDiodeBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            length: 2.0e-6,
            area: 1.0e-8,
            p_doping: 1.0e23, // 1e17 cm^-3
            n_doping: 1.0e22, // 1e16 cm^-3
        }
    }

    pub fn length(mut self, length: f64) -> Self {
        self.length = length;
        self
    }

    pub fn area(mut self, area: f64) -> Self {
        self.area = area;
        self
    }

    pub fn p_doping(mut self, na: f64) -> Self {
        self.p_doping = na;
        self
    }

    pub fn n_doping(mut self, nd: f64) -> Self {
        self.n_doping = nd;
        self
    }

    /// Synthesizes the device as a low-level structural TCAD device with spatial mesh.
    pub fn into_tcad(self) -> TcadDevice {
        TcadDeviceBuilder::new_pn_junction(&self.name)
            .length(self.length)
            .cross_section_area(self.area)
            .p_doping(self.p_doping)
            .n_doping(self.n_doping)
            .build()
    }

    /// Synthesizes the device as an optimized analytical Shockley compact model (O(1) evaluation).
    pub fn into_compact(self) -> DiodeModel {
        let mat = MaterialProperties::silicon();
        let ni = mat.intrinsic_carrier_conc_300k;
        let dn = mat.electron_diffusion_coeff(T_REF);
        let dp = mat.hole_diffusion_coeff(T_REF);
        let ln = (dn * mat.electron_lifetime).sqrt().min(self.length * 0.5);
        let lp = (dp * mat.hole_lifetime).sqrt().min(self.length * 0.5);

        // Theoretical Shockley ideal reverse saturation current Is = q * A * ni^2 * (Dn / (Ln * Na) + Dp / (Lp * Nd))
        let is = ELEMENTARY_CHARGE
            * self.area
            * ni
            * ni
            * (dn / (ln * self.p_doping) + dp / (lp * self.n_doping));

        DiodeModel {
            is: is.clamp(1e-18, 1e-6),
            n: 1.0,
            ..DiodeModel::default()
        }
    }

    /// Automatically runs TCAD simulation to extract calibrated compact model parameters.
    pub fn into_calibrated_compact(self, temp_k: f64) -> Result<DiodeModel, String> {
        let tcad = self.into_tcad();
        extract_diode_model(&tcad, temp_k)
    }
}

/// Unified builder for synthesizing MOSFET transistors at both low-level TCAD and high-level compact tiers.
#[derive(Debug, Clone)]
pub struct HierarchicalTransistorBuilder {
    pub name: String,
    pub channel_length: f64,
    pub channel_width: f64,
    pub oxide_thickness: f64,
    pub p_substrate_doping: f64,
    pub n_source_drain_doping: f64,
}

impl HierarchicalTransistorBuilder {
    pub fn new_nmos(name: &str) -> Self {
        Self {
            name: name.to_string(),
            channel_length: 180.0e-9,
            channel_width: 1.0e-6,
            oxide_thickness: 4.0e-9,
            p_substrate_doping: 1.0e23,    // 1e17 cm^-3
            n_source_drain_doping: 1.0e26, // 1e20 cm^-3
        }
    }

    pub fn length(mut self, l: f64) -> Self {
        self.channel_length = l;
        self
    }

    pub fn width(mut self, w: f64) -> Self {
        self.channel_width = w;
        self
    }

    pub fn oxide_thickness(mut self, tox: f64) -> Self {
        self.oxide_thickness = tox;
        self
    }

    pub fn substrate_doping(mut self, na: f64) -> Self {
        self.p_substrate_doping = na;
        self
    }

    /// Synthesizes the device as a low-level structural TCAD device with spatial mesh.
    pub fn into_tcad(self) -> TcadDevice {
        TcadDeviceBuilder::new_mosfet(&self.name)
            .length(self.channel_length * 2.0) // include source/drain extensions
            .cross_section_area(self.channel_width * 1.0e-6)
            .oxide_thickness(self.oxide_thickness)
            .p_doping(self.p_substrate_doping)
            .n_doping(self.n_source_drain_doping)
            .build()
    }

    /// Synthesizes the device as an optimized analytical Level-1/BSIM compact model (O(1) evaluation).
    pub fn into_compact(self) -> MosfetModel {
        let mat = MaterialProperties::silicon();
        let eps_ox = 3.9 * EPSILON_0;
        let c_ox = eps_ox / self.oxide_thickness;
        let vt = MaterialProperties::thermal_voltage(T_REF);
        let ni = mat.intrinsic_carrier_conc_300k;

        // Built-in bulk Fermi potential: phi_F = Vt * ln(Na / ni)
        let phi_f = vt * (self.p_substrate_doping / ni).ln().max(0.1);
        let gamma =
            (2.0 * ELEMENTARY_CHARGE * mat.permittivity() * self.p_substrate_doping).sqrt() / c_ox;
        let v_th0 = -0.1 + 2.0 * phi_f + gamma * (2.0 * phi_f).sqrt();

        MosfetModel {
            vth0: v_th0.clamp(0.2, 1.2),
            mu0: mat.electron_mobility_300k,
            gamma,
            phi_s: 2.0 * phi_f,
            lambda: 0.02,
            w: self.channel_width,
            l: self.channel_length,
            tox: self.oxide_thickness,
            ..MosfetModel::default()
        }
    }

    /// Automatically runs TCAD simulation to extract calibrated compact model parameters.
    pub fn into_calibrated_compact(self, temp_k: f64) -> Result<MosfetModel, String> {
        let w = self.channel_width;
        let tcad = self.into_tcad();
        extract_mosfet_model(&tcad, w, temp_k)
    }
}
