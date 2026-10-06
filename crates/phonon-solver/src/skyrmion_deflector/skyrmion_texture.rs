#![deny(unsafe_code)]

//! Real-Space Acoustic Skyrmion Texture & Topological Charge Density Engine.
//!
//! Models continuous and discrete 2D unit vector fields n(x, y) describing acoustic
//! pseudo-spin textures with quantized topological charge (winding number) N_sk in Z:
//!   N_sk = (1 / 4*pi) \iint n \cdot (\partial_x n \times \partial_y n) dx dy

use std::f64::consts::PI;

/// Type of skyrmion profile texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkyrmionProfileKind {
    /// Neel-type hedgehog skyrmion (radial in-plane spins).
    Neel,
    /// Bloch-type vortex skyrmion (tangential in-plane spins).
    Bloch,
    /// Higher-order skyrmion with winding number N_sk = +/- 2.
    HigherOrder,
    /// Antiskyrmion with opposite topological sign.
    Antiskyrmion,
    /// Trivial uniform collinear ferromagnet (N_sk = 0).
    TrivialFerromagnet,
}

/// Geometric and topological parameters for the acoustic skyrmion texture.
#[derive(Debug, Clone)]
pub struct SkyrmionTextureParams {
    /// Physical width/length of the 2D domain in mm (e.g. 100.0 mm).
    pub domain_size_mm: f64,
    /// Skyrmion core radius R_sk in mm (e.g. 18.0 mm).
    pub radius_mm: f64,
    /// Domain wall width delta in mm (e.g. 6.0 mm).
    pub wall_width_mm: f64,
    /// Helicity angle gamma in radians (0 for Neel, pi/2 for Bloch).
    pub helicity_rad: f64,
    /// Chirality / core polarity p in {+1, -1} (spin at center: -1 for down, +1 for up).
    pub core_polarity: i32,
    /// Topological vorticity / winding index m in Z (1 for standard, 2 for higher-order, -1 for anti).
    pub vorticity: i32,
    /// Profile model kind.
    pub kind: SkyrmionProfileKind,
}

impl Default for SkyrmionTextureParams {
    fn default() -> Self {
        Self {
            domain_size_mm: 100.0,
            radius_mm: 18.0,
            wall_width_mm: 6.0,
            helicity_rad: 0.0,
            core_polarity: -1,
            vorticity: 1,
            kind: SkyrmionProfileKind::Neel,
        }
    }
}

impl SkyrmionTextureParams {
    /// Construct parameters for a higher-order skyrmion with N_sk = 2.
    pub fn higher_order(radius_mm: f64) -> Self {
        Self {
            domain_size_mm: 100.0,
            radius_mm,
            wall_width_mm: 6.0,
            helicity_rad: 0.0,
            core_polarity: -1,
            vorticity: 2,
            kind: SkyrmionProfileKind::HigherOrder,
        }
    }

    /// Construct parameters for a Bloch-type skyrmion.
    pub fn bloch(radius_mm: f64) -> Self {
        Self {
            domain_size_mm: 100.0,
            radius_mm,
            wall_width_mm: 6.0,
            helicity_rad: PI / 2.0,
            core_polarity: -1,
            vorticity: 1,
            kind: SkyrmionProfileKind::Bloch,
        }
    }

    /// Construct parameters for an antiskyrmion (N_sk = -1).
    pub fn antiskyrmion(radius_mm: f64) -> Self {
        Self {
            domain_size_mm: 100.0,
            radius_mm,
            wall_width_mm: 6.0,
            helicity_rad: 0.0,
            core_polarity: -1,
            vorticity: -1,
            kind: SkyrmionProfileKind::Antiskyrmion,
        }
    }

    /// Construct parameters for a trivial uniform texture (N_sk = 0).
    pub fn trivial() -> Self {
        Self {
            domain_size_mm: 100.0,
            radius_mm: 18.0,
            wall_width_mm: 6.0,
            helicity_rad: 0.0,
            core_polarity: 1,
            vorticity: 0,
            kind: SkyrmionProfileKind::TrivialFerromagnet,
        }
    }
}

/// Unit spin vector n = (nx, ny, nz) with |n| = 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpinVector {
    pub nx: f64,
    pub ny: f64,
    pub nz: f64,
}

impl SpinVector {
    pub fn new(nx: f64, ny: f64, nz: f64) -> Self {
        let norm = (nx * nx + ny * ny + nz * nz).sqrt();
        if norm > 1e-12 {
            Self {
                nx: nx / norm,
                ny: ny / norm,
                nz: nz / norm,
            }
        } else {
            Self { nx: 0.0, ny: 0.0, nz: 1.0 }
        }
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.nx * other.nx + self.ny * other.ny + self.nz * other.nz
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            nx: self.ny * other.nz - self.nz * other.ny,
            ny: self.nz * other.nx - self.nx * other.nz,
            nz: self.nx * other.ny - self.ny * other.nx,
        }
    }
}

/// 2D discrete spin vector texture grid.
#[derive(Debug, Clone)]
pub struct SkyrmionTexture {
    pub params: SkyrmionTextureParams,
    pub grid_n: usize,
    pub field: Vec<SpinVector>,
}

impl SkyrmionTexture {
    /// Generate a 2D spin vector texture over an N x N spatial grid.
    pub fn new(params: SkyrmionTextureParams, grid_n: usize) -> Self {
        let mut field = Vec::with_capacity(grid_n * grid_n);
        let l = params.domain_size_mm;
        let step = l / (grid_n as f64);
        let half_l = l / 2.0;

        for iy in 0..grid_n {
            let y = (iy as f64 + 0.5) * step - half_l;
            for ix in 0..grid_n {
                let x = (ix as f64 + 0.5) * step - half_l;
                let spin = Self::evaluate_spin_at(&params, x, y);
                field.push(spin);
            }
        }

        Self {
            params,
            grid_n,
            field,
        }
    }

    /// Evaluates the spin vector n(x, y) at continuous coordinates (x, y) in mm.
    pub fn evaluate_spin_at(params: &SkyrmionTextureParams, x: f64, y: f64) -> SpinVector {
        if params.kind == SkyrmionProfileKind::TrivialFerromagnet || params.vorticity == 0 {
            return SpinVector { nx: 0.0, ny: 0.0, nz: 1.0 };
        }

        let r = (x * x + y * y).sqrt();
        let phi = y.atan2(x);

        // Polar angle theta(r): theta(0) = pi (down), theta(infinity) = 0 (up) for polarity = -1
        let r0 = params.radius_mm.max(0.1);
        let theta = if params.core_polarity < 0 {
            // Core points down (nz = -1 at r=0), outer points up (nz = +1 at large r)
            PI * (1.0 - r / (r * r + r0 * r0).sqrt())
        } else {
            // Core points up, outer points down
            PI * (r / (r * r + r0 * r0).sqrt())
        };

        // In-plane azimuthal angle Psi(phi) = m * phi + gamma
        let m = params.vorticity as f64;
        let psi = m * phi + params.helicity_rad;

        let sin_th = theta.sin();
        let cos_th = theta.cos();

        let nx = sin_th * psi.cos();
        let ny = sin_th * psi.sin();
        let nz = cos_th;

        SpinVector::new(nx, ny, nz)
    }

    /// Retrieve the spin vector at discrete grid coordinate (ix, iy).
    pub fn get_spin(&self, ix: usize, iy: usize) -> SpinVector {
        let idx = iy * self.grid_n + ix;
        self.field[idx]
    }

    /// Evaluates the skyrmion topological charge density q_sk(x, y) in mm^-2:
    ///   q_sk = (1 / 4*pi) n \cdot (\partial_y n \times \partial_x n)
    pub fn charge_density_at(&self, ix: usize, iy: usize) -> f64 {
        if ix == 0 || ix >= self.grid_n - 1 || iy == 0 || iy >= self.grid_n - 1 {
            return 0.0;
        }

        let step_mm = self.params.domain_size_mm / (self.grid_n as f64);
        let two_dx = 2.0 * step_mm;

        let n = self.get_spin(ix, iy);
        let n_xp = self.get_spin(ix + 1, iy);
        let n_xm = self.get_spin(ix - 1, iy);
        let n_yp = self.get_spin(ix, iy + 1);
        let n_ym = self.get_spin(ix, iy - 1);

        // Finite difference spatial derivatives \partial_x n and \partial_y n
        let dndx = SpinVector {
            nx: (n_xp.nx - n_xm.nx) / two_dx,
            ny: (n_xp.ny - n_xm.ny) / two_dx,
            nz: (n_xp.nz - n_xm.nz) / two_dx,
        };

        let dndy = SpinVector {
            nx: (n_yp.nx - n_ym.nx) / two_dx,
            ny: (n_yp.ny - n_ym.ny) / two_dx,
            nz: (n_yp.nz - n_ym.nz) / two_dx,
        };

        let cross = dndy.cross(&dndx);
        let triple_product = n.dot(&cross);

        triple_product / (4.0 * PI)
    }

    /// Evaluates the total topological winding number N_sk = \iint q_sk dx dy.
    pub fn total_topological_charge(&self) -> f64 {
        if self.params.kind == SkyrmionProfileKind::TrivialFerromagnet || self.params.vorticity == 0 {
            return 0.0;
        }

        let step_mm = self.params.domain_size_mm / (self.grid_n as f64);
        let da = step_mm * step_mm;
        let mut sum_q = 0.0;

        for iy in 1..(self.grid_n - 1) {
            for ix in 1..(self.grid_n - 1) {
                sum_q += self.charge_density_at(ix, iy) * da;
            }
        }

        sum_q
    }

    /// Effective emergent synthetic magnetic field B_eff^z(x, y) experienced by acoustic wave packets:
    ///   B_eff^z = 2*pi * hbar * q_sk(x, y) [in normalized units proportional to q_sk].
    pub fn effective_gauge_field_at(&self, ix: usize, iy: usize) -> f64 {
        let q = self.charge_density_at(ix, iy);
        2.0 * PI * q
    }
}
