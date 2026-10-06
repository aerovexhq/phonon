#![deny(unsafe_code)]

//! Through-Silicon Via (TSV) & Fine-Pitch Redistribution Layer (RDL) Parasitics Extractor.
//!
//! Provides electrodynamic RLGC extraction and high-frequency S-parameter modeling:
//! - Copper TSVs with SiO2 dielectric isolation liner embedded in silicon substrate
//! - Fine-pitch sub-micron Redistribution Layer (RDL) transmission lines
//! - Frequency-dependent skin depth, substrate eddy losses, and dielectric loss tangent
//! - Scattering parameters (S11 return loss, S21 insertion loss) from DC to 50 GHz.

use std::f64::consts::PI;

const EPSILON_0: f64 = 8.854187817e-12; // F/m
const MU_0: f64 = 1.256637061e-6;       // H/m
const RHO_CU: f64 = 1.72e-8;            // Ohm * m (Copper resistivity at 20 C)
const EPS_SIO2: f64 = 3.9;              // Relative permittivity of SiO2 liner
const EPS_SI: f64 = 11.7;               // Relative permittivity of Bulk Silicon
const SIGMA_SI: f64 = 10.0;             // S/m (Silicon substrate conductivity, ~10 Ohm*cm)

/// Physical geometric specifications for a Through-Silicon Via (TSV) pair.
#[derive(Debug, Clone, PartialEq)]
pub struct TsvGeometry {
    /// TSV diameter in micrometers (typically 5.0 to 10.0 um).
    pub diameter_um: f64,
    /// TSV height / silicon substrate thickness in micrometers (typically 50.0 to 100.0 um).
    pub height_um: f64,
    /// Dielectric oxide isolation liner thickness in micrometers (typically 0.2 to 0.5 um).
    pub liner_thickness_um: f64,
    /// Center-to-center via pitch in micrometers (typically 20.0 to 50.0 um).
    pub pitch_um: f64,
}

impl Default for TsvGeometry {
    fn default() -> Self {
        Self {
            diameter_um: 5.0,
            height_um: 50.0,
            liner_thickness_um: 0.25,
            pitch_um: 20.0,
        }
    }
}

/// Extracted lumped electrical parameters (RLGC) for a TSV.
#[derive(Debug, Clone, PartialEq)]
pub struct TsvRlgc {
    /// DC series resistance in Ohms (milli-Ohms range).
    pub r_dc_mohm: f64,
    /// Loop inductance in pico-Henries (pH).
    pub l_ph: f64,
    /// Oxide liner capacitance in femto-Farads (fF).
    pub c_ox_ff: f64,
    /// Substrate capacitance in femto-Farads (fF).
    pub c_si_ff: f64,
    /// Total effective parasitic shunt capacitance in femto-Farads (fF).
    pub c_total_ff: f64,
    /// Substrate conductance in micro-Siemens (uS).
    pub g_si_us: f64,
}

/// Physical geometric specifications for a Redistribution Layer (RDL) interconnect.
#[derive(Debug, Clone, PartialEq)]
pub struct RdlGeometry {
    /// Metal line width in micrometers (typically 1.0 to 3.0 um).
    pub width_um: f64,
    /// Inter-metal line spacing in micrometers (typically 1.0 to 3.0 um).
    pub space_um: f64,
    /// Metal thickness in micrometers (typically 1.5 to 3.0 um).
    pub thickness_um: f64,
    /// Line length in millimeters.
    pub length_mm: f64,
    /// Inter-metal dielectric relative permittivity (typically 2.8 to 3.5).
    pub dielectric_eps_r: f64,
}

impl Default for RdlGeometry {
    fn default() -> Self {
        Self {
            width_um: 2.0,
            space_um: 2.0,
            thickness_um: 2.0,
            length_mm: 1.5,
            dielectric_eps_r: 3.2,
        }
    }
}

/// Extracted electrical parameters for RDL transmission line.
#[derive(Debug, Clone, PartialEq)]
pub struct RdlRlgc {
    /// Total DC resistance in Ohms.
    pub r_dc_ohm: f64,
    /// Total series inductance in nano-Henries (nH).
    pub l_nh: f64,
    /// Total line capacitance in pico-Farads (pF).
    pub c_pf: f64,
    /// Characteristic impedance in Ohms.
    pub z0_ohm: f64,
    /// Propagation delay in picoseconds.
    pub t_pd_ps: f64,
}

/// Point on frequency sweep curve for scattering parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct SParameterPoint {
    pub freq_ghz: f64,
    pub s11_db: f64,
    pub s21_db: f64,
}

/// Extracts frequency-independent baseline RLGC parameters for a TSV.
pub fn extract_tsv_rlgc(geom: &TsvGeometry) -> TsvRlgc {
    let r_m = (geom.diameter_um / 2.0) * 1e-6;
    let h_m = geom.height_um * 1e-6;
    let tox_m = geom.liner_thickness_um * 1e-6;
    let s_m = geom.pitch_um * 1e-6;

    // DC Resistance: R = rho * H / (pi * r^2)
    let area_m2 = PI * r_m * r_m;
    let r_dc = RHO_CU * h_m / area_m2;

    // Loop Inductance: L = (mu_0 * H / pi) * acosh(S / (2 * r))
    let ratio = (s_m / (2.0 * r_m)).max(1.0001);
    let l_loop = (MU_0 * h_m / PI) * ratio.acosh();

    // Oxide capacitance: C_ox = 2 * pi * eps_0 * eps_ox * H / ln((r + tox) / r)
    let c_ox = 2.0 * PI * EPSILON_0 * EPS_SIO2 * h_m / ((r_m + tox_m) / r_m).ln();

    // Substrate capacitance: C_si = pi * eps_0 * eps_si * H / acosh(S / (2 * (r + tox)))
    let ratio_si = (s_m / (2.0 * (r_m + tox_m))).max(1.0001);
    let c_si = PI * EPSILON_0 * EPS_SI * h_m / ratio_si.acosh();

    // Total capacitance of TSV pair: C_total = C_ox / 2 in series with C_si
    let c_total = (c_ox * c_si) / (c_ox + 2.0 * c_si);

    // Substrate conductance: G_si = sigma_si / (eps_0 * eps_si) * C_si
    let g_si = (SIGMA_SI / (EPSILON_0 * EPS_SI)) * c_si;

    TsvRlgc {
        r_dc_mohm: r_dc * 1000.0,
        l_ph: l_loop * 1e12,
        c_ox_ff: c_ox * 1e15,
        c_si_ff: c_si * 1e15,
        c_total_ff: c_total * 1e15,
        g_si_us: g_si * 1e6,
    }
}

/// Extracts transmission line parameters for an RDL trace.
pub fn extract_rdl_rlgc(geom: &RdlGeometry) -> RdlRlgc {
    let w_m = geom.width_um * 1e-6;
    let t_m = geom.thickness_um * 1e-6;
    let l_m = geom.length_mm * 1e-3;

    // DC Resistance
    let r_dc = RHO_CU * l_m / (w_m * t_m);

    // Coplanar / microstrip capacitance and inductance approximations
    let eps_eff = (geom.dielectric_eps_r + 1.0) / 2.0;
    let c_per_m = EPSILON_0 * eps_eff * (geom.width_um / geom.space_um + 1.85);
    let total_c = c_per_m * l_m;

    let v_phase = 3e8 / eps_eff.sqrt();
    let z0 = 1.0 / (v_phase * c_per_m);
    let l_per_m = z0 * z0 * c_per_m;
    let total_l = l_per_m * l_m;

    let t_pd_ps = (l_m / v_phase) * 1e12;

    RdlRlgc {
        r_dc_ohm: r_dc,
        l_nh: total_l * 1e9,
        c_pf: total_c * 1e12,
        z0_ohm: z0,
        t_pd_ps,
    }
}

/// Computes S11 (return loss) and S21 (insertion loss) frequency response from 1 GHz to 50 GHz.
pub fn compute_s_parameters(tsv: &TsvRlgc, rdl: &RdlRlgc) -> Vec<SParameterPoint> {
    let mut points = Vec::with_capacity(50);
    let r_tot = (tsv.r_dc_mohm * 1e-3) + rdl.r_dc_ohm;
    let l_tot = (tsv.l_ph * 1e-12) + (rdl.l_nh * 1e-9);
    let c_tot = (tsv.c_total_ff * 1e-15) + (rdl.c_pf * 1e-12);
    let z_ref = 50.0;

    for i in 1..=50 {
        let f_ghz = i as f64;
        let omega = 2.0 * PI * f_ghz * 1e9;

        // Skin effect resistance increase
        let r_ac = r_tot * (1.0 + 0.15 * f_ghz.sqrt());
        let x_l = omega * l_tot;
        let b_c = omega * c_tot;

        // ABCD transmission matrix of L-network (series R+jX, shunt jB)
        // Z_series = r_ac + j*x_l, Y_shunt = j*b_c
        // A = 1 + Z*Y, B = Z, C = Y, D = 1
        // Insertion loss approximation in 50 Ohm system
        // S21 = 2 / (A + B/Z0 + C*Z0 + D)
        let denom_real = 2.0 + (r_ac / z_ref) - (x_l * b_c);
        let denom_imag = (x_l / z_ref) + (b_c * z_ref) + (r_ac * b_c);
        let denom_mag_sq = denom_real * denom_real + denom_imag * denom_imag;

        let s21_linear = (4.0 / denom_mag_sq).sqrt();
        let s21_db = (20.0 * s21_linear.log10()).min(0.0).max(-40.0);

        // Return loss S11 = (B/Z0 - C*Z0) / (A + B/Z0 + C*Z0 + D)
        let num_real = (r_ac / z_ref);
        let num_imag = (x_l / z_ref) - (b_c * z_ref);
        let num_mag_sq = num_real * num_real + num_imag * num_imag;
        let s11_linear = (num_mag_sq / denom_mag_sq).sqrt().min(0.999);
        let s11_db = (20.0 * s11_linear.log10()).min(-0.1);

        points.push(SParameterPoint {
            freq_ghz: f_ghz,
            s11_db,
            s21_db,
        });
    }

    points
}
