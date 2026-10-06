#![deny(unsafe_code)]

//! Interconnect Electromigration (EM) Engine across Metallization Layers M1 to M15.
//!
//! Implements Black's Equation, Joule self-heating derating, Blech length immortality limits,
//! void nucleation/growth kinetics, and multi-tier interconnect reliability scoring.

use crate::silicon_aging::bti::K_BOLTZMANN_EV_PER_K;

/// Standard metal layer designations from local M1 through global RDL M15.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetalLayerId {
    M1,
    M2,
    M3,
    M4,
    M5,
    M6,
    M7,
    M8,
    M9,
    M10,
    M11,
    M12,
    M13,
    M14,
    M15,
}

impl MetalLayerId {
    /// Returns human-readable designator string.
    pub fn name(&self) -> &'static str {
        match self {
            MetalLayerId::M1 => "M1 (Local Contact FinFET)",
            MetalLayerId::M2 => "M2 (Local Signal)",
            MetalLayerId::M3 => "M3 (Local Signal)",
            MetalLayerId::M4 => "M4 (Semi-Global Clock)",
            MetalLayerId::M5 => "M5 (Semi-Global Signal)",
            MetalLayerId::M6 => "M6 (Intermediate Bus)",
            MetalLayerId::M7 => "M7 (Intermediate Bus)",
            MetalLayerId::M8 => "M8 (Intermediate Clock)",
            MetalLayerId::M9 => "M9 (Intermediate Clock)",
            MetalLayerId::M10 => "M10 (Semi-Global Power)",
            MetalLayerId::M11 => "M11 (Global Power Grid)",
            MetalLayerId::M12 => "M12 (Global Power Mesh)",
            MetalLayerId::M13 => "M13 (Thick Top Power)",
            MetalLayerId::M14 => "M14 (Ultra-Thick Bus)",
            MetalLayerId::M15 => "M15 (RDL / Bump Metal)",
        }
    }

    /// Returns short label string.
    pub fn short_name(&self) -> &'static str {
        match self {
            MetalLayerId::M1 => "M1",
            MetalLayerId::M2 => "M2",
            MetalLayerId::M3 => "M3",
            MetalLayerId::M4 => "M4",
            MetalLayerId::M5 => "M5",
            MetalLayerId::M6 => "M6",
            MetalLayerId::M7 => "M7",
            MetalLayerId::M8 => "M8",
            MetalLayerId::M9 => "M9",
            MetalLayerId::M10 => "M10",
            MetalLayerId::M11 => "M11",
            MetalLayerId::M12 => "M12",
            MetalLayerId::M13 => "M13",
            MetalLayerId::M14 => "M14",
            MetalLayerId::M15 => "M15",
        }
    }

    /// All layers in bottom-to-top order.
    pub fn all() -> [MetalLayerId; 15] {
        [
            MetalLayerId::M1,
            MetalLayerId::M2,
            MetalLayerId::M3,
            MetalLayerId::M4,
            MetalLayerId::M5,
            MetalLayerId::M6,
            MetalLayerId::M7,
            MetalLayerId::M8,
            MetalLayerId::M9,
            MetalLayerId::M10,
            MetalLayerId::M11,
            MetalLayerId::M12,
            MetalLayerId::M13,
            MetalLayerId::M14,
            MetalLayerId::M15,
        ]
    }
}

/// Physical geometry and electrical specification for an individual metal interconnect layer.
#[derive(Debug, Clone)]
pub struct MetalLayerProperties {
    pub layer_id: MetalLayerId,
    /// Wire drawn width in nanometers.
    pub width_nm: f64,
    /// Wire height / thickness in nanometers.
    pub thickness_nm: f64,
    /// Effective copper thin-film resistivity in nOhm*m (accounting for surface/grain scattering).
    pub rho_nohm_m: f64,
    /// Operating RMS current in milliamperes.
    pub current_ma: f64,
    /// Interconnect segment length in micrometers.
    pub length_um: f64,
    /// Thermal resistance of surrounding low-k dielectric to silicon substrate in K/W.
    pub dielectric_thermal_resistance_k_per_w: f64,
}

/// Black's equation and material parameters for electromigration evaluation.
#[derive(Debug, Clone)]
pub struct BlacksEquationParams {
    /// Empirical geometry scaling constant A_em in hours * (A/cm^2)^n.
    pub a_em: f64,
    /// Current density exponent n (typically 1.8 for void growth).
    pub current_exponent_n: f64,
    /// Electromigration activation energy E_a in eV (typically 0.88 to 0.95 eV for Cu/Co cap).
    pub e_a_ev: f64,
    /// Critical Blech product threshold (j * L)_crit in A/cm.
    pub blech_crit_a_per_cm: f64,
    /// Failure threshold percentage resistance rise (e.g. 10.0%).
    pub failure_delta_r_pct: f64,
}

impl Default for BlacksEquationParams {
    fn default() -> Self {
        Self {
            a_em: 2.8e14,
            current_exponent_n: 1.85,
            e_a_ev: 0.885,
            blech_crit_a_per_cm: 3500.0,
            failure_delta_r_pct: 10.0,
        }
    }
}

/// Electromigration analysis result for a specific metallization layer.
#[derive(Debug, Clone)]
pub struct MetalLayerEmResult {
    pub layer_id: MetalLayerId,
    pub name: String,
    pub cross_section_um2: f64,
    pub current_density_ma_per_um2: f64,
    pub current_density_a_per_cm2: f64,
    pub joule_heating_delta_t_k: f64,
    pub effective_metal_temp_k: f64,
    pub jl_product_a_per_cm: f64,
    pub is_blech_immortal: bool,
    pub mttf_hours: f64,
    pub mttf_years: f64,
    pub resistance_drift_10yr_pct: f64,
    pub is_em_compliant: bool,
}

/// Evaluates electromigration metrics for a given metal layer under operating substrate temperature.
pub fn evaluate_metal_layer_em(
    layer: &MetalLayerProperties,
    params: &BlacksEquationParams,
    t_substrate_k: f64,
) -> MetalLayerEmResult {
    // Cross section in um^2: (w_nm * 1e-3) * (t_nm * 1e-3)
    let w_um = layer.width_nm * 1.0e-3;
    let t_um = layer.thickness_nm * 1.0e-3;
    let cross_section_um2 = (w_um * t_um).max(1.0e-6);

    // Current density:
    // in mA / um^2:
    let j_ma_per_um2 = layer.current_ma / cross_section_um2;
    // in A / cm^2: 1 mA / um^2 = (1e-3 A) / (1e-4 cm)^2 = 1e5 A / cm^2
    let j_a_per_cm2 = j_ma_per_um2 * 1.0e5;

    // Resistance of wire segment: R = rho * (L / A)
    // rho in nOhm*m = rho * 1e-9 Ohm*m
    // L in m = L_um * 1e-6 m
    // A in m^2 = cross_section_um2 * 1e-12 m^2
    let rho_ohm_m = layer.rho_nohm_m * 1.0e-9;
    let length_m = layer.length_um * 1.0e-6;
    let cross_section_m2 = cross_section_um2 * 1.0e-12;
    let r_wire = rho_ohm_m * (length_m / cross_section_m2);

    // Joule heating power P = I^2 * R
    let current_a = layer.current_ma * 1.0e-3;
    let p_joule_w = current_a.powi(2) * r_wire;

    // Joule heating temperature rise Delta T
    let joule_delta_t_k = (p_joule_w * layer.dielectric_thermal_resistance_k_per_w).clamp(0.01, 80.0);
    let effective_metal_temp_k = t_substrate_k + joule_delta_t_k;

    // Blech product calculation: (J * L) in A/cm
    // j in A/cm^2, L in cm: L_cm = layer.length_um * 1e-4
    let length_cm = layer.length_um * 1.0e-4;
    let jl_product_a_per_cm = j_a_per_cm2 * length_cm;
    let is_blech_immortal = jl_product_a_per_cm <= params.blech_crit_a_per_cm;

    // Black's equation evaluation:
    // MTTF = A_em / (J^n) * exp(E_a / (k_B * T_metal))
    let j_factor = j_a_per_cm2.powf(params.current_exponent_n).max(1.0);
    let arrhenius = (params.e_a_ev / (K_BOLTZMANN_EV_PER_K * effective_metal_temp_k)).exp();

    let mttf_hours = if is_blech_immortal {
        // Immortal wire: essentially infinite MTTF (represented as 1.0e8 hours ~ 11,400 years)
        1.0e8
    } else {
        (params.a_em / j_factor) * arrhenius
    };

    let mttf_years = mttf_hours / 8766.0;

    // Resistance drift over 10-year mission life (87,660 hours)
    let ten_year_hours = 10.0 * 8766.0;
    let resistance_drift_10yr_pct = if is_blech_immortal {
        0.05 // negligible stress backflow
    } else {
        ((ten_year_hours / mttf_hours.max(1.0)) * params.failure_delta_r_pct).clamp(0.0, 100.0)
    };

    // 10-year compliance criteria: MTTF >= 10.0 years AND resistance drift <= failure threshold
    let is_em_compliant = is_blech_immortal || (mttf_years >= 10.0 && resistance_drift_10yr_pct <= params.failure_delta_r_pct);

    MetalLayerEmResult {
        layer_id: layer.layer_id,
        name: layer.layer_id.name().to_string(),
        cross_section_um2,
        current_density_ma_per_um2: j_ma_per_um2,
        current_density_a_per_cm2: j_a_per_cm2,
        joule_heating_delta_t_k: joule_delta_t_k,
        effective_metal_temp_k,
        jl_product_a_per_cm,
        is_blech_immortal,
        mttf_hours,
        mttf_years,
        resistance_drift_10yr_pct,
        is_em_compliant,
    }
}

/// Builds standard industrial 15-tier metal stack for advanced FinFET / GAA-FET server nodes.
pub fn build_default_m1_to_m15_stack() -> Vec<MetalLayerProperties> {
    vec![
        MetalLayerProperties {
            layer_id: MetalLayerId::M1,
            width_nm: 24.0,
            thickness_nm: 48.0,
            rho_nohm_m: 44.0,
            current_ma: 0.12,
            length_um: 1.8,
            dielectric_thermal_resistance_k_per_w: 12000.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M2,
            width_nm: 28.0,
            thickness_nm: 56.0,
            rho_nohm_m: 39.0,
            current_ma: 0.16,
            length_um: 2.2,
            dielectric_thermal_resistance_k_per_w: 10500.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M3,
            width_nm: 32.0,
            thickness_nm: 64.0,
            rho_nohm_m: 36.0,
            current_ma: 0.32,
            length_um: 8.0,
            dielectric_thermal_resistance_k_per_w: 9200.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M4,
            width_nm: 42.0,
            thickness_nm: 84.0,
            rho_nohm_m: 32.0,
            current_ma: 0.52,
            length_um: 14.0,
            dielectric_thermal_resistance_k_per_w: 7800.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M5,
            width_nm: 52.0,
            thickness_nm: 104.0,
            rho_nohm_m: 29.0,
            current_ma: 0.78,
            length_um: 22.0,
            dielectric_thermal_resistance_k_per_w: 6500.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M6,
            width_nm: 68.0,
            thickness_nm: 136.0,
            rho_nohm_m: 26.5,
            current_ma: 1.15,
            length_um: 35.0,
            dielectric_thermal_resistance_k_per_w: 5200.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M7,
            width_nm: 88.0,
            thickness_nm: 176.0,
            rho_nohm_m: 24.5,
            current_ma: 1.65,
            length_um: 50.0,
            dielectric_thermal_resistance_k_per_w: 4200.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M8,
            width_nm: 115.0,
            thickness_nm: 230.0,
            rho_nohm_m: 23.0,
            current_ma: 2.45,
            length_um: 80.0,
            dielectric_thermal_resistance_k_per_w: 3400.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M9,
            width_nm: 155.0,
            thickness_nm: 310.0,
            rho_nohm_m: 21.8,
            current_ma: 3.80,
            length_um: 120.0,
            dielectric_thermal_resistance_k_per_w: 2600.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M10,
            width_nm: 220.0,
            thickness_nm: 440.0,
            rho_nohm_m: 20.5,
            current_ma: 6.20,
            length_um: 180.0,
            dielectric_thermal_resistance_k_per_w: 1900.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M11,
            width_nm: 320.0,
            thickness_nm: 640.0,
            rho_nohm_m: 19.8,
            current_ma: 11.50,
            length_um: 280.0,
            dielectric_thermal_resistance_k_per_w: 1400.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M12,
            width_nm: 500.0,
            thickness_nm: 1000.0,
            rho_nohm_m: 19.0,
            current_ma: 24.00,
            length_um: 450.0,
            dielectric_thermal_resistance_k_per_w: 950.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M13,
            width_nm: 800.0,
            thickness_nm: 1600.0,
            rho_nohm_m: 18.2,
            current_ma: 52.00,
            length_um: 750.0,
            dielectric_thermal_resistance_k_per_w: 620.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M14,
            width_nm: 1250.0,
            thickness_nm: 2500.0,
            rho_nohm_m: 17.6,
            current_ma: 110.00,
            length_um: 1200.0,
            dielectric_thermal_resistance_k_per_w: 380.0,
        },
        MetalLayerProperties {
            layer_id: MetalLayerId::M15,
            width_nm: 2000.0,
            thickness_nm: 3600.0,
            rho_nohm_m: 17.0,
            current_ma: 260.00,
            length_um: 2200.0,
            dielectric_thermal_resistance_k_per_w: 190.0,
        },
    ]
}

/// Evaluates full multi-tier metallization stack M1 through M15.
pub fn evaluate_full_metal_stack(
    stack: &[MetalLayerProperties],
    params: &BlacksEquationParams,
    t_substrate_k: f64,
) -> Vec<MetalLayerEmResult> {
    stack
        .iter()
        .map(|layer| evaluate_metal_layer_em(layer, params, t_substrate_k))
        .collect()
}
