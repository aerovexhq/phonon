//! Total Ionizing Dose (TID) radiation degradation physics:
//! Oxide trapped charge ($N_{ot}$), interface trap generation ($N_{it}$),
//! threshold voltage shift, subthreshold swing degradation, and STI edge leakage.
//!
//! Formulates:
//! - Dose accumulation $D$ in $\text{krad}(\text{SiO}_2)$.
//! - Oxide hole trapping:
//!   $$\Delta N_{ot}(D) = N_{ot,sat} \left(1 - \exp\left(-\frac{D}{D_{ot}}\right)\right)$$
//! - Interface trap generation (hydrogen dissociation power law):
//!   $$\Delta N_{it}(D) = N_{it,0} \left(\frac{D}{D_{it}}\right)^\gamma$$
//! - Net threshold voltage shift:
//!   $$\Delta V_{th}(D) = -\frac{q \Delta N_{ot}(D)}{C_{ox}} \pm \frac{q \Delta N_{it}(D)}{C_{ox}}$$
//! - Subthreshold swing degradation:
//!   $$S(D) = S_0 + \ln(10) \frac{k_B T}{q} \frac{q \Delta N_{it}(D)}{C_{ox}}$$
//! - Channel carrier mobility degradation:
//!   $$\mu(D) = \frac{\mu_0}{1 + \alpha_{it} \cdot (\Delta N_{it}(D) \cdot 10^{-12})}$$
//! - STI sidewall inversion leakage current:
//!   $$I_{off, STI}(D) = I_{off, 0} \cdot 10^{\frac{-\Delta V_{th, STI}(D)}{S_{STI}}}$$

use phonon_core::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, EPSILON_SIO2};

/// Electron-hole pair creation energy in silicon dioxide at 300 K in eV ($17.0\text{ eV}$).
pub const DEFAULT_E_EH_OXIDE_EV: f64 = 17.0;

/// Transistor channel type for TID polarity determination.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelPolarity {
    Nmos,
    Pmos,
}

/// Physical parameters for Total Ionizing Dose (TID) degradation in MOSFET gate and isolation oxides.
#[derive(Debug, Clone, PartialEq)]
pub struct TotalIonizingDoseModel {
    pub polarity: ChannelPolarity,
    /// Physical gate oxide thickness $t_{ox}$ in meters ($m$).
    pub t_ox_m: f64,
    /// Saturated oxide trapped hole density $N_{ot,sat}$ in $\text{m}^{-2}$ (typically $10^{16} - 10^{17}\text{ m}^{-2}$).
    pub n_ot_sat_m2: f64,
    /// Characteristic dose for oxide trapping saturation $D_{ot}$ in $\text{krad}(\text{SiO}_2)$.
    pub d_ot_krad: f64,
    /// Interface trap buildup prefactor $N_{it,0}$ in $\text{m}^{-2}$.
    pub n_it_0_m2: f64,
    /// Characteristic dose for interface state buildup $D_{it}$ in $\text{krad}(\text{SiO}_2)$.
    pub d_it_krad: f64,
    /// Power-law exponent for interface trap buildup $\gamma$ (typically $0.5 - 0.7$).
    pub gamma_it: f64,
    /// Interface scattering mobility degradation coefficient $\alpha_{it}$ (typically $0.05 - 0.20$ per $10^{12}\text{ cm}^{-2}$).
    pub alpha_it: f64,
    /// Nominal subthreshold swing $S_0$ in Volts/decade ($V/\text{dec}$) (typically $0.070\text{ V/dec}$).
    pub nominal_swing_v_per_dec: f64,
    /// Initial STI sidewall off-state leakage current in Amperes ($A$).
    pub initial_sti_leakage_a: f64,
}

impl TotalIonizingDoseModel {
    /// Standard bulk CMOS 28nm NMOS TID preset.
    pub fn nmos_28nm() -> Self {
        Self {
            polarity: ChannelPolarity::Nmos,
            t_ox_m: 2.0e-9,      // 2 nm equivalent oxide
            n_ot_sat_m2: 2.0e16, // 2e12 cm^-2
            d_ot_krad: 30.0,     // 30 krad saturation
            n_it_0_m2: 3.0e15,   // 3e11 cm^-2
            d_it_krad: 30.0,
            gamma_it: 0.65,
            alpha_it: 0.10,
            nominal_swing_v_per_dec: 0.075,
            initial_sti_leakage_a: 1.0e-11, // 10 pA
        }
    }

    /// Standard bulk CMOS 28nm PMOS TID preset.
    pub fn pmos_28nm() -> Self {
        Self {
            polarity: ChannelPolarity::Pmos,
            t_ox_m: 2.0e-9,
            n_ot_sat_m2: 2.0e16,
            d_ot_krad: 30.0,
            n_it_0_m2: 3.0e15,
            d_it_krad: 30.0,
            gamma_it: 0.65,
            alpha_it: 0.10,
            nominal_swing_v_per_dec: 0.075,
            initial_sti_leakage_a: 1.0e-11,
        }
    }

    /// Gate oxide capacitance per unit area $C_{ox} = \frac{\epsilon_{ox}}{t_{ox}}$ in $\text{F/m}^2$.
    #[inline]
    pub fn c_ox(&self) -> f64 {
        EPSILON_SIO2 / self.t_ox_m.max(1e-10)
    }

    /// Accumulated oxide trapped charge density $\Delta N_{ot}(D)$ in $\text{m}^{-2}$.
    pub fn delta_n_ot(&self, dose_krad: f64) -> f64 {
        let d = dose_krad.max(0.0);
        self.n_ot_sat_m2 * (1.0 - (-d / self.d_ot_krad.max(1e-3)).exp())
    }

    /// Accumulated interface trap density $\Delta N_{it}(D)$ in $\text{m}^{-2}$.
    pub fn delta_n_it(&self, dose_krad: f64) -> f64 {
        let d = dose_krad.max(0.0);
        self.n_it_0_m2 * (d / self.d_it_krad.max(1e-3)).powf(self.gamma_it)
    }

    /// Radiation-induced threshold voltage shift $\Delta V_{th}(D)$ in Volts ($V$).
    pub fn threshold_voltage_shift(&self, dose_krad: f64) -> f64 {
        let c_ox = self.c_ox();
        let d_not = self.delta_n_ot(dose_krad);
        let d_nit = self.delta_n_it(dose_krad);

        // Oxide trapped holes always contribute a negative shift: -q * Not / Cox
        let delta_vth_ot = -(ELEMENTARY_CHARGE * d_not) / c_ox;

        // Interface traps:
        // In NMOS (acceptor-like in upper half of bandgap): traps fill with electrons, giving positive shift
        // In PMOS (donor-like in lower half of bandgap): traps fill with holes, giving negative shift
        let delta_vth_it = match self.polarity {
            ChannelPolarity::Nmos => (ELEMENTARY_CHARGE * d_nit) / c_ox,
            ChannelPolarity::Pmos => -(ELEMENTARY_CHARGE * d_nit) / c_ox,
        };

        delta_vth_ot + delta_vth_it
    }

    /// Degraded subthreshold swing $S(D)$ in Volts per decade ($V/\text{dec}$).
    pub fn subthreshold_swing(&self, dose_krad: f64, temp_k: f64) -> f64 {
        let c_ox = self.c_ox();
        let d_nit = self.delta_n_it(dose_krad);
        let vt = (BOLTZMANN_CONSTANT * temp_k) / ELEMENTARY_CHARGE;
        let delta_s = std::f64::consts::LN_10 * vt * (ELEMENTARY_CHARGE * d_nit) / c_ox;
        self.nominal_swing_v_per_dec + delta_s
    }

    /// Mobility degradation factor $\frac{\mu(D)}{\mu_0} \in (0, 1]$.
    pub fn mobility_degradation_factor(&self, dose_krad: f64) -> f64 {
        let d_nit = self.delta_n_it(dose_krad);
        let nit_cm2_1e12 = (d_nit * 1e-4) * 1e-12; // In units of 1e12 cm^-2
        1.0 / (1.0 + self.alpha_it * nit_cm2_1e12)
    }

    /// STI sidewall parasitic leakage current in Amperes ($A$).
    pub fn sti_sidewall_leakage(&self, dose_krad: f64) -> f64 {
        // Trapped positive charge in thick STI oxide produces negative shift:
        let d_not_sti = self.delta_n_ot(dose_krad);
        // STI capacitance is lower (thicker oxide ~100 nm), causing larger delta Vth
        let delta_vth_sti = (ELEMENTARY_CHARGE * d_not_sti * 100e-9) / EPSILON_SIO2;
        let s_sti = 0.150; // 150 mV/dec parasitic edge swing
        let log_leakage_boost = (delta_vth_sti / s_sti).min(8.0);
        self.initial_sti_leakage_a * 10.0_f64.powf(log_leakage_boost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nmos_tid_threshold_rebound_curve() {
        let tid = TotalIonizingDoseModel::nmos_28nm();

        // At low dose (50 krad), oxide trapping dominates -> negative Vth shift
        let vth_shift_50k = tid.threshold_voltage_shift(50.0);
        assert!(
            vth_shift_50k < 0.0,
            "Initial TID dose must cause negative Vth shift in NMOS: got {} V",
            vth_shift_50k
        );

        // At very high dose (1000 krad), interface traps accumulate with power-law exponent,
        // causing threshold rebound (shift becomes less negative or positive)
        let vth_shift_1000k = tid.threshold_voltage_shift(1000.0);
        assert!(
            vth_shift_1000k > vth_shift_50k,
            "Interface trap buildup must cause threshold rebound: 50k = {} V, 1000k = {} V",
            vth_shift_50k,
            vth_shift_1000k
        );
    }

    #[test]
    fn test_pmos_tid_monotonic_negative_shift() {
        let tid = TotalIonizingDoseModel::pmos_28nm();

        let vth_shift_50k = tid.threshold_voltage_shift(50.0);
        let vth_shift_200k = tid.threshold_voltage_shift(200.0);

        // In PMOS, both Not and Nit shifts are negative, causing monotonic negative shift
        assert!(vth_shift_50k < 0.0);
        assert!(vth_shift_200k < vth_shift_50k);
    }

    #[test]
    fn test_subthreshold_swing_and_sti_leakage_degradation() {
        let tid = TotalIonizingDoseModel::nmos_28nm();

        let s_0 = tid.subthreshold_swing(0.0, phonon_core::T_REF);
        let s_100 = tid.subthreshold_swing(100.0, phonon_core::T_REF);
        assert!(
            s_100 > s_0,
            "Subthreshold swing must stretch with dose: S(0) = {}, S(100) = {}",
            s_0,
            s_100
        );

        let i_off_0 = tid.sti_sidewall_leakage(0.0);
        let i_off_100 = tid.sti_sidewall_leakage(100.0);
        assert!(
            i_off_100 > 10.0 * i_off_0,
            "STI sidewall leakage must increase significantly: got {} A vs initial {} A",
            i_off_100,
            i_off_0
        );
    }
}
