#![deny(unsafe_code)]

//! Ground-state sideband cooling and quantum noise analysis engine.
//!
//! Models Bose-Einstein thermal bath statistics, laser sideband dynamical cooling,
//! ground-state purity, input-referred added quantum noise, and dilution refrigerator
//! heat load margins for the intermediate phononic resonator.

/// Configuration parameters for ground-state sideband cooling and quantum noise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebandCoolingParams {
    /// Cryogenic dilution refrigerator bath temperature in milliKelvin (default: 20.0 mK).
    pub bath_temp_mk: f64,
    /// Mechanical resonator frequency $\Omega_m / (2\pi)$ in GHz (default: 4.0 GHz).
    pub omega_m_ghz: f64,
    /// Intrinsic mechanical dissipation rate $\gamma_m / (2\pi)$ in kHz (default: 1.8 kHz).
    pub gamma_m_khz: f64,
    /// Optical dynamical cooling rate $\Gamma_{\text{opt}} / (2\pi)$ in MHz (default: 0.35 MHz).
    pub optical_cooling_rate_mhz: f64,
    /// Microwave dynamical cooling rate $\Gamma_e / (2\pi)$ in MHz (default: 0.35 MHz).
    pub microwave_cooling_rate_mhz: f64,
    /// Quantum backaction cooling floor $n_{\text{min}} = (\kappa_o / 4\Omega_m)^2$ (default: 3.5e-6).
    pub quantum_backaction_n_min: f64,
    /// Dilution refrigerator 20 mK cooling capacity in microwatts (default: 20.0 uW).
    pub fridge_cooling_capacity_uw: f64,
    /// Total continuous optical + microwave dissipation heat load in microwatts (default: 0.85 uW).
    pub heat_load_uw: f64,
    /// Input microwave extraction efficiency $\eta_{e,\text{ext}}$ (default: 0.90).
    pub eta_e_ext: f64,
    /// Output optical extraction efficiency $\eta_{o,\text{ext}}$ (default: 0.90).
    pub eta_o_ext: f64,
}

impl Default for SidebandCoolingParams {
    fn default() -> Self {
        Self {
            bath_temp_mk: 20.0,
            omega_m_ghz: 4.0,
            gamma_m_khz: 1.8,
            optical_cooling_rate_mhz: 0.35,
            microwave_cooling_rate_mhz: 0.35,
            quantum_backaction_n_min: 3.5e-6,
            fridge_cooling_capacity_uw: 20.0,
            heat_load_uw: 0.85,
            eta_e_ext: 0.90,
            eta_o_ext: 0.90,
        }
    }
}

impl SidebandCoolingParams {
    /// Validates physical constraints and clamps parameter ranges.
    pub fn sanitized(&self) -> Self {
        Self {
            bath_temp_mk: self.bath_temp_mk.clamp(1.0, 1000.0),
            omega_m_ghz: self.omega_m_ghz.clamp(0.5, 20.0),
            gamma_m_khz: self.gamma_m_khz.clamp(0.01, 100.0),
            optical_cooling_rate_mhz: self.optical_cooling_rate_mhz.clamp(0.001, 50.0),
            microwave_cooling_rate_mhz: self.microwave_cooling_rate_mhz.clamp(0.001, 50.0),
            quantum_backaction_n_min: self.quantum_backaction_n_min.clamp(1.0e-9, 0.1),
            fridge_cooling_capacity_uw: self.fridge_cooling_capacity_uw.clamp(1.0, 1000.0),
            heat_load_uw: self.heat_load_uw.clamp(0.01, 50.0),
            eta_e_ext: self.eta_e_ext.clamp(0.1, 1.0),
            eta_o_ext: self.eta_o_ext.clamp(0.1, 1.0),
        }
    }
}

/// Evaluates dynamical cooling performance, added noise, and ground-state statistics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebandCoolingEngine {
    pub params: SidebandCoolingParams,
}

impl SidebandCoolingEngine {
    /// Creates a new sideband cooling engine instance.
    pub fn new(params: SidebandCoolingParams) -> Self {
        Self {
            params: params.sanitized(),
        }
    }

    /// Evaluates equilibrium Bose-Einstein thermal phonon occupancy $n_{\text{th}} = \frac{1}{e^{\hbar\Omega_m / k_B T} - 1}$.
    pub fn compute_thermal_occupancy(&self) -> f64 {
        let p = &self.params;
        // Constants: hbar = 1.054571817e-34 J*s, k_B = 1.380649e-23 J/K
        let hbar = 1.054_571_817e-34;
        let k_b = 1.380_649e-23;
        let omega_m_rad = p.omega_m_ghz * 1.0e9 * 2.0 * std::f64::consts::PI;
        let temp_k = (p.bath_temp_mk * 1.0e-3).max(1.0e-6);

        let exponent = (hbar * omega_m_rad) / (k_b * temp_k);
        if exponent > 60.0 {
            (-exponent).exp()
        } else if exponent < 1.0e-5 {
            1.0 / exponent
        } else {
            1.0 / (exponent.exp() - 1.0)
        }
    }

    /// Evaluates effective dynamically sideband-cooled phonon occupancy $\bar{n}_{\text{eff}}$.
    ///
    /// $\bar{n}_{\text{eff}} = \frac{\gamma_m n_{\text{th}} + \Gamma_{\text{opt}} n_{\text{min}}}{\gamma_m + \Gamma_{\text{opt}} + \Gamma_e}$
    pub fn compute_effective_occupancy(&self) -> f64 {
        let p = &self.params;
        let n_th = self.compute_thermal_occupancy();
        let gamma_m_mhz = p.gamma_m_khz * 1.0e-3;
        let gamma_total_mhz = gamma_m_mhz + p.optical_cooling_rate_mhz + p.microwave_cooling_rate_mhz;

        let num = gamma_m_mhz * n_th + p.optical_cooling_rate_mhz * p.quantum_backaction_n_min;
        let n_eff = num / gamma_total_mhz.max(1.0e-9);
        n_eff.max(p.quantum_backaction_n_min)
    }

    /// Evaluates quantum mechanical ground state probability $P_{\text{ground}} = \frac{1}{1 + \bar{n}_{\text{eff}}}$.
    pub fn compute_ground_state_purity(&self) -> f64 {
        let n_eff = self.compute_effective_occupancy();
        1.0 / (1.0 + n_eff)
    }

    /// Evaluates total input-referred added quantum noise $N_{\text{add}}$ in quanta.
    ///
    /// $N_{\text{add}} = \frac{1 - \eta_{e,\text{ext}}}{2 \eta_{e,\text{ext}}} + \frac{n_{\text{th}}}{C_e} + \frac{\bar{n}_{\text{eff}}}{\eta_{e,\text{ext}}}$
    pub fn compute_added_quantum_noise(&self, c_e: f64) -> f64 {
        let p = &self.params;
        let c_safe = c_e.max(1.0);
        let n_th = self.compute_thermal_occupancy();
        let n_eff = self.compute_effective_occupancy();

        let loss_noise = (1.0 - p.eta_e_ext) / (2.0 * p.eta_e_ext.max(1.0e-4));
        let thermal_noise = n_th / c_safe;
        let sideband_noise = n_eff / p.eta_e_ext.max(1.0e-4);

        (loss_noise + thermal_noise + sideband_noise).max(0.01)
    }

    /// Evaluates dilution refrigerator cooling capacity headroom margin in percent: $(P_{\text{fridge}} - P_{\text{load}}) / P_{\text{fridge}} \times 100\%$.
    pub fn compute_cryo_capacity_margin_percent(&self) -> f64 {
        let p = &self.params;
        let margin = (p.fridge_cooling_capacity_uw - p.heat_load_uw)
            / p.fridge_cooling_capacity_uw.max(1.0e-4);
        (margin * 100.0).clamp(0.0, 100.0)
    }

    /// Sweeps effective phonon occupancy vs total dynamical cooling rate $\Gamma / \gamma_m$.
    pub fn sweep_cooling_curve(&self, steps: usize) -> Vec<(f64, f64)> {
        let p = &self.params;
        let n_th = self.compute_thermal_occupancy();
        let gamma_m_mhz = p.gamma_m_khz * 1.0e-3;

        let count = steps.max(5);
        let c_min = 0.1_f64;
        let c_max = 500.0_f64;

        (0..count)
            .map(|i| {
                let frac = i as f64 / (count - 1) as f64;
                let cooling_ratio = c_min * (c_max / c_min).powf(frac);
                let gamma_cool_mhz = cooling_ratio * gamma_m_mhz;
                let total_gamma = gamma_m_mhz + 2.0 * gamma_cool_mhz;
                let n_eff = (gamma_m_mhz * n_th + gamma_cool_mhz * p.quantum_backaction_n_min)
                    / total_gamma;
                (cooling_ratio, n_eff.max(p.quantum_backaction_n_min))
            })
            .collect()
    }
}
