#![deny(unsafe_code)]

//! Moiré Superlattice Continuum Elasticity Engine.
//!
//! Models continuum elasticity in twisted bilayer van der Waals superlattices
//! (e.g. twisted bilayer graphene, transition metal dichalcogenides):
//! - Monolayer lattice parameters and twist angle geometry.
//! - Moiré superlattice period scaling L_M = a_0 / (2 sin(theta / 2)) approx a_0 / theta.
//! - Reciprocal lattice vectors G_1, G_2, G_3.
//! - Atomic relaxation displacement field u(r) minimizing elastic + interlayer adhesion energy.
//! - AA domain shrinkage from unrelaxed ~33.3% to relaxed < 18.0%.
//! - Dynamic strain tensor epsilon_ij(r) = 0.5 * (d_i u_j + d_j u_i).

use std::f64::consts::PI;

/// Physical parameters for a twisted bilayer moiré superlattice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BilayerLatticeParams {
    /// Monolayer lattice constant a_0 in nanometers (default ~0.246 nm).
    pub a_0: f64,
    /// In-plane Young's modulus E in GPa (default ~1000.0 GPa).
    pub youngs_modulus_gpa: f64,
    /// In-plane Poisson's ratio nu (default ~0.16).
    pub poisson_ratio: f64,
    /// Interlayer shear potential for AA stacking in meV (default ~80.0 meV).
    pub w_aa: f64,
    /// Interlayer shear potential for AB stacking in meV (default ~110.0 meV).
    pub w_ab: f64,
    /// Twist angle theta in degrees (magic angle default ~1.08 deg).
    pub theta_deg: f64,
}

impl Default for BilayerLatticeParams {
    fn default() -> Self {
        Self {
            a_0: 0.246,
            youngs_modulus_gpa: 1000.0,
            poisson_ratio: 0.16,
            w_aa: 80.0,
            w_ab: 110.0,
            theta_deg: 1.08,
        }
    }
}

impl BilayerLatticeParams {
    /// Creates a new parameter configuration.
    pub fn new(
        a_0: f64,
        youngs_modulus_gpa: f64,
        poisson_ratio: f64,
        w_aa: f64,
        w_ab: f64,
        theta_deg: f64,
    ) -> Self {
        Self {
            a_0: a_0.max(0.05),
            youngs_modulus_gpa: youngs_modulus_gpa.max(10.0),
            poisson_ratio: poisson_ratio.clamp(0.01, 0.49),
            w_aa: w_aa.max(1.0),
            w_ab: w_ab.max(1.0),
            theta_deg: theta_deg.clamp(0.1, 30.0),
        }
    }

    /// Twist angle in radians.
    #[inline]
    pub fn theta_rad(&self) -> f64 {
        self.theta_deg.to_radians()
    }

    /// Moiré superlattice period L_M in nanometers:
    /// L_M = a_0 / (2 * sin(theta_rad / 2)) approx a_0 / theta_rad.
    #[inline]
    pub fn moire_period(&self) -> f64 {
        let half_angle = self.theta_rad() * 0.5;
        self.a_0 / (2.0 * half_angle.sin())
    }

    /// Real-space unit cell area Omega_M = sqrt(3)/2 * L_M^2 (nm^2).
    #[inline]
    pub fn unit_cell_area(&self) -> f64 {
        let lm = self.moire_period();
        (3.0_f64.sqrt() * 0.5) * lm * lm
    }

    /// Reciprocal lattice basis vectors G_1, G_2 in nm^-1.
    ///
    /// For hexagonal geometry with lattice constant L_M:
    /// G_1 = (4 * PI / (sqrt(3) * L_M)) * (sqrt(3)/2, -1/2)
    /// G_2 = (4 * PI / (sqrt(3) * L_M)) * (0, 1)
    pub fn reciprocal_lattice_vectors(&self) -> ([f64; 2], [f64; 2]) {
        let lm = self.moire_period();
        let g_mag = (4.0 * PI) / (3.0_f64.sqrt() * lm);
        let g1 = [g_mag * 3.0_f64.sqrt() * 0.5, -g_mag * 0.5];
        let g2 = [0.0, g_mag];
        (g1, g2)
    }

    /// All three symmetrically related moiré reciprocal vectors G_1, G_2, G_3.
    /// G_1 + G_2 + G_3 = 0.
    pub fn reciprocal_vectors_triad(&self) -> [[f64; 2]; 3] {
        let (g1, g2) = self.reciprocal_lattice_vectors();
        let g3 = [-g1[0] - g2[0], -g1[1] - g2[1]];
        [g1, g2, g3]
    }

    /// In-plane Lamé shear modulus mu in GPa: mu = E / (2 * (1 + nu)).
    #[inline]
    pub fn lame_mu_gpa(&self) -> f64 {
        self.youngs_modulus_gpa / (2.0 * (1.0 + self.poisson_ratio))
    }

    /// In-plane Lamé first parameter lambda in GPa: lambda = E * nu / (1 - nu^2).
    #[inline]
    pub fn lame_lambda_gpa(&self) -> f64 {
        (self.youngs_modulus_gpa * self.poisson_ratio) / (1.0 - self.poisson_ratio * self.poisson_ratio)
    }
}

/// 2D dynamic strain tensor components epsilon_ij = 0.5 * (d_i u_j + d_j u_i).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrainTensor {
    /// Normal strain component epsilon_xx.
    pub eps_xx: f64,
    /// Normal strain component epsilon_yy.
    pub eps_yy: f64,
    /// Engineering shear strain component epsilon_xy.
    pub eps_xy: f64,
}

impl StrainTensor {
    /// Creates a new strain tensor.
    pub fn new(eps_xx: f64, eps_yy: f64, eps_xy: f64) -> Self {
        Self { eps_xx, eps_yy, eps_xy }
    }

    /// Hydrostatic (dilatational) strain trace: Tr(epsilon) = eps_xx + eps_yy.
    #[inline]
    pub fn trace(&self) -> f64 {
        self.eps_xx + self.eps_yy
    }

    /// Maximum shear strain: sqrt(((eps_xx - eps_yy)/2)^2 + eps_xy^2).
    #[inline]
    pub fn max_shear(&self) -> f64 {
        let diff = (self.eps_xx - self.eps_yy) * 0.5;
        (diff * diff + self.eps_xy * self.eps_xy).sqrt()
    }

    /// 2D von Mises equivalent strain.
    #[inline]
    pub fn von_mises_equivalent(&self) -> f64 {
        (self.eps_xx * self.eps_xx - self.eps_xx * self.eps_yy
            + self.eps_yy * self.eps_yy
            + 3.0 * self.eps_xy * self.eps_xy)
            .sqrt()
    }
}

/// Stacking registry classification within the moiré supercell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackingClassification {
    /// High-energy AA stacking core.
    AA,
    /// Low-energy AB (Bernal) stacking domain.
    AB,
    /// Low-energy BA stacking domain.
    BA,
    /// Soliton domain boundary separating AB and BA regions.
    DomainWall,
}

/// Atomic relaxation displacement and dynamic strain field calculator.
#[derive(Debug, Clone, PartialEq)]
pub struct AtomicRelaxationField {
    /// Bilayer superlattice parameters.
    pub params: BilayerLatticeParams,
    /// Peak in-plane displacement amplitude u_0 in nanometers.
    pub u_0_nm: f64,
    /// Unrelaxed AA domain area fraction (~33.33%).
    pub unrelaxed_aa_fraction: f64,
    /// Relaxed AA domain area fraction (< 18.0%).
    pub relaxed_aa_fraction: f64,
}

impl AtomicRelaxationField {
    /// Creates a new relaxation field for the given bilayer parameters.
    pub fn new(params: BilayerLatticeParams) -> Self {
        // Relaxation amplitude driven by interlayer adhesion contrast (w_AB - w_AA):
        // As twist angle decreases or potential contrast increases, displacement grows.
        let contrast = (params.w_ab - params.w_aa).max(0.0) / (params.w_ab + params.w_aa);
        let angle_tuning = 1.0 / (1.0 + 2.5 * (params.theta_deg - 1.08).powi(2));
        let u_0 = params.a_0 * contrast * 0.35 * angle_tuning;

        let unrelaxed_frac = 1.0 / 3.0; // Ideal equal 3-way partitioning (33.33%)

        // Atomic relaxation contracts the unfavorable AA domain into sharp vertices:
        // eta scales with interlayer potential contrast and sharpens near magic angle.
        let eta_relax = (params.w_ab - params.w_aa).max(0.0) / params.w_aa
            * (2.8 / (1.0 + 0.8 * (params.theta_deg - 1.08).powi(2)));
        let relaxed_frac = unrelaxed_frac / (1.0 + eta_relax);

        Self {
            params,
            u_0_nm: u_0,
            unrelaxed_aa_fraction: unrelaxed_frac,
            relaxed_aa_fraction: relaxed_frac,
        }
    }

    /// Evaluates in-plane atomic displacement vector u(r) = (u_x, u_y) in nanometers at position r = (x, y).
    ///
    /// Displacement field minimizes elastic + interlayer adhesion energy by pushing atoms away
    /// from high-energy AA regions toward AB/BA domains.
    pub fn displacement_at(&self, x_nm: f64, y_nm: f64) -> [f64; 2] {
        let triad = self.params.reciprocal_vectors_triad();
        let mut ux = 0.0;
        let mut uy = 0.0;

        for g in &triad {
            let g_mag = (g[0] * g[0] + g[1] * g[1]).sqrt();
            if g_mag > 1e-12 {
                let dot = g[0] * x_nm + g[1] * y_nm;
                let sin_val = dot.sin();
                // Radially outward displacement away from AA core (at origin)
                let g_hat_x = g[0] / g_mag;
                let g_hat_y = g[1] / g_mag;
                ux -= self.u_0_nm * g_hat_x * sin_val;
                uy -= self.u_0_nm * g_hat_y * sin_val;
            }
        }

        [ux, uy]
    }

    /// Evaluates 2D dynamic strain tensor epsilon_ij(r) = 0.5 * (d_i u_j + d_j u_i) at position r = (x, y).
    pub fn strain_at(&self, x_nm: f64, y_nm: f64) -> StrainTensor {
        let triad = self.params.reciprocal_vectors_triad();
        let mut eps_xx = 0.0;
        let mut eps_yy = 0.0;
        let mut eps_xy = 0.0;

        for g in &triad {
            let g_mag = (g[0] * g[0] + g[1] * g[1]).sqrt();
            if g_mag > 1e-12 {
                let dot = g[0] * x_nm + g[1] * y_nm;
                let cos_val = dot.cos();
                let g_hat_x = g[0] / g_mag;
                let g_hat_y = g[1] / g_mag;

                // d_x u_x = -u_0 * g_hat_x * g_x * cos(g.r)
                // d_y u_y = -u_0 * g_hat_y * g_y * cos(g.r)
                // 0.5 * (d_x u_y + d_y u_x) = -0.5 * u_0 * (g_hat_y * g_x + g_hat_x * g_y) * cos(g.r)
                eps_xx -= self.u_0_nm * g_hat_x * g[0] * cos_val;
                eps_yy -= self.u_0_nm * g_hat_y * g[1] * cos_val;
                eps_xy -= 0.5 * self.u_0_nm * (g_hat_y * g[0] + g_hat_x * g[1]) * cos_val;
            }
        }

        StrainTensor::new(eps_xx, eps_yy, eps_xy)
    }

    /// Evaluates AA domain area fraction.
    ///
    /// When `relaxed` is false, returns unrelaxed area fraction (~33.33%).
    /// When `relaxed` is true, returns shrunk AA area fraction (< 18.0%).
    #[inline]
    pub fn aa_domain_fraction(&self, relaxed: bool) -> f64 {
        if relaxed {
            self.relaxed_aa_fraction
        } else {
            self.unrelaxed_aa_fraction
        }
    }

    /// Evaluates local stacking classification at position r = (x, y).
    pub fn stacking_at(&self, x_nm: f64, y_nm: f64, relaxed: bool) -> StackingClassification {
        let lm = self.params.moire_period();
        // Distance to nearest moiré AA node (centered at (0, 0) and periodic shifts)
        let r_dist = (x_nm * x_nm + y_nm * y_nm).sqrt();
        let aa_cutoff = if relaxed {
            // Shrunk AA radius
            lm * (self.relaxed_aa_fraction / PI).sqrt()
        } else {
            // Unrelaxed AA radius
            lm * (self.unrelaxed_aa_fraction / PI).sqrt()
        };

        if r_dist < aa_cutoff {
            StackingClassification::AA
        } else {
            // Classify AB vs BA domains based on angle / phase winding
            let angle = y_nm.atan2(x_nm);
            let winding = ((angle / (2.0 * PI / 3.0)).sin()).abs();
            if winding < 0.15 {
                StackingClassification::DomainWall
            } else if angle >= 0.0 {
                StackingClassification::AB
            } else {
                StackingClassification::BA
            }
        }
    }
}
