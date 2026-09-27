//! Multi-Valued Logic (MVL) physical semiconductor devices.
//!
//! Provides physically formulated models for:
//! 1. Multi-threshold MOSFETs engineered via metal gate workfunction tuning (\(\Phi_m\)) and stepped oxides.
//! 2. Multi-peak resonant tunneling devices (RTD) exhibiting dual negative differential resistance
//!    regions and three stable operating states for compact ternary latches.
//! 3. Chirality-tuned Carbon Nanotube FETs (CNTFET) offering pristine diameter-controlled threshold voltages.

use crate::common::safe_exp;

/// Physical constants for semiconductor calculations.
const Q_E: f64 = 1.602_176_634e-19; // Elementary charge [C]
const K_B: f64 = 1.380_649e-23; // Boltzmann constant [J/K]
const EPS_0: f64 = 8.854_187_812_8e-12; // Vacuum permittivity [F/m]
const EPS_SI: f64 = 11.68 * EPS_0; // Silicon permittivity [F/m]
const EPS_OX_SIO2: f64 = 3.9 * EPS_0; // SiO2 permittivity [F/m]

/// Transistor threshold flavor for multi-valued logic synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MosfetFlavor {
    /// Standard NMOS switching at approximately +0.35 V
    StandardN,
    /// High-threshold NMOS switching only at elevated voltages (> 0.55 V, above half-supply)
    HighN,
    /// Low-threshold NMOS conducting at both intermediate (Vdd/2) and high voltages (> 0.15 V)
    LowN,
    /// Standard PMOS switching at approximately -0.35 V
    StandardP,
    /// High-magnitude threshold PMOS switching only near ground (< 0.35 V, |Vth| > 0.55 V)
    HighP,
    /// Low-magnitude threshold PMOS conducting at both intermediate and low voltages (< 0.75 V, |Vth| < 0.15 V)
    LowP,
}

impl MosfetFlavor {
    /// Nominal metal gate workfunction in eV corresponding to this flavor.
    pub fn nominal_workfunction_ev(&self) -> f64 {
        match self {
            MosfetFlavor::LowN => 4.15,
            MosfetFlavor::StandardN => 4.35,
            MosfetFlavor::HighN => 4.55,
            MosfetFlavor::LowP => 5.05,
            MosfetFlavor::StandardP => 4.85,
            MosfetFlavor::HighP => 4.65,
        }
    }

    /// True if device is an n-channel field effect transistor.
    pub fn is_n_channel(&self) -> bool {
        matches!(
            self,
            MosfetFlavor::StandardN | MosfetFlavor::HighN | MosfetFlavor::LowN
        )
    }
}

/// Physical parameters for a multi-threshold MOSFET device.
#[derive(Debug, Clone)]
pub struct MultiThresholdMosfetParams {
    /// Channel length [m]
    pub length: f64,
    /// Channel width [m]
    pub width: f64,
    /// Equivalent Oxide Thickness (EOT) [m]
    pub eot: f64,
    /// Substrate doping density [m^-3]
    pub n_substrate: f64,
    /// Gate workfunction [eV]
    pub workfunction_ev: f64,
    /// Low-field carrier mobility [m^2 / (V*s)]
    pub mobility: f64,
    /// Saturation injection velocity [m/s]
    pub v_sat: f64,
    /// Subthreshold swing factor
    pub subthreshold_factor: f64,
    /// Drain-Induced Barrier Lowering (DIBL) coefficient [V/V]
    pub dibl_eta: f64,
    /// Device flavor classification
    pub flavor: MosfetFlavor,
}

impl MultiThresholdMosfetParams {
    /// Creates a device parameter set customized for a specific flavor at a given supply voltage.
    pub fn from_flavor(flavor: MosfetFlavor, eot_m: f64, length_m: f64, width_m: f64) -> Self {
        let is_n = flavor.is_n_channel();
        let workfunction_ev = flavor.nominal_workfunction_ev();
        let mobility = if is_n { 0.045 } else { 0.018 }; // 450 cm^2/Vs vs 180 cm^2/Vs
        let v_sat = if is_n { 1.0e5 } else { 8.0e4 };

        Self {
            length: length_m,
            width: width_m,
            eot: eot_m,
            n_substrate: 1.0e24, // 1e18 cm^-3
            workfunction_ev,
            mobility,
            v_sat,
            subthreshold_factor: 1.15,
            dibl_eta: 0.06,
            flavor,
        }
    }
}

/// Output evaluation of a multi-threshold MOSFET.
#[derive(Debug, Clone, Copy)]
pub struct MosfetMvlEvaluation {
    /// Drain current [A] (positive into drain for NMOS, positive out of drain for PMOS)
    pub ids: f64,
    /// Transconductance gm = dIds / dVgs [S]
    pub gm: f64,
    /// Output conductance gds = dIds / dVds [S]
    pub gds: f64,
    /// Effective physical threshold voltage [V]
    pub v_th: f64,
}

/// Physically formulated Multi-Threshold MOSFET model.
#[derive(Debug, Clone)]
pub struct MultiThresholdMosfet {
    params: MultiThresholdMosfetParams,
    c_ox: f64,
    phi_f: f64,
    v_th0: f64,
}

impl MultiThresholdMosfet {
    /// Constructs a new MultiThresholdMosfet from physical parameters and operating temperature.
    pub fn new(params: MultiThresholdMosfetParams, temp_k: f64) -> Self {
        let t = temp_k.max(10.0);
        let v_t = K_B * t / Q_E;
        let (exp_eg, _) = safe_exp(-1.12 * Q_E / (2.0 * K_B * t));
        let n_i = 1.0e16 * (t / 300.0).powf(1.5) * exp_eg;
        let phi_f = (v_t * (params.n_substrate / n_i.max(1.0)).ln()).clamp(0.2, 0.6);

        let c_ox = EPS_OX_SIO2 / params.eot.max(0.5e-9);
        let q_dep = (4.0 * Q_E * EPS_SI * params.n_substrate * phi_f).sqrt();

        // Silicon electron affinity chi_si = 4.05 eV, bandgap Eg = 1.12 eV
        let chi_si = 4.05;
        let eg = 1.12;
        let phi_s = if params.flavor.is_n_channel() {
            chi_si + (eg / 2.0) + phi_f
        } else {
            chi_si + (eg / 2.0) - phi_f
        };

        // Flatband voltage V_fb = Phi_m - Phi_s
        let v_fb = params.workfunction_ev - phi_s;

        // Long-channel zero-bias threshold voltage
        let v_th0 = if params.flavor.is_n_channel() {
            v_fb + 2.0 * phi_f + (q_dep / c_ox)
        } else {
            v_fb - 2.0 * phi_f - (q_dep / c_ox)
        };

        Self {
            params,
            c_ox,
            phi_f,
            v_th0,
        }
    }

    /// Evaluates drain current, transconductance, and output conductance for terminal voltages.
    ///
    /// Voltages are defined relative to source (Vgs, Vds) and substrate (Vbs).
    pub fn evaluate(&self, v_gs: f64, v_ds: f64, v_bs: f64, temp_k: f64) -> MosfetMvlEvaluation {
        let is_n = self.params.flavor.is_n_channel();
        let t = temp_k.max(50.0);
        let v_t = K_B * t / Q_E;

        // Sign orientation: normalize to equivalent positive-polarity NMOS equations
        let (vgs_eff, vds_eff, vbs_eff) = if is_n {
            (v_gs, v_ds.max(0.0), v_bs.min(0.0))
        } else {
            (-v_gs, (-v_ds).max(0.0), (-v_bs).min(0.0))
        };

        // Body effect threshold shift
        let gamma = (2.0 * Q_E * EPS_SI * self.params.n_substrate).sqrt() / self.c_ox;
        let body_shift = gamma * ((2.0 * self.phi_f - vbs_eff).sqrt() - (2.0 * self.phi_f).sqrt());
        let v_th_nominal = if is_n {
            self.v_th0 + body_shift
        } else {
            self.v_th0.abs() + body_shift
        };

        // DIBL threshold lowering
        let v_th = v_th_nominal - self.params.dibl_eta * vds_eff;

        // Subthreshold swing factor
        let s_swing = self.params.subthreshold_factor * (v_t * (10.0_f64).ln());
        let n_sub = s_swing / (v_t * (10.0_f64).ln());

        let v_ov = vgs_eff - v_th;
        let beta = (self.params.width / self.params.length) * self.params.mobility * self.c_ox;

        let (ids_norm, gm_norm, gds_norm) = if v_ov <= 0.0 {
            // Subthreshold region: exponential conduction
            let i_off_0 = beta * (n_sub - 1.0) * v_t * v_t;
            let (exp_term, _) = safe_exp(v_ov / (n_sub * v_t));
            let (exp_vds, _) = safe_exp(-vds_eff / v_t);
            let vds_sat_factor = 1.0 - exp_vds;
            let ids = i_off_0 * exp_term * vds_sat_factor;
            let gm = ids / (n_sub * v_t);
            let gds = (ids / v_t) * exp_vds + (self.params.dibl_eta / (n_sub * v_t)) * ids;
            (ids, gm, gds)
        } else {
            // Strong inversion with velocity saturation
            let e_sat = 2.0 * self.params.v_sat / self.params.mobility;
            let v_dsat = (v_ov * e_sat * self.params.length) / (v_ov + e_sat * self.params.length);

            if vds_eff < v_dsat {
                // Linear region
                let denom = 1.0 + (vds_eff / (e_sat * self.params.length));
                let ids = beta * (v_ov * vds_eff - 0.5 * vds_eff * vds_eff) / denom;
                let gm = beta * vds_eff / denom;
                let gds = beta * (v_ov - vds_eff) / denom;
                (ids, gm, gds)
            } else {
                // Saturation region with channel length modulation
                let clm = 1.0 + 0.08 * (vds_eff - v_dsat) / self.params.length.max(10e-9);
                let ids_sat =
                    0.5 * beta * v_ov * v_dsat / (1.0 + (v_dsat / (e_sat * self.params.length)));
                let ids = ids_sat * clm;
                let gm = beta * v_dsat * clm / (1.0 + (v_dsat / (e_sat * self.params.length)));
                let gds =
                    ids_sat * 0.08 / self.params.length.max(10e-9) + gm * self.params.dibl_eta;
                (ids, gm, gds)
            }
        };

        let final_vth = if is_n { v_th } else { -v_th };
        let final_ids = if is_n { ids_norm } else { -ids_norm };

        MosfetMvlEvaluation {
            ids: final_ids,
            gm: gm_norm.max(1e-12),
            gds: gds_norm.max(1e-12),
            v_th: final_vth,
        }
    }

    /// Access the underlying parameters.
    pub fn params(&self) -> &MultiThresholdMosfetParams {
        &self.params
    }
}

/// Physical parameters for a cascaded dual-barrier Resonant Tunneling Diode (RTD)
/// providing multi-peak negative differential resistance.
#[derive(Debug, Clone)]
pub struct MultiPeakRtdParams {
    /// Active mesa area [m^2]
    pub area: f64,
    /// First resonance peak voltage [V]
    pub v_p1: f64,
    /// First peak current density [A/m^2]
    pub j_p1: f64,
    /// First valley voltage [V]
    pub v_v1: f64,
    /// First valley current density [A/m^2]
    pub j_v1: f64,
    /// Second resonance peak voltage [V]
    pub v_p2: f64,
    /// Second peak current density [A/m^2]
    pub j_p2: f64,
    /// Second valley voltage [V]
    pub v_v2: f64,
    /// Second valley current density [A/m^2]
    pub j_v2: f64,
    /// Excess thermal current pre-exponential [A/m^2]
    pub j_0: f64,
}

impl Default for MultiPeakRtdParams {
    fn default() -> Self {
        Self {
            area: 1.0e-12, // 1 um^2
            v_p1: 0.22,
            j_p1: 1.0e9, // 100 kA/cm^2
            v_v1: 0.38,
            j_v1: 1.2e8, // PVCR_1 = 8.33
            v_p2: 0.58,
            j_p2: 1.05e9, // 105 kA/cm^2 (series matched current)
            v_v2: 0.74,
            j_v2: 1.4e8, // PVCR_2 = 7.5
            j_0: 1.0e4,
        }
    }
}

/// Physical evaluation of a multi-peak RTD device.
#[derive(Debug, Clone, Copy)]
pub struct RtdEvaluation {
    /// Total terminal current [A]
    pub current: f64,
    /// Dynamic differential conductance dI / dV [S]
    pub conductance: f64,
    /// Flag indicating whether device is in a negative differential conductance region (g < 0)
    pub is_ndr: bool,
}

/// Dual-peak resonant tunneling diode device model producing three stable operating states.
#[derive(Debug, Clone)]
pub struct MultiPeakRtdModel {
    params: MultiPeakRtdParams,
}

impl MultiPeakRtdModel {
    /// Constructs a multi-peak RTD model from parameters.
    pub fn new(params: MultiPeakRtdParams) -> Self {
        Self { params }
    }

    /// Evaluates the terminal current and dynamic conductance for applied voltage.
    pub fn evaluate(&self, _v: f64, temp_k: f64) -> RtdEvaluation {
        let _t = temp_k.max(50.0);
        let v_abs = _v.abs().max(1e-12);
        let sgn = if _v >= 0.0 { 1.0 } else { -1.0 };

        // Branch 1 & 2: First resonant tunneling diode peak and NDR
        let i_p1_total = self.params.j_p1 * self.params.area;
        let i_v1_total = self.params.j_v1 * self.params.area;

        // Branch 3 & 4: Second resonant tunneling diode peak and NDR
        let i_p2_total = self.params.j_p2 * self.params.area;
        let i_v2_total = self.params.j_v2 * self.params.area;

        let (current_mag, conductance) = if v_abs < self.params.v_v1 {
            // Region 1: First RTD active
            let norm1 = v_abs / self.params.v_p1;
            let (exp1, _) = safe_exp(1.0 - norm1);
            let res1 = i_p1_total * norm1 * exp1;
            let d_res1 = (i_p1_total / self.params.v_p1) * (1.0 - norm1) * exp1;

            let floor1 = i_v1_total * (v_abs / self.params.v_v1).min(1.0);
            let d_floor1 = i_v1_total / self.params.v_v1;

            (
                res1.max(floor1),
                if res1 > floor1 { d_res1 } else { d_floor1 },
            )
        } else if v_abs < self.params.v_v2 {
            // Region 2: Second RTD active above valley 1
            let span2 = self.params.v_p2 - self.params.v_v1;
            let norm2 = (v_abs - self.params.v_v1) / span2;
            let (exp2, _) = safe_exp(1.0 - norm2);
            let res2 = i_p2_total * norm2 * exp2;
            let d_res2 = (i_p2_total / span2) * (1.0 - norm2) * exp2;

            let floor2 = i_v1_total
                + (i_v2_total - i_v1_total)
                    * ((v_abs - self.params.v_v1) / (self.params.v_v2 - self.params.v_v1));
            let d_floor2 = (i_v2_total - i_v1_total) / (self.params.v_v2 - self.params.v_v1);

            let i_total = (res2 + i_v1_total).max(floor2);
            let g_total = if res2 + i_v1_total > floor2 {
                d_res2
            } else {
                d_floor2
            };
            (i_total, g_total)
        } else {
            // Region 3: Post-valley field emission and series conduction (positive slope)
            let g_series = (i_p1_total / self.params.v_p1) * 0.8;
            let dv = v_abs - self.params.v_v2;
            let i_post = i_v2_total + g_series * dv;
            (i_post, g_series)
        };

        RtdEvaluation {
            current: sgn * current_mag,
            conductance,
            is_ndr: conductance < 0.0,
        }
    }

    /// Calculates Peak-to-Valley Current Ratios (PVCR) for both resonant peaks.
    pub fn pvcr(&self) -> (f64, f64) {
        let pvcr1 = self.params.j_p1 / self.params.j_v1.max(1e-12);
        let pvcr2 = self.params.j_p2 / self.params.j_v2.max(1e-12);
        (pvcr1, pvcr2)
    }

    /// Finds the three stable equilibrium voltage states when connected to a supply Vdd
    /// through a pull-up load resistor R_load.
    pub fn solve_stable_states(&self, v_dd: f64, r_load: f64, temp_k: f64) -> Vec<f64> {
        let mut stable_voltages = Vec::new();
        let steps = 400;
        let dv = v_dd / (steps as f64);

        let mut prev_v = 0.0;
        let mut prev_f = self.load_line_mismatch(0.0, v_dd, r_load, temp_k);

        for step in 1..=steps {
            let v = (step as f64) * dv;
            let f = self.load_line_mismatch(v, v_dd, r_load, temp_k);

            // Zero crossing detected: load line intersects RTD I-V characteristic
            if prev_f * f <= 0.0 {
                let v_root = self.refine_root(prev_v, v, v_dd, r_load, temp_k);
                let eval = self.evaluate(v_root, temp_k);
                let total_conductance = eval.conductance + (1.0 / r_load);
                // Stable if total differential conductance > 0 (positive slope)
                if total_conductance > 0.0 {
                    stable_voltages.push(v_root);
                }
            }

            prev_v = v;
            prev_f = f;
        }

        stable_voltages
    }

    fn load_line_mismatch(&self, v: f64, v_dd: f64, r_load: f64, temp_k: f64) -> f64 {
        let i_rtd = self.evaluate(v, temp_k).current;
        let i_load = (v_dd - v) / r_load;
        i_rtd - i_load
    }

    fn refine_root(
        &self,
        mut v_low: f64,
        mut v_high: f64,
        v_dd: f64,
        r_load: f64,
        temp_k: f64,
    ) -> f64 {
        for _ in 0..20 {
            let v_mid = 0.5 * (v_low + v_high);
            let f_mid = self.load_line_mismatch(v_mid, v_dd, r_load, temp_k);
            let f_low = self.load_line_mismatch(v_low, v_dd, r_load, temp_k);

            if f_low * f_mid <= 0.0 {
                v_high = v_mid;
            } else {
                v_low = v_mid;
            }
            if (v_high - v_low).abs() < 1e-6 {
                break;
            }
        }
        0.5 * (v_low + v_high)
    }
}

/// Chirality-tuned Carbon Nanotube FET (CNTFET) physical model for Multi-Valued Logic.
#[derive(Debug, Clone)]
pub struct CntfetTernaryModel {
    /// Chirality index n
    pub n: u32,
    /// Chirality index m
    pub m: u32,
    /// Nanotube diameter [m]
    pub diameter_m: f64,
    /// Energy bandgap [eV]
    pub bandgap_ev: f64,
    /// Intrinsic physical threshold voltage [V]
    pub v_th: f64,
}

impl CntfetTernaryModel {
    /// Constructs a CNTFET from integer chirality indices (n, m).
    pub fn from_chirality(n: u32, m: u32) -> Self {
        let a_cc = 0.1421e-9; // Carbon-carbon bond length [m]
        let sqrt3 = 3.0_f64.sqrt();
        let c_ch = a_cc * sqrt3 * ((n * n + n * m + m * m) as f64).sqrt();
        let diameter_m = c_ch / std::f64::consts::PI;

        // Bandgap Eg = (2 * a_cc * t_hop) / (sqrt(3) * d) = ~0.84 eV / d(nm)
        let diameter_nm = diameter_m * 1.0e9;
        let bandgap_ev = 0.84 / diameter_nm.max(0.4);

        // Intrinsic threshold voltage Vth ~ Eg / (2 * q)
        let v_th = 0.5 * bandgap_ev;

        Self {
            n,
            m,
            diameter_m,
            bandgap_ev,
            v_th,
        }
    }

    /// Presets for ternary multi-threshold CNTFET library.
    pub fn low_vth() -> Self {
        Self::from_chirality(19, 0) // d ~ 1.49 nm -> Vth ~ 0.28 V
    }

    pub fn standard_vth() -> Self {
        Self::from_chirality(13, 0) // d ~ 1.02 nm -> Vth ~ 0.41 V
    }

    pub fn high_vth() -> Self {
        Self::from_chirality(10, 0) // d ~ 0.78 nm -> Vth ~ 0.54 V
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_threshold_workfunction_shifts() {
        let eot = 1.0e-9;
        let l = 20e-9;
        let w = 100e-9;
        let temp = 300.0;

        let mos_low = MultiThresholdMosfet::new(
            MultiThresholdMosfetParams::from_flavor(MosfetFlavor::LowN, eot, l, w),
            temp,
        );
        let mos_std = MultiThresholdMosfet::new(
            MultiThresholdMosfetParams::from_flavor(MosfetFlavor::StandardN, eot, l, w),
            temp,
        );
        let mos_high = MultiThresholdMosfet::new(
            MultiThresholdMosfetParams::from_flavor(MosfetFlavor::HighN, eot, l, w),
            temp,
        );

        let eval_low = mos_low.evaluate(0.3, 0.8, 0.0, temp);
        let eval_std = mos_std.evaluate(0.3, 0.8, 0.0, temp);
        let eval_high = mos_high.evaluate(0.3, 0.8, 0.0, temp);

        assert!(eval_low.v_th < eval_std.v_th);
        assert!(eval_std.v_th < eval_high.v_th);
        assert!(eval_low.ids > eval_std.ids);
        assert!(eval_std.ids > eval_high.ids);
    }

    #[test]
    fn test_multi_peak_rtd_stable_states() {
        let rtd = MultiPeakRtdModel::new(MultiPeakRtdParams::default());
        let (pvcr1, pvcr2) = rtd.pvcr();
        assert!(pvcr1 > 5.0);
        assert!(pvcr2 > 5.0);

        // Solve stable states across 800 ohm load resistor at 1.0 V supply
        let states = rtd.solve_stable_states(1.0, 800.0, 300.0);
        // Cascaded dual-peak RTD produces all 3 stable operating states on the load line
        assert_eq!(states.len(), 3);
    }

    #[test]
    fn test_cntfet_chirality_bandgaps() {
        let cnt_low = CntfetTernaryModel::low_vth();
        let cnt_std = CntfetTernaryModel::standard_vth();
        let cnt_high = CntfetTernaryModel::high_vth();

        assert!(cnt_low.v_th < cnt_std.v_th);
        assert!(cnt_std.v_th < cnt_high.v_th);
        assert!(cnt_low.diameter_m > cnt_std.diameter_m);
    }
}
