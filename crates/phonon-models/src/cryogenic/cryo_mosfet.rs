//! Cryo-CMOS compact model with cryogenic subthreshold swing and freeze-out dynamics.
//!
//! Captures the steepening of the subthreshold swing $S(T) \propto T$ down to cryogenic temperatures (4.2 K - 77 K),
//! threshold voltage shift $V_{th}(T)$, and multi-mechanism mobility degradation.

use super::cryo_mobility::CryogenicMobilityModel;
use super::freezeout::CryogenicFreezeoutModel;
use phonon_core::{thermal_voltage, EPSILON_0, EPSILON_R_OX};

/// Cryogenic MOSFET compact model output state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CryoMosfetOutput {
    /// Drain-to-source current $I_{ds}$ in Amperes ($A$).
    pub ids: f64,
    /// Transconductance $g_m = \frac{\partial I_{ds}}{\partial V_{gs}}$ in Siemens ($S$).
    pub gm: f64,
    /// Output conductance $g_{ds} = \frac{\partial I_{ds}}{\partial V_{ds}}$ in Siemens ($S$).
    pub gds: f64,
    /// Effective cryogenic threshold voltage $V_{th}(T)$ in Volts ($V$).
    pub vth: f64,
    /// Subthreshold swing $S(T)$ in $mV/\text{decade}$.
    pub swing_mv_dec: f64,
}

/// Cryo-CMOS compact physical model.
#[derive(Debug, Clone, PartialEq)]
pub struct CryoMosfetModel {
    /// Threshold voltage at reference 300K in Volts ($V$).
    pub vth0: f64,
    /// Temperature coefficient of threshold voltage $\alpha_{vth} = \frac{d V_{th}}{d(300 - T)}$ in $V/K$ ($\approx 1.0\text{ mV/K}$).
    pub alpha_vth: f64,
    /// Gate channel width $W$ in meters ($m$).
    pub width: f64,
    /// Gate channel length $L$ in meters ($m$).
    pub length: f64,
    /// Gate dielectric physical oxide thickness $t_{ox}$ in meters ($m$).
    pub tox: f64,
    /// Subthreshold body effect ideality factor $n_{ss} = 1 + C_d / C_{ox}$ ($\approx 1.1 - 1.3$).
    pub n_subthreshold: f64,
    /// Minimum saturation subthreshold swing floor $S_{min}$ in $mV/\text{decade}$ due to band-tail interface traps.
    pub min_swing_mv_dec: f64,
    /// Channel length modulation parameter $\lambda$ in $V^{-1}$.
    pub lambda: f64,
    /// Critical saturation velocity field $\mathcal{E}_{sat}$ in $V/m$ ($\approx 10^7 \text{ V/m}$).
    pub e_sat: f64,
    /// Cryogenic carrier mobility model.
    pub mobility_model: CryogenicMobilityModel,
    /// Cryogenic dopant freeze-out model.
    pub freezeout_model: CryogenicFreezeoutModel,
}

impl Default for CryoMosfetModel {
    fn default() -> Self {
        Self {
            vth0: 0.35,
            alpha_vth: 1.0e-3, // 1 mV/K shift -> at 4K, Vth increases by ~0.3V
            width: 1.0e-6,
            length: 45e-9,
            tox: 1.2e-9,
            n_subthreshold: 1.15,
            min_swing_mv_dec: 5.0, // Interface trap limit at 4.2K
            lambda: 0.05,
            e_sat: 1.0e7,
            mobility_model: CryogenicMobilityModel::default(),
            freezeout_model: CryogenicFreezeoutModel::default(),
        }
    }
}

impl CryoMosfetModel {
    /// Oxide capacitance per unit area $C_{ox} = \frac{\epsilon_{ox} \epsilon_0}{t_{ox}}$ in $F/m^2$.
    #[inline]
    pub fn cox(&self) -> f64 {
        (EPSILON_R_OX * EPSILON_0) / self.tox
    }

    /// Cryogenic threshold voltage at temperature $T$:
    /// $$V_{th}(T) = V_{th0} + \alpha_{vth} \cdot (300.0 - T)$$
    #[inline]
    pub fn threshold_voltage(&self, temp_k: f64) -> f64 {
        let t = temp_k.clamp(1.0, 500.0);
        self.vth0 + self.alpha_vth * (300.0 - t)
    }

    /// Subthreshold swing $S(T)$ in $mV/\text{decade}$:
    /// $$S(T) = \max\left( \ln(10) \frac{k_B T}{q} n_{ss} \cdot 10^3, S_{min} \right)$$
    pub fn subthreshold_swing_mv_dec(&self, temp_k: f64) -> f64 {
        let vt = thermal_voltage(temp_k.max(1.0));
        let ideal_swing_mv = std::f64::consts::LN_10 * vt * self.n_subthreshold * 1000.0;
        ideal_swing_mv.max(self.min_swing_mv_dec)
    }

    /// Internal evaluation of drain current $I_{ds}(V_{gs}, V_{ds}, T)$.
    fn compute_ids(&self, vgs: f64, vds: f64, temp_k: f64) -> f64 {
        if vds.abs() < 1e-14 {
            return 0.0;
        }

        let is_negative = vds < 0.0;
        let vds_mag = vds.abs();
        let t = temp_k.max(1.0);
        let vth = self.threshold_voltage(t);

        // Effective subthreshold smoothing voltage parameter
        let swing_v = (self.subthreshold_swing_mv_dec(t) / 1000.0) / std::f64::consts::LN_10;
        let eta_ss = swing_v.max(1e-4);

        // Smooth overdrive voltage connecting subthreshold and strong inversion
        let vov = vgs - vth;
        let exp_arg = (vov / (2.0 * eta_ss)).clamp(-60.0, 60.0);
        let vgst_eff = 2.0 * eta_ss * (1.0 + exp_arg.exp()).ln();

        // Effective transverse electric field for mobility & freeze-out
        let e_eff = (vgst_eff / (6.0 * self.tox) + 1.0e5).clamp(1e5, 1e9);

        // Ionized and neutral dopant fractions
        let ionized_frac = self.freezeout_model.ionized_donor_fraction(t, 0.0, e_eff);
        let n_d = self.freezeout_model.donor_concentration;
        let n_ion = n_d * ionized_frac;
        let n_neut = n_d * (1.0 - ionized_frac);

        let cox = self.cox();
        let inv_charge = (cox * vgst_eff).max(1e-10);
        let carrier_density = inv_charge / (phonon_core::ELEMENTARY_CHARGE * 5e-9); // 5nm inversion layer depth

        let mu_eff =
            self.mobility_model
                .effective_mobility(t, n_ion, n_neut, carrier_density, e_eff);

        // Velocity saturation
        let v_sat_drive = (self.e_sat * self.length).max(0.01);
        let vdsat = (vgst_eff * v_sat_drive) / (vgst_eff + v_sat_drive);

        // Smooth drain saturation interpolation
        let ratio = vds_mag / vdsat.max(1e-3);
        let vds_eff = vds_mag / (1.0 + ratio.powi(2)).sqrt();

        // Core drift-diffusion current with channel length modulation
        let w_over_l = self.width / self.length;
        let linear_factor = vgst_eff * vds_eff - 0.5 * vds_eff * vds_eff;
        let clm = 1.0 + self.lambda * vds_mag;

        let ids_mag = mu_eff * cox * w_over_l * linear_factor * clm;

        if is_negative {
            -ids_mag
        } else {
            ids_mag
        }
    }

    /// Evaluates current, transconductance, and output conductance at $(V_{gs}, V_{ds}, T)$.
    pub fn evaluate(&self, vgs: f64, vds: f64, temp_k: f64) -> CryoMosfetOutput {
        let t = temp_k.max(1.0);
        let vth = self.threshold_voltage(t);
        let swing_mv_dec = self.subthreshold_swing_mv_dec(t);

        let ids = self.compute_ids(vgs, vds, t);

        // Numerical derivatives via high-precision central difference
        let d_v = 1e-6;
        let ids_vgs_p = self.compute_ids(vgs + d_v, vds, t);
        let ids_vgs_m = self.compute_ids(vgs - d_v, vds, t);
        let gm = (ids_vgs_p - ids_vgs_m) / (2.0 * d_v);

        let ids_vds_p = self.compute_ids(vgs, vds + d_v, t);
        let ids_vds_m = self.compute_ids(vgs, vds - d_v, t);
        let gds = (ids_vds_p - ids_vds_m) / (2.0 * d_v);

        CryoMosfetOutput {
            ids,
            gm: gm.max(0.0),
            gds: gds.max(1e-12),
            vth,
            swing_mv_dec,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subthreshold_swing_steepening() {
        let mosfet = CryoMosfetModel::default();
        let swing_300 = mosfet.subthreshold_swing_mv_dec(300.0);
        let swing_77 = mosfet.subthreshold_swing_mv_dec(77.0);
        let swing_4 = mosfet.subthreshold_swing_mv_dec(4.2);

        // 300K: ~68 mV/dec
        assert!((swing_300 - 68.0).abs() < 10.0);
        // 77K: ~17 mV/dec
        assert!((swing_77 - 17.5).abs() < 5.0);
        // 4.2K: limited by min_swing_mv_dec (5.0 mV/dec)
        assert_eq!(swing_4, 5.0);
    }

    #[test]
    fn test_threshold_voltage_cryo_shift() {
        let mosfet = CryoMosfetModel::default();
        let vth_300 = mosfet.threshold_voltage(300.0);
        let vth_4 = mosfet.threshold_voltage(4.2);

        assert_eq!(vth_300, 0.35);
        // At 4.2K, Vth increases by ~1 mV/K * (300 - 4.2) ~= 0.2958 V -> ~0.6458 V
        assert!((vth_4 - 0.6458).abs() < 0.01);
    }

    #[test]
    fn test_cryo_mosfet_iv_continuity() {
        let mosfet = CryoMosfetModel::default();
        let out_sub = mosfet.evaluate(0.2, 0.8, 77.0);
        let out_on = mosfet.evaluate(1.0, 0.8, 77.0);

        assert!(out_sub.ids < out_on.ids);
        assert!(out_on.ids > 0.0);
        assert!(out_on.gm > 0.0);
        assert!(out_on.gds > 0.0);
    }
}
