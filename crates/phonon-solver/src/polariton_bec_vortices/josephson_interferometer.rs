#![deny(unsafe_code)]

//! Polariton Josephson acoustic interferometer and sub-nanostrain sensing engine.
//!
//! Models macroscopic quantum interference between coupled polariton condensates,
//! plasma oscillations, macroscopic quantum self-trapping (MQST), and strain-induced
//! phase shifts for high-precision acoustic sensing.

use std::f64::consts::PI;
use crate::polariton_bec_vortices::gross_pitaevskii::HBAR_MEV_PS;

/// Parameters governing the polariton Josephson junction and acoustic strain coupling.
#[derive(Debug, Clone, PartialEq)]
pub struct JosephsonInterferometerParams {
    /// Josephson tunneling coupling energy E_J in meV (typically ~ 0.02 to 0.10 meV).
    pub josephson_coupling_ej_mev: f64,
    /// On-site charging / interaction energy E_C in meV (typically ~ 0.05 to 0.20 meV).
    pub charging_energy_ec_mev: f64,
    /// Acoustic deformation potential Xi_ac in eV (e.g. 8.5 eV for GaAs conduction band).
    pub deformation_potential_ev: f64,
    /// Applied mechanical strain epsilon_xx (dimensionless, e.g. 1e-9 to 1e-5).
    pub applied_strain: f64,
    /// Initial population imbalance z_0 = (N1 - N2) / (N1 + N2) in [-1, 1].
    pub initial_imbalance: f64,
    /// Initial phase difference Delta_phi_0 in radians.
    pub initial_phase_rad: f64,
    /// Total evolution time in picoseconds (e.g. 50 ps to 300 ps).
    pub total_time_ps: f64,
    /// Number of time integration steps.
    pub time_steps: usize,
}

impl Default for JosephsonInterferometerParams {
    fn default() -> Self {
        Self {
            josephson_coupling_ej_mev: 0.045,
            charging_energy_ec_mev: 0.080,
            deformation_potential_ev: 8.5,
            applied_strain: 2.5e-7, // 250 nanostrain
            initial_imbalance: 0.20,
            initial_phase_rad: 0.0,
            total_time_ps: 150.0,
            time_steps: 120,
        }
    }
}

/// A point along the Josephson time-evolution trajectory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JosephsonTrajectoryPoint {
    pub time_ps: f64,
    /// Fractional population imbalance z(t) in [-1, 1].
    pub imbalance_z: f64,
    /// Relative phase difference Delta_phi(t) in radians.
    pub relative_phase_rad: f64,
    /// Instantaneous interference fringe intensity at center.
    pub fringe_intensity: f64,
}

/// Metrics describing the acoustic Josephson interferometric sensor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JosephsonSensorMetrics {
    /// Josephson plasma oscillation frequency in GHz.
    pub plasma_frequency_ghz: f64,
    /// Critical population imbalance z_c for MQST transition.
    pub mqst_critical_imbalance: f64,
    /// Whether the system is in Macroscopic Quantum Self-Trapping (MQST) regime.
    pub is_self_trapped: bool,
    /// Strain-induced chemical potential shift Delta_V in meV.
    pub strain_potential_shift_mev: f64,
    /// Strain sensitivity d(Delta_phi) / d(epsilon) in rad / strain.
    pub phase_strain_responsivity_rad: f64,
    /// Minimum detectable acoustic strain epsilon_min in 1 / sqrt(Hz).
    pub minimum_detectable_strain: f64,
    /// Interference fringe visibility contrast V in [0, 1].
    pub fringe_visibility: f64,
}

/// A point in the spatial interference fringe pattern across detector screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FringePatternPoint {
    pub position_um: f64,
    pub intensity: f64,
}

/// Engine evaluating acoustic Josephson interferometry and strain sensing.
#[derive(Debug, Clone)]
pub struct JosephsonInterferometerSolver {
    pub params: JosephsonInterferometerParams,
}

impl JosephsonInterferometerSolver {
    /// Creates a new solver with the specified parameters.
    pub fn new(params: JosephsonInterferometerParams) -> Self {
        Self { params }
    }

    /// Evaluates Josephson plasma oscillation frequency omega_J in rad / ps.
    ///
    /// omega_J = sqrt(2 * E_C * E_J) / hbar
    pub fn plasma_frequency_rad_ps(&self) -> f64 {
        let ec = self.params.charging_energy_ec_mev;
        let ej = self.params.josephson_coupling_ej_mev;
        (2.0 * ec * ej).sqrt() / HBAR_MEV_PS
    }

    /// Evaluates plasma oscillation frequency f_J in GHz.
    pub fn plasma_frequency_ghz(&self) -> f64 {
        let omega = self.plasma_frequency_rad_ps();
        (omega / (2.0 * PI)) * 1.0e3 // convert rad/ps to GHz
    }

    /// Evaluates MQST critical imbalance z_crit = sqrt(2 * E_J / E_C).
    pub fn mqst_critical_imbalance(&self) -> f64 {
        let ej = self.params.josephson_coupling_ej_mev;
        let ec = self.params.charging_energy_ec_mev.max(1e-6);
        (2.0 * ej / ec).sqrt().min(1.0)
    }

    /// Computes acoustic strain potential shift Delta_V_strain = Xi_ac * epsilon in meV.
    pub fn strain_potential_shift_mev(&self) -> f64 {
        (self.params.deformation_potential_ev * self.params.applied_strain) * 1.0e3
    }

    /// Computes phase responsivity d(Delta_phi) / d(epsilon) in rad per strain over 100 ps.
    pub fn phase_strain_responsivity_rad(&self) -> f64 {
        let xi_mev = self.params.deformation_potential_ev * 1.0e3;
        let tau_ps = 100.0;
        (xi_mev * tau_ps) / HBAR_MEV_PS
    }

    /// Computes minimum detectable strain in 1 / sqrt(Hz).
    pub fn minimum_detectable_strain(&self) -> f64 {
        let resp = self.phase_strain_responsivity_rad().max(1.0);
        // Phase resolution limited by shot noise (typically 1e-4 rad / sqrt(Hz) in microcavities)
        let delta_phi_noise = 1.0e-4;
        delta_phi_noise / resp
    }

    /// Solves time-evolution of population imbalance and relative phase via Runge-Kutta 4.
    pub fn solve_dynamics(&self) -> Vec<JosephsonTrajectoryPoint> {
        let n_steps = self.params.time_steps.max(10);
        let dt = self.params.total_time_ps / (n_steps - 1) as f64;
        let mut trajectory = Vec::with_capacity(n_steps);

        let mut z = self.params.initial_imbalance.clamp(-0.999, 0.999);
        let mut phi = self.params.initial_phase_rad;
        let delta_v = self.strain_potential_shift_mev();
        let ej = self.params.josephson_coupling_ej_mev;
        let ec = self.params.charging_energy_ec_mev;
        let hbar = HBAR_MEV_PS;

        // Equations:
        // dz/dt = -(2 * E_J / hbar) * sqrt(1 - z^2) * sin(phi)
        // dphi/dt = (2 * E_C * z + delta_v) / hbar
        let derivatives = |curr_z: f64, curr_phi: f64| -> (f64, f64) {
            let cz = curr_z.clamp(-0.999, 0.999);
            let dz = -(2.0 * ej / hbar) * (1.0 - cz * cz).sqrt() * curr_phi.sin();
            let dphi = (2.0 * ec * cz + delta_v) / hbar;
            (dz, dphi)
        };

        for i in 0..n_steps {
            let t = i as f64 * dt;
            let fringe = 0.5 * (1.0 + phi.cos());

            trajectory.push(JosephsonTrajectoryPoint {
                time_ps: t,
                imbalance_z: z,
                relative_phase_rad: phi,
                fringe_intensity: fringe,
            });

            // RK4 integration step
            let (k1_z, k1_phi) = derivatives(z, phi);
            let (k2_z, k2_phi) = derivatives(z + 0.5 * dt * k1_z, phi + 0.5 * dt * k1_phi);
            let (k3_z, k3_phi) = derivatives(z + 0.5 * dt * k2_z, phi + 0.5 * dt * k2_phi);
            let (k4_z, k4_phi) = derivatives(z + dt * k3_z, phi + dt * k3_phi);

            z += (dt / 6.0) * (k1_z + 2.0 * k2_z + 2.0 * k3_z + k4_z);
            phi += (dt / 6.0) * (k1_phi + 2.0 * k2_phi + 2.0 * k3_phi + k4_phi);
            z = z.clamp(-0.999, 0.999);
        }

        trajectory
    }

    /// Evaluates complete sensor performance metrics.
    pub fn evaluate_sensor_metrics(&self) -> JosephsonSensorMetrics {
        let f_j = self.plasma_frequency_ghz();
        let z_c = self.mqst_critical_imbalance();
        let delta_v = self.strain_potential_shift_mev();
        let resp = self.phase_strain_responsivity_rad();
        let eps_min = self.minimum_detectable_strain();

        let is_mqst = self.params.initial_imbalance.abs() > z_c;
        let visibility = 0.94; // high-contrast microcavity interference

        JosephsonSensorMetrics {
            plasma_frequency_ghz: f_j,
            mqst_critical_imbalance: z_c,
            is_self_trapped: is_mqst,
            strain_potential_shift_mev: delta_v,
            phase_strain_responsivity_rad: resp,
            minimum_detectable_strain: eps_min,
            fringe_visibility: visibility,
        }
    }

    /// Generates spatial interference fringe pattern I(x) on detector screen.
    pub fn compute_fringe_pattern(&self, phase_shift_rad: f64, num_points: usize) -> Vec<FringePatternPoint> {
        let n = num_points.max(2);
        let x_span_um = 30.0;
        let step = (2.0 * x_span_um) / (n - 1) as f64;
        let mut result = Vec::with_capacity(n);

        let k_fringe = 2.0 * PI / 6.0; // fringe period 6 um
        let visibility = 0.94;

        for i in 0..n {
            let x = -x_span_um + i as f64 * step;
            let envelope = (-2.0 * (x / 20.0).powi(2)).exp();
            let pattern = envelope * (1.0 + visibility * (k_fringe * x + phase_shift_rad).cos());

            result.push(FringePatternPoint {
                position_um: x,
                intensity: pattern.max(0.0),
            });
        }

        result
    }
}
