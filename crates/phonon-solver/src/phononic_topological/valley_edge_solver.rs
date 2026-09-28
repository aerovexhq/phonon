//! Domain Wall Chiral Edge State & Backscattering Immunity Solvers.
//!
//! Solves localized topological edge mode profiles traversing the valley
//! bandgap along inverted domain walls, and evaluates backscattering immunity
//! across sharp corner bends.

use phonon_models::phononic_topological::HoneycombAcousticLattice;

/// Localized topological edge state profile across the domain wall.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyEdgeProfile {
    /// Transverse spatial coordinates $y$ across supercell in meters.
    pub coordinate_y_m: Vec<f64>,
    /// Normalized acoustic pressure amplitude $|p(y)|$.
    pub pressure_amplitude: Vec<f64>,
    /// Characteristic exponential decay length $\xi$ in meters.
    pub decay_length_m: f64,
    /// Edge mode angular frequency $\omega(k_x)$ in rad/s.
    pub frequency_rad_per_s: f64,
    /// Chiral group velocity $v_g = \frac{\partial\omega}{\partial k_x}$ in m/s.
    pub group_velocity_m_per_s: f64,
}

/// Valley acoustic edge mode solver for domain walls.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyEdgeSolver {
    /// Honeycomb metamaterial lattice configuration.
    pub lattice: HoneycombAcousticLattice,
    /// Total transverse supercell width $W$ in meters.
    pub supercell_width_m: f64,
    /// Number of spatial discretization points along transverse coordinate $y$.
    pub num_spatial_points: usize,
}

impl Default for ValleyEdgeSolver {
    fn default() -> Self {
        Self {
            lattice: HoneycombAcousticLattice::default(),
            supercell_width_m: 0.30,
            num_spatial_points: 101,
        }
    }
}

impl ValleyEdgeSolver {
    /// Creates a new valley edge state solver.
    pub fn new(
        lattice: HoneycombAcousticLattice,
        supercell_width_m: f64,
        num_spatial_points: usize,
    ) -> Self {
        Self {
            lattice,
            supercell_width_m,
            num_spatial_points: num_spatial_points.max(3),
        }
    }

    /// Solves the localized domain wall edge mode profile for a given wavevector $k_x$.
    pub fn solve_edge_profile(&self, kx: f64) -> ValleyEdgeProfile {
        let n = self.num_spatial_points;
        let w = self.supercell_width_m;
        let half_w = 0.5 * w;
        let dy = w / ((n - 1) as f64);

        let xi = self.lattice.edge_state_decay_length_m();
        let w0 = self.lattice.dirac_frequency_rad_per_s();
        let vd = self.lattice.dirac_velocity_m_per_s();
        let chern = self.lattice.valley_chern_number();

        // Edge state dispersion: omega(kx) = omega_0 + sgn(delta_A) * v_D * k_x
        let vg = (chern as f64) * vd;
        let omega = w0 + vg * kx;

        let mut coords = Vec::with_capacity(n);
        let mut pressure = Vec::with_capacity(n);

        let decay_rate = if xi.is_finite() && xi > 1e-12 {
            1.0 / xi
        } else {
            0.0
        };

        for i in 0..n {
            let y = -half_w + (i as f64) * dy;
            coords.push(y);
            // Exponentially localized at the domain wall interface y = 0
            let p = (-decay_rate * y.abs()).exp();
            pressure.push(p);
        }

        ValleyEdgeProfile {
            coordinate_y_m: coords,
            pressure_amplitude: pressure,
            decay_length_m: xi,
            frequency_rad_per_s: omega,
            group_velocity_m_per_s: vg,
        }
    }

    /// Computes the dispersion relation $(k_x, \omega(k_x))$ across the valley bandgap.
    pub fn compute_dispersion(&self, kx_samples: usize) -> Vec<(f64, f64)> {
        let n = kx_samples.max(2);
        let gap = self.lattice.valley_gap_rad_per_s();
        let vd = self.lattice.dirac_velocity_m_per_s();
        let kx_max = if vd > 0.0 { 0.5 * gap / vd } else { 10.0 };

        let mut dispersion = Vec::with_capacity(n);
        for i in 0..n {
            let frac = (i as f64) / ((n - 1) as f64);
            let kx = -kx_max + 2.0 * kx_max * frac;
            let profile = self.solve_edge_profile(kx);
            dispersion.push((kx, profile.frequency_rad_per_s));
        }
        dispersion
    }

    /// Evaluates total acoustic transmission through a sequence of sharp waveguide bends (in degrees).
    /// Topologically protected valley edge modes navigate sharp $60^\circ$ and $120^\circ$
    /// turns with negligible backscattering ($T \ge 90\%$).
    pub fn evaluate_bend_transmission(&self, bend_angles_deg: &[f64]) -> f64 {
        let mut total_t = 1.0;
        for &angle in bend_angles_deg {
            let angle_rad = angle.to_radians();
            let t = self.lattice.corner_transmission(angle_rad);
            total_t *= t;
        }
        total_t.clamp(0.0, 1.0)
    }
}
