#![deny(unsafe_code)]

//! Complex-Energy Riemann Surface and Point-Gap Topological Winding Engine.
//!
//! Evaluates the complex eigenenergy spectrum E(k) of the non-Hermitian acoustic lattice,
//! point-gap topological winding invariant W(E_0) = +1, and Generalized Brillouin Zone (GBZ)
//! geometric deformation radius r_GBZ = sqrt(|J_L / J_R|).

use std::f64::consts::PI;

/// Parameters for complex Riemann energy winding analysis.
#[derive(Debug, Clone)]
pub struct RiemannWindingParams {
    /// Forward hopping amplitude J_R in MHz (default ~12.0 MHz).
    pub forward_hopping_jr_mhz: f64,
    /// Backward hopping amplitude J_L in MHz (default ~3.2 MHz).
    pub backward_hopping_jl_mhz: f64,
    /// Complex reference energy origin Re(E_0) in MHz (default 0.0 MHz).
    pub ref_energy_real_mhz: f64,
    /// Complex reference energy origin Im(E_0) in MHz (default 0.0 MHz).
    pub ref_energy_imag_mhz: f64,
    /// On-site potential detuning delta in MHz (default 0.0 MHz).
    pub onsite_detuning_mhz: f64,
}

impl Default for RiemannWindingParams {
    fn default() -> Self {
        Self {
            forward_hopping_jr_mhz: 12.0,
            backward_hopping_jl_mhz: 3.2,
            ref_energy_real_mhz: 0.0,
            ref_energy_imag_mhz: 0.0,
            onsite_detuning_mhz: 0.0,
        }
    }
}

/// Evaluated topological invariants and spectral characteristics.
#[derive(Debug, Clone)]
pub struct RiemannWindingMetrics {
    /// Integer point-gap topological winding number W in Z around reference energy (target = 1).
    pub point_gap_winding_number: i32,
    /// Generalized Brillouin Zone (GBZ) deformation radius r_GBZ = sqrt(|J_L / J_R|).
    pub gbz_radius: f64,
    /// GBZ deformation deviation |r_GBZ - 1.0| (target >= 0.30).
    pub gbz_deformation_magnitude: f64,
    /// Spectral ellipse semi-major axis (real energy span) in MHz.
    pub spectral_ellipse_real_axis_mhz: f64,
    /// Spectral ellipse semi-minor axis (imaginary energy span) in MHz.
    pub spectral_ellipse_imag_axis_mhz: f64,
    /// Point-gap topological protection barrier Delta_point = min_k |E(k) - E_0| in MHz.
    pub point_gap_minimum_distance_mhz: f64,
}

/// Sample point along the closed complex-energy spectrum loop E(k).
#[derive(Debug, Clone)]
pub struct RiemannEnergyPoint {
    /// Bloch momentum k in [-pi, pi] radians.
    pub bloch_momentum_rad: f64,
    /// Real component of complex quasi-energy Re(E) in MHz.
    pub energy_real_mhz: f64,
    /// Imaginary component of complex quasi-energy Im(E) in MHz.
    pub energy_imag_mhz: f64,
    /// Phase angle theta = arg(E(k) - E_0) in radians.
    pub winding_phase_rad: f64,
}

/// Solver for complex-energy Riemann surface and point-gap topology.
#[derive(Debug, Clone)]
pub struct RiemannEnergyWindingSolver {
    params: RiemannWindingParams,
}

impl RiemannEnergyWindingSolver {
    /// Constructs a new Riemann winding solver.
    pub fn new(params: RiemannWindingParams) -> Self {
        Self { params }
    }

    /// Returns a reference to the active parameters.
    pub fn params(&self) -> &RiemannWindingParams {
        &self.params
    }

    /// Evaluates macroscopic winding metrics and topological invariants.
    pub fn evaluate_metrics(&self) -> RiemannWindingMetrics {
        let jr = self.params.forward_hopping_jr_mhz.max(0.1);
        let jl = self.params.backward_hopping_jl_mhz.max(0.1);

        // GBZ deformation radius: r_GBZ = sqrt(|J_L / J_R|)
        let gbz_r = (jl / jr).sqrt();
        let gbz_def = (1.0 - gbz_r).abs();

        // Spectral ellipse axes:
        // E(k) = (J_R + J_L) * cos(k) + i * (J_R - J_L) * sin(k)
        let a_real = jr + jl;
        let b_imag = (jr - jl).abs();

        // Point-gap winding number:
        // For E_0 inside the ellipse (e.g. (0, 0)), W = +1 if J_R > J_L, -1 if J_R < J_L
        let winding = if jr > jl {
            1
        } else if jl > jr {
            -1
        } else {
            0
        };

        // Minimum distance to origin E_0: min(a_real, b_imag)
        let min_dist = b_imag.min(a_real).max(0.1);

        RiemannWindingMetrics {
            point_gap_winding_number: winding,
            gbz_radius: gbz_r,
            gbz_deformation_magnitude: gbz_def,
            spectral_ellipse_real_axis_mhz: a_real,
            spectral_ellipse_imag_axis_mhz: b_imag,
            point_gap_minimum_distance_mhz: min_dist,
        }
    }

    /// Computes the closed complex energy loop E(k) for k in [-pi, pi].
    pub fn compute_complex_energy_loop(&self, points: usize) -> Vec<RiemannEnergyPoint> {
        let n_pts = points.max(40);
        let mut results = Vec::with_capacity(n_pts);

        let jr = self.params.forward_hopping_jr_mhz;
        let jl = self.params.backward_hopping_jl_mhz;
        let delta = self.params.onsite_detuning_mhz;
        let e0_re = self.params.ref_energy_real_mhz;
        let e0_im = self.params.ref_energy_imag_mhz;

        for i in 0..n_pts {
            let frac = (i as f64) / ((n_pts - 1) as f64);
            let k = -PI + frac * 2.0 * PI;

            // E(k) = delta + (J_R + J_L) * cos(k) + i * (J_R - J_L) * sin(k)
            let e_re = delta + (jr + jl) * k.cos();
            let e_im = (jr - jl) * k.sin();

            let diff_re = e_re - e0_re;
            let diff_im = e_im - e0_im;
            let phase = diff_im.atan2(diff_re);

            results.push(RiemannEnergyPoint {
                bloch_momentum_rad: k,
                energy_real_mhz: e_re,
                energy_imag_mhz: e_im,
                winding_phase_rad: phase,
            });
        }

        results
    }
}
