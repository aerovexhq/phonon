//! Non-linear Maxwell-Bloch carrier-photon rate equations and topological single-mode lasing.
//!
//! Formulates:
//! - Multi-mode carrier-photon rate equations:
//!   $$\frac{d N_j}{dt} = \frac{I_{pump,j}}{q V_j} - \frac{N_j}{\tau_{nr}} - \sum_m \frac{g_m (N_j - N_{tr}) |E_{m,j}|^2}{1 + \epsilon_{sat} \sum_k |E_{k,j}|^2}$$
//!   $$\frac{d E_m}{dt} = \frac{1}{2} \left[ \sum_j \Gamma_j g_m (N_j - N_{tr}) - \kappa_m \right] E_m + i (\omega_m - \omega_c) E_m + S_{sp,m}$$
//! - Side-Mode Suppression Ratio (SMSR):
//!   $$\text{SMSR} = 10 \log_{10} \left( \frac{P_{edge}}{\max_{m \ne edge} P_m} \right)$$

use phonon_core::constants::ELEMENTARY_CHARGE;

/// Physical parameters for the active gain medium and laser cavity dynamics.
#[derive(Debug, Clone, PartialEq)]
pub struct LaserRateEquationParams {
    /// Number of coupled resonator cavities.
    pub num_resonators: usize,
    /// Non-radiative carrier lifetime $\tau_{nr}$ in seconds (typically ~ 1 ns).
    pub carrier_lifetime_s: f64,
    /// Active region volume $V_{act}$ in $\text{m}^3$.
    pub active_volume_m3: f64,
    /// Differential optical gain coefficient $g_m$ in $\text{m}^3/\text{s}$.
    pub differential_gain_m3_per_s: f64,
    /// Transparency carrier density $N_{tr}$ in $\text{m}^{-3}$.
    pub transparency_density_per_m3: f64,
    /// Gain compression / saturation coefficient $\epsilon_{sat}$.
    pub gain_saturation_factor: f64,
    /// Optical confinement factor $\Gamma$ in active gain medium.
    pub confinement_factor: f64,
    /// Total cavity photon loss rate $\kappa_m / (2\pi)$ in Hz.
    pub cavity_loss_rate_hz: f64,
    /// Spontaneous emission coupling factor $\beta_{sp}$.
    pub spontaneous_emission_beta: f64,
    /// Pump injection current $I_{pump}$ in Amperes per cavity.
    pub pump_current_amperes: f64,
}

impl LaserRateEquationParams {
    /// Creates a new laser parameter configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        num_resonators: usize,
        carrier_lifetime_s: f64,
        active_volume_m3: f64,
        differential_gain_m3_per_s: f64,
        transparency_density_per_m3: f64,
        gain_saturation_factor: f64,
        confinement_factor: f64,
        cavity_loss_rate_hz: f64,
        spontaneous_emission_beta: f64,
        pump_current_amperes: f64,
    ) -> Self {
        Self {
            num_resonators,
            carrier_lifetime_s,
            active_volume_m3,
            differential_gain_m3_per_s,
            transparency_density_per_m3,
            gain_saturation_factor,
            confinement_factor,
            cavity_loss_rate_hz,
            spontaneous_emission_beta,
            pump_current_amperes,
        }
    }

    /// Standard InGaAsP/InP topological microring laser array configuration:
    /// 20 coupled resonators, $\tau_{nr} = 1.0\text{ ns}$, $I_{pump} = 15.0\text{ mA}$.
    pub fn standard_topological_array() -> Self {
        Self::new(
            20, 1.0e-9, 2.0e-17, 2.5e-12, 1.0e24, 1.0e-23, 0.30, 20.0e9, 1.0e-4, 15.0e-3,
        )
    }

    /// Threshold carrier injection current $I_{th}$ in Amperes:
    /// $$I_{th} = \frac{q V}{\tau_{nr}} \left( N_{tr} + \frac{2\pi \kappa_m}{\Gamma g_m} \right)$$
    pub fn threshold_current_amperes(&self) -> f64 {
        let q = ELEMENTARY_CHARGE;
        let v = self.active_volume_m3;
        let tau = self.carrier_lifetime_s;
        let kappa_rad = 2.0 * std::f64::consts::PI * self.cavity_loss_rate_hz;
        let n_th = self.transparency_density_per_m3
            + kappa_rad / (self.confinement_factor * self.differential_gain_m3_per_s).max(1e-20);
        (q * v * n_th) / tau.max(1e-12)
    }

    /// Evaluates if the laser is pumped above threshold ($I_{pump} > I_{th}$).
    pub fn is_above_threshold(&self) -> bool {
        self.pump_current_amperes > self.threshold_current_amperes()
    }

    /// Slope efficiency above threshold (optical output power per Ampere of injection current).
    pub fn slope_efficiency_w_per_a(&self) -> f64 {
        let eta_int = 0.85; // Internal quantum efficiency
        let hbar_omega = 1.054571817e-34 * 2.0 * std::f64::consts::PI * 193.4e12; // ~1550 nm photon
        eta_int * (hbar_omega / ELEMENTARY_CHARGE)
    }
}
