#![deny(unsafe_code)]

/// Boltzmann constant (J/K).
pub const K_BOLTZMANN: f64 = 1.380649e-23;
/// Elementary charge (Coulombs).
pub const Q_ELEM: f64 = 1.602176634e-19;
/// Absolute zero reference temperature in Celsius (0 C = 273.15 K).
pub const T_ZERO_CELSIUS_K: f64 = 273.15;

/// Leakage and semiconductor physics parameters for the silicon die.
#[derive(Debug, Clone)]
pub struct LeakageModelParams {
    /// Reference subthreshold leakage current at T0 (Amperes), e.g. 8.0 A for high-density multi-core.
    pub i_sub0: f64,
    /// Baseline threshold voltage at T0 (Volts), e.g. 0.35 V.
    pub v_th0: f64,
    /// Threshold voltage temperature coefficient (V/K), typical silicon ~0.0012 V/K (1.2 mV/K).
    pub alpha_vth: f64,
    /// Subthreshold ideality factor (dimensionless, ~1.2 to 1.5).
    pub m_ideality: f64,
    /// Reference temperature (Kelvin), default 300.0 K.
    pub t0_kelvin: f64,
    /// Gate oxide / Band-to-Band Tunneling (BTBT) base leakage (Amperes).
    pub gate_leakage_base: f64,
    /// Gate leakage temperature coefficient (1/K).
    pub gate_temp_coeff: f64,
}

impl Default for LeakageModelParams {
    fn default() -> Self {
        Self {
            i_sub0: 6.5,
            v_th0: 0.34,
            alpha_vth: 0.0012,
            m_ideality: 1.35,
            t0_kelvin: 300.0,
            gate_leakage_base: 0.8,
            gate_temp_coeff: 0.0035,
        }
    }
}

/// Dynamic power model parameters.
#[derive(Debug, Clone)]
pub struct DynamicPowerParams {
    /// Total effective switched capacitance C_eff = alpha * C_total (Farads), e.g. 15.0 nF.
    pub effective_capacitance: f64,
    /// Baseline activity factor [0.0, 1.0].
    pub activity_factor: f64,
}

impl Default for DynamicPowerParams {
    fn default() -> Self {
        Self {
            effective_capacitance: 1.8e-8,
            activity_factor: 0.85,
        }
    }
}

/// Status of thermal stability and runaway risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalStabilityStatus {
    /// System is thermally stable (S > 0.30).
    Stable,
    /// System is near the bifurcation boundary (0.0 < S <= 0.30).
    Marginal,
    /// Thermal runaway condition reached (S <= 0.0, cooling cannot dissipate leakage escalation).
    Runaway,
}

/// Result of self-consistent electro-thermal solving.
#[derive(Debug, Clone)]
pub struct EquilibriumResult {
    /// Equilibrium junction temperature in Kelvin.
    pub junction_temp_k: f64,
    /// Equilibrium junction temperature in Celsius.
    pub junction_temp_c: f64,
    /// Dynamic power dissipation (Watts).
    pub dynamic_power_w: f64,
    /// Temperature-dependent leakage power dissipation (Watts).
    pub leakage_power_w: f64,
    /// Total power dissipation (Watts).
    pub total_power_w: f64,
    /// Leakage percentage of total power (%).
    pub leakage_percentage: f64,
    /// Stability factor S = 1 - R_th * d(P_leak)/dT.
    pub stability_factor: f64,
    /// Thermal stability classification.
    pub status: ThermalStabilityStatus,
    /// Number of Newton-Raphson iterations to converge.
    pub iterations: usize,
    /// Convergence achieved within tolerance.
    pub converged: bool,
}

/// Bifurcation sweep curve containing heat generation and dissipation curves.
#[derive(Debug, Clone)]
pub struct BifurcationCurve {
    /// Temperature axis in Celsius.
    pub temperatures_c: Vec<f64>,
    /// Total heat generation curve P_gen(T) in Watts.
    pub heat_generation_w: Vec<f64>,
    /// Heat dissipation line (T - T_amb) / R_th in Watts.
    pub heat_dissipation_w: Vec<f64>,
    /// Leakage power component in Watts.
    pub leakage_component_w: Vec<f64>,
    /// Stability factor S(T) across temperatures.
    pub stability_factor: Vec<f64>,
    /// Critical bifurcation temperature in Celsius where d(P_leak)/dT = 1/R_th.
    pub bifurcation_temp_c: Option<f64>,
}

/// Evaluates temperature-dependent leakage current and power.
pub fn calculate_leakage_current(params: &LeakageModelParams, temp_k: f64, v_dd: f64) -> f64 {
    let t = temp_k.max(100.0);
    let v_t = (K_BOLTZMANN * t) / Q_ELEM;
    let v_t0 = (K_BOLTZMANN * params.t0_kelvin) / Q_ELEM;

    // Threshold voltage degradation with temperature: V_th(T) = V_th0 - alpha * (T - T0)
    let delta_t = t - params.t0_kelvin;
    let v_th_t = params.v_th0 - params.alpha_vth * delta_t;

    // Subthreshold current exponential term
    let exp_num = -v_th_t / (params.m_ideality * v_t);
    let exp_den = -params.v_th0 / (params.m_ideality * v_t0);
    let temp_ratio = t / params.t0_kelvin;

    let subthreshold_factor = (temp_ratio * temp_ratio) * (exp_num - exp_den).exp();
    let drain_voltage_factor = (1.0 - (-v_dd / v_t).exp()).max(0.0).min(1.0);

    let i_sub = params.i_sub0 * subthreshold_factor * drain_voltage_factor;

    // Gate oxide / BTBT leakage: mild temperature dependence
    let i_gate = params.gate_leakage_base * (1.0 + params.gate_temp_coeff * delta_t).max(0.1);

    i_sub + i_gate
}

/// Evaluates leakage power P_leak = I_leak * V_dd.
pub fn calculate_leakage_power(params: &LeakageModelParams, temp_k: f64, v_dd: f64) -> f64 {
    calculate_leakage_current(params, temp_k, v_dd) * v_dd
}

/// Evaluates dynamic power P_dyn = activity * C_eff * V_dd^2 * frequency.
pub fn calculate_dynamic_power(
    params: &DynamicPowerParams,
    v_dd: f64,
    frequency_hz: f64,
    clock_gating_duty: f64,
) -> f64 {
    let duty = clock_gating_duty.clamp(0.05, 1.0);
    params.activity_factor * duty * params.effective_capacitance * v_dd * v_dd * frequency_hz
}

/// Evaluates numerical derivative d(P_leak)/dT at temperature T.
pub fn calculate_leakage_temp_derivative(params: &LeakageModelParams, temp_k: f64, v_dd: f64) -> f64 {
    let dt = 0.5; // Kelvin
    let p_plus = calculate_leakage_power(params, temp_k + dt, v_dd);
    let p_minus = calculate_leakage_power(params, temp_k - dt, v_dd);
    (p_plus - p_minus) / (2.0 * dt)
}

/// Solves for self-consistent electro-thermal equilibrium temperature T_eq via damped Newton-Raphson.
pub fn solve_thermal_equilibrium(
    leak_params: &LeakageModelParams,
    dyn_params: &DynamicPowerParams,
    v_dd: f64,
    frequency_hz: f64,
    clock_gating_duty: f64,
    ambient_temp_c: f64,
    total_r_th_kw: f64,
) -> EquilibriumResult {
    let t_amb_k = ambient_temp_c + T_ZERO_CELSIUS_K;
    let p_dyn = calculate_dynamic_power(dyn_params, v_dd, frequency_hz, clock_gating_duty);

    let max_iterations = 60;
    let tolerance = 1e-4; // Kelvin
    let t_runaway_trip_k = t_amb_k + 150.0; // Over-temperature runaway threshold (150 K above ambient)

    let mut t_k = t_amb_k + total_r_th_kw * p_dyn; // Initial guess using cold dynamic power
    let mut converged = false;
    let mut iter_count = 0;

    for iter in 0..max_iterations {
        iter_count = iter + 1;
        let p_leak = calculate_leakage_power(leak_params, t_k, v_dd);
        let p_tot = p_dyn + p_leak;

        // Residual: g(T) = T - T_amb - R_th * P_tot(T)
        let residual = t_k - t_amb_k - total_r_th_kw * p_tot;

        if residual.abs() < tolerance {
            converged = true;
            break;
        }

        // Derivative: g'(T) = 1 - R_th * d(P_leak)/dT
        let d_pleak_dt = calculate_leakage_temp_derivative(leak_params, t_k, v_dd);
        let g_prime = 1.0 - total_r_th_kw * d_pleak_dt;

        // If derivative is zero or negative, runaway bifurcation occurs
        if g_prime <= 0.01 || t_k > t_runaway_trip_k {
            t_k = t_runaway_trip_k;
            converged = false;
            break;
        }

        // Damped Newton step
        let step = residual / g_prime;
        let damping = 0.75;
        t_k -= damping * step;

        if t_k < t_amb_k {
            t_k = t_amb_k;
        }
    }

    let p_leak_final = calculate_leakage_power(leak_params, t_k, v_dd);
    let p_tot_final = p_dyn + p_leak_final;
    let d_pleak_dt = calculate_leakage_temp_derivative(leak_params, t_k, v_dd);
    let stability_factor = 1.0 - total_r_th_kw * d_pleak_dt;

    let status = if !converged || stability_factor <= 0.0 || t_k >= t_runaway_trip_k {
        ThermalStabilityStatus::Runaway
    } else if stability_factor <= 0.25 {
        ThermalStabilityStatus::Marginal
    } else {
        ThermalStabilityStatus::Stable
    };

    let leakage_pct = if p_tot_final > 0.0 {
        (p_leak_final / p_tot_final) * 100.0
    } else {
        0.0
    };

    EquilibriumResult {
        junction_temp_k: t_k,
        junction_temp_c: t_k - T_ZERO_CELSIUS_K,
        dynamic_power_w: p_dyn,
        leakage_power_w: p_leak_final,
        total_power_w: p_tot_final,
        leakage_percentage: leakage_pct,
        stability_factor,
        status,
        iterations: iter_count,
        converged,
    }
}

/// Generates sweep curves for thermal runaway bifurcation analysis.
pub fn generate_bifurcation_curve(
    leak_params: &LeakageModelParams,
    dyn_params: &DynamicPowerParams,
    v_dd: f64,
    frequency_hz: f64,
    clock_gating_duty: f64,
    ambient_temp_c: f64,
    total_r_th_kw: f64,
    num_points: usize,
) -> BifurcationCurve {
    let t_amb_k = ambient_temp_c + T_ZERO_CELSIUS_K;
    let t_start_c = ambient_temp_c;
    let t_end_c = ambient_temp_c + 120.0;
    let step_c = (t_end_c - t_start_c) / (num_points.max(10) - 1) as f64;

    let p_dyn = calculate_dynamic_power(dyn_params, v_dd, frequency_hz, clock_gating_duty);

    let mut temps_c = Vec::with_capacity(num_points);
    let mut p_gen = Vec::with_capacity(num_points);
    let mut p_diss = Vec::with_capacity(num_points);
    let mut p_leak_vec = Vec::with_capacity(num_points);
    let mut s_vec = Vec::with_capacity(num_points);
    let mut bif_temp_c = None;

    for i in 0..num_points {
        let tc = t_start_c + (i as f64) * step_c;
        let tk = tc + T_ZERO_CELSIUS_K;
        let pleak = calculate_leakage_power(leak_params, tk, v_dd);
        let ptot = p_dyn + pleak;
        let pdiss = (tk - t_amb_k) / total_r_th_kw.max(0.001);

        let d_pleak_dt = calculate_leakage_temp_derivative(leak_params, tk, v_dd);
        let s = 1.0 - total_r_th_kw * d_pleak_dt;

        if s <= 0.0 && bif_temp_c.is_none() {
            bif_temp_c = Some(tc);
        }

        temps_c.push(tc);
        p_gen.push(ptot);
        p_diss.push(pdiss);
        p_leak_vec.push(pleak);
        s_vec.push(s);
    }

    BifurcationCurve {
        temperatures_c: temps_c,
        heat_generation_w: p_gen,
        heat_dissipation_w: p_diss,
        leakage_component_w: p_leak_vec,
        stability_factor: s_vec,
        bifurcation_temp_c: bif_temp_c,
    }
}
