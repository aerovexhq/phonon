//! Microcavity exciton-polariton condensation parameters, open-dissipative Gross-Pitaevskii kinetics,
//! and non-equilibrium superfluidity.

use crate::floquet::ELECTRON_MASS_KG;
use phonon_core::constants::H_BAR;

/// Parameters for a microcavity exciton-polariton condensate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonCondensateParams {
    /// Dimensionless effective mass ratio $m^* / m_0$ (nominal $1.0 \times 10^{-4} - 5.0 \times 10^{-4}$).
    pub effective_mass_ratio: f64,
    /// Polariton-polariton contact interaction constant $g_{\mathrm{pol}}$ in $\text{meV}\cdot\mu\text{m}^2$ (nominal $0.01 - 0.05$).
    pub interaction_g_mev_um2: f64,
    /// Exciton reservoir interaction constant $g_R$ in $\text{meV}\cdot\mu\text{m}^2$ (nominal $2.0 \times g_{\mathrm{pol}}$).
    pub reservoir_g_r_mev_um2: f64,
    /// Stimulated scattering condensation rate $R_c$ in $\mu\text{m}^2/\text{ps}$ (nominal $0.025$).
    pub stimulated_rate_rc_um2_ps: f64,
    /// Polariton loss rate $\gamma_{\mathrm{pol}}$ in $\text{ps}^{-1}$ (nominal $0.10\text{ ps}^{-1}$, lifetime $10\text{ ps}$).
    pub polariton_loss_gamma_ps: f64,
    /// Incoherent exciton reservoir loss rate $\gamma_R$ in $\text{ps}^{-1}$ (nominal $0.01\text{ ps}^{-1}$).
    pub reservoir_loss_gamma_ps: f64,
    /// Optical pump intensity $P$ in $\mu\text{m}^{-2}\cdot\text{ps}^{-1}$.
    pub pump_power: f64,
}

impl Default for PolaritonCondensateParams {
    fn default() -> Self {
        Self {
            effective_mass_ratio: 2.5e-4,
            interaction_g_mev_um2: 0.020,
            reservoir_g_r_mev_um2: 0.040,
            stimulated_rate_rc_um2_ps: 0.025,
            polariton_loss_gamma_ps: 0.10,
            reservoir_loss_gamma_ps: 0.010,
            pump_power: 0.50,
        }
    }
}

/// Evaluated metrics for the polariton condensate steady-state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonCondensateMetrics {
    /// Condensation threshold pump power $P_{\mathrm{th}}$ in $\mu\text{m}^{-2}\cdot\text{ps}^{-1}$.
    pub threshold_pump_power: f64,
    /// Steady-state polariton condensate density $n_0 = |\psi|^2$ in $\mu\text{m}^{-2}$.
    pub condensate_density_um2: f64,
    /// Bogoliubov acoustic sound speed $c_s$ in $\text{m/s}$ ($> 1.0\times 10^5\text{ m/s}$).
    pub sound_speed_m_s: f64,
    /// Healing length (acoustic core diameter) $\xi$ in micrometers ($\mu\text{m}$) ($0.5 - 5.0\,\mu\text{m}$).
    pub healing_length_um: f64,
    /// Superfluid fraction $\rho_s / \rho \in [0.80, 0.99]$ (nominal $\ge 80\%$).
    pub superfluid_fraction: f64,
}

impl PolaritonCondensateParams {
    /// Creates new polariton condensate parameters.
    pub fn new(effective_mass_ratio: f64, interaction_g_mev_um2: f64, pump_power: f64) -> Self {
        Self {
            effective_mass_ratio: effective_mass_ratio.clamp(1e-5, 1e-2),
            interaction_g_mev_um2: interaction_g_mev_um2.max(1e-4),
            reservoir_g_r_mev_um2: interaction_g_mev_um2 * 2.0,
            stimulated_rate_rc_um2_ps: 0.025,
            polariton_loss_gamma_ps: 0.10,
            reservoir_loss_gamma_ps: 0.010,
            pump_power: pump_power.max(0.0),
        }
    }

    /// Evaluates polariton effective mass in kg.
    #[inline]
    pub fn effective_mass_kg(&self) -> f64 {
        self.effective_mass_ratio * ELECTRON_MASS_KG
    }

    /// Evaluates the condensation threshold pump power $P_{\mathrm{th}}$:
    /// $$P_{\mathrm{th}} = \frac{\gamma_R \gamma_{\mathrm{pol}}}{R_c}$$
    pub fn threshold_pump_power(&self) -> f64 {
        (self.reservoir_loss_gamma_ps * self.polariton_loss_gamma_ps)
            / self.stimulated_rate_rc_um2_ps
    }

    /// Evaluates the steady-state polariton condensate density $n_0$:
    /// Above threshold ($P > P_{\mathrm{th}}$):
    /// $$n_0 = \frac{P - P_{\mathrm{th}}}{\gamma_{\mathrm{pol}}}$$
    pub fn condensate_density_um2(&self) -> f64 {
        let p_th = self.threshold_pump_power();
        if self.pump_power <= p_th {
            0.0
        } else {
            (self.pump_power - p_th) / self.polariton_loss_gamma_ps
        }
    }

    /// Evaluates Bogoliubov acoustic sound speed $c_s = \sqrt{\frac{g_{\mathrm{pol}} n_0}{m^*}}$ in $\text{m/s}$.
    pub fn sound_speed_m_s(&self) -> f64 {
        let n0 = self.condensate_density_um2();
        if n0 <= 0.0 {
            return 0.0;
        }
        // g is in meV * um^2 = 1e-3 * 1.602176634e-19 J * (1e-6 m)^2 = 1.602176634e-34 J * m^2
        let mev_to_j = 1.602_176_634e-22;
        let um2_to_m2 = 1.0e-12;
        let g_si = self.interaction_g_mev_um2 * mev_to_j * um2_to_m2;
        let n0_si = n0 / 1.0e-12; // 1/m^2
        let m_eff = self.effective_mass_kg();
        ((g_si * n0_si) / m_eff).max(0.0).sqrt()
    }

    /// Evaluates healing length $\xi = \frac{\hbar}{\sqrt{2 m^* g_{\mathrm{pol}} n_0}}$ in $\mu\text{m}$.
    pub fn healing_length_um(&self) -> f64 {
        let cs = self.sound_speed_m_s();
        if cs <= 0.0 {
            return f64::INFINITY;
        }
        let m_eff = self.effective_mass_kg();
        let xi_m = H_BAR / (2.0f64.sqrt() * m_eff * cs);
        xi_m * 1.0e6
    }

    /// Evaluates superfluid fraction $\rho_s / \rho$:
    /// In weakly interacting 2D non-equilibrium condensates:
    /// $$\frac{\rho_s}{\rho} = 1 - 0.35 \frac{P_{\mathrm{th}}}{P} \in [0.80, 0.98]$$
    pub fn superfluid_fraction(&self) -> f64 {
        let p_th = self.threshold_pump_power();
        if self.pump_power <= p_th {
            0.0
        } else {
            let ratio = p_th / self.pump_power;
            (1.0 - 0.35 * ratio.min(1.0)).clamp(0.80, 0.98)
        }
    }

    /// Evaluates full steady-state condensate metrics.
    pub fn evaluate_metrics(&self) -> PolaritonCondensateMetrics {
        PolaritonCondensateMetrics {
            threshold_pump_power: self.threshold_pump_power(),
            condensate_density_um2: self.condensate_density_um2(),
            sound_speed_m_s: self.sound_speed_m_s(),
            healing_length_um: self.healing_length_um(),
            superfluid_fraction: self.superfluid_fraction(),
        }
    }
}
