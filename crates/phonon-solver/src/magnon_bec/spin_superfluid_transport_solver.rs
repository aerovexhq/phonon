//! Hydrodynamic Spin Superfluid Transport Solver.
//!
//! Solves spin superfluid velocity, Landau critical margins,
//! non-local spin injection and detection voltages via SHE/ISHE,
//! and evaluates long-range algebraic (1/L) vs exponential (exp(-L/lambda)) transport.

use phonon_models::magnon_bec::{HeavyMetalElectrode, SpinSuperfluidChannel};

/// Results from hydrodynamic spin superfluid transport analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct SpinSuperfluidTransportResult {
    /// Channel length L in meters.
    pub channel_length_m: f64,
    /// Actual spin superfluid velocity v_s in m/s.
    pub superfluid_velocity_m_per_s: f64,
    /// Landau critical velocity v_c in m/s.
    pub landau_critical_velocity_m_per_s: f64,
    /// Flag indicating whether v_s is safely below v_c (v_s < v_c).
    pub is_below_critical_velocity: bool,
    /// Dissipationless spin current density J_s in J/m^2.
    pub spin_current_density_j_per_m2: f64,
    /// Superfluid transmission factor T_superfluid.
    pub superfluid_transmission: f64,
    /// Diffusive magnon transmission factor T_diffusive.
    pub diffusive_transmission: f64,
    /// Transmission advantage ratio T_superfluid / T_diffusive.
    pub transmission_advantage: f64,
    /// Non-local ISHE detector voltage in the spin superfluid regime (Volts).
    pub non_local_voltage_superfluid_v: f64,
    /// Non-local ISHE detector voltage in the normal diffusive regime (Volts).
    pub non_local_voltage_diffusive_v: f64,
    /// Non-local transresistance R_nl = V_nl / I_inj in Ohms.
    pub non_local_resistance_superfluid_ohm: f64,
}

/// Hydrodynamic transport solver for spin superfluid channels.
#[derive(Debug, Default, Clone)]
pub struct SpinSuperfluidTransportSolver;

impl SpinSuperfluidTransportSolver {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates spin transport across the channel for a given injector current.
    pub fn solve_transport(
        &self,
        channel: &SpinSuperfluidChannel,
        electrode: &HeavyMetalElectrode,
        injector_current_a: f64,
        phase_gradient_rad_per_m: f64,
    ) -> SpinSuperfluidTransportResult {
        let l = channel.channel_length_m;
        let v_c = channel.landau_critical_velocity_m_per_s();
        let v_s = channel.superfluid_velocity_m_per_s(phase_gradient_rad_per_m);
        let is_below_vc = v_s < v_c;

        let j_s = channel.spin_current_density_j_per_m2(phase_gradient_rad_per_m);

        let t_super = channel.superfluid_transmission_factor(l);
        let t_diff = channel.diffusive_transmission_factor(l);
        let advantage = channel.transmission_advantage_ratio(l);

        let v_nl_super = channel.non_local_voltage_volts(injector_current_a, electrode, true);
        let v_nl_diff = channel.non_local_voltage_volts(injector_current_a, electrode, false);

        let r_nl_super = if injector_current_a.abs() > 1e-12 {
            v_nl_super / injector_current_a
        } else {
            0.0
        };

        SpinSuperfluidTransportResult {
            channel_length_m: l,
            superfluid_velocity_m_per_s: v_s,
            landau_critical_velocity_m_per_s: v_c,
            is_below_critical_velocity: is_below_vc,
            spin_current_density_j_per_m2: j_s,
            superfluid_transmission: t_super,
            diffusive_transmission: t_diff,
            transmission_advantage: advantage,
            non_local_voltage_superfluid_v: v_nl_super,
            non_local_voltage_diffusive_v: v_nl_diff,
            non_local_resistance_superfluid_ohm: r_nl_super,
        }
    }
}
