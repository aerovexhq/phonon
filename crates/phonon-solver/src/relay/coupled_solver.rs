//! Multi-physics coupled transient solver for atomic relays and electrochemical switches.
//!
//! Monolithically or tightly couples:
//! 1. Electrical nodal conductance stamping and terminal currents.
//! 2. Mechanical 2nd-order cantilever beam dynamics (damping, spring, adhesion, electrostatic force).
//! 3. Butler-Volmer ionic hopping and redox filament kinetics.
//! 4. Dynamic Joule self-heating (\(P = I^2 R\)) and thermal dissipation.

use phonon_models::relay::{
    AtomicRelayModel, AtomicRelayParameters, EcmCellModel, EcmCellParameters, RelayContactState,
};

/// Simulation output report for a coupled electro-mechanical-thermal transient run.
#[derive(Debug, Clone)]
pub struct CoupledRelayTransientResult {
    /// Time sequence [s]
    pub time_s: Vec<f64>,
    /// Terminal voltage waveform [V]
    pub voltage_v: Vec<f64>,
    /// Device current waveform [A]
    pub current_a: Vec<f64>,
    /// Cantilever displacement waveform [m]
    pub displacement_m: Vec<f64>,
    /// Device junction temperature [K]
    pub temperature_k: Vec<f64>,
    /// Equivalent electrical conductance [S]
    pub conductance_s: Vec<f64>,
    /// Measured mechanical pull-in switching delay [s]
    pub pull_in_delay_s: Option<f64>,
    /// Peak temperature reached [K]
    pub peak_temperature_k: f64,
    /// Total energy dissipated [J]
    pub total_energy_j: f64,
}

/// Coupled multi-physics solver configuration.
#[derive(Debug, Clone)]
pub struct CoupledSolverConfig {
    /// Ambient reference temperature [K]
    pub ambient_temp_k: f64,
    /// Thermal resistance to substrate R_th [K/W] (e.g. 5e4 K/W)
    pub thermal_resistance_k_per_w: f64,
    /// Thermal capacitance C_th [J/K] (e.g. 1e-15 J/K for nanoscale device)
    pub thermal_capacitance_j_per_k: f64,
}

impl Default for CoupledSolverConfig {
    fn default() -> Self {
        Self {
            ambient_temp_k: 300.0,
            thermal_resistance_k_per_w: 5.0e4,
            thermal_capacitance_j_per_k: 1.0e-15,
        }
    }
}

/// Multi-physics transient solver for atomic relays and electrochemical switches.
#[derive(Debug, Clone, Default)]
pub struct CoupledRelaySolver {
    pub config: CoupledSolverConfig,
}

impl CoupledRelaySolver {
    /// Creates a new solver with specified thermal configuration.
    pub fn new(config: CoupledSolverConfig) -> Self {
        Self { config }
    }

    /// Simulates electro-mechanical transient response of a 3-terminal atomic relay
    /// driven by a voltage pulse V_gate(t).
    pub fn simulate_atomic_relay_transient(
        &self,
        params: AtomicRelayParameters,
        v_pulse_v: f64,
        t_stop_s: f64,
        dt_s: f64,
    ) -> CoupledRelayTransientResult {
        let mut relay = AtomicRelayModel::new(params);
        let mut temp_k = self.config.ambient_temp_k;

        let num_steps = (t_stop_s / dt_s).ceil() as usize;
        let mut time_s = Vec::with_capacity(num_steps);
        let mut voltage_v = Vec::with_capacity(num_steps);
        let mut current_a = Vec::with_capacity(num_steps);
        let mut displacement_m = Vec::with_capacity(num_steps);
        let mut temperature_k = Vec::with_capacity(num_steps);
        let mut conductance_s = Vec::with_capacity(num_steps);

        let mut pull_in_delay_s: Option<f64> = None;
        let mut peak_temp_k = temp_k;
        let mut total_energy_j = 0.0;

        let mut t = 0.0;
        for _ in 0..num_steps {
            // Apply gate pulse after brief 5ns setup
            let v_gate = if t >= 5.0e-9 { v_pulse_v } else { 0.0 };

            // Advance mechanical dynamics
            relay.step(v_gate, dt_s);

            let r_device = relay.resistance_ohm();
            let g_device = 1.0 / r_device.max(1e-18);
            let v_ds = 0.40; // 400 mV drain-source bias
            let i_device = v_ds * g_device;

            // Joule dissipation P = I * V
            let p_joule = i_device * v_ds;
            total_energy_j += p_joule * dt_s;

            // Thermal dynamic state: dT/dt = (P_joule - (T - T_amb)/R_th) / C_th
            let q_diss =
                (temp_k - self.config.ambient_temp_k) / self.config.thermal_resistance_k_per_w;
            let d_temp = ((p_joule - q_diss) / self.config.thermal_capacitance_j_per_k) * dt_s;
            temp_k = (temp_k + d_temp).max(self.config.ambient_temp_k);
            if temp_k > peak_temp_k {
                peak_temp_k = temp_k;
            }

            // Check pull-in delay
            if relay.state.contact_state == RelayContactState::Closed && pull_in_delay_s.is_none() {
                pull_in_delay_s = Some(t);
            }

            time_s.push(t);
            voltage_v.push(v_gate);
            current_a.push(i_device);
            displacement_m.push(relay.state.displacement_m);
            temperature_k.push(temp_k);
            conductance_s.push(g_device);

            t += dt_s;
        }

        CoupledRelayTransientResult {
            time_s,
            voltage_v,
            current_a,
            displacement_m,
            temperature_k,
            conductance_s,
            pull_in_delay_s,
            peak_temperature_k: peak_temp_k,
            total_energy_j,
        }
    }

    /// Simulates electrochemical metallization (ECM) cell transient switching with Joule heating.
    pub fn simulate_ecm_transient(
        &self,
        params: EcmCellParameters,
        v_prog_v: f64,
        t_stop_s: f64,
        dt_s: f64,
    ) -> CoupledRelayTransientResult {
        let mut ecm = EcmCellModel::new(params);
        let mut temp_k = self.config.ambient_temp_k;

        let num_steps = (t_stop_s / dt_s).ceil() as usize;
        let mut time_s = Vec::with_capacity(num_steps);
        let mut voltage_v = Vec::with_capacity(num_steps);
        let mut current_a = Vec::with_capacity(num_steps);
        let mut displacement_m = Vec::with_capacity(num_steps);
        let mut temperature_k = Vec::with_capacity(num_steps);
        let mut conductance_s = Vec::with_capacity(num_steps);

        let mut pull_in_delay_s: Option<f64> = None;
        let mut peak_temp_k = temp_k;
        let mut total_energy_j = 0.0;

        let mut t = 0.0;
        for _ in 0..num_steps {
            ecm.state.temp_k = temp_k;
            ecm.step(v_prog_v, dt_s);

            let g_device = ecm.conductance_s();
            let i_device = v_prog_v * g_device;

            // Joule heating
            let p_joule = i_device * v_prog_v;
            total_energy_j += p_joule * dt_s;

            let q_diss =
                (temp_k - self.config.ambient_temp_k) / self.config.thermal_resistance_k_per_w;
            let d_temp = ((p_joule - q_diss) / self.config.thermal_capacitance_j_per_k) * dt_s;
            temp_k = (temp_k + d_temp).max(self.config.ambient_temp_k);
            if temp_k > peak_temp_k {
                peak_temp_k = temp_k;
            }

            if ecm.state.filament_length_ratio >= 0.99 && pull_in_delay_s.is_none() {
                pull_in_delay_s = Some(t);
            }

            time_s.push(t);
            voltage_v.push(v_prog_v);
            current_a.push(i_device);
            displacement_m.push(ecm.state.filament_length_ratio * ecm.params.gap_thickness_m);
            temperature_k.push(temp_k);
            conductance_s.push(g_device);

            t += dt_s;
        }

        CoupledRelayTransientResult {
            time_s,
            voltage_v,
            current_a,
            displacement_m,
            temperature_k,
            conductance_s,
            pull_in_delay_s,
            peak_temperature_k: peak_temp_k,
            total_energy_j,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coupled_atomic_relay_transient_switching() {
        let solver = CoupledRelaySolver::default();
        let params = AtomicRelayParameters {
            initial_gap_m: 3.0e-9,
            contact_gap_m: 1.5e-9,
            gate_area_m2: 100.0e-9 * 300.0e-9,
            spring_constant_n_per_m: 0.3,
            ..Default::default()
        };

        // Apply 120 mV gate pulse (> V_pi)
        let res = solver.simulate_atomic_relay_transient(params, 0.12, 100.0e-9, 0.5e-9);

        // Pull-in delay should be detected within 100 ns
        assert!(res.pull_in_delay_s.is_some());
        let delay = res.pull_in_delay_s.unwrap();
        assert!(delay > 5.0e-9 && delay < 90.0e-9);

        // Conductance must switch abruptly
        let initial_g = res.conductance_s[0];
        let final_g = *res.conductance_s.last().unwrap();
        assert!(final_g > initial_g * 1.0e10);
    }
}
