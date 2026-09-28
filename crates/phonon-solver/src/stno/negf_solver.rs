//! Non-Equilibrium Green's Function (NEGF) transport solver:
//! transmission spectra, Kondo resonance peak splitting, differential conductance,
//! and spin-polarized tunnel current.

use phonon_models::stno::NegfMolecularJunctionParams;

/// Result of an NEGF molecular transport evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct NegfTransportResult {
    /// Zero-bias linear conductance G_0 in units of quantum conductance G_0 = 2 e^2 / h.
    pub zero_bias_conductance_2e2_over_h: f64,
    /// Kondo resonance peak width in eV (~ 2 * k_B * T_K).
    pub kondo_resonance_fwhm_ev: f64,
    /// Measured Zeeman splitting of Kondo peak in eV.
    pub zeeman_splitting_ev: f64,
    /// Tunneling magnetoresistance ratio (TMR).
    pub tmr_ratio: f64,
    /// Total current I in microamperes at bias voltage V.
    pub current_microamps: f64,
    /// Differential conductance dI/dV in micro-Siemens.
    pub differential_conductance_microsiemens: f64,
}

/// NEGF transport solver for single-molecule junctions.
pub struct NegfMolecularSolver;

impl NegfMolecularSolver {
    /// Evaluates spin-resolved transmission T(E) at energy E (in eV) under external magnetic field B (Tesla).
    pub fn transmission_at_energy(
        params: &NegfMolecularJunctionParams,
        energy_ev: f64,
        b_field_tesla: f64,
    ) -> (f64, f64) {
        let (gamma_l_up, gamma_l_dn) = params.spin_lead_couplings();
        let gamma_r = params.gamma_right_ev;

        let delta_z = 0.5 * params.kondo_zeeman_splitting_ev(b_field_tesla, 2.0);
        let tk_k = params.kondo_temperature_k();
        let tk_ev = (tk_k * 1.380649e-23) / 1.602176634e-19;
        let gamma_kondo = 2.0 * tk_ev.max(1e-4);

        // Spin up transmission: Abrikosov-Suhl resonance at +delta_z plus single-particle background
        let denom_up = (energy_ev - delta_z).powi(2) + (0.5 * gamma_kondo).powi(2);
        let t_kondo_up = (0.5 * gamma_kondo).powi(2) / denom_up;
        let t_up = (4.0 * gamma_l_up * gamma_r) / (gamma_l_up + gamma_r).powi(2) * t_kondo_up;

        // Spin down transmission: resonance at -delta_z
        let denom_dn = (energy_ev + delta_z).powi(2) + (0.5 * gamma_kondo).powi(2);
        let t_kondo_dn = (0.5 * gamma_kondo).powi(2) / denom_dn;
        let t_dn = (4.0 * gamma_l_dn * gamma_r) / (gamma_l_dn + gamma_r).powi(2) * t_kondo_dn;

        (t_up, t_dn)
    }

    /// Evaluates transport properties at specified bias voltage V_bias (Volts) and field B (Tesla).
    pub fn solve(
        params: &NegfMolecularJunctionParams,
        v_bias_volts: f64,
        b_field_tesla: f64,
    ) -> NegfTransportResult {
        let n_points = 200;
        let e_max = 0.05 + v_bias_volts.abs();
        let e_min = -e_max;
        let de = (e_max - e_min) / (n_points as f64 - 1.0);

        let mut total_current_a = 0.0;
        let e_charge: f64 = 1.602_176_634e-19;
        let h_planck: f64 = 6.626_070_15e-34;
        let g0_si = (2.0 * e_charge.powi(2)) / h_planck; // ~7.748e-5 S

        // Simpson / midpoint integration of Landauer current:
        // I = (e / h) \int [T_up(E) + T_dn(E)] * [f_L(E) - f_R(E)] dE
        for i in 0..n_points {
            let e = e_min + (i as f64 + 0.5) * de;
            let (t_up, t_dn) = Self::transmission_at_energy(params, e, b_field_tesla);
            let t_total = t_up + t_dn;

            let f_l = fermi_dirac(e - 0.5 * v_bias_volts, params.temperature_k);
            let f_r = fermi_dirac(e + 0.5 * v_bias_volts, params.temperature_k);

            let d_current = (e_charge / h_planck) * t_total * (f_l - f_r) * (de * e_charge);
            total_current_a += d_current;
        }

        // Zero-bias conductance
        let (t0_up, t0_dn) = Self::transmission_at_energy(params, 0.0, b_field_tesla);
        let g_zero_bias = 0.5 * (t0_up + t0_dn);

        // Differential conductance near V_bias
        let (t_v_up, t_v_dn) =
            Self::transmission_at_energy(params, 0.5 * v_bias_volts, b_field_tesla);
        let diff_g = 0.5 * (t_v_up + t_v_dn) * g0_si;

        let tk_k = params.kondo_temperature_k();
        let tk_ev = (tk_k * 1.380649e-23) / 1.602176634e-19;

        NegfTransportResult {
            zero_bias_conductance_2e2_over_h: g_zero_bias,
            kondo_resonance_fwhm_ev: 2.0 * tk_ev,
            zeeman_splitting_ev: params.kondo_zeeman_splitting_ev(b_field_tesla, 2.0),
            tmr_ratio: params.tunnel_magnetoresistance_ratio(),
            current_microamps: total_current_a * 1.0e6,
            differential_conductance_microsiemens: diff_g * 1.0e6,
        }
    }
}

fn fermi_dirac(energy_ev: f64, temperature_k: f64) -> f64 {
    let kb_ev = 8.617_333_262e-5;
    let arg = energy_ev / (kb_ev * temperature_k.max(0.1));
    if arg > 40.0 {
        0.0
    } else if arg < -40.0 {
        1.0
    } else {
        1.0 / (1.0 + arg.exp())
    }
}
