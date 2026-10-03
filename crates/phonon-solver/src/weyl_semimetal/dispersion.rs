#![deny(unsafe_code)]

//! 3D Weyl and Dirac dispersion solver, conical band topology,
//! and Berry curvature monopole calculations.

/// Classification of Weyl nodes based on tilt parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeylNodeType {
    /// Standard upright or moderately tilted cone (tilt t < 1.0) with point Fermi surface.
    TypeI,
    /// Overtilted Weyl cone (tilt t > 1.0) with touching electron and hole Fermi pockets.
    TypeII,
}

impl WeylNodeType {
    /// User-facing descriptive label.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TypeI => "Type-I (Point Fermi surface)",
            Self::TypeII => "Type-II (Overtilted electron-hole pockets)",
        }
    }
}

/// A 3D Weyl node in momentum space.
#[derive(Debug, Clone, PartialEq)]
pub struct WeylNode {
    /// Position in 3D momentum space k0 = (kx, ky, kz) in 1/m (or rad/a).
    pub k0: [f64; 3],
    /// Chiral topological charge C in {+1, -1}.
    pub chirality: i32,
    /// Fermi velocity components v_F = (vx, vy, vz) in m/s.
    pub vf: [f64; 3],
    /// Tilt vector w = (wx, wy, wz) in m/s.
    pub tilt: [f64; 3],
}

impl WeylNode {
    /// Creates a new Weyl node.
    pub fn new(k0: [f64; 3], chirality: i32, vf: [f64; 3], tilt: [f64; 3]) -> Self {
        let charge = if chirality >= 0 { 1 } else { -1 };
        Self {
            k0,
            chirality: charge,
            vf,
            tilt,
        }
    }

    /// Evaluates normalized tilt parameter t = sqrt( (w_x/v_x)^2 + (w_y/v_y)^2 + (w_z/v_z)^2 ).
    pub fn tilt_parameter(&self) -> f64 {
        let mut sum_sq = 0.0;
        for i in 0..3 {
            let v = self.vf[i].abs();
            if v > 1e-12 {
                let ratio = self.tilt[i] / v;
                sum_sq += ratio * ratio;
            } else if self.tilt[i].abs() > 1e-12 {
                sum_sq += 1e6;
            }
        }
        sum_sq.sqrt()
    }

    /// Classifies the node as Type-I (t < 1.0) or Type-II (t > 1.0).
    pub fn node_type(&self) -> WeylNodeType {
        if self.tilt_parameter() > 1.0 {
            WeylNodeType::TypeII
        } else {
            WeylNodeType::TypeI
        }
    }

    /// Calculates energy dispersion E_±(k) = w · \delta k ± \sqrt{(vx \delta kx)^2 + (vy \delta ky)^2 + (vz \delta kz)^2}.
    /// Returns (E_-, E_+) where E_- is the lower band and E_+ is the upper band.
    pub fn dispersion_at(&self, k: [f64; 3]) -> (f64, f64) {
        let dk = [k[0] - self.k0[0], k[1] - self.k0[1], k[2] - self.k0[2]];
        let w_dot = self.tilt[0] * dk[0] + self.tilt[1] * dk[1] + self.tilt[2] * dk[2];
        let delta = ((self.vf[0] * dk[0]).powi(2)
            + (self.vf[1] * dk[1]).powi(2)
            + (self.vf[2] * dk[2]).powi(2))
        .sqrt();
        (w_dot - delta, w_dot + delta)
    }

    /// Conduction (upper) band energy E_+(k).
    pub fn energy_plus(&self, k: [f64; 3]) -> f64 {
        self.dispersion_at(k).1
    }

    /// Valence (lower) band energy E_-(k).
    pub fn energy_minus(&self, k: [f64; 3]) -> f64 {
        self.dispersion_at(k).0
    }

    /// Berry curvature monopole \Omega(k) = C * \delta k / (2 |\delta k|^3).
    pub fn berry_curvature(&self, k: [f64; 3]) -> [f64; 3] {
        let dk = [k[0] - self.k0[0], k[1] - self.k0[1], k[2] - self.k0[2]];
        let dist = (dk[0] * dk[0] + dk[1] * dk[1] + dk[2] * dk[2]).sqrt();
        if dist < 1e-14 {
            return [0.0, 0.0, 0.0];
        }
        let denom = 2.0 * dist.powi(3);
        let factor = (self.chirality as f64) / denom;
        [factor * dk[0], factor * dk[1], factor * dk[2]]
    }

    /// Berry flux over enclosing sphere: \oint \Omega · dS = 2\pi C.
    /// Performs numerical surface integration over a sphere of radius R centered at k0.
    pub fn berry_flux_sphere(&self, radius: f64, num_theta: usize, num_phi: usize) -> f64 {
        let n_th = num_theta.max(10);
        let n_ph = num_phi.max(10);
        let d_theta = std::f64::consts::PI / (n_th as f64);
        let d_phi = 2.0 * std::f64::consts::PI / (n_ph as f64);
        let mut flux = 0.0;

        for i in 0..n_th {
            let theta = (i as f64 + 0.5) * d_theta;
            let sin_th = theta.sin();
            let cos_th = theta.cos();
            for j in 0..n_ph {
                let phi = (j as f64 + 0.5) * d_phi;
                let r_hat = [sin_th * phi.cos(), sin_th * phi.sin(), cos_th];
                let k = [
                    self.k0[0] + radius * r_hat[0],
                    self.k0[1] + radius * r_hat[1],
                    self.k0[2] + radius * r_hat[2],
                ];
                let omega = self.berry_curvature(k);
                let da = radius * radius * sin_th * d_theta * d_phi;
                let dot = omega[0] * r_hat[0] + omega[1] * r_hat[1] + omega[2] * r_hat[2];
                flux += dot * da;
            }
        }
        flux
    }

    /// Analytical quantized Berry flux 2\pi * C.
    pub fn analytical_berry_flux(&self) -> f64 {
        2.0 * std::f64::consts::PI * (self.chirality as f64)
    }
}

/// A point along a high-symmetry path in the Brillouin zone with band energies.
#[derive(Debug, Clone, PartialEq)]
pub struct BandPoint {
    /// 3D momentum coordinate.
    pub k: [f64; 3],
    /// Cumulative path coordinate along the trajectory.
    pub path_coordinate: f64,
    /// Valence band energies from all nodes.
    pub valence_energies: Vec<f64>,
    /// Conduction band energies from all nodes.
    pub conduction_energies: Vec<f64>,
    /// All sorted band eigenvalues.
    pub all_energies: Vec<f64>,
}

/// Weyl / Dirac semimetal metamaterial model managing node pairs and symmetry.
#[derive(Debug, Clone, PartialEq)]
pub struct WeylSemimetalModel {
    /// Collection of Weyl nodes in the Brillouin zone.
    pub nodes: Vec<WeylNode>,
    /// Whether the model represents a 4-band Dirac semimetal (degenerate pairs with zero net chirality).
    pub is_dirac_mode: bool,
    /// Metamaterial lattice parameter a (in meters).
    pub lattice_constant_a: f64,
}

impl WeylSemimetalModel {
    /// Creates a model from an explicit list of Weyl nodes.
    pub fn new(nodes: Vec<WeylNode>, is_dirac_mode: bool, lattice_constant_a: f64) -> Self {
        Self {
            nodes,
            is_dirac_mode,
            lattice_constant_a,
        }
    }

    /// Minimal 2-node Weyl semimetal with broken time-reversal symmetry (TRS broken).
    /// Nodes placed at (±Delta_k/2, 0, 0) with chiralities +1 and -1.
    pub fn new_trs_broken_pair(
        separation_delta_k: f64,
        vf: [f64; 3],
        tilt: [f64; 3],
        lattice_constant_a: f64,
    ) -> Self {
        let half_sep = separation_delta_k * 0.5;
        let w_plus = WeylNode::new([half_sep, 0.0, 0.0], 1, vf, tilt);
        let w_minus = WeylNode::new([-half_sep, 0.0, 0.0], -1, vf, tilt);
        Self {
            nodes: vec![w_plus, w_minus],
            is_dirac_mode: false,
            lattice_constant_a,
        }
    }

    /// 4-node Weyl semimetal with broken inversion symmetry and preserved time-reversal symmetry.
    /// Nodes placed at (±Delta_k/2, 0, 0) and (0, ±Delta_k/2, 0).
    pub fn new_inversion_broken_quad(
        separation_delta_k: f64,
        vf: [f64; 3],
        tilt: [f64; 3],
        lattice_constant_a: f64,
    ) -> Self {
        let half_sep = separation_delta_k * 0.5;
        let n1 = WeylNode::new([half_sep, 0.0, 0.0], 1, vf, tilt);
        let n2 = WeylNode::new([-half_sep, 0.0, 0.0], 1, vf, tilt);
        let n3 = WeylNode::new([0.0, half_sep, 0.0], -1, vf, tilt);
        let n4 = WeylNode::new([0.0, -half_sep, 0.0], -1, vf, tilt);
        Self {
            nodes: vec![n1, n2, n3, n4],
            is_dirac_mode: false,
            lattice_constant_a,
        }
    }

    /// Dirac semimetal mode with degenerate node pairs at (±Delta_k/2, 0, 0), each having zero net chirality.
    pub fn new_dirac_semimetal(
        separation_delta_k: f64,
        vf: [f64; 3],
        lattice_constant_a: f64,
    ) -> Self {
        let half_sep = separation_delta_k * 0.5;
        let zero_tilt = [0.0, 0.0, 0.0];
        // At +half_sep: two overlapping Weyl nodes with opposite chiralities
        let d1_plus = WeylNode::new([half_sep, 0.0, 0.0], 1, vf, zero_tilt);
        let d1_minus = WeylNode::new([half_sep, 0.0, 0.0], -1, vf, zero_tilt);
        // At -half_sep: two overlapping Weyl nodes with opposite chiralities
        let d2_plus = WeylNode::new([-half_sep, 0.0, 0.0], 1, vf, zero_tilt);
        let d2_minus = WeylNode::new([-half_sep, 0.0, 0.0], -1, vf, zero_tilt);

        Self {
            nodes: vec![d1_plus, d1_minus, d2_plus, d2_minus],
            is_dirac_mode: true,
            lattice_constant_a,
        }
    }

    /// Total chiral charge sum \sum C_i.
    pub fn total_chirality(&self) -> i32 {
        self.nodes.iter().map(|n| n.chirality).sum()
    }

    /// Validates compliance with the Nielsen-Ninomiya theorem (\sum C_i = 0).
    pub fn satisfies_nielsen_ninomiya(&self) -> bool {
        self.total_chirality() == 0
    }

    /// Evaluates band structure along a multi-segment trajectory in the Brillouin zone.
    pub fn band_structure_along_path(
        &self,
        waypoints: &[[f64; 3]],
        samples_per_segment: usize,
    ) -> Vec<BandPoint> {
        if waypoints.len() < 2 {
            return Vec::new();
        }

        let n_samples = samples_per_segment.max(2);
        let mut path_points = Vec::new();
        let mut cumulative_dist = 0.0;

        for w in 0..(waypoints.len() - 1) {
            let start = waypoints[w];
            let end = waypoints[w + 1];
            let seg_vec = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
            let seg_len = (seg_vec[0] * seg_vec[0] + seg_vec[1] * seg_vec[1] + seg_vec[2] * seg_vec[2]).sqrt();

            for s in 0..n_samples {
                if w > 0 && s == 0 {
                    continue;
                }
                let fraction = s as f64 / (n_samples - 1) as f64;
                let k = [
                    start[0] + fraction * seg_vec[0],
                    start[1] + fraction * seg_vec[1],
                    start[2] + fraction * seg_vec[2],
                ];
                let current_dist = cumulative_dist + fraction * seg_len;

                let mut valence = Vec::with_capacity(self.nodes.len());
                let mut conduction = Vec::with_capacity(self.nodes.len());
                let mut all = Vec::with_capacity(self.nodes.len() * 2);

                for node in &self.nodes {
                    let (e_minus, e_plus) = node.dispersion_at(k);
                    valence.push(e_minus);
                    conduction.push(e_plus);
                    all.push(e_minus);
                    all.push(e_plus);
                }

                all.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                path_points.push(BandPoint {
                    k,
                    path_coordinate: current_dist,
                    valence_energies: valence,
                    conduction_energies: conduction,
                    all_energies: all,
                });
            }

            cumulative_dist += seg_len;
        }

        path_points
    }
}
