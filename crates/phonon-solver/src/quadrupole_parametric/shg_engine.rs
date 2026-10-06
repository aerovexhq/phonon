#![deny(unsafe_code)]

//! Second-Harmonic Generation (SHG) Engine in Topological Acoustic Quadrupole Waveguide.
//!
//! Solves spatial non-linear coupled-mode equations for fundamental (f_1) and second-harmonic (f_2 = 2*f_1)
//! phononic edge modes along propagation coordinate z in [0, L], evaluating conversion efficiency eta(z),
//! phase-matching sinc^2(Delta_k * L / 2) curves, and spatial modal overlap integrals.

use std::f64::consts::PI;

/// Parameters governing non-linear second-harmonic generation along the waveguide.
#[derive(Debug, Clone, PartialEq)]
pub struct ShgParams {
    /// Quadratic acoustic non-linear coupling coefficient kappa in rad / (sqrt(W) * mm).
    pub kappa: f64,
    /// Input fundamental pump power P_0 in Watts (default ~0.1 W / 100 mW).
    pub pump_power_w: f64,
    /// Fundamental acoustic frequency f_1 in GHz (default ~1.0 GHz).
    pub f1_ghz: f64,
    /// Second harmonic acoustic frequency f_2 in GHz (default ~2.0 GHz = 2 * f_1).
    pub f2_ghz: f64,
    /// Propagation integration step count N_z (default 100).
    pub nz_steps: usize,
    /// Waveguide linear attenuation for fundamental mode alpha_1 in dB/cm (default 0.2 dB/cm).
    pub alpha1_db_cm: f64,
    /// Waveguide linear attenuation for second-harmonic mode alpha_2 in dB/cm (default 0.2 dB/cm).
    pub alpha2_db_cm: f64,
    /// Acoustic phase mismatch Delta_k = k(2*omega_1) - 2*k(omega_1) in rad/mm (0.0 for exact phase matching).
    pub delta_k_rad_mm: f64,
    /// Waveguide propagation length L in mm (default ~50.0 mm).
    pub length_mm: f64,
}

impl Default for ShgParams {
    fn default() -> Self {
        Self {
            kappa: 0.05,
            pump_power_w: 0.1,
            f1_ghz: 1.0,
            f2_ghz: 2.0,
            nz_steps: 100,
            alpha1_db_cm: 0.2,
            alpha2_db_cm: 0.2,
            delta_k_rad_mm: 0.0,
            length_mm: 50.0,
        }
    }
}

impl ShgParams {
    /// Converts alpha_1 from dB/cm to amplitude attenuation per mm: alpha = alpha_dB * ln(10) / 200.
    #[inline]
    pub fn alpha1_per_mm(&self) -> f64 {
        self.alpha1_db_cm * 2.302585092994046 / 200.0
    }

    /// Converts alpha_2 from dB/cm to amplitude attenuation per mm: alpha = alpha_dB * ln(10) / 200.
    #[inline]
    pub fn alpha2_per_mm(&self) -> f64 {
        self.alpha2_db_cm * 2.302585092994046 / 200.0
    }
}

/// Snapshot of the acoustic field amplitudes and powers at a single propagation coordinate z.
#[derive(Debug, Clone, PartialEq)]
pub struct ShgStepPoint {
    /// Propagation coordinate z in mm.
    pub z_mm: f64,
    /// Fundamental modal field amplitude A_1 in sqrt(W).
    pub a1: f64,
    /// Second-harmonic modal field amplitude A_2 in sqrt(W).
    pub a2: f64,
    /// Fundamental acoustic power P_1(z) in Watts.
    pub p1_w: f64,
    /// Second-harmonic acoustic power P_2(z) in Watts.
    pub p2_w: f64,
    /// Second-harmonic conversion efficiency eta_shg(z) = P_2(z) / P_1(0).
    pub efficiency: f64,
}

/// Point on the phase-matching sinc^2 curve vs Delta_k.
#[derive(Debug, Clone, PartialEq)]
pub struct ShgPhaseMatchSample {
    /// Phase mismatch Delta_k in rad/mm.
    pub delta_k_rad_mm: f64,
    /// Numerically integrated conversion efficiency eta(Delta_k).
    pub efficiency: f64,
    /// Analytical low-depletion sinc^2(Delta_k * L / 2) curve.
    pub sinc2_theoretical: f64,
}

/// Non-linear Second-Harmonic Generation coupled-mode solver.
#[derive(Debug, Clone)]
pub struct ShgSolver {
    pub params: ShgParams,
    /// Spatial modal overlap integral I_overlap between fundamental and SHG edge states in [0.0, 1.0].
    pub modal_overlap_integral: f64,
    /// Propagation trajectory containing discrete steps along z in [0, L].
    pub trajectory: Vec<ShgStepPoint>,
    /// Terminal conversion efficiency eta_shg(L) = P_2(L) / P_1(0).
    pub conversion_efficiency: f64,
    /// Terminal generated second-harmonic power P_2(L) in Watts.
    pub terminal_shg_power_w: f64,
}

impl ShgSolver {
    /// Creates and solves a new SHG engine instance with the given parameters.
    pub fn new(params: ShgParams) -> Self {
        let mut solver = Self {
            params,
            modal_overlap_integral: 0.94,
            trajectory: Vec::new(),
            conversion_efficiency: 0.0,
            terminal_shg_power_w: 0.0,
        };
        solver.recompute();
        solver
    }

    /// Recomputes spatial modal overlap and solves the coupled-mode trajectory using 4th-order Runge-Kutta.
    pub fn recompute(&mut self) {
        // Compute transverse modal overlap between fundamental edge state and second harmonic state
        self.modal_overlap_integral = Self::compute_modal_overlap(0.72, 0.36, 6);

        self.trajectory = self.integrate_rk4(self.params.delta_k_rad_mm);

        if let Some(last) = self.trajectory.last() {
            self.conversion_efficiency = last.efficiency;
            self.terminal_shg_power_w = last.p2_w;
        } else {
            self.conversion_efficiency = 0.0;
            self.terminal_shg_power_w = 0.0;
        }
    }

    /// Evaluates the spatial modal overlap integral I_overlap between fundamental and second-harmonic edge states:
    ///
    /// I_overlap = integral(psi_1^2 * psi_2 dy) / [sqrt(integral(psi_1^4 dy)) * sqrt(integral(psi_2^2 dy))]
    pub fn compute_modal_overlap(xi1: f64, xi2: f64, ny_cells: usize) -> f64 {
        let n_sites = ny_cells.max(3) * 2;
        let mut num = 0.0;
        let mut den1 = 0.0;
        let mut den2 = 0.0;

        for iy in 0..n_sites {
            let y_cell = (iy as f64) * 0.5;
            let u1 = (-y_cell / xi1).exp();
            let u2 = (-y_cell / xi2).exp();

            let u1_sq = u1 * u1;
            let u1_fourth = u1_sq * u1_sq;
            let u2_sq = u2 * u2;

            num += u1_sq * u2;
            den1 += u1_fourth;
            den2 += u2_sq;
        }

        let den = (den1 * den2).max(1e-18).sqrt();
        (num / den).clamp(0.0, 1.0)
    }

    /// Solves the spatial coupled-mode equations using 4th-order Runge-Kutta (RK4) for a given Delta_k:
    ///
    /// dA_1/dz = -alpha_1 * A_1 - kappa * A_1 * A_2 * sin(Delta_k * z)
    /// dA_2/dz = -alpha_2 * A_2 + kappa * A_1^2 * cos(Delta_k * z)
    pub fn integrate_rk4(&self, delta_k: f64) -> Vec<ShgStepPoint> {
        let n_steps = self.params.nz_steps.max(10);
        let l = self.params.length_mm;
        let dz = l / (n_steps as f64);

        let p0 = self.params.pump_power_w.max(1e-12);
        let a1_0 = p0.sqrt();
        let a2_0 = 0.0;

        let alpha1 = self.params.alpha1_per_mm();
        let alpha2 = self.params.alpha2_per_mm();
        let kappa = self.params.kappa;

        let mut trajectory = Vec::with_capacity(n_steps + 1);

        let mut a1 = a1_0;
        let mut a2 = a2_0;
        let mut z = 0.0;

        trajectory.push(ShgStepPoint {
            z_mm: 0.0,
            a1,
            a2,
            p1_w: a1 * a1,
            p2_w: a2 * a2,
            efficiency: 0.0,
        });

        let rhs = |z_val: f64, y1: f64, y2: f64| -> (f64, f64) {
            let phase = delta_k * z_val;
            let da1 = -alpha1 * y1 - kappa * y1 * y2 * phase.sin();
            let da2 = -alpha2 * y2 + kappa * y1 * y1 * phase.cos();
            (da1, da2)
        };

        for _ in 0..n_steps {
            // RK4 step
            let (k1_1, k1_2) = rhs(z, a1, a2);
            let (k2_1, k2_2) = rhs(z + 0.5 * dz, a1 + 0.5 * dz * k1_1, a2 + 0.5 * dz * k1_2);
            let (k3_1, k3_2) = rhs(z + 0.5 * dz, a1 + 0.5 * dz * k2_1, a2 + 0.5 * dz * k2_2);
            let (k4_1, k4_2) = rhs(z + dz, a1 + dz * k3_1, a2 + dz * k3_2);

            a1 += (dz / 6.0) * (k1_1 + 2.0 * k2_1 + 2.0 * k3_1 + k4_1);
            a2 += (dz / 6.0) * (k1_2 + 2.0 * k2_2 + 2.0 * k3_2 + k4_2);
            z += dz;

            // Physical boundary condition: field amplitudes remain non-negative in envelope representation
            a1 = a1.max(0.0);
            a2 = a2.max(0.0);

            let p1 = a1 * a1;
            let p2 = a2 * a2;
            let eff = (p2 / p0).clamp(0.0, 1.0);

            trajectory.push(ShgStepPoint {
                z_mm: z,
                a1,
                a2,
                p1_w: p1,
                p2_w: p2,
                efficiency: eff,
            });
        }

        trajectory
    }

    /// Verifies whether the second-harmonic power conversion growth is monotonic along z.
    ///
    /// Under exact phase matching (Delta_k = 0), second-harmonic acoustic power P_2(z) strictly
    /// increases monotonically across the waveguide.
    pub fn is_monotonic_conversion(&self) -> bool {
        if self.trajectory.len() < 2 {
            return false;
        }

        self.trajectory
            .windows(2)
            .all(|w| w[1].p2_w >= w[0].p2_w - 1e-12)
    }

    /// Sweeps phase mismatch Delta_k in [-delta_k_max, +delta_k_max] rad/mm to evaluate
    /// the characteristic phase-matching sinc^2 curve.
    pub fn sweep_phase_mismatch(&self, delta_k_max: f64, num_points: usize) -> Vec<ShgPhaseMatchSample> {
        let n = num_points.max(16);
        let mut samples = Vec::with_capacity(n);

        // Peak conversion at Delta_k = 0
        let peak_eff = self.conversion_efficiency.max(1e-6);
        let l = self.params.length_mm;

        for i in 0..n {
            let frac = (i as f64) / ((n - 1) as f64);
            let delta_k = -delta_k_max + frac * (2.0 * delta_k_max);

            // Numerical integration for this Delta_k
            let traj = self.integrate_rk4(delta_k);
            let num_eff = traj.last().map(|p| p.efficiency).unwrap_or(0.0);

            // Analytical low-depletion sinc^2: sinc(x) = sin(x) / x where x = Delta_k * L / 2
            let arg = delta_k * l * 0.5;
            let sinc_val = if arg.abs() < 1e-9 {
                1.0
            } else {
                arg.sin() / arg
            };
            let sinc2_theo = peak_eff * sinc_val * sinc_val;

            samples.push(ShgPhaseMatchSample {
                delta_k_rad_mm: delta_k,
                efficiency: num_eff,
                sinc2_theoretical: sinc2_theo,
            });
        }

        samples
    }
}
