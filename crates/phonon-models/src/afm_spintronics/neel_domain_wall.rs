//! Relativistic Néel vector domain wall dynamics, sub-picosecond synaptic memristors,
//! and non-reciprocal Terahertz magnon diodes.

use super::afm_params::AfmMaterialParams;

/// Parameters for Néel vector domain wall dynamics and spintronic memristors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeelDomainWallParams {
    /// Domain wall width parameter $\Delta_{\mathrm{DW}}$ in meters (nominal $2.0 - 5.0\text{ nm}$).
    pub domain_wall_width_m: f64,
    /// Synaptic nanogap track length $L_{\mathrm{cell}}$ in meters (nominal $5.0 - 20.0\text{ nm}$).
    pub track_length_m: f64,
    /// Spin-Hall / staggered spin-orbit torque efficiency $\theta_{\mathrm{SOT}}$ (nominal $0.20 - 0.40$).
    pub sot_torque_efficiency: f64,
    /// Minimum (anti-parallel) synaptic conductance $G_{\mathrm{min}}$ in Siemens ($S$).
    pub min_conductance_s: f64,
    /// Maximum (parallel) synaptic conductance $G_{\mathrm{max}}$ in Siemens ($S$).
    pub max_conductance_s: f64,
    /// Magnon diode forward-to-reverse DMI isolation asymmetry factor.
    pub dmi_diode_asymmetry: f64,
}

impl Default for NeelDomainWallParams {
    fn default() -> Self {
        Self {
            domain_wall_width_m: 3.0e-9,
            track_length_m: 10.0e-9,
            sot_torque_efficiency: 0.30,
            min_conductance_s: 1.0e-5,
            max_conductance_s: 1.0e-4,
            dmi_diode_asymmetry: 0.85,
        }
    }
}

/// Evaluated metrics for Néel domain wall transit and memristive plastic updates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeelTransitMetrics {
    /// Relativistic domain wall steady-state velocity $v_{\mathrm{DW}}$ in $\text{m/s}$ ($> 5000\text{ m/s}$).
    pub domain_wall_velocity_m_s: f64,
    /// Transit time across the synaptic track $\tau_{\mathrm{transit}}$ in picoseconds ($\text{ps}$) ($< 1.0\text{ ps}$).
    pub transit_time_ps: f64,
    /// Synaptic conductance state $G$ in Siemens ($S$).
    pub conductance_s: f64,
    /// Magnon diode rectification contrast in decibels $\mathcal{R}_{\mathrm{diode}} \ge 15.0\text{ dB}$.
    pub diode_rectification_db: f64,
}

impl NeelDomainWallParams {
    /// Creates new Néel domain wall parameters.
    pub fn new(domain_wall_width_m: f64, track_length_m: f64, sot_torque_efficiency: f64) -> Self {
        Self {
            domain_wall_width_m: domain_wall_width_m.max(1.0e-9),
            track_length_m: track_length_m.max(2.0e-9),
            sot_torque_efficiency: sot_torque_efficiency.clamp(0.05, 0.9),
            min_conductance_s: 1.0e-5,
            max_conductance_s: 1.0e-4,
            dmi_diode_asymmetry: 0.85,
        }
    }

    /// Evaluates the relativistic steady-state domain wall velocity $v_{\mathrm{DW}}$ under current drive $J_e$ in $\text{A/m}^2$:
    /// In antiferromagnets, absence of Walker breakdown allows reaching relativistic spin-wave group velocities:
    /// $$v_{\mathrm{DW}} = \frac{\mu_{\mathrm{DW}} J_e}{\sqrt{1 + (\mu_{\mathrm{DW}} J_e / v_{\mathrm{ex}})^2}}$$
    pub fn evaluate_dw_velocity(
        &self,
        afm_params: &AfmMaterialParams,
        current_density_a_m2: f64,
    ) -> f64 {
        let v_max = afm_params.exchange_velocity_m_s();
        // Domain wall mobility ~ 1.2e-7 m/s per (A/m^2) * sot_efficiency
        let mu_dw = 1.2e-7 * self.sot_torque_efficiency;
        let v_lin = mu_dw * current_density_a_m2.abs();
        let lorentz_factor = 1.0 + (v_lin / v_max).powi(2);
        v_lin / lorentz_factor.sqrt()
    }

    /// Evaluates transit metrics across the synaptic track.
    pub fn evaluate_transit(
        &self,
        afm_params: &AfmMaterialParams,
        current_density_a_m2: f64,
        normalized_position: f64,
    ) -> NeelTransitMetrics {
        let v_dw = self
            .evaluate_dw_velocity(afm_params, current_density_a_m2)
            .max(10.0);
        let tau_s = self.track_length_m / v_dw;
        let tau_ps = tau_s * 1.0e12;

        let pos = normalized_position.clamp(0.0, 1.0);
        let g = self.min_conductance_s + (self.max_conductance_s - self.min_conductance_s) * pos;

        // Magnon diode rectification via DMI non-reciprocal spin-wave transmission:
        // Forward transmission T_+ = 0.95, Reverse T_- = 0.95 * (1.0 - asymmetry)
        let forward_t = 0.95;
        let reverse_t = (0.95 * (1.0 - self.dmi_diode_asymmetry)).max(1e-4);
        let rectification_db = 20.0 * (forward_t / reverse_t).log10();

        NeelTransitMetrics {
            domain_wall_velocity_m_s: v_dw,
            transit_time_ps: tau_ps,
            conductance_s: g,
            diode_rectification_db: rectification_db,
        }
    }
}
