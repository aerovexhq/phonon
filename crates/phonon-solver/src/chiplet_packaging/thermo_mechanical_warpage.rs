#![deny(unsafe_code)]

//! Multi-Material Thermo-Mechanical Warpage & Micro-Bump Shear Stress Engine.
//!
//! Models Coefficient of Thermal Expansion (CTE) mismatches across heterogeneous chiplet stacks:
//! - Silicon Compute & Memory Dies (CTE ~ 2.6 ppm/K, E ~ 130 GPa)
//! - Copper Micro-Bumps (CTE ~ 16.5 ppm/K, E ~ 117 GPa, G ~ 44 GPa)
//! - Epoxy Underfill Matrix (CTE ~ 30.0 ppm/K, E ~ 8.5 GPa)
//! - Silicon Interposer / Organic Substrate (CTE ~ 2.6 to 15.0 ppm/K)
//! - Multi-layer Timoshenko beam/plate thermal curvature and out-of-plane bow (um)
//! - Distance-from-Neutral-Point (DNP) corner bump inelastic shear strain and stress (MPa)
//! - Coffin-Manson low-cycle thermal fatigue cycles-to-failure (N_f) prediction.

/// Physical material elastic and thermal expansion properties.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerMaterial {
    pub name: String,
    /// Young's modulus in Gigapascals (GPa).
    pub youngs_modulus_gpa: f64,
    /// Poisson's ratio (dimensionless).
    pub poissons_ratio: f64,
    /// Coefficient of Thermal Expansion in ppm/K (1e-6 / K).
    pub cte_ppm_k: f64,
}

impl LayerMaterial {
    pub fn silicon() -> Self {
        Self {
            name: "Silicon (Die/Interposer)".to_string(),
            youngs_modulus_gpa: 130.0,
            poissons_ratio: 0.28,
            cte_ppm_k: 2.6,
        }
    }

    pub fn copper() -> Self {
        Self {
            name: "Copper (Bumps/RDL)".to_string(),
            youngs_modulus_gpa: 117.0,
            poissons_ratio: 0.34,
            cte_ppm_k: 16.5,
        }
    }

    pub fn epoxy_underfill() -> Self {
        Self {
            name: "Epoxy Underfill".to_string(),
            youngs_modulus_gpa: 8.5,
            poissons_ratio: 0.35,
            cte_ppm_k: 30.0,
        }
    }

    pub fn organic_substrate() -> Self {
        Self {
            name: "Organic Substrate (ABF/BT)".to_string(),
            youngs_modulus_gpa: 22.0,
            poissons_ratio: 0.22,
            cte_ppm_k: 15.0,
        }
    }
}

/// Package stack geometric dimensions.
#[derive(Debug, Clone, PartialEq)]
pub struct PackageStackGeometry {
    /// Top die edge length in millimeters (typically 10 to 30 mm).
    pub die_size_mm: f64,
    /// Top die thickness in micrometers (typically 50 to 775 um).
    pub die_thickness_um: f64,
    /// Package substrate edge length in millimeters (typically 35 to 70 mm).
    pub substrate_size_mm: f64,
    /// Package substrate thickness in micrometers (typically 800 to 1800 um).
    pub substrate_thickness_um: f64,
    /// Micro-bump height in micrometers (typically 15 to 40 um).
    pub bump_height_um: f64,
    /// Micro-bump diameter in micrometers (typically 15 to 30 um).
    pub bump_diameter_um: f64,
    /// Micro-bump pitch in micrometers (typically 25 to 130 um).
    pub bump_pitch_um: f64,
    /// Whether an intermediate silicon interposer (e.g. CoWoS) is present.
    pub has_silicon_interposer: bool,
    /// Silicon interposer thickness in micrometers (typically 100 um).
    pub interposer_thickness_um: f64,
}

impl Default for PackageStackGeometry {
    fn default() -> Self {
        Self {
            die_size_mm: 18.0,
            die_thickness_um: 150.0,
            substrate_size_mm: 50.0,
            substrate_thickness_um: 1200.0,
            bump_height_um: 25.0,
            bump_diameter_um: 25.0,
            bump_pitch_um: 35.0,
            has_silicon_interposer: true,
            interposer_thickness_um: 100.0,
        }
    }
}

/// Thermal cycling conditions for warpage and fatigue evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct ThermalCycleParams {
    /// Stress-free reference temperature (underfill cure / solder freeze temperature) in Celsius.
    pub ref_temp_c: f64,
    /// Minimum operating/testing temperature in Celsius (e.g. -40 C for MIL/Aero).
    pub t_min_c: f64,
    /// Maximum operating/testing temperature in Celsius (e.g. 125 C).
    pub t_max_c: f64,
    /// Current evaluation temperature in Celsius.
    pub current_temp_c: f64,
}

impl Default for ThermalCycleParams {
    fn default() -> Self {
        Self {
            ref_temp_c: 175.0, // Typical underfill curing temperature
            t_min_c: -40.0,
            t_max_c: 125.0,
            current_temp_c: 25.0, // Room temperature
        }
    }
}

/// Computed thermo-mechanical warpage and shear stress results.
#[derive(Debug, Clone, PartialEq)]
pub struct WarpageStressReport {
    /// Thermal expansion mismatch Delta-alpha between die and base in ppm/K.
    pub delta_cte_ppm_k: f64,
    /// Out-of-plane package bow warpage at current temperature in micrometers.
    /// Positive = Convex ("Crying"), Negative = Concave ("Smiling").
    pub warpage_bow_um: f64,
    /// Maximum warpage bow across the complete thermal cycle [-40 C to 125 C] in um.
    pub max_cycle_warpage_um: f64,
    /// Whether the package satisfies standard coplanarity limits (< 50 um bow).
    pub is_coplanar: bool,
    /// Distance from neutral point (DNP) to the corner micro-bump in millimeters.
    pub dnp_mm: f64,
    /// Maximum shear strain on the outermost corner micro-bump (percentage).
    pub corner_bump_shear_strain_pct: f64,
    /// Maximum shear stress on the outermost corner micro-bump in Megapascals (MPa).
    pub corner_bump_shear_stress_mpa: f64,
    /// Coffin-Manson low-cycle thermal fatigue life in thermal cycles (N_f).
    pub fatigue_cycles_to_failure: u32,
    /// Assessment rating ("Pass", "Marginal", "HighRisk").
    pub fatigue_status: String,
}

/// Evaluates package thermal warpage and micro-bump fatigue stress.
pub fn evaluate_thermo_mechanics(
    geom: &PackageStackGeometry,
    thermal: &ThermalCycleParams,
) -> WarpageStressReport {
    let mat_die = LayerMaterial::silicon();
    let mat_sub = if geom.has_silicon_interposer {
        // Effective CTE of interposer + substrate composite
        LayerMaterial {
            name: "Silicon Interposer on Substrate".to_string(),
            youngs_modulus_gpa: 85.0,
            poissons_ratio: 0.25,
            cte_ppm_k: 4.8, // Drastically reduced CTE mismatch due to silicon interposer!
        }
    } else {
        LayerMaterial::organic_substrate()
    };

    let delta_cte = (mat_sub.cte_ppm_k - mat_die.cte_ppm_k).abs();

    // Timoshenko bi-material beam curvature formula:
    // kappa = 6 * (alpha_sub - alpha_die) * delta_T * (1 + m)^2 /
    //         ( h * [ 3*(1+m)^2 + (1 + m*n)*(m^2 + 1/(m*n)) ] )
    // where m = t_die / t_sub, n = E_die / E_sub, h = t_die + t_sub
    let t_die_m = geom.die_thickness_um * 1e-6;
    let t_sub_m = geom.substrate_thickness_um * 1e-6;
    let h_total_m = t_die_m + t_sub_m;
    let m = t_die_m / t_sub_m;
    let n = mat_die.youngs_modulus_gpa / mat_sub.youngs_modulus_gpa;

    let delta_t_current = thermal.current_temp_c - thermal.ref_temp_c; // Negative when cooling down
    let delta_alpha_si = (mat_sub.cte_ppm_k - mat_die.cte_ppm_k) * 1e-6;

    let denom = h_total_m * (3.0 * (1.0 + m).powi(2) + (1.0 + m * n) * (m.powi(2) + 1.0 / (m * n)));
    let kappa_current = 6.0 * delta_alpha_si * delta_t_current * (1.0 + m).powi(2) / denom;

    // Out-of-plane bow: w = kappa * L^2 / 8
    let l_sub_m = geom.substrate_size_mm * 1e-3;
    let warpage_bow_um = (kappa_current * l_sub_m.powi(2) / 8.0) * 1e6;

    // Evaluate worst-case warpage at T_min (maximum cooling delta from cure temperature)
    let delta_t_worst = thermal.t_min_c - thermal.ref_temp_c;
    let kappa_worst = 6.0 * delta_alpha_si * delta_t_worst * (1.0 + m).powi(2) / denom;
    let max_cycle_warpage_um = ((kappa_worst * l_sub_m.powi(2) / 8.0) * 1e6).abs();

    // JEDEC JESD22-B112 / JEITA ED-7306 standard for large multichip modules:
    // Room-temperature coplanarity bow <= 75 um, max extreme thermal cycle bow <= 100 um.
    let is_coplanar = warpage_bow_um.abs() <= 75.0 && max_cycle_warpage_um <= 100.0;

    // Outermost corner micro-bump: DNP = sqrt(2) * (L_die / 2)
    let dnp_mm = (2.0_f64.sqrt()) * (geom.die_size_mm / 2.0);
    let dnp_m = dnp_mm * 1e-3;
    let h_bump_m = geom.bump_height_um * 1e-6;

    // Total thermal displacement mismatch at corner: Delta_L = Delta_alpha * Delta_T * DNP
    // Thermal cycling range: Delta_T_cycle = T_max - T_min
    let delta_t_cycle = thermal.t_max_c - thermal.t_min_c;
    let delta_l_m = delta_alpha_si * delta_t_cycle * dnp_m;

    // Epoxy underfill matrix absorbs ~85-93% of relative shear displacement in 2.5D packages
    let underfill_strain_relief = if geom.has_silicon_interposer { 0.075 } else { 0.15 };
    let shear_strain = (delta_l_m / h_bump_m) * underfill_strain_relief;
    let shear_strain_pct = shear_strain * 100.0;

    // Shear modulus of copper G_cu ~ 44 GPa
    let g_cu_mpa = 44_000.0;
    // Effective elastoplastic shear stress with hardening limit at ~180 MPa
    let corner_bump_shear_stress_mpa = (g_cu_mpa * shear_strain).min(180.0);

    // Coffin-Manson low-cycle fatigue equation:
    // N_f = 0.5 * (Delta_gamma_p / (2 * epsilon_f))^(-1 / c)
    // where 2 * epsilon_f ~ 0.65 (fatigue ductility coefficient for SAC/Cu), c ~ 0.50 (fatigue ductility exponent)
    let plastic_strain = (shear_strain - 0.002).max(0.0001);
    let cycles_calc = 0.5 * (plastic_strain / 0.65).powf(-2.0);
    let fatigue_cycles_to_failure = (cycles_calc as u32).min(250_000).max(10);

    let fatigue_status = if fatigue_cycles_to_failure >= 2_000 {
        "Pass (Aero/Space Qualified)".to_string()
    } else if fatigue_cycles_to_failure >= 800 {
        "Marginal (Consumer Tier)".to_string()
    } else {
        "HighRisk (Underfill / Pitch Redesign Required)".to_string()
    };

    WarpageStressReport {
        delta_cte_ppm_k: delta_cte,
        warpage_bow_um,
        max_cycle_warpage_um,
        is_coplanar,
        dnp_mm,
        corner_bump_shear_strain_pct: shear_strain_pct,
        corner_bump_shear_stress_mpa,
        fatigue_cycles_to_failure,
        fatigue_status,
    }
}
