//! Solvers for chiral phonon-magnon spin Seebeck cascades,
//! angular momentum transfer, and cryogenic phononic thermocells.

use phonon_models::chiral_spin_seebeck::{ChiralSpinSeebeckMetrics, ChiralSpinSeebeckParams};

/// Solver for chiral phonon-magnon spin Seebeck cascades and thermocells.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralSpinSeebeckSolver {
    pub params: ChiralSpinSeebeckParams,
}

impl ChiralSpinSeebeckSolver {
    /// Creates a new chiral spin Seebeck solver instance.
    pub fn new(params: ChiralSpinSeebeckParams) -> Self {
        Self { params }
    }

    /// Evaluates the temperature difference $\Delta T = T_{\mathrm{hot}} - T_{\mathrm{cold}}$ in Kelvin.
    pub fn compute_temperature_difference_k(&self) -> f64 {
        (self.params.temp_hot_k - self.params.temp_cold_k).max(1.0e-4)
    }

    /// Evaluates the Carnot thermodynamic efficiency limit $\eta_{\mathrm{Carnot}} = \frac{\Delta T}{T_{\mathrm{hot}}}$.
    pub fn compute_carnot_efficiency(&self) -> f64 {
        let delta_t = self.compute_temperature_difference_k();
        (delta_t / self.params.temp_hot_k.max(1.0e-3)).clamp(0.0, 1.0)
    }

    /// Evaluates the thermal rectification ratio $\mathcal{R}_{\mathrm{th}} = \kappa_+ / \kappa_-$.
    pub fn compute_thermal_rectification_ratio(&self) -> f64 {
        let k_plus = self.params.forward_thermal_conductance_w_k;
        let k_minus = self.params.backward_thermal_conductance_w_k.max(1.0e-12);
        (k_plus / k_minus).max(1.0)
    }

    /// Evaluates the effective spin Seebeck coefficient $S_{\mathrm{SSE}}$ in $\mu\text{V/K}$.
    pub fn compute_spin_seebeck_coefficient_uv_per_k(&self) -> f64 {
        let p = &self.params;
        let g_norm = p.spin_mixing_conductance_m2 / 1.5e19;
        let theta_norm = p.spin_hall_angle / 0.12;
        let rho_norm = p.detector_resistivity_ohm_m / 2.5e-7;
        let p_circ = p.chiral_phonon_polarization.clamp(0.1, 1.0);
        let g_sp_norm = p.spin_phonon_coupling_ghz / 1.2;
        let l_norm = p.detector_length_um / 20.0;

        let base_s0 = 24.0; // Nominal Seebeck coefficient: 24.0 uV/K
        base_s0 * g_norm * theta_norm * rho_norm * p_circ * g_sp_norm * l_norm
    }

    /// Evaluates the Inverse Spin Hall Effect (ISHE) generated voltage $V_{\mathrm{ISHE}}$ in $\mu\text{V}$.
    pub fn compute_spin_seebeck_voltage_uv(&self) -> f64 {
        let s_sse = self.compute_spin_seebeck_coefficient_uv_per_k();
        let delta_t = self.compute_temperature_difference_k();
        (s_sse * delta_t).max(1.0e-6)
    }

    /// Evaluates the injected interfacial spin current density $J_s$ in $\text{A/m}^2$.
    pub fn compute_spin_current_density_a_m2(&self) -> f64 {
        let v_uv = self.compute_spin_seebeck_voltage_uv();
        let v_volts = v_uv * 1.0e-6;
        let theta = self.params.spin_hall_angle.max(0.01);
        let rho = self.params.detector_resistivity_ohm_m.max(1.0e-9);
        let l_meters = (self.params.detector_length_um * 1.0e-6).max(1.0e-7);

        // V_ISHE = theta * rho * J_s * L => J_s = V_ISHE / (theta * rho * L)
        v_volts / (theta * rho * l_meters)
    }

    /// Evaluates forward and backward heat currents $(J_{\mathrm{heat}, +}, J_{\mathrm{heat}, -})$ in $\mu\text{W}$.
    pub fn compute_heat_currents_uw(&self) -> (f64, f64) {
        let delta_t = self.compute_temperature_difference_k();
        let j_plus = self.params.forward_thermal_conductance_w_k * delta_t * 1.0e6;
        let j_minus = self.params.backward_thermal_conductance_w_k * delta_t * 1.0e6;
        (j_plus, j_minus)
    }

    /// Evaluates the thermocell electrical power output $P_{\mathrm{gen}} = \frac{V_{\mathrm{ISHE}}^2}{4 R_{\mathrm{load}}}$ in picowatts ($\text{pW}$).
    pub fn compute_thermocell_power_output_pw(&self) -> f64 {
        let v_volts = self.compute_spin_seebeck_voltage_uv() * 1.0e-6;
        let r_load = self.params.thermocell_internal_resistance_ohm.max(1.0);
        let p_watts = (v_volts * v_volts) / (4.0 * r_load);
        p_watts * 1.0e12
    }

    /// Evaluates the thermocell heat-to-electricity conversion efficiency $\eta_{\mathrm{th}} \times 100\%$.
    pub fn compute_thermocell_efficiency_percent(&self) -> f64 {
        let p_watts = self.compute_thermocell_power_output_pw() * 1.0e-12;
        let (j_plus_uw, _) = self.compute_heat_currents_uw();
        let q_watts = (j_plus_uw * 1.0e-6).max(1.0e-15);
        (p_watts / q_watts) * 100.0
    }

    /// Solves the full chiral phonon-magnon spin Seebeck metrics.
    pub fn solve(&self) -> ChiralSpinSeebeckMetrics {
        let r_th = self.compute_thermal_rectification_ratio();
        let v_uv = self.compute_spin_seebeck_voltage_uv();
        let (j_plus, j_minus) = self.compute_heat_currents_uw();
        let p_pw = self.compute_thermocell_power_output_pw();
        let carnot_pct = self.compute_carnot_efficiency() * 100.0;
        let eff_pct = self.compute_thermocell_efficiency_percent();
        let j_s = self.compute_spin_current_density_a_m2();

        ChiralSpinSeebeckMetrics {
            thermal_rectification_ratio: r_th,
            spin_seebeck_voltage_uv: v_uv,
            forward_heat_current_uw: j_plus,
            backward_heat_current_uw: j_minus,
            thermocell_power_output_pw: p_pw,
            carnot_efficiency_percent: carnot_pct,
            thermocell_efficiency_percent: eff_pct,
            spin_current_density_a_m2: j_s,
        }
    }
}
