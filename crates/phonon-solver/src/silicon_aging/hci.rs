#![deny(unsafe_code)]

//! Hot Carrier Injection (HCI) Physics Engine.
//!
//! Models lateral drain electric field acceleration, impact ionization, substrate current (I_sub),
//! gate interface trap generation, threshold voltage shift, transconductance loss, and subthreshold swing degradation.

use crate::silicon_aging::bti::K_BOLTZMANN_EV_PER_K;

/// Elementary charge in Coulombs.
pub const Q_ELECTRON_COULOMBS: f64 = 1.602176634e-19;

/// Vacuum permittivity in F/m.
pub const EPSILON_0_F_PER_M: f64 = 8.8541878128e-12;

/// Relative permittivity of SiO2 / high-k dielectric stack.
pub const EPSILON_OX_REL: f64 = 3.9;

/// Physical parameters for Hot Carrier Injection (HCI) modeling.
#[derive(Debug, Clone)]
pub struct HciParams {
    /// HCI aging prefactor A_hci (V / (A/m)^m / s^n).
    pub a_hci: f64,
    /// Substrate current acceleration exponent m_hci (typically 0.6 to 1.0).
    pub m_hci: f64,
    /// Time growth exponent n_hci (typically 0.40 to 0.50).
    pub n_hci: f64,
    /// Impact ionization threshold energy phi_i in eV (typically 1.3 eV for Si).
    pub phi_i_ev: f64,
    /// Carrier optical phonon mean free path at 300K in nm (typically 6.0 to 9.0 nm).
    pub lambda_0_nm: f64,
    /// Drain pinch-off length l_d in nm (typically 8.0 to 15.0 nm).
    pub pinch_off_length_nm: f64,
    /// Drawn channel length L_g in nm.
    pub channel_length_nm: f64,
    /// Total channel width W in um.
    pub channel_width_um: f64,
    /// Saturation drain voltage V_dsat in V.
    pub v_dsat_v: f64,
    /// Substrate current empirical coupling coefficient C_1.
    pub c1_sub: f64,
    /// Transconductance degradation coefficient A_gm (1/V).
    pub a_gm: f64,
}

impl Default for HciParams {
    fn default() -> Self {
        Self {
            a_hci: 2.5e-6,
            m_hci: 0.78,
            n_hci: 0.46,
            phi_i_ev: 1.30,
            lambda_0_nm: 7.2,
            pinch_off_length_nm: 10.5,
            channel_length_nm: 16.0,
            channel_width_um: 1.0,
            v_dsat_v: 0.38,
            c1_sub: 1.85,
            a_gm: 1.75,
        }
    }
}

/// Evaluates maximum lateral electric field near drain edge in V/nm.
pub fn calculate_max_lateral_field(params: &HciParams, v_ds: f64) -> f64 {
    let v_pinch = (v_ds - params.v_dsat_v).max(0.01);
    v_pinch / params.pinch_off_length_nm
}

/// Evaluates temperature-dependent electron mean free path lambda(T) in nm.
/// Note: Increased acoustic/optical phonon scattering at elevated temperatures reduces lambda,
/// causing impact ionization to be stronger at lower/moderate temperatures.
pub fn calculate_mean_free_path_nm(params: &HciParams, temp_k: f64) -> f64 {
    let t_norm = (temp_k / 300.0).max(0.5);
    params.lambda_0_nm / t_norm
}

/// Evaluates substrate current I_sub in Amperes using the lucky-electron model.
pub fn calculate_substrate_current(
    params: &HciParams,
    v_ds: f64,
    v_gs: f64,
    temp_k: f64,
) -> f64 {
    if v_ds <= params.v_dsat_v {
        return 1.0e-12; // Negligible substrate current below saturation
    }

    let e_m = calculate_max_lateral_field(params, v_ds); // V/nm
    let lambda_nm = calculate_mean_free_path_nm(params, temp_k); // nm

    // Exponent: -phi_i / (q * lambda * E_m)
    // In eV and V: phi_i_ev / (lambda_nm * e_m)
    let exponent = -params.phi_i_ev / (lambda_nm * e_m).max(1.0e-3);
    let prob_ionization = exponent.exp().clamp(1.0e-15, 1.0);

    // Approximate channel on-current I_ds in A
    let v_overdrive = (v_gs - 0.32).max(0.05);
    let i_ds = 1.2e-3 * params.channel_width_um * (v_overdrive / 0.53).powi(2);

    params.c1_sub * i_ds * prob_ionization
}

/// Evaluates HCI threshold voltage shift (Delta V_th in Volts) over time.
pub fn calculate_hci_vth_shift(
    params: &HciParams,
    v_ds: f64,
    v_gs: f64,
    temp_k: f64,
    time_sec: f64,
) -> f64 {
    if time_sec <= 0.0 {
        return 0.0;
    }

    let i_sub = calculate_substrate_current(params, v_ds, v_gs, temp_k);
    let linear_i_sub_density = (i_sub / (params.channel_width_um * 1.0e-6)).max(1.0e-9);

    let sub_factor = linear_i_sub_density.powf(params.m_hci);
    let time_factor = time_sec.powf(params.n_hci);

    // HCI duty cycle / switching factor for digital logic (HCI occurs during switching transitions)
    let dynamic_switching_factor = 0.12;

    (params.a_hci * sub_factor * time_factor * dynamic_switching_factor).clamp(0.0, 0.25)
}

/// Evaluates transconductance degradation percentage: Delta g_m / g_m0.
pub fn calculate_hci_gm_degradation(vth_shift_v: f64, params: &HciParams) -> f64 {
    (params.a_gm * vth_shift_v * 100.0).clamp(0.0, 50.0)
}

/// Evaluates subthreshold swing degradation Delta SS in mV/decade due to generated interface traps.
pub fn calculate_subthreshold_swing_degradation(
    vth_shift_v: f64,
    temp_k: f64,
    eot_nm: f64,
) -> f64 {
    // Thermal voltage in mV: (k_B * T / q) * 1000
    let v_t_mv = (K_BOLTZMANN_EV_PER_K * temp_k) * 1000.0;
    let ln10 = 2.302585093;

    // Approximate interface trap charge fraction relative to gate capacitance
    // Delta SS approx ln(10) * (k_B * T / q) * (q * Delta N_it / C_ox)
    // Normalized to threshold shift:
    let delta_ss_mv_per_dec = ln10 * v_t_mv * (vth_shift_v / 0.50) * (eot_nm / 1.15);
    delta_ss_mv_per_dec.clamp(0.0, 45.0)
}

/// Generates time-series trajectories for HCI threshold shift across evaluation timestamps.
pub fn generate_hci_trajectories(
    params: &HciParams,
    v_ds: f64,
    v_gs: f64,
    temp_k: f64,
    times_sec: &[f64],
) -> Vec<f64> {
    times_sec
        .iter()
        .map(|&t| calculate_hci_vth_shift(params, v_ds, v_gs, temp_k, t))
        .collect()
}
