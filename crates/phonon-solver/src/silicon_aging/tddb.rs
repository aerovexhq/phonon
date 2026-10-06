#![deny(unsafe_code)]

//! Time-Dependent Dielectric Breakdown (TDDB) Reliability Engine.
//!
//! Models gate oxide percolation defect generation, Weibull statistical failure distributions,
//! full-chip area Poisson scaling (A_chip / A_0), Failure-in-Time (FIT) rates, and pre-breakdown leakage escalation.

use crate::silicon_aging::bti::K_BOLTZMANN_EV_PER_K;

/// Number of hours in one year (365.25 days * 24 hours).
pub const HOURS_PER_YEAR: f64 = 8766.0;

/// Number of seconds in one year.
pub const SECONDS_PER_YEAR: f64 = HOURS_PER_YEAR * 3600.0;

/// Physical and statistical parameters for gate dielectric TDDB.
#[derive(Debug, Clone)]
pub struct TddbParams {
    /// Characteristic scale prefactor A_tddb in seconds.
    pub a_tddb_sec: f64,
    /// Electric field acceleration parameter gamma_e in cm/MV.
    pub gamma_e_cm_per_mv: f64,
    /// Defect generation thermal activation energy in eV (typically 0.70 to 0.85 eV).
    pub e_a_ev: f64,
    /// Weibull shape parameter (slope) beta (typically 1.2 to 1.6 for thin high-k oxides).
    pub weibull_beta: f64,
    /// Reference test structure gate area A_0 in um^2.
    pub reference_area_um2: f64,
    /// Full chip active gate area in mm^2 (e.g. 50 mm^2 = 5.0e7 um^2).
    pub chip_gate_area_mm2: f64,
    /// Equivalent oxide thickness (EOT) in nm.
    pub eot_nm: f64,
    /// Baseline fresh gate leakage current density in nA/um^2.
    pub initial_leakage_density_na_per_um2: f64,
    /// Leakage acceleration exponent prior to hard breakdown.
    pub leakage_acceleration_exponent: f64,
}

impl Default for TddbParams {
    fn default() -> Self {
        Self {
            a_tddb_sec: 2.2e15,
            gamma_e_cm_per_mv: 3.4,
            e_a_ev: 0.78,
            weibull_beta: 1.42,
            reference_area_um2: 10.0,
            chip_gate_area_mm2: 45.0,
            eot_nm: 1.15,
            initial_leakage_density_na_per_um2: 0.05,
            leakage_acceleration_exponent: 1.65,
        }
    }
}

/// Evaluates characteristic time-to-breakdown eta (63.2% failure time in seconds) for a given area.
pub fn calculate_t63_eta_sec(
    params: &TddbParams,
    v_dd: f64,
    temp_k: f64,
    target_area_um2: f64,
) -> f64 {
    // Electric field across gate dielectric in MV/cm
    let e_ox_mv_per_cm = (v_dd / params.eot_nm) * 10.0;

    // Field acceleration factor exp(-gamma_e * E_ox)
    let field_factor = (-params.gamma_e_cm_per_mv * e_ox_mv_per_cm).exp();

    // Arrhenius temperature activation exp(E_a / (k_B * T))
    // Note: higher temperature REDUCES lifetime (increases rate), so lifetime scales with exp(E_a / (k_B * T))
    let arrhenius = (params.e_a_ev / (K_BOLTZMANN_EV_PER_K * temp_k)).exp();

    let t63_ref = params.a_tddb_sec * field_factor * arrhenius;

    // Poisson area scaling: eta(A) = eta(A_0) * (A_0 / A)^(1 / beta)
    let area_ratio = (params.reference_area_um2 / target_area_um2.max(1.0e-3)).max(1.0e-12);
    t63_ref * area_ratio.powf(1.0 / params.weibull_beta)
}

/// Evaluates cumulative failure probability F(t) = 1 - exp(-(t / eta)^beta).
pub fn calculate_weibull_failure_probability(
    time_sec: f64,
    eta_sec: f64,
    weibull_beta: f64,
) -> f64 {
    if time_sec <= 0.0 {
        return 0.0;
    }
    let ratio = (time_sec / eta_sec.max(1.0e-6)).max(0.0);
    let exponent = -ratio.powf(weibull_beta);
    // Use exp_m1 for numerical precision when exponent is close to 0
    let f = -exponent.exp_m1();
    f.clamp(0.0, 1.0)
}

/// Evaluates linearized Weibull plot coordinate: W = ln(-ln(1 - F(t))).
pub fn calculate_weibull_plot_w(failure_prob: f64) -> f64 {
    let f_clamped = failure_prob.clamp(1.0e-10, 0.99999);
    // ln(-ln(1 - F))
    (-(-(1.0 - f_clamped).ln()).ln()).clamp(-15.0, 5.0)
}

/// Evaluates chip-level Failure-in-Time (FIT, failures per 10^9 operating hours) over mission lifespan.
pub fn calculate_fit_rate(failure_prob: f64, mission_hours: f64) -> f64 {
    if mission_hours <= 0.0 || failure_prob <= 0.0 {
        return 0.0;
    }
    let f_clamped = failure_prob.clamp(1.0e-12, 0.9999);
    let lambda_per_hour = -(1.0 - f_clamped).ln() / mission_hours;
    (lambda_per_hour * 1.0e9).max(0.0)
}

/// Evaluates gate leakage current escalation density in nA/um^2 as oxide degrades over time.
pub fn calculate_progressive_gate_leakage_density(
    params: &TddbParams,
    time_sec: f64,
    eta_sec: f64,
) -> f64 {
    if time_sec <= 0.0 {
        return params.initial_leakage_density_na_per_um2;
    }
    let wear_fraction = (time_sec / eta_sec.max(1.0e-6)).clamp(0.0, 0.98);
    let escalation = 1.0 + 8.5 * wear_fraction.powf(params.leakage_acceleration_exponent);
    params.initial_leakage_density_na_per_um2 * escalation
}

/// Computes Weibull reliability curve points across time from 1 day to 15 years.
pub fn generate_weibull_reliability_curve(
    params: &TddbParams,
    v_dd: f64,
    temp_k: f64,
    times_sec: &[f64],
) -> (Vec<f64>, Vec<f64>) {
    let chip_area_um2 = params.chip_gate_area_mm2 * 1.0e6;
    let eta_chip = calculate_t63_eta_sec(params, v_dd, temp_k, chip_area_um2);

    let mut failure_probs = Vec::with_capacity(times_sec.len());
    let mut weibull_ws = Vec::with_capacity(times_sec.len());

    for &t in times_sec {
        let f = calculate_weibull_failure_probability(t, eta_chip, params.weibull_beta);
        let w = calculate_weibull_plot_w(f);
        failure_probs.push(f);
        weibull_ws.push(w);
    }

    (failure_probs, weibull_ws)
}
