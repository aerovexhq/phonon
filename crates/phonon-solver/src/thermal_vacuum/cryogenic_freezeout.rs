#![deny(unsafe_code)]

//! Cryogenic Semiconductor Carrier Freeze-Out, Subthreshold Steepening & Kink TCAD Engine.
//!
//! Models deep cryogenic CMOS physics from room temperature (300K) down to liquid helium (4.2K).
//! Includes dopant carrier freeze-out n(T)/N_D (Phosphorus, Boron, Arsenic, Indium),
//! subthreshold slope steepening S(T) = (k_B * T / q) * ln(10) * (1 + C_dep/C_ox) achieving
//! S < 20 mV/dec at 77K and S < 5 mV/dec at 4.2K, and substrate freeze-out impact-ionization kink effect.

use std::f64::consts::LN_10;

/// Boltzmann constant k_B in Joules / Kelvin.
pub const BOLTZMANN_K: f64 = 1.380649e-23;

/// Elementary electron charge q in Coulombs.
pub const ELEMENTARY_CHARGE_Q: f64 = 1.602176634e-19;

/// Semiconductor shallow dopant impurity kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryogenicDopantKind {
    /// Phosphorus in Silicon (n-type donor, Delta E_D ~ 45 meV).
    PhosphorusInSilicon,
    /// Boron in Silicon (p-type acceptor, Delta E_A ~ 45 meV).
    BoronInSilicon,
    /// Arsenic in Silicon (n-type donor, Delta E_D ~ 54 meV).
    ArsenicInSilicon,
    /// Antimony in Silicon (n-type donor, Delta E_D ~ 43 meV).
    AntimonyInSilicon,
    /// Indium in Silicon (p-type deep acceptor, Delta E_A ~ 160 meV).
    IndiumInSilicon,
}

impl CryogenicDopantKind {
    /// Dopant ionization energy Delta E in electron-volts (eV).
    pub fn ionization_energy_ev(&self) -> f64 {
        match self {
            Self::PhosphorusInSilicon => 0.045,
            Self::BoronInSilicon => 0.045,
            Self::ArsenicInSilicon => 0.054,
            Self::AntimonyInSilicon => 0.043,
            Self::IndiumInSilicon => 0.160,
        }
    }

    /// Dopant ground-state degeneracy factor g (typically 2 for donors, 4 for acceptors).
    pub fn degeneracy_factor(&self) -> f64 {
        match self {
            Self::PhosphorusInSilicon | Self::ArsenicInSilicon | Self::AntimonyInSilicon => 2.0,
            Self::BoronInSilicon | Self::IndiumInSilicon => 4.0,
        }
    }
}

/// Carrier freeze-out statistics model for semiconductors.
#[derive(Debug, Clone)]
pub struct CarrierFreezeoutModel {
    /// Dopant impurity kind.
    pub dopant: CryogenicDopantKind,
    /// Nominal background doping concentration in cm^-3 (e.g. 1.0e17 cm^-3).
    pub nominal_doping_cm3: f64,
    /// Effective density of states at 300K in cm^-3 (e.g. N_C ~ 2.8e19 cm^-3 for Si).
    pub effective_dos_300k_cm3: f64,
    /// Compensation acceptor/donor concentration in cm^-3.
    pub compensation_doping_cm3: f64,
}

impl Default for CarrierFreezeoutModel {
    fn default() -> Self {
        Self {
            dopant: CryogenicDopantKind::PhosphorusInSilicon,
            nominal_doping_cm3: 1.0e17,
            effective_dos_300k_cm3: 2.8e19,
            compensation_doping_cm3: 1.0e15,
        }
    }
}

impl CarrierFreezeoutModel {
    /// Effective density of states N_C(T) or N_V(T) at temperature T in Kelvin (cm^-3).
    /// Scales with (T / 300)^(3/2).
    pub fn effective_dos_at_temp(&self, temp_k: f64) -> f64 {
        let t = temp_k.max(0.1);
        self.effective_dos_300k_cm3 * (t / 300.0).powf(1.5)
    }

    /// Thermal voltage V_t = k_B * T / q in Volts.
    pub fn thermal_voltage_v(temp_k: f64) -> f64 {
        let t = temp_k.max(0.1);
        (BOLTZMANN_K * t) / ELEMENTARY_CHARGE_Q
    }

    /// Calculate ionized carrier concentration n(T) in cm^-3 at temperature T (Kelvin).
    /// Solves the standard semiconductor statistical equation with compensation:
    /// n * (N_A + n) / (N_D - N_A - n) = (N_C / g) * exp(-Delta E / k_B T).
    pub fn ionized_carrier_concentration_cm3(&self, temp_k: f64) -> f64 {
        let t = temp_k.max(0.1);
        let delta_e_j = self.dopant.ionization_energy_ev() * ELEMENTARY_CHARGE_Q;
        let kb_t = BOLTZMANN_K * t;
        let exp_factor = (-delta_e_j / kb_t).exp().max(1.0e-300);

        let n_c = self.effective_dos_at_temp(t);
        let g = self.dopant.degeneracy_factor();
        let n_1 = (n_c / g) * exp_factor;

        let n_d = self.nominal_doping_cm3;
        let n_a = self.compensation_doping_cm3.min(0.5 * n_d);

        // Quadratic equation: n^2 + (N_A + n_1) * n - n_1 * (N_D - N_A) = 0
        let b = n_a + n_1;
        let c = -n_1 * (n_d - n_a);

        let discr = (b * b - 4.0 * c).max(0.0);
        let n = 0.5 * (-b + discr.sqrt());

        n.clamp(1.0e2, n_d)
    }

    /// Calculate ionized carrier fraction eta(T) = n(T) / N_D in [0.0, 1.0].
    pub fn ionized_carrier_fraction(&self, temp_k: f64) -> f64 {
        let n = self.ionized_carrier_concentration_cm3(temp_k);
        (n / self.nominal_doping_cm3).clamp(0.0, 1.0)
    }

    /// Generate ionized fraction sweep from T_min to 300K.
    pub fn simulate_freezeout_curve(&self, min_temp_k: f64, points: usize) -> Vec<(f64, f64)> {
        let pts = points.clamp(30, 500);
        let t_min = min_temp_k.clamp(2.0, 50.0);
        let t_max = 300.0;
        let dt = (t_max - t_min) / (pts - 1) as f64;

        (0..pts)
            .map(|i| {
                let t = t_min + i as f64 * dt;
                let eta = self.ionized_carrier_fraction(t);
                (t, eta)
            })
            .collect()
    }
}

/// Cryogenic subthreshold steepening TCAD model.
#[derive(Debug, Clone)]
pub struct SubthresholdSteepeningModel {
    /// Ideality body factor n = 1 + C_dep / C_ox (typically 1.1 to 1.4 at 300K).
    pub ideality_factor: f64,
    /// Band-tail / interface localized defect saturation floor in mV/dec at T -> 0.
    pub cryogenic_saturation_floor_mv_per_dec: f64,
}

impl Default for SubthresholdSteepeningModel {
    fn default() -> Self {
        Self {
            ideality_factor: 1.15,
            cryogenic_saturation_floor_mv_per_dec: 3.5, // Interface localized state limit
        }
    }
}

impl SubthresholdSteepeningModel {
    /// Calculate subthreshold swing S(T) in mV / decade at temperature T in Kelvin.
    ///
    /// Ideal thermionic emission: S_ideal(T) = (k_B * T / q) * ln(10) * n * 1000.
    /// Includes cryogenic interface band-tail localization saturation floor S_min:
    /// S(T) = sqrt( S_ideal(T)^2 + S_min^2 ).
    pub fn subthreshold_swing_mv_per_dec(&self, temp_k: f64) -> f64 {
        let t = temp_k.max(0.1);
        let thermal_v = CarrierFreezeoutModel::thermal_voltage_v(t);
        let s_ideal_mv = thermal_v * LN_10 * self.ideality_factor * 1000.0;

        let s_floor = self.cryogenic_saturation_floor_mv_per_dec;
        (s_ideal_mv * s_ideal_mv + s_floor * s_floor).sqrt()
    }

    /// Generate subthreshold swing trajectory from 4.2K to 300K.
    pub fn simulate_subthreshold_swing_curve(&self, points: usize) -> Vec<(f64, f64)> {
        let pts = points.clamp(30, 500);
        let t_min = 4.2;
        let t_max = 300.0;
        let dt = (t_max - t_min) / (pts - 1) as f64;

        (0..pts)
            .map(|i| {
                let t = t_min + i as f64 * dt;
                let s = self.subthreshold_swing_mv_per_dec(t);
                (t, s)
            })
            .collect()
    }
}

/// Cryogenic MOSFET substrate freeze-out impact ionization kink model.
#[derive(Debug, Clone)]
pub struct CryogenicKinkModel {
    /// Operating temperature in Kelvin (e.g. 4.2K - 50K).
    pub temp_k: f64,
    /// Threshold voltage V_th in Volts at operating temperature.
    pub threshold_voltage_v: f64,
    /// Low-field transconductance / transconductance parameter beta in mA / V^2.
    pub beta_ma_per_v2: f64,
    /// Drain voltage onset of substrate impact ionization kink V_kink in Volts (e.g. 0.8V - 1.2V).
    pub kink_onset_voltage_v: f64,
    /// Impact ionization multiplication kink intensity factor.
    pub kink_intensity_factor: f64,
}

impl Default for CryogenicKinkModel {
    fn default() -> Self {
        Self {
            temp_k: 4.2,
            threshold_voltage_v: 0.65,
            beta_ma_per_v2: 2.4,
            kink_onset_voltage_v: 1.0,
            kink_intensity_factor: 0.35,
        }
    }
}

impl CryogenicKinkModel {
    /// Calculate drain current I_D in milliamperes (mA) at gate voltage V_gs and drain voltage V_ds.
    ///
    /// At cryogenic temperatures where substrate is frozen out (R_sub -> infty), impact ionization
    /// near the drain junction induces bulk charging, shifting threshold voltage downward:
    /// Delta V_th = -kink_factor * (V_ds - V_kink)^2 for V_ds > V_kink.
    pub fn drain_current_ma(&self, v_gs: f64, v_ds: f64) -> f64 {
        let vds = v_ds.max(0.0);

        // Effective cryogenic threshold voltage with bulk-charging kink shift
        let delta_vth = if vds > self.kink_onset_voltage_v {
            let dv = vds - self.kink_onset_voltage_v;
            self.kink_intensity_factor * (dv * dv) / (1.0 + dv)
        } else {
            0.0
        };

        let vth_eff = (self.threshold_voltage_v - delta_vth).max(0.05);
        let vov = v_gs - vth_eff;
        if vov <= 0.0 {
            // Weak subthreshold leakage
            let ss = SubthresholdSteepeningModel::default().subthreshold_swing_mv_per_dec(self.temp_k);
            let sub_ratio = vov / (ss * 0.001);
            return (1.0e-5 * 10.0_f64.powf(sub_ratio.max(-12.0))).clamp(0.0, 1.0e-2);
        }

        // MOSFET piecewise linear/saturation characteristics
        if vds < vov {
            // Linear triode regime
            self.beta_ma_per_v2 * (vov * vds - 0.5 * vds * vds)
        } else {
            // Saturation regime with channel length modulation and substrate kink multiplication
            let i_sat = 0.5 * self.beta_ma_per_v2 * vov * vov;
            let lambda_ch = 0.05; // Channel length modulation
            let mult = 1.0 + lambda_ch * (vds - vov);
            let kink_boost = if vds > self.kink_onset_voltage_v {
                1.0 + 0.6 * (vds - self.kink_onset_voltage_v).powi(2)
            } else {
                1.0
            };

            i_sat * mult * kink_boost
        }
    }

    /// Simulate output characteristic curve I_D(V_DS) for a given gate voltage V_gs.
    /// Returns pairs of (V_ds in Volts, I_d in mA).
    pub fn simulate_id_vds_curve(&self, v_gs: f64, max_vds: f64, points: usize) -> Vec<(f64, f64)> {
        let pts = points.clamp(30, 500);
        let v_max = max_vds.max(0.5);
        let dv = v_max / (pts - 1) as f64;

        (0..pts)
            .map(|i| {
                let vds = i as f64 * dv;
                let id = self.drain_current_ma(v_gs, vds);
                (vds, id)
            })
            .collect()
    }
}
