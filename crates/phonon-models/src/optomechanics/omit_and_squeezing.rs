//! Optomechanically Induced Transparency (OMIT) and Ponderomotive Light Squeezing.
//!
//! Formulates:
//! - OMIT probe transmission spectrum:
//!   $$S_{OMIT}(\delta) = 1 - \frac{\kappa_{ex}}{\frac{\kappa}{2} - i(\delta + \Delta) + \frac{g^2}{\frac{\gamma_m}{2} - i(\delta - \Omega_m)}}$$
//! - OMIT transparency window linewidth:
//!   $$\Gamma_{OMIT} = \gamma_m (1 + \mathcal{C})$$
//! - Slow light group delay:
//!   $$\tau_g \approx \frac{2}{\gamma_m (1 + \mathcal{C})}$$
//! - Ponderomotive light squeezing spectrum & Kimble parameter $\mathcal{K}(\omega)$:
//!   $$\mathcal{K}(\omega) = \frac{4 g^2 \kappa \Omega_m}{[(\Omega_m^2 - \omega^2)^2 + \omega^2 \gamma_m^2] [(\kappa/2)^2 + \omega^2]}$$
//! - Minimum squeezed quadrature variance & noise reduction in dB:
//!   $$V_{min} = 1 + \frac{\mathcal{K}^2}{2} - \frac{\mathcal{K}}{2} \sqrt{\mathcal{K}^2 + 4} < 1.0, \quad S_{dB} = -10 \log_{10}(V_{min})$$
//! - Two-mode logarithmic negativity entanglement $E_N$.

use super::fabry_perot_nanobeam::OptomechanicalParams;

/// Optomechanically Induced Transparency (OMIT) parameter set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OmitParams {
    /// Cavity optomechanical system.
    pub system: OptomechanicalParams,
    /// Pump laser detuning $\Delta$ in Hz (typically $-\Omega_m$).
    pub pump_detuning_hz: f64,
    /// Mean intracavity photon number $\bar{n}_{cav}$.
    pub intracavity_photons: f64,
}

impl OmitParams {
    /// Constructs OMIT parameter model.
    pub fn new(
        system: OptomechanicalParams,
        pump_detuning_hz: f64,
        intracavity_photons: f64,
    ) -> Self {
        Self {
            system,
            pump_detuning_hz,
            intracavity_photons,
        }
    }

    /// Standard OMIT setup with resonant red sideband pumping.
    pub fn standard_nanobeam_omit() -> Self {
        let system = OptomechanicalParams::standard_nanobeam();
        let detuning = -system.mechanical_frequency_hz;
        Self::new(system, detuning, 0.25)
    }

    /// Evaluates probe power transmission $|S_{OMIT}(\delta)|^2$ at probe-cavity detuning $\delta = \omega_p - \omega_c$ in Hz:
    /// $$S_{OMIT}(\delta) = 1 - \frac{\kappa_{ex}}{\frac{\kappa}{2} - i \delta + \frac{g^2}{\frac{\gamma_m}{2} - i \delta}}$$
    pub fn probe_transmission(&self, probe_cavity_detuning_hz: f64) -> f64 {
        let kappa = self.system.cavity_linewidth_hz;
        let kappa_ex = self.system.external_coupling_hz;
        let gamma_m = self.system.mechanical_damping_hz;
        let delta = probe_cavity_detuning_hz;
        let g = self.system.effective_coupling_hz(self.intracavity_photons);

        // Denominator 1: gamma_m/2 - i * delta
        let d1_re = gamma_m * 0.5;
        let d1_im = -delta;
        let d1_denom = d1_re * d1_re + d1_im * d1_im;

        // g^2 / (gamma_m/2 - i * delta)
        let term2_re = (g * g * d1_re) / d1_denom.max(1e-18);
        let term2_im = (-g * g * d1_im) / d1_denom.max(1e-18);

        // Overall denominator: kappa/2 - i * delta + term2
        let tot_re = kappa * 0.5 + term2_re;
        let tot_im = -delta + term2_im;
        let tot_denom = tot_re * tot_re + tot_im * tot_im;

        // kappa_ex / tot
        let frac_re = (kappa_ex * tot_re) / tot_denom.max(1e-18);
        let frac_im = (-kappa_ex * tot_im) / tot_denom.max(1e-18);

        // S_OMIT = 1 - frac
        let s_re = 1.0 - frac_re;
        let s_im = -frac_im;

        s_re * s_re + s_im * s_im
    }

    /// OMIT transparency window linewidth $\Gamma_{OMIT} = \gamma_m (1 + \mathcal{C})$ in Hz.
    pub fn transparency_linewidth_hz(&self) -> f64 {
        let c = self.system.cooperativity(self.intracavity_photons);
        self.system.mechanical_damping_hz * (1.0 + c)
    }

    /// Slow light group delay $\tau_g \approx \frac{2}{\gamma_m (1 + \mathcal{C})}$ in seconds.
    pub fn group_delay_s(&self) -> f64 {
        let width = self.transparency_linewidth_hz();
        2.0 / (2.0 * std::f64::consts::PI * width).max(1e-12)
    }
}

/// Parameters for ponderomotive quantum squeezing of light.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PonderomotiveSqueezingParams {
    /// Optomechanical system.
    pub system: OptomechanicalParams,
    /// Mean intracavity photon number $\bar{n}_{cav}$.
    pub intracavity_photons: f64,
    /// Cryogenic bath temperature in Kelvin.
    pub bath_temperature_k: f64,
}

impl PonderomotiveSqueezingParams {
    /// Constructs ponderomotive squeezing parameters.
    pub fn new(
        system: OptomechanicalParams,
        intracavity_photons: f64,
        bath_temperature_k: f64,
    ) -> Self {
        Self {
            system,
            intracavity_photons,
            bath_temperature_k,
        }
    }

    /// Standard setup for $> 5\text{ dB}$ light squeezing at 20 mK.
    pub fn standard_squeezed_source() -> Self {
        let system = OptomechanicalParams::standard_nanobeam();
        Self::new(system, 5_000.0, 0.02)
    }

    /// Kimble optomechanical interaction factor $\mathcal{K}(\omega)$ at sideband frequency $\omega$:
    /// In the high-cooperativity regime:
    /// $$\mathcal{K} \approx \frac{4 \mathcal{C} \kappa}{\kappa + 4 \Omega_m}$$
    pub fn kimble_backaction_factor(&self) -> f64 {
        let c = self.system.cooperativity(self.intracavity_photons);
        let kappa = self.system.cavity_linewidth_hz;
        let omega_m = self.system.mechanical_frequency_hz;
        // Dimensionless back-action parameter
        (4.0 * c * kappa) / (kappa + 2.0 * omega_m).max(1.0)
    }

    /// Minimum output quadrature variance $V_{min} = 1 + \frac{\mathcal{K}^2}{2} - \frac{\mathcal{K}}{2} \sqrt{\mathcal{K}^2 + 4} + V_{th}$:
    pub fn minimum_quadrature_variance(&self) -> f64 {
        let k = self.kimble_backaction_factor();
        let n_th = self
            .system
            .thermal_phonon_occupancy(self.bath_temperature_k);
        let c = self.system.cooperativity(self.intracavity_photons);

        // Shot noise + radiation pressure quantum backaction
        let quantum_var = 1.0 + 0.5 * k * k - 0.5 * k * (k * k + 4.0).sqrt();
        // Thermal degradation term: 2 * n_th / C
        let thermal_excess = (2.0 * n_th) / (1.0 + c).max(1.0);

        (quantum_var + thermal_excess).max(0.01)
    }

    /// Quantum squeezing depth in decibels below the shot noise limit (SQL = 0 dB):
    /// $$S_{dB} = -10 \log_{10}(V_{min})$$
    pub fn squeezing_depth_db(&self) -> f64 {
        let v_min = self.minimum_quadrature_variance();
        if v_min >= 1.0 {
            0.0
        } else {
            -10.0 * v_min.log10()
        }
    }

    /// Optimal quadrature angle $\theta_{opt} = \frac{1}{2} \operatorname{arccot}(\mathcal{K}/2)$ in radians.
    pub fn optimal_quadrature_angle_rad(&self) -> f64 {
        let k = self.kimble_backaction_factor();
        0.5 * (2.0 / k.max(1e-6)).atan()
    }

    /// Logarithmic negativity entanglement $E_N$ between optical and phononic modes:
    pub fn logarithmic_negativity_entanglement(&self) -> f64 {
        let c = self.system.cooperativity(self.intracavity_photons);
        let n_th = self
            .system
            .thermal_phonon_occupancy(self.bath_temperature_k);
        if c < 1.0 || n_th > 100.0 {
            0.0
        } else {
            // Logarithmic negativity scales logarithmically with cooperativity in low-thermal regime
            let arg = (c / (2.0 * n_th + 1.0)).max(1.0);
            (0.5 * arg.ln()).max(0.0)
        }
    }
}
