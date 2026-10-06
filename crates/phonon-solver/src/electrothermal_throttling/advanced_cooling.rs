#![deny(unsafe_code)]

/// Acceleration of gravity (m/s^2).
pub const GRAVITY_ACCEL: f64 = 9.80665;
/// Copper thermal conductivity for fins/spreader (W/(m*K)).
pub const COPPER_THERMAL_CONDUCTIVITY: f64 = 390.0;

/// Coolant fluid selection for liquid cold-plate microchannels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoolantFluid {
    PureWater,
    PropyleneGlycol50,
}

/// Thermophysical properties of a liquid coolant.
#[derive(Debug, Clone)]
pub struct LiquidCoolantProperties {
    pub density_kg_m3: f64,
    pub dynamic_viscosity_pa_s: f64,
    pub thermal_conductivity_w_m_k: f64,
    pub specific_heat_j_kg_k: f64,
}

impl CoolantFluid {
    pub fn properties(&self) -> LiquidCoolantProperties {
        match self {
            CoolantFluid::PureWater => LiquidCoolantProperties {
                density_kg_m3: 997.0,
                dynamic_viscosity_pa_s: 8.9e-4,
                thermal_conductivity_w_m_k: 0.606,
                specific_heat_j_kg_k: 4182.0,
            },
            CoolantFluid::PropyleneGlycol50 => LiquidCoolantProperties {
                density_kg_m3: 1045.0,
                dynamic_viscosity_pa_s: 3.5e-3,
                thermal_conductivity_w_m_k: 0.380,
                specific_heat_j_kg_k: 3500.0,
            },
        }
    }
}

/// Geometric and operational parameters for a microchannel liquid cold-plate.
#[derive(Debug, Clone)]
pub struct MicrochannelParams {
    /// Channel width in meters (e.g. 100 um = 0.0001 m).
    pub channel_width_m: f64,
    /// Channel height in meters (e.g. 500 um = 0.0005 m).
    pub channel_height_m: f64,
    /// Fin thickness in meters (e.g. 100 um = 0.0001 m).
    pub fin_thickness_m: f64,
    /// Channel flow length in meters (e.g. 25 mm = 0.025 m).
    pub channel_length_m: f64,
    /// Cold-plate width across channels in meters (e.g. 25 mm = 0.025 m).
    pub plate_width_m: f64,
    /// Coolant volumetric flow rate in Liters per minute (L/min).
    pub flow_rate_lpm: f64,
    /// Coolant inlet temperature in Celsius.
    pub inlet_temp_c: f64,
    /// Coolant type.
    pub coolant: CoolantFluid,
}

impl Default for MicrochannelParams {
    fn default() -> Self {
        Self {
            channel_width_m: 1.2e-4, // 120 um
            channel_height_m: 6.0e-4, // 600 um
            fin_thickness_m: 1.0e-4, // 100 um
            channel_length_m: 0.028, // 28 mm
            plate_width_m: 0.028, // 28 mm
            flow_rate_lpm: 1.2, // 1.2 L/min
            inlet_temp_c: 25.0,
            coolant: CoolantFluid::PureWater,
        }
    }
}

/// Hydrodynamic and thermal results for liquid cold-plate microchannels.
#[derive(Debug, Clone)]
pub struct MicrochannelPerformance {
    pub channel_count: usize,
    pub hydraulic_diameter_m: f64,
    pub fluid_velocity_m_s: f64,
    pub reynolds_number: f64,
    pub prandtl_number: f64,
    pub nusselt_number: f64,
    pub heat_transfer_coeff_w_m2_k: f64,
    pub fin_efficiency: f64,
    pub total_wetted_area_m2: f64,
    pub convective_resistance_k_w: f64,
    pub caloric_resistance_k_w: f64,
    pub total_coldplate_resistance_k_w: f64,
    pub pressure_drop_kpa: f64,
    pub pumping_power_w: f64,
}

/// Evaluates hydrodynamics and thermal resistance of a microchannel cold-plate.
pub fn calculate_microchannel_performance(params: &MicrochannelParams) -> MicrochannelPerformance {
    let props = params.coolant.properties();

    let pitch = params.channel_width_m + params.fin_thickness_m;
    let channel_count = ((params.plate_width_m / pitch).floor() as usize).max(1);

    // Channel cross-sectional area and hydraulic diameter
    let a_c = params.channel_width_m * params.channel_height_m;
    let p_wet = 2.0 * (params.channel_width_m + params.channel_height_m);
    let d_h = (4.0 * a_c) / p_wet;

    // Volumetric flow rate in m^3/s: 1 L/min = 1e-3 / 60 m^3/s
    let flow_rate_m3_s = (params.flow_rate_lpm * 1e-3) / 60.0;
    let mass_flow_kg_s = flow_rate_m3_s * props.density_kg_m3;

    let fluid_velocity = flow_rate_m3_s / ((channel_count as f64) * a_c).max(1e-9);

    let reynolds = (props.density_kg_m3 * fluid_velocity * d_h) / props.dynamic_viscosity_pa_s;
    let prandtl = (props.dynamic_viscosity_pa_s * props.specific_heat_j_kg_k) / props.thermal_conductivity_w_m_k;

    // Nusselt number correlation
    let nusselt = if reynolds < 2300.0 {
        // Laminar developing flow in rectangular duct
        let gz = (d_h / params.channel_length_m) * reynolds * prandtl;
        4.36 + (0.0668 * gz) / (1.0 + 0.04 * gz.powf(2.0 / 3.0))
    } else {
        // Turbulent Gnielinski correlation
        let f_darcy = (0.790 * reynolds.ln() - 1.64).powi(-2);
        let num = (f_darcy / 8.0) * (reynolds - 1000.0) * prandtl;
        let den = 1.0 + 12.7 * (f_darcy / 8.0).sqrt() * (prandtl.powf(2.0 / 3.0) - 1.0);
        (num / den.max(0.1)).max(4.36)
    };

    let h_coeff = (nusselt * props.thermal_conductivity_w_m_k) / d_h;

    // Fin efficiency
    let m_fin = ((2.0 * h_coeff) / (COPPER_THERMAL_CONDUCTIVITY * params.fin_thickness_m)).sqrt();
    let m_h = m_fin * params.channel_height_m;
    let fin_eff = if m_h > 1e-5 {
        m_h.tanh() / m_h
    } else {
        1.0
    };

    // Wetted surface area
    let base_area = (channel_count as f64) * params.channel_width_m * params.channel_length_m;
    let fin_area = (channel_count as f64) * 2.0 * params.channel_height_m * params.channel_length_m;
    let total_wetted_area = base_area + fin_area;
    let eff_area = base_area + fin_eff * fin_area;

    // Convective resistance
    let r_conv = 1.0 / (h_coeff * eff_area).max(1e-6);

    // Caloric (sensible heating of the coolant along the channel length)
    let r_caloric = 1.0 / (2.0 * mass_flow_kg_s * props.specific_heat_j_kg_k).max(1e-6);

    let r_total_cp = r_conv + r_caloric;

    // Pressure drop (Darcy-Weisbach)
    let friction_factor = if reynolds < 2300.0 {
        64.0 / reynolds.max(1.0)
    } else {
        (0.790 * reynolds.ln() - 1.64).powi(-2)
    };
    let delta_p_pa = friction_factor * (params.channel_length_m / d_h) * 0.5 * props.density_kg_m3 * fluid_velocity * fluid_velocity;
    let delta_p_kpa = delta_p_pa / 1000.0;
    let pumping_power_w = flow_rate_m3_s * delta_p_pa;

    MicrochannelPerformance {
        channel_count,
        hydraulic_diameter_m: d_h,
        fluid_velocity_m_s: fluid_velocity,
        reynolds_number: reynolds,
        prandtl_number: prandtl,
        nusselt_number: nusselt,
        heat_transfer_coeff_w_m2_k: h_coeff,
        fin_efficiency: fin_eff,
        total_wetted_area_m2: total_wetted_area,
        convective_resistance_k_w: r_conv,
        caloric_resistance_k_w: r_caloric,
        total_coldplate_resistance_k_w: r_total_cp,
        pressure_drop_kpa: delta_p_kpa,
        pumping_power_w,
    }
}

/// Dielectric immersion cooling fluid kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImmersionFluidKind {
    FluorinertFC72,
    Novec7100,
}

/// Thermophysical boiling parameters for dielectric immersion cooling.
#[derive(Debug, Clone)]
pub struct ImmersionFluidProperties {
    pub boiling_point_c: f64,
    pub latent_heat_j_kg: f64,
    pub liquid_density_kg_m3: f64,
    pub vapor_density_kg_m3: f64,
    pub surface_tension_n_m: f64,
    pub liquid_specific_heat_j_kg_k: f64,
    pub liquid_viscosity_pa_s: f64,
    pub liquid_prandtl: f64,
    pub surface_fluid_c_sf: f64,
    pub boiling_exponent_n: f64,
}

impl ImmersionFluidKind {
    pub fn properties(&self) -> ImmersionFluidProperties {
        match self {
            ImmersionFluidKind::FluorinertFC72 => ImmersionFluidProperties {
                boiling_point_c: 56.0,
                latent_heat_j_kg: 88000.0,
                liquid_density_kg_m3: 1680.0,
                vapor_density_kg_m3: 13.5,
                surface_tension_n_m: 0.010,
                liquid_specific_heat_j_kg_k: 1100.0,
                liquid_viscosity_pa_s: 6.4e-4,
                liquid_prandtl: 9.2,
                surface_fluid_c_sf: 0.0054,
                boiling_exponent_n: 1.7,
            },
            ImmersionFluidKind::Novec7100 => ImmersionFluidProperties {
                boiling_point_c: 61.0,
                latent_heat_j_kg: 112000.0,
                liquid_density_kg_m3: 1510.0,
                vapor_density_kg_m3: 9.6,
                surface_tension_n_m: 0.0136,
                liquid_specific_heat_j_kg_k: 1183.0,
                liquid_viscosity_pa_s: 5.8e-4,
                liquid_prandtl: 8.8,
                surface_fluid_c_sf: 0.0062,
                boiling_exponent_n: 1.7,
            },
        }
    }
}

/// Parameters for two-phase immersion cooling.
#[derive(Debug, Clone)]
pub struct ImmersionCoolingParams {
    pub fluid: ImmersionFluidKind,
    /// Active die dissipation area in m^2 (e.g. 6.25 cm^2 = 6.25e-4 m^2).
    pub die_area_m2: f64,
    /// Liquid bulk pool temperature in Celsius.
    pub bulk_liquid_temp_c: f64,
}

impl Default for ImmersionCoolingParams {
    fn default() -> Self {
        Self {
            fluid: ImmersionFluidKind::FluorinertFC72,
            die_area_m2: 6.25e-4, // 25 mm x 25 mm
            bulk_liquid_temp_c: 45.0,
        }
    }
}

/// Two-phase boiling curve point.
#[derive(Debug, Clone)]
pub struct BoilingCurvePoint {
    pub superheat_delta_t_sat_k: f64,
    pub surface_temp_c: f64,
    pub heat_flux_w_cm2: f64,
    pub heat_transfer_coeff_w_m2_k: f64,
    pub thermal_resistance_k_w: f64,
    pub in_chf_risk: bool,
}

/// Two-phase immersion performance report.
#[derive(Debug, Clone)]
pub struct ImmersionPerformance {
    pub boiling_point_c: f64,
    pub critical_heat_flux_w_cm2: f64,
    pub current_heat_flux_w_cm2: f64,
    pub chf_margin_percentage: f64,
    pub effective_heat_transfer_coeff_w_m2_k: f64,
    pub boiling_thermal_resistance_k_w: f64,
    pub is_nucleate_boiling_active: bool,
}

/// Calculates Critical Heat Flux (CHF) using Zuber pool boiling correlation.
pub fn calculate_zuber_chf(props: &ImmersionFluidProperties) -> f64 {
    let delta_rho = props.liquid_density_kg_m3 - props.vapor_density_kg_m3;
    let factor = (props.surface_tension_n_m * GRAVITY_ACCEL * delta_rho).powf(0.25);
    let q_chf_w_m2 = 0.131 * props.vapor_density_kg_m3.sqrt() * props.latent_heat_j_kg * factor;
    q_chf_w_m2 / 1e4 // Convert W/m^2 to W/cm^2
}

/// Calculates heat flux for a given wall superheat Delta T_sat = T_wall - T_sat using Rohsenow correlation.
pub fn calculate_rohsenow_heat_flux(props: &ImmersionFluidProperties, delta_t_sat_k: f64) -> f64 {
    if delta_t_sat_k <= 0.0 {
        return 0.0;
    }

    // Natural convection onset threshold
    let t_onb = 2.5; // K
    if delta_t_sat_k < t_onb {
        // Natural convection regime: h ~ 300 W/(m^2*K)
        let q_nc = 300.0 * delta_t_sat_k;
        return q_nc / 1e4; // W/cm^2
    }

    let delta_rho = (props.liquid_density_kg_m3 - props.vapor_density_kg_m3).max(1.0);
    let buo_term = (GRAVITY_ACCEL * delta_rho / props.surface_tension_n_m).sqrt();
    let lead_term = props.liquid_viscosity_pa_s * props.latent_heat_j_kg * buo_term;

    let pr_term = props.liquid_prandtl.powf(props.boiling_exponent_n);
    let bracket = (props.liquid_specific_heat_j_kg_k * delta_t_sat_k)
        / (props.surface_fluid_c_sf * props.latent_heat_j_kg * pr_term);

    let q_rohsenow_w_m2 = lead_term * bracket.powi(3);
    q_rohsenow_w_m2 / 1e4 // W/cm^2
}

/// Evaluates two-phase immersion cooling performance for a given operating surface temperature.
pub fn calculate_immersion_performance(
    params: &ImmersionCoolingParams,
    surface_temp_c: f64,
) -> ImmersionPerformance {
    let props = params.fluid.properties();
    let delta_t_sat = (surface_temp_c - props.boiling_point_c).max(0.0);
    let chf_w_cm2 = calculate_zuber_chf(&props);

    let q_w_cm2 = calculate_rohsenow_heat_flux(&props, delta_t_sat);
    let q_w_m2 = q_w_cm2 * 1e4;

    let h_eff = if delta_t_sat > 0.1 {
        q_w_m2 / delta_t_sat
    } else {
        300.0 // Single-phase baseline
    };

    let r_boil = 1.0 / (h_eff * params.die_area_m2).max(1e-6);
    let chf_margin = (1.0 - (q_w_cm2 / chf_w_cm2)).max(0.0) * 100.0;
    let is_nucleate = delta_t_sat >= 2.5;

    ImmersionPerformance {
        boiling_point_c: props.boiling_point_c,
        critical_heat_flux_w_cm2: chf_w_cm2,
        current_heat_flux_w_cm2: q_w_cm2,
        chf_margin_percentage: chf_margin,
        effective_heat_transfer_coeff_w_m2_k: h_eff,
        boiling_thermal_resistance_k_w: r_boil,
        is_nucleate_boiling_active: is_nucleate,
    }
}

/// Generates boiling curve samples for plotting.
pub fn generate_boiling_curve(
    params: &ImmersionCoolingParams,
    num_points: usize,
) -> Vec<BoilingCurvePoint> {
    let props = params.fluid.properties();
    let chf_w_cm2 = calculate_zuber_chf(&props);
    let max_delta_t = 25.0; // K
    let step = max_delta_t / (num_points.max(10) - 1) as f64;

    let mut curve = Vec::with_capacity(num_points);
    for i in 0..num_points {
        let dt = (i as f64) * step;
        let t_surf = props.boiling_point_c + dt;
        let q_flux = calculate_rohsenow_heat_flux(&props, dt);
        let q_w_m2 = q_flux * 1e4;
        let h_eff = if dt > 0.1 { q_w_m2 / dt } else { 300.0 };
        let r_boil = 1.0 / (h_eff * params.die_area_m2).max(1e-6);

        curve.push(BoilingCurvePoint {
            superheat_delta_t_sat_k: dt,
            surface_temp_c: t_surf,
            heat_flux_w_cm2: q_flux,
            heat_transfer_coeff_w_m2_k: h_eff,
            thermal_resistance_k_w: r_boil,
            in_chf_risk: q_flux >= 0.85 * chf_w_cm2,
        });
    }
    curve
}

/// Thermal Interface Material (TIM-1 & TIM-2) pump-out aging model.
#[derive(Debug, Clone)]
pub struct TimAgingModel {
    /// Initial bondline thickness in micrometers (um), e.g. 35.0 um.
    pub initial_blt_um: f64,
    /// Bulk thermal conductivity of TIM in W/(m*K), e.g. 4.8 W/(m*K).
    pub thermal_conductivity_w_m_k: f64,
    /// Contact resistance in K*cm^2/W, e.g. 0.04 K*cm^2/W.
    pub contact_resistance_k_cm2_w: f64,
    /// Pump-out rate coefficient per cycle^0.5 (e.g. 0.0035).
    pub pump_out_rate_coeff: f64,
    /// Number of cumulative thermal power cycles.
    pub power_cycles: usize,
    /// Active die area in cm^2 (e.g. 6.25 cm^2).
    pub die_area_cm2: f64,
}

impl Default for TimAgingModel {
    fn default() -> Self {
        Self {
            initial_blt_um: 35.0,
            thermal_conductivity_w_m_k: 4.8,
            contact_resistance_k_cm2_w: 0.04,
            pump_out_rate_coeff: 0.0035,
            power_cycles: 0,
            die_area_cm2: 6.25,
        }
    }
}

/// TIM aging performance point.
#[derive(Debug, Clone)]
pub struct TimAgingPoint {
    pub cycles: usize,
    pub unit_resistance_k_cm2_w: f64,
    pub total_tim_resistance_k_w: f64,
    pub degradation_percentage: f64,
}

impl TimAgingModel {
    /// Evaluates unit thermal resistance r_tim in K*cm^2/W for a given cycle count.
    pub fn unit_resistance_at_cycles(&self, cycles: usize) -> f64 {
        // r_tim0 = (BLT_m / k_tim) * 1e4 + r_contact
        let blt_m = self.initial_blt_um * 1e-6;
        let r_bulk_cm2 = (blt_m / self.thermal_conductivity_w_m_k) * 1e4;
        let r_tim0 = r_bulk_cm2 + self.contact_resistance_k_cm2_w;

        // Fickian pump-out degradation: r_tim(N) = r_tim0 * (1 + gamma * N^0.5)
        let degradation_factor = 1.0 + self.pump_out_rate_coeff * (cycles as f64).sqrt();
        r_tim0 * degradation_factor
    }

    /// Evaluates total package TIM thermal resistance in K/W.
    pub fn total_resistance_k_w(&self) -> f64 {
        self.unit_resistance_at_cycles(self.power_cycles) / self.die_area_cm2.max(0.1)
    }

    /// Evaluates baseline (unaged) total TIM thermal resistance in K/W.
    pub fn baseline_resistance_k_w(&self) -> f64 {
        self.unit_resistance_at_cycles(0) / self.die_area_cm2.max(0.1)
    }

    /// Generates aging trajectory over thermal cycle range.
    pub fn generate_aging_trajectory(&self, max_cycles: usize, num_points: usize) -> Vec<TimAgingPoint> {
        let r0 = self.unit_resistance_at_cycles(0);
        let step = (max_cycles as f64) / (num_points.max(10) - 1) as f64;

        let mut points = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let cycles = ((i as f64) * step).round() as usize;
            let r_unit = self.unit_resistance_at_cycles(cycles);
            let r_tot = r_unit / self.die_area_cm2.max(0.1);
            let deg_pct = ((r_unit - r0) / r0) * 100.0;

            points.push(TimAgingPoint {
                cycles,
                unit_resistance_k_cm2_w: r_unit,
                total_tim_resistance_k_w: r_tot,
                degradation_percentage: deg_pct,
            });
        }
        points
    }
}
