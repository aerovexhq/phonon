#![deny(unsafe_code)]

//! Multi-Terminal Chiral Phononic Anyon Interferometer Engine.
//!
//! Models multi-arm Mach-Zehnder and Fabry-Perot anyon interferometry along
//! chiral topological phononic edge waveguides, resolving Aharonov-Bohm flux
//! oscillations, enclosed topological anyon phase shifts, and high-visibility
//! multi-terminal conductance modulations.

use std::f64::consts::PI;

/// Configuration parameters for the multi-terminal anyon interferometer.
#[derive(Debug, Clone)]
pub struct MultiTerminalInterferometerParams {
    /// Number of routing terminals (e.g. 4 or 6).
    pub terminal_count: usize,
    /// Transmissivity of first acoustic beam splitter / QPC (0.01..0.99).
    pub qpc1_transmissivity: f64,
    /// Transmissivity of second acoustic beam splitter / QPC (0.01..0.99).
    pub qpc2_transmissivity: f64,
    /// Number of localized topological anyons enclosed by the interfering arms.
    pub enclosed_anyon_count: usize,
    /// Qudit Hilbert space dimension (d = 3 for qutrit, d = 4 for ququart).
    pub qudit_dimension: usize,
    /// Interferometer enclosed area in square micrometers.
    pub interferometer_area_um2: f64,
    /// Acoustic wavelength in micrometers.
    pub acoustic_wavelength_um: f64,
    /// Acoustic edge coherence length in micrometers.
    pub coherence_length_um: f64,
    /// Path length mismatch between the two interfering arms in micrometers.
    pub path_mismatch_um: f64,
    /// Dimensionless synthetic / magnetic flux in units of flux quantum Phi_0.
    pub magnetic_flux_phi0: f64,
}

impl Default for MultiTerminalInterferometerParams {
    fn default() -> Self {
        Self {
            terminal_count: 4,
            qpc1_transmissivity: 0.50,
            qpc2_transmissivity: 0.50,
            enclosed_anyon_count: 1,
            qudit_dimension: 3,
            interferometer_area_um2: 25.0,
            acoustic_wavelength_um: 0.80,
            coherence_length_um: 120.0,
            path_mismatch_um: 6.0,
            magnetic_flux_phi0: 1.0,
        }
    }
}

/// Evaluated metrics for the multi-terminal anyon interferometer.
#[derive(Debug, Clone)]
pub struct MultiTerminalInterferometerMetrics {
    /// Interferometric fringe visibility in percent (V >= 85.0%).
    pub interference_visibility_pct: f64,
    /// Forward constructive port transmission coefficient (0.0..1.0).
    pub transmission_port2: f64,
    /// Cross destructive port transmission coefficient (0.0..1.0).
    pub transmission_port3: f64,
    /// Aharonov-Bohm oscillation period in units of Phi_0.
    pub aharonov_bohm_period_phi0: f64,
    /// Quantized topological anyon phase shift in radians.
    pub topological_phase_shift_rad: f64,
    /// Acoustic coherence attenuation factor (exp(-Delta_L / L_coh)).
    pub coherence_decay_factor: f64,
    /// Conductance peak-to-valley ratio (I_max / I_min).
    pub peak_to_valley_ratio: f64,
    /// Cross-terminal routing isolation in decibels.
    pub routing_isolation_db: f64,
}

/// A point along the magnetic flux sweep curve of the interferometer.
#[derive(Debug, Clone)]
pub struct InterferometerFluxSweepPoint {
    /// Normalized magnetic flux in units of Phi_0.
    pub flux_phi0: f64,
    /// Transmission to Terminal 2 (constructive channel).
    pub conductance_terminal2: f64,
    /// Transmission to Terminal 3 (destructive channel).
    pub conductance_terminal3: f64,
    /// Leakage to Terminal 4 (cross-isolation channel).
    pub conductance_terminal4: f64,
    /// Net accumulated interference phase in radians.
    pub total_phase_rad: f64,
}

/// Solver for multi-terminal chiral phononic anyon interferometry.
#[derive(Debug, Clone)]
pub struct MultiTerminalInterferometerSolver {
    params: MultiTerminalInterferometerParams,
}

impl MultiTerminalInterferometerSolver {
    /// Creates a new solver instance with given parameters.
    pub fn new(params: MultiTerminalInterferometerParams) -> Self {
        Self { params }
    }

    /// Evaluates the key physical performance metrics.
    pub fn evaluate_metrics(&self) -> MultiTerminalInterferometerMetrics {
        let t1 = self.params.qpc1_transmissivity.clamp(0.01, 0.99);
        let t2 = self.params.qpc2_transmissivity.clamp(0.01, 0.99);
        let r1 = 1.0 - t1;
        let r2 = 1.0 - t2;

        let delta_l = self.params.path_mismatch_um.abs();
        let l_coh = self.params.coherence_length_um.max(1.0);
        let coherence_factor = (-delta_l / l_coh).exp();

        // Fringe visibility: V = 2*sqrt(T1*T2*R1*R2) / (T1*T2 + R1*R2) * eta_coh
        let denom = t1 * t2 + r1 * r2;
        let numer = 2.0 * (t1 * t2 * r1 * r2).sqrt();
        let visibility_ideal = if denom > 1e-12 { numer / denom } else { 1.0 };
        let visibility = (visibility_ideal * coherence_factor).clamp(0.0, 1.0);
        let visibility_pct = visibility * 100.0;

        // Topological anyon phase shift: theta_topo = 2 * pi * n_anyons / d
        let d = self.params.qudit_dimension.max(2) as f64;
        let topo_phase = 2.0 * PI * (self.params.enclosed_anyon_count as f64) / d;

        // Aharonov-Bohm phase: phi_AB = 2 * pi * (Phi / Phi_0)
        let phi_ab = 2.0 * PI * self.params.magnetic_flux_phi0;
        let geom_phase = 2.0 * PI * delta_l / self.params.acoustic_wavelength_um.max(0.01);
        let total_phase = geom_phase + phi_ab + topo_phase;

        // Transmissions into output ports
        let cross_term = (t1 * t2 * r1 * r2).sqrt() * coherence_factor * total_phase.cos();
        let t_port2 = (t1 * t2 + r1 * r2 + 2.0 * cross_term).clamp(0.001, 0.999);
        let t_port3 = (t1 * r2 + r1 * t2 - 2.0 * cross_term).clamp(0.001, 0.999);

        let i_max = (t1 * t2 + r1 * r2 + 2.0 * (t1 * t2 * r1 * r2).sqrt() * coherence_factor).max(1e-6);
        let i_min = (t1 * t2 + r1 * r2 - 2.0 * (t1 * t2 * r1 * r2).sqrt() * coherence_factor).max(1e-6);
        let pv_ratio = (i_max / i_min).clamp(1.0, 1000.0);

        let isolation_db = -10.0 * (t_port3 / t_port2.max(1e-6)).log10().max(0.0);

        MultiTerminalInterferometerMetrics {
            interference_visibility_pct: visibility_pct,
            transmission_port2: t_port2,
            transmission_port3: t_port3,
            aharonov_bohm_period_phi0: 1.0,
            topological_phase_shift_rad: topo_phase,
            coherence_decay_factor: coherence_factor,
            peak_to_valley_ratio: pv_ratio,
            routing_isolation_db: isolation_db,
        }
    }

    /// Computes conductance traces across a normalized flux sweep in units of Phi_0.
    pub fn sweep_flux(&self, n_points: usize) -> Vec<InterferometerFluxSweepPoint> {
        let count = n_points.max(20);
        let t1 = self.params.qpc1_transmissivity.clamp(0.01, 0.99);
        let t2 = self.params.qpc2_transmissivity.clamp(0.01, 0.99);
        let r1 = 1.0 - t1;
        let r2 = 1.0 - t2;

        let delta_l = self.params.path_mismatch_um.abs();
        let l_coh = self.params.coherence_length_um.max(1.0);
        let coherence_factor = (-delta_l / l_coh).exp();

        let d = self.params.qudit_dimension.max(2) as f64;
        let topo_phase = 2.0 * PI * (self.params.enclosed_anyon_count as f64) / d;
        let geom_phase = 2.0 * PI * delta_l / self.params.acoustic_wavelength_um.max(0.01);

        let flux_min = 0.0;
        let flux_max = 3.0; // 3 Aharonov-Bohm periods

        let mut points = Vec::with_capacity(count);
        for i in 0..count {
            let frac = i as f64 / (count - 1) as f64;
            let flux = flux_min + frac * (flux_max - flux_min);
            let phi_ab = 2.0 * PI * flux;
            let total_phase = geom_phase + phi_ab + topo_phase;

            let cross_term = (t1 * t2 * r1 * r2).sqrt() * coherence_factor * total_phase.cos();
            let g2 = (t1 * t2 + r1 * r2 + 2.0 * cross_term).clamp(0.0, 1.0);
            let g3 = (t1 * r2 + r1 * t2 - 2.0 * cross_term).clamp(0.0, 1.0);
            // Higher order terminal leakage (e.g. terminal 4 cross talk)
            let g4 = (0.01 * (1.0 - coherence_factor * 0.5) * (1.0 + 0.5 * (2.0 * total_phase).sin())).clamp(0.0, 0.05);

            points.push(InterferometerFluxSweepPoint {
                flux_phi0: flux,
                conductance_terminal2: g2,
                conductance_terminal3: g3,
                conductance_terminal4: g4,
                total_phase_rad: total_phase,
            });
        }
        points
    }
}
