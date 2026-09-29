//! Transient solver and dynamic Hall routing engine for ultrafast Floquet optical switches.

use phonon_models::floquet_topological::{FloquetOpticalSwitch, FloquetSwitchResponse};

/// Transient time-point snapshot of the Floquet optical switch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetSwitchTransientPoint {
    /// Simulation time in seconds.
    pub time_s: f64,
    /// Optical pulse envelope factor $f(t) \in [0, 1]$.
    pub envelope: f64,
    /// Dynamic mass gap opening in $\text{meV}$.
    pub dynamic_gap_mev: f64,
    /// Dynamic Hall current density $j_y(t)$ in $\text{A/m}$.
    pub hall_current_density_a_m: f64,
    /// Longitudinal current density $j_x(t)$ in $\text{A/m}$.
    pub longitudinal_current_density_a_m: f64,
    /// Instantaneous ON/OFF contrast in decibels.
    pub contrast_db: f64,
}

/// Dynamic solver for Floquet optical switches and routers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetSwitchSolver {
    pub switch: FloquetOpticalSwitch,
}

impl FloquetSwitchSolver {
    /// Creates a new Floquet switch solver.
    pub fn new(switch: FloquetOpticalSwitch) -> Self {
        Self { switch }
    }

    /// Evaluates steady-state switching metrics and ON/OFF contrast.
    pub fn solve_steady_state(&self) -> FloquetSwitchResponse {
        self.switch.evaluate_response()
    }

    /// Solves the time-dependent transient switching response across $[t_0, t_1]$.
    pub fn solve_transient(
        &self,
        t_start_s: f64,
        t_end_s: f64,
        num_points: usize,
    ) -> Vec<FloquetSwitchTransientPoint> {
        let n = num_points.max(2);
        let dt = (t_end_s - t_start_s) / ((n - 1) as f64);

        let steady = self.solve_steady_state();
        let max_gap_mev = self.switch.material.dynamic_gap_ev(&self.switch.drive) * 1000.0;
        let peak_jy = steady.hall_current_density_a_m;
        let jx = steady.longitudinal_current_density_a_m;
        let j_off = self.switch.off_leakage_conductance_si * self.switch.bias_field_x_v_m;

        (0..n)
            .map(|i| {
                let t = t_start_s + (i as f64) * dt;
                let env = self.switch.drive.envelope(t);
                let gap = max_gap_mev * env.powi(2);
                let jy = peak_jy * env.powi(2);

                let contrast = (jy.abs() / j_off.abs().max(1e-15)).max(1.0);
                let contrast_db = 20.0 * contrast.log10();

                FloquetSwitchTransientPoint {
                    time_s: t,
                    envelope: env,
                    dynamic_gap_mev: gap,
                    hall_current_density_a_m: jy,
                    longitudinal_current_density_a_m: jx,
                    contrast_db,
                }
            })
            .collect()
    }
}
