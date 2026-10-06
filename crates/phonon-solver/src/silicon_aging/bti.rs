#![deny(unsafe_code)]

//! Bias Temperature Instability (BTI) Physics Engine.
//!
//! Models Negative Bias Temperature Instability (NBTI) in pMOS and Positive Bias Temperature
//! Instability (PBTI) in nMOS using reaction-diffusion (R-D) kinetics, electric field acceleration,
//! Arrhenius temperature activation, and dynamic AC stress recovery.

/// Boltzmann constant in eV/K.
pub const K_BOLTZMANN_EV_PER_K: f64 = 8.617333262e-5;

/// Transistor polarity for BTI degradation evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransistorPolarity {
    /// Negative Bias Temperature Instability in pMOS (dominant mechanism).
    PmosNbti,
    /// Positive Bias Temperature Instability in nMOS (high-k metal gate electron trapping).
    NmosPbti,
}

/// Physical and empirical parameters for BTI reaction-diffusion model.
#[derive(Debug, Clone)]
pub struct BtiParams {
    /// Reaction-diffusion pre-factor A_bti (V / (MV/cm)^gamma / s^n).
    pub a_bti: f64,
    /// Electric field acceleration exponent gamma (typically 1.2 to 1.5).
    pub gamma: f64,
    /// Thermal activation energy in eV (typically 0.12 to 0.20 eV).
    pub e_a_ev: f64,
    /// Dispersive diffusion time exponent n (typically 1/6 ~ 0.1667 for H2 diffusion).
    pub time_exponent_n: f64,
    /// Baseline nominal threshold voltage in V.
    pub v_th0: f64,
    /// Equivalent oxide thickness (EOT) in nm.
    pub eot_nm: f64,
    /// Relative susceptibility factor between pMOS and nMOS (pMOS is typically ~2.5x higher).
    pub pmos_to_nmos_ratio: f64,
}

impl Default for BtiParams {
    fn default() -> Self {
        Self {
            a_bti: 0.024,
            gamma: 1.35,
            e_a_ev: 0.155,
            time_exponent_n: 1.0 / 6.0,
            v_th0: 0.32,
            eot_nm: 1.15,
            pmos_to_nmos_ratio: 2.65,
        }
    }
}

/// Calculates static DC threshold voltage shift (Delta V_th in Volts).
pub fn calculate_bti_dc_vth_shift(
    params: &BtiParams,
    v_dd: f64,
    temp_k: f64,
    time_sec: f64,
    polarity: TransistorPolarity,
) -> f64 {
    if time_sec <= 0.0 || temp_k <= 0.0 {
        return 0.0;
    }

    // Overdrive voltage across oxide
    let v_overdrive = (v_dd - params.v_th0).max(0.05);

    // Oxide electric field in MV/cm: (V_overdrive / eot_nm) * 10
    // E.g. (0.85 - 0.32) / 1.15 nm = 0.46 V/nm = 4.6 MV/cm
    let e_ox_mv_per_cm = (v_overdrive / params.eot_nm) * 10.0;
    let field_factor = e_ox_mv_per_cm.powf(params.gamma);

    // Arrhenius thermal acceleration
    let arrhenius = (-params.e_a_ev / (K_BOLTZMANN_EV_PER_K * temp_k)).exp();

    // Reaction-diffusion power-law time growth
    let time_factor = time_sec.powf(params.time_exponent_n);

    let base_shift = params.a_bti * field_factor * arrhenius * time_factor;

    match polarity {
        TransistorPolarity::PmosNbti => base_shift,
        TransistorPolarity::NmosPbti => base_shift / params.pmos_to_nmos_ratio,
    }
}

/// Calculates dynamic AC recovery degradation scaling factor beta_rec(duty_cycle).
/// Under periodic switching (duty cycle alpha in (0, 1]), unbonded hydrogen diffuses back
/// during the low phase, passivating interface traps and recovering 30% to 50% of shift.
pub fn calculate_bti_ac_recovery_factor(duty_cycle: f64, time_exponent_n: f64) -> f64 {
    let alpha = duty_cycle.clamp(0.01, 1.0);
    if alpha >= 0.999 {
        return 1.0; // Pure DC stress
    }

    // Standard reaction-diffusion AC duty cycle formulation:
    // beta_rec = alpha^n * (0.55 + 0.45 * alpha)
    let alpha_pow = alpha.powf(time_exponent_n);
    let recovery_damping = 0.55 + 0.45 * alpha;
    (alpha_pow * recovery_damping).clamp(0.1, 1.0)
}

/// Evaluates effective threshold voltage shift (Delta V_th in Volts) considering AC duty cycle.
pub fn calculate_bti_effective_vth_shift(
    params: &BtiParams,
    v_dd: f64,
    temp_k: f64,
    time_sec: f64,
    duty_cycle: f64,
    polarity: TransistorPolarity,
) -> f64 {
    let dc_shift = calculate_bti_dc_vth_shift(params, v_dd, temp_k, time_sec, polarity);
    let recovery = calculate_bti_ac_recovery_factor(duty_cycle, params.time_exponent_n);
    dc_shift * recovery
}

/// Calculates percentage saturation drive current degradation: Delta I_on / I_on0.
pub fn calculate_on_current_degradation(vth_shift_v: f64, v_dd: f64, v_th0: f64) -> f64 {
    let overdrive = (v_dd - v_th0).max(0.05);
    (vth_shift_v / overdrive * 100.0).clamp(0.0, 60.0)
}

/// Calculates percentage gate stage propagation delay penalty: Delta tau / tau_0.
pub fn calculate_stage_delay_penalty(vth_shift_v: f64, v_dd: f64, v_th0: f64) -> f64 {
    let overdrive = (v_dd - v_th0).max(0.05);
    // Alpha-power law delay sensitivity (alpha ~ 1.25 for advanced velocity-saturated CMOS)
    let alpha_v = 1.25;
    (alpha_v * vth_shift_v / overdrive * 100.0).clamp(0.0, 80.0)
}

/// Generates time-series trajectories for pMOS NBTI and nMOS PBTI across evaluation timestamps.
pub fn generate_bti_trajectories(
    params: &BtiParams,
    v_dd: f64,
    temp_k: f64,
    times_sec: &[f64],
    duty_cycle: f64,
) -> (Vec<f64>, Vec<f64>) {
    let mut pmos_shifts = Vec::with_capacity(times_sec.len());
    let mut nmos_shifts = Vec::with_capacity(times_sec.len());

    for &t in times_sec {
        let pmos = calculate_bti_effective_vth_shift(
            params,
            v_dd,
            temp_k,
            t,
            duty_cycle,
            TransistorPolarity::PmosNbti,
        );
        let nmos = calculate_bti_effective_vth_shift(
            params,
            v_dd,
            temp_k,
            t,
            duty_cycle,
            TransistorPolarity::NmosPbti,
        );
        pmos_shifts.push(pmos);
        nmos_shifts.push(nmos);
    }

    (pmos_shifts, nmos_shifts)
}
