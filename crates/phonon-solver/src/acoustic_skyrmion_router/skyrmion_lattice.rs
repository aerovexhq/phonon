#![deny(unsafe_code)]

//! Acoustic Skyrmion Lattice and Topological Charge Calculation Engine.
//!
//! Models 2D pseudospin / acoustic velocity vector fields n(x, y) = (n_x, n_y, n_z)
//! with |n| = 1.0, representing isolated single skyrmions, hexagonal vortex lattices,
//! square lattices, and chiral domain wall interfaces.
//! Computes continuous and discrete topological charge densities q(x, y)
//! and verifies quantized topological invariant Q = double_integral q(x, y) dx dy.

use std::f64::consts::PI;

/// Type of acoustic skyrmion configuration or topological defect lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkyrmionLatticeType {
    /// Single isolated acoustic skyrmion (core at center, background at perimeter).
    SingleSkyrmion,
    /// Hexagonal (triangular) periodic skyrmion vortex crystal (triple-q state).
    HexagonalVortexLattice,
    /// Square lattice periodic skyrmion array (double-q state).
    SquareLattice,
    /// Chiral domain wall interface separating opposite topological pseudospin domains.
    DomainWallInterface,
}

/// Physical and computational grid parameters for the acoustic skyrmion lattice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionLatticeParams {
    /// Configuration category.
    pub lattice_type: SkyrmionLatticeType,
    /// Radius of the skyrmion core in micrometers (R_sk).
    pub skyrmion_radius_um: f64,
    /// Lattice constant / pitch for periodic arrays in micrometers (a_sk).
    pub lattice_pitch_um: f64,
    /// Helicity angle in radians (0.0 for Neel skyrmion, pi/2 for Bloch skyrmion).
    pub helicity_gamma_rad: f64,
    /// Vorticity / winding number (+1 for skyrmion, -1 for antiskyrmion).
    pub vorticity_m: i32,
    /// Spatial grid resolution along X.
    pub grid_nx: usize,
    /// Spatial grid resolution along Y.
    pub grid_ny: usize,
    /// Full side length of the square simulation domain in micrometers.
    pub domain_size_um: f64,
    /// Bulk acoustic propagation speed in m/s (default 343.0 m/s in air).
    pub acoustic_speed_ms: f64,
    /// Dimensionless viscous dissipation damping factor alpha.
    pub dissipation_alpha: f64,
}

impl Default for SkyrmionLatticeParams {
    fn default() -> Self {
        Self {
            lattice_type: SkyrmionLatticeType::SingleSkyrmion,
            skyrmion_radius_um: 50.0,
            lattice_pitch_um: 160.0,
            helicity_gamma_rad: 0.0, // Neel skyrmion
            vorticity_m: 1,
            grid_nx: 64,
            grid_ny: 64,
            domain_size_um: 400.0,
            acoustic_speed_ms: 343.0,
            dissipation_alpha: 0.05,
        }
    }
}

/// 3D unit pseudo-spin / acoustic velocity vector field n(x, y) across a 2D discrete grid.
///
/// Guaranteed to satisfy |n(x, y)| = 1.0 at every grid point.
#[derive(Debug, Clone, PartialEq)]
pub struct Vector3Field {
    /// Number of grid points along X.
    pub nx: usize,
    /// Number of grid points along Y.
    pub ny: usize,
    /// Grid spacing along X in micrometers.
    pub dx: f64,
    /// Grid spacing along Y in micrometers.
    pub dy: f64,
    /// Domain extent in micrometers.
    pub domain_size_um: f64,
    /// Flat vector of 3D unit vectors [n_x, n_y, n_z], indexed by j * nx + i.
    pub data: Vec<[f64; 3]>,
}

impl Vector3Field {
    /// Allocates an empty vector field of dimension `nx * ny` initialized to [0.0, 0.0, 1.0].
    pub fn new(nx: usize, ny: usize, domain_size_um: f64) -> Self {
        let dx = domain_size_um / (nx.max(1) as f64);
        let dy = domain_size_um / (ny.max(1) as f64);
        let data = vec![[0.0, 0.0, 1.0]; nx * ny];
        Self {
            nx,
            ny,
            dx,
            dy,
            domain_size_um,
            data,
        }
    }

    /// Linear index from 2D discrete coordinates.
    #[inline]
    pub fn index(&self, i: usize, j: usize) -> usize {
        j * self.nx + i
    }

    /// Retrieves unit vector [n_x, n_y, n_z] at grid index (i, j).
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> [f64; 3] {
        self.data[self.index(i, j)]
    }

    /// Sets unit vector at grid index (i, j), normalizing to ensure |n| = 1.0.
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, val: [f64; 3]) {
        let norm = (val[0] * val[0] + val[1] * val[1] + val[2] * val[2]).sqrt();
        let idx = self.index(i, j);
        if norm > 1e-15 {
            self.data[idx] = [val[0] / norm, val[1] / norm, val[2] / norm];
        } else {
            self.data[idx] = [0.0, 0.0, 1.0];
        }
    }

    /// Converts discrete grid index (i, j) to continuous coordinate (x, y) in micrometers,
    /// centered at the origin (x, y) in [-domain/2, domain/2].
    #[inline]
    pub fn grid_to_coord(&self, i: usize, j: usize) -> (f64, f64) {
        let x = -self.domain_size_um / 2.0 + (i as f64 + 0.5) * self.dx;
        let y = -self.domain_size_um / 2.0 + (j as f64 + 0.5) * self.dy;
        (x, y)
    }

    /// Constructs the vector field according to the provided `SkyrmionLatticeParams`.
    pub fn from_params(params: &SkyrmionLatticeParams) -> Self {
        let mut field = Self::new(params.grid_nx, params.grid_ny, params.domain_size_um);

        match params.lattice_type {
            SkyrmionLatticeType::SingleSkyrmion => {
                field.build_single_skyrmion(
                    0.0,
                    0.0,
                    params.skyrmion_radius_um,
                    params.helicity_gamma_rad,
                    params.vorticity_m,
                );
            }
            SkyrmionLatticeType::HexagonalVortexLattice => {
                field.build_hexagonal_lattice(
                    params.lattice_pitch_um,
                    params.helicity_gamma_rad,
                    params.vorticity_m,
                );
            }
            SkyrmionLatticeType::SquareLattice => {
                field.build_square_lattice(
                    params.lattice_pitch_um,
                    params.helicity_gamma_rad,
                    params.vorticity_m,
                );
            }
            SkyrmionLatticeType::DomainWallInterface => {
                field.build_domain_wall(
                    0.0,
                    params.skyrmion_radius_um * 0.5,
                    params.helicity_gamma_rad,
                );
            }
        }

        field
    }

    /// Builds a single isolated skyrmion centered at (x0, y0).
    ///
    /// Profile:
    /// - For r < R: theta(r) = pi * (1.0 - r / R), n_z(r) = cos(theta(r)).
    /// - For r >= R: theta(r) = 0.0, n_z(r) = +1.0.
    /// - In-plane: n_x = sin(theta) * cos(m * phi + gamma), n_y = sin(theta) * sin(m * phi + gamma).
    pub fn build_single_skyrmion(
        &mut self,
        x0: f64,
        y0: f64,
        radius_um: f64,
        gamma_rad: f64,
        vorticity_m: i32,
    ) {
        let r_safe = radius_um.max(1e-6);
        let m = vorticity_m as f64;

        for j in 0..self.ny {
            for i in 0..self.nx {
                let (x, y) = self.grid_to_coord(i, j);
                let dx = x - x0;
                let dy = y - y0;
                let r = (dx * dx + dy * dy).sqrt();
                let phi = dy.atan2(dx);

                if r < r_safe {
                    let theta = PI * (1.0 - r / r_safe);
                    let sin_theta = theta.sin();
                    let cos_theta = theta.cos();
                    let psi = m * phi + gamma_rad;

                    let nx = sin_theta * psi.cos();
                    let ny = sin_theta * psi.sin();
                    let nz = cos_theta;

                    self.set(i, j, [nx, ny, nz]);
                } else {
                    self.set(i, j, [0.0, 0.0, 1.0]);
                }
            }
        }
    }

    /// Builds a periodic hexagonal skyrmion crystal by superposing 3 helical spin spirals
    /// oriented at 120-degree angles (triple-q state).
    pub fn build_hexagonal_lattice(&mut self, pitch_um: f64, gamma_rad: f64, vorticity_m: i32) {
        let a_sk = pitch_um.max(1e-6);
        // Reciprocal wavevector magnitude for triangular lattice of pitch a_sk
        let q = 4.0 * PI / (3.0_f64.sqrt() * a_sk);
        let m = vorticity_m as f64;

        // 3 spiral angles at 120-degree increments: 0, 2pi/3, 4pi/3
        let angles = [0.0, 2.0 * PI / 3.0, 4.0 * PI / 3.0];
        let mut q_vecs = [[0.0, 0.0]; 3];
        let mut u_vecs = [[0.0, 0.0]; 3];

        for k in 0..3 {
            let theta_k = angles[k];
            q_vecs[k] = [q * theta_k.cos(), q * theta_k.sin()];

            // In-plane unit polarization
            let cos_k = (m * theta_k).cos();
            let sin_k = (m * theta_k).sin();
            let ux = gamma_rad.cos() * cos_k - gamma_rad.sin() * sin_k;
            let uy = gamma_rad.cos() * sin_k + gamma_rad.sin() * cos_k;
            u_vecs[k] = [ux, uy];
        }

        for j in 0..self.ny {
            for i in 0..self.nx {
                let (x, y) = self.grid_to_coord(i, j);

                let mut mx = 0.0;
                let mut my = 0.0;
                let mut mz = 1.0; // Positive background bias

                for k in 0..3 {
                    let phase = q_vecs[k][0] * x + q_vecs[k][1] * y;
                    let sin_p = phase.sin();
                    let cos_p = phase.cos();

                    mx -= sin_p * u_vecs[k][0];
                    my -= sin_p * u_vecs[k][1];
                    mz -= cos_p;
                }

                self.set(i, j, [mx, my, mz]);
            }
        }
    }

    /// Builds a periodic square skyrmion lattice (double-q state).
    pub fn build_square_lattice(&mut self, pitch_um: f64, gamma_rad: f64, vorticity_m: i32) {
        let a_sk = pitch_um.max(1e-6);
        let q = 2.0 * PI / a_sk;
        let m = vorticity_m as f64;

        for j in 0..self.ny {
            for i in 0..self.nx {
                let (x, y) = self.grid_to_coord(i, j);

                let p1 = q * x;
                let p2 = q * y;

                let s1 = p1.sin();
                let s2 = p2.sin();
                let c1 = p1.cos();
                let c2 = p2.cos();

                let ux1 = gamma_rad.cos();
                let uy1 = gamma_rad.sin() * m;
                let ux2 = -gamma_rad.sin() * m;
                let uy2 = gamma_rad.cos();

                let mx = -(s1 * ux1 + s2 * ux2);
                let my = -(s1 * uy1 + s2 * uy2);
                let mz = -(c1 + c2) + 0.8;

                self.set(i, j, [mx, my, mz]);
            }
        }
    }

    /// Builds a chiral domain wall interface at y = y0 with transition width delta_um.
    ///
    /// Profile: n_z(y) = tanh((y - y0) / delta), with in-plane sech((y - y0) / delta)
    /// rotated by helicity gamma_rad.
    pub fn build_domain_wall(&mut self, y0: f64, delta_um: f64, gamma_rad: f64) {
        let delta = delta_um.max(1e-6);

        for j in 0..self.ny {
            for i in 0..self.nx {
                let (_x, y) = self.grid_to_coord(i, j);
                let xi = (y - y0) / delta;
                let nz = xi.tanh();
                let in_plane = (1.0 - nz * nz).max(0.0).sqrt(); // sech(xi)

                let nx = in_plane * gamma_rad.sin();
                let ny = in_plane * gamma_rad.cos();

                self.set(i, j, [nx, ny, nz]);
            }
        }
    }
}

/// High-accuracy Topological Charge Calculator.
///
/// Computes local topological charge density:
/// q(x, y) = (1 / (4 * pi)) * n . (partial_x n x partial_y n)
/// and integrates total topological invariant Q = double_integral q(x, y) dx dy.
///
/// Also provides discrete solid-angle (Berg-Luscher) winding evaluation on lattice triangles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TopologicalChargeCalculator;

impl TopologicalChargeCalculator {
    /// Creates a new `TopologicalChargeCalculator`.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates partial derivative with respect to X at grid index (i, j).
    /// Uses periodic boundary conditions if `periodic` is true, otherwise central differences
    /// with one-sided boundary handling.
    pub fn partial_x(field: &Vector3Field, i: usize, j: usize, periodic: bool) -> [f64; 3] {
        let nx = field.nx;
        let (i_prev, i_next) = if periodic {
            ((i + nx - 1) % nx, (i + 1) % nx)
        } else if i == 0 {
            (0, 1.min(nx - 1))
        } else if i == nx - 1 {
            ((nx - 2).max(0), nx - 1)
        } else {
            (i - 1, i + 1)
        };

        let scale = if !periodic && (i == 0 || i == nx - 1) {
            field.dx
        } else {
            2.0 * field.dx
        };

        let n_prev = field.get(i_prev, j);
        let n_next = field.get(i_next, j);

        [
            (n_next[0] - n_prev[0]) / scale,
            (n_next[1] - n_prev[1]) / scale,
            (n_next[2] - n_prev[2]) / scale,
        ]
    }

    /// Evaluates partial derivative with respect to Y at grid index (i, j).
    pub fn partial_y(field: &Vector3Field, i: usize, j: usize, periodic: bool) -> [f64; 3] {
        let ny = field.ny;
        let (j_prev, j_next) = if periodic {
            ((j + ny - 1) % ny, (j + 1) % ny)
        } else if j == 0 {
            (0, 1.min(ny - 1))
        } else if j == ny - 1 {
            ((ny - 2).max(0), ny - 1)
        } else {
            (j - 1, j + 1)
        };

        let scale = if !periodic && (j == 0 || j == ny - 1) {
            field.dy
        } else {
            2.0 * field.dy
        };

        let n_prev = field.get(i, j_prev);
        let n_next = field.get(i, j_next);

        [
            (n_next[0] - n_prev[0]) / scale,
            (n_next[1] - n_prev[1]) / scale,
            (n_next[2] - n_prev[2]) / scale,
        ]
    }

    /// Computes the local topological charge density q(x, y) across the entire grid:
    /// q(x, y) = (1 / (4 * pi)) * n . (partial_x n x partial_y n).
    pub fn compute_charge_density(&self, field: &Vector3Field, periodic: bool) -> Vec<f64> {
        let mut density = vec![0.0; field.nx * field.ny];

        for j in 0..field.ny {
            for i in 0..field.nx {
                let n = field.get(i, j);
                let dx_n = Self::partial_x(field, i, j, periodic);
                let dy_n = Self::partial_y(field, i, j, periodic);

                // Cross product: (partial_x n x partial_y n)
                let cross_x = dx_n[1] * dy_n[2] - dx_n[2] * dy_n[1];
                let cross_y = dx_n[2] * dy_n[0] - dx_n[0] * dy_n[2];
                let cross_z = dx_n[0] * dy_n[1] - dx_n[1] * dy_n[0];

                // Scalar triple product: n . cross
                let triple = n[0] * cross_x + n[1] * cross_y + n[2] * cross_z;

                let q = triple / (4.0 * PI);
                density[field.index(i, j)] = q;
            }
        }

        density
    }

    /// Evaluates total integrated topological charge Q by integrating q(x, y) dx dy
    /// over the 2D domain.
    pub fn compute_total_charge(&self, field: &Vector3Field, periodic: bool) -> f64 {
        let density = self.compute_charge_density(field, periodic);
        let cell_area = field.dx * field.dy;
        density.iter().sum::<f64>() * cell_area
    }

    /// Computes the exact discrete lattice topological charge using the Berg-Luscher
    /// solid angle formula across elementary triangles of the 2D grid:
    /// tan(Omega / 2) = (n1 . (n2 x n3)) / (1 + n1.n2 + n2.n3 + n3.n1).
    pub fn compute_lattice_solid_angle_charge(&self, field: &Vector3Field, periodic: bool) -> f64 {
        let nx = field.nx;
        let ny = field.ny;
        let mut total_solid_angle = 0.0;

        let triangle_solid_angle = |v1: [f64; 3], v2: [f64; 3], v3: [f64; 3]| -> f64 {
            let cross_x = v2[1] * v3[2] - v2[2] * v3[1];
            let cross_y = v2[2] * v3[0] - v2[0] * v3[2];
            let cross_z = v2[0] * v3[1] - v2[1] * v3[0];
            let num = v1[0] * cross_x + v1[1] * cross_y + v1[2] * cross_z;

            let dot12 = v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2];
            let dot23 = v2[0] * v3[0] + v2[1] * v3[1] + v2[2] * v3[2];
            let dot31 = v3[0] * v1[0] + v3[1] * v1[1] + v3[2] * v1[2];
            let den = 1.0 + dot12 + dot23 + dot31;

            2.0 * num.atan2(den)
        };

        let i_limit = if periodic { nx } else { nx - 1 };
        let j_limit = if periodic { ny } else { ny - 1 };

        for j in 0..j_limit {
            let j_next = (j + 1) % ny;
            for i in 0..i_limit {
                let i_next = (i + 1) % nx;

                let n_00 = field.get(i, j);
                let n_10 = field.get(i_next, j);
                let n_11 = field.get(i_next, j_next);
                let n_01 = field.get(i, j_next);

                // Triangle 1: (00, 10, 11)
                total_solid_angle += triangle_solid_angle(n_00, n_10, n_11);
                // Triangle 2: (00, 11, 01)
                total_solid_angle += triangle_solid_angle(n_00, n_11, n_01);
            }
        }

        total_solid_angle / (4.0 * PI)
    }

    /// Verifies whether the calculated topological charge is quantized within tolerance.
    pub fn verify_charge_quantization(&self, q: f64, target_q: f64, max_residual: f64) -> bool {
        (q - target_q).abs() <= max_residual
    }
}
