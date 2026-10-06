#![deny(unsafe_code)]

//! Magnetic Skyrmion Lattice & Micromagnetic Texture Engine.
//!
//! Models 2D magnetic thin films hosting chiral Néel/Bloch skyrmions,
//! calculates topological invariant charge Q = +-1, evaluates the
//! Thiele equation of motion with Skyrmion Hall deflection, and
//! computes interaction with artificial synaptic pinning potentials.

use super::llgs_engine::{Vector3, GYROMAGNETIC_RATIO};
use std::f64::consts::PI;

/// Physical parameters of the 2D chiral magnetic thin film.
#[derive(Debug, Clone)]
pub struct SkyrmionGridParams {
    /// Number of grid cells in x direction.
    pub nx: usize,
    /// Number of grid cells in y direction.
    pub ny: usize,
    /// Discretization cell pitch in meters (e.g. 1.5 nm = 1.5e-9 m).
    pub cell_size_m: f64,
    /// Exchange stiffness constant A_ex in J / m (e.g. 15.0 pJ/m = 15e-12 J/m).
    pub a_ex: f64,
    /// Interfacial Dzyaloshinskii-Moriya Interaction (DMI) D in J / m^2 (e.g. 3.0 mJ/m^2 = 3e-3 J/m^2).
    pub dmi: f64,
    /// Perpendicular Magnetic Anisotropy (PMA) K_u in J / m^3 (e.g. 6.0e5 J/m^3).
    pub ku: f64,
    /// Saturation magnetization M_s in A / m (e.g. 5.8e5 A/m).
    pub ms: f64,
    /// Magnetic film thickness d_z in meters (e.g. 1.0 nm = 1.0e-9 m).
    pub thickness_m: f64,
    /// Gilbert damping alpha.
    pub alpha: f64,
}

impl Default for SkyrmionGridParams {
    fn default() -> Self {
        Self {
            nx: 32,
            ny: 32,
            cell_size_m: 1.5e-9, // 1.5 nm per cell (48 nm total width)
            a_ex: 15.0e-12,     // 15 pJ/m
            dmi: 3.2e-3,        // 3.2 mJ/m^2
            ku: 6.0e5,          // 600 kJ/m^3
            ms: 5.8e5,          // 580 kA/m
            thickness_m: 1.0e-9,// 1 nm
            alpha: 0.03,
        }
    }
}

/// Artificial synaptic pinning center on the magnetic track.
#[derive(Debug, Clone)]
pub struct PinningSite {
    /// Center coordinate x in meters.
    pub x_m: f64,
    /// Center coordinate y in meters.
    pub y_m: f64,
    /// Effective pinning radius in meters (e.g. 3.0 nm).
    pub radius_m: f64,
    /// Potential well depth U_0 in Joules (e.g. 1.0 eV = 1.6e-19 J).
    pub depth_j: f64,
}

impl PinningSite {
    pub fn new(x_m: f64, y_m: f64, radius_m: f64, depth_j: f64) -> Self {
        Self {
            x_m,
            y_m,
            radius_m,
            depth_j,
        }
    }

    /// Evaluates Gaussian pinning potential energy at position (x, y).
    pub fn energy_at(&self, x: f64, y: f64) -> f64 {
        let dx = x - self.x_m;
        let dy = y - self.y_m;
        let r_sq = dx * dx + dy * dy;
        -self.depth_j * (-r_sq / (2.0 * self.radius_m * self.radius_m)).exp()
    }

    /// Evaluates restoring force F = -grad(V) on the skyrmion at (x, y).
    pub fn force_at(&self, x: f64, y: f64) -> (f64, f64) {
        let dx = x - self.x_m;
        let dy = y - self.y_m;
        let r_sq = dx * dx + dy * dy;
        let factor = (self.depth_j / (self.radius_m * self.radius_m))
            * (-r_sq / (2.0 * self.radius_m * self.radius_m)).exp();
        (-factor * dx, -factor * dy)
    }
}

/// Discrete 2D thin film micromagnetic texture holding spin configurations.
#[derive(Debug, Clone)]
pub struct MagneticSkyrmionTexture {
    pub params: SkyrmionGridParams,
    /// 1D-flattened 2D grid of unit magnetization vectors m(x, y), len = nx * ny.
    pub spins: Vec<Vector3>,
    /// Array of artificial pinning sites.
    pub pinning_sites: Vec<PinningSite>,
}

impl MagneticSkyrmionTexture {
    /// Creates a new texture initialized to uniform ferromagnetic state (all spins pointing +z).
    pub fn new_ferromagnetic(params: SkyrmionGridParams) -> Self {
        let count = params.nx * params.ny;
        let spins = vec![Vector3::EZ; count];
        Self {
            params,
            spins,
            pinning_sites: Vec::new(),
        }
    }

    /// Returns index into the flat spin array.
    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.params.nx + x
    }

    /// Reads spin at (x, y) with clamping.
    pub fn get_spin(&self, x: usize, y: usize) -> Vector3 {
        let clamped_x = x.min(self.params.nx - 1);
        let clamped_y = y.min(self.params.ny - 1);
        self.spins[self.idx(clamped_x, clamped_y)]
    }

    /// Sets spin at (x, y).
    pub fn set_spin(&mut self, x: usize, y: usize, m: Vector3) {
        let i = self.idx(x, y);
        self.spins[i] = m.normalize();
    }

    /// Generates an isolated chiral Néel skyrmion centered at (center_x, center_y) cell coordinates.
    ///
    /// The polar profile is given by theta(r) = 2 * arctan(exp(-(r - R_0) / w)),
    /// where at r = 0, m_z = -1 (core down), and at r >> R_0, m_z = +1 (background up).
    pub fn initialize_neel_skyrmion(
        &mut self,
        center_x: f64,
        center_y: f64,
        radius_cells: f64,
        wall_width_cells: f64,
        chirality: f64, // 0.0 for hedgehog outward Néel, PI for inward Néel
    ) {
        let nx = self.params.nx;
        let ny = self.params.ny;

        for y in 0..ny {
            for x in 0..nx {
                let dx = x as f64 - center_x;
                let dy = y as f64 - center_y;
                let r = (dx * dx + dy * dy).sqrt();
                let phi = dy.atan2(dx);

                // Skyrmion profile: theta from PI (center) to 0 (periphery)
                let arg = -(r - radius_cells) / wall_width_cells.max(0.1);
                let theta = 2.0 * arg.exp().atan();

                // Magnetization vector:
                // m_x = sin(theta) * cos(phi + chirality)
                // m_y = sin(theta) * sin(phi + chirality)
                // m_z = -cos(theta)  (so at r=0: theta=pi -> -cos(pi) = +1, wait: theta=2*atan(exp(R0/w)) approx pi)
                // When arg is large positive (r << R0), exp(arg) >> 1, theta -> pi, cos(theta) -> -1.
                // We want core m_z = -1, background m_z = +1.
                // cos(theta) at theta=pi is -1, at theta=0 is +1.
                // Therefore m_z = cos(theta).
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();

                let mx = sin_theta * (phi + chirality).cos();
                let my = sin_theta * (phi + chirality).sin();
                let mz = cos_theta;

                self.set_spin(x, y, Vector3::new(mx, my, mz));
            }
        }
    }

    /// Computes the topological winding number (topological charge) Q:
    ///
    /// Q = (1 / (4 * pi)) * integral [ m . (dm/dx x dm/dy) ] dx dy
    ///
    /// For an ideal single skyrmion, Q evaluates to -1.0 (or +1.0).
    /// For a uniform ferromagnet, Q evaluates to 0.0.
    pub fn compute_topological_charge(&self) -> f64 {
        let nx = self.params.nx;
        let ny = self.params.ny;
        if nx < 3 || ny < 3 {
            return 0.0;
        }

        let mut sum_solid_angle = 0.0;

        for y in 1..ny - 1 {
            for x in 1..nx - 1 {
                let m = self.get_spin(x, y);

                // Central differences for dm/dx and dm/dy (in units of 2 * dx)
                let dm_dx = self.get_spin(x + 1, y).sub(self.get_spin(x - 1, y)).scale(0.5);
                let dm_dy = self.get_spin(x, y + 1).sub(self.get_spin(x, y - 1)).scale(0.5);

                let cross = dm_dx.cross(dm_dy);
                sum_solid_angle += m.dot(cross);
            }
        }

        sum_solid_angle / (4.0 * PI)
    }

    /// Evaluates the center of mass (X_c, Y_c) of the skyrmion core (where m_z < 0.5) in meters.
    pub fn compute_center_of_mass(&self) -> (f64, f64) {
        let nx = self.params.nx;
        let ny = self.params.ny;
        let a = self.params.cell_size_m;

        let mut sum_weight = 0.0;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;

        for y in 0..ny {
            for x in 0..nx {
                let m = self.get_spin(x, y);
                // Weight is 1.0 - m_z, peaking at m_z = -1 (weight = 2) and 0 at m_z = +1
                let w = (1.0 - m.z).max(0.0);
                let px = (x as f64 + 0.5) * a;
                let py = (y as f64 + 0.5) * a;

                sum_x += px * w;
                sum_y += py * w;
                sum_weight += w;
            }
        }

        if sum_weight > 1e-12 {
            (sum_x / sum_weight, sum_y / sum_weight)
        } else {
            (0.5 * nx as f64 * a, 0.5 * ny as f64 * a)
        }
    }

    /// Evaluates the effective skyrmion diameter in nanometers.
    pub fn compute_effective_diameter_nm(&self) -> f64 {
        let nx = self.params.nx;
        let ny = self.params.ny;
        let a = self.params.cell_size_m;

        // Count cells where m_z <= 0.0 (core region inside the wall)
        let mut core_cells = 0usize;
        for y in 0..ny {
            for x in 0..nx {
                if self.get_spin(x, y).z <= 0.0 {
                    core_cells += 1;
                }
            }
        }

        let area_m2 = core_cells as f64 * a * a;
        let diameter_m = 2.0 * (area_m2 / PI).sqrt();
        diameter_m * 1e9 // in nm
    }

    /// Evaluates the Thiele equation gyrovector G and dissipation tensor D:
    ///
    /// G = (0, 0, -4 * pi * Q * M_s * d_z / gamma)
    /// D = (M_s * d_z / gamma) * integral [ (dm/dx)^2 ] dx dy
    ///
    /// Computes the analytical Skyrmion Hall angle:
    /// theta_SkH = arctan( G_z / (alpha * D_xx) )
    pub fn compute_thiele_parameters(&self) -> (f64, f64, f64) {
        let q = self.compute_topological_charge();
        let ms = self.params.ms;
        let dz = self.params.thickness_m;
        let gamma = GYROMAGNETIC_RATIO;
        let alpha = self.params.alpha;

        // Gyrovector magnitude G_z in kg / s:
        let g_z = -4.0 * PI * q * (ms * dz / gamma);

        // Dissipation tensor D_xx:
        let mut d_sum = 0.0;
        let nx = self.params.nx;
        let ny = self.params.ny;

        for y in 1..ny - 1 {
            for x in 1..nx - 1 {
                let dm_dx = self.get_spin(x + 1, y).sub(self.get_spin(x - 1, y)).scale(0.5);
                d_sum += dm_dx.norm_sq();
            }
        }
        let d_xx = (ms * dz / gamma) * d_sum;

        // Skyrmion Hall angle in degrees:
        let hall_angle_rad = (g_z / (alpha * d_xx.max(1e-30))).atan();
        let hall_angle_deg = hall_angle_rad * 180.0 / PI;

        (g_z, d_xx, hall_angle_deg)
    }

    /// Evaluates total potential energy from artificial pinning centers:
    pub fn compute_pinning_energy(&self) -> f64 {
        let (cx, cy) = self.compute_center_of_mass();
        self.pinning_sites.iter().map(|site| site.energy_at(cx, cy)).sum()
    }

    /// Evaluates pinning restoring force F_pin = -grad(V) on the skyrmion center:
    pub fn compute_pinning_force(&self) -> (f64, f64) {
        let (cx, cy) = self.compute_center_of_mass();
        let mut total_fx = 0.0;
        let mut total_fy = 0.0;
        for site in &self.pinning_sites {
            let (fx, fy) = site.force_at(cx, cy);
            total_fx += fx;
            total_fy += fy;
        }
        (total_fx, total_fy)
    }

    /// Simulates skyrmion center displacement under drive current density J_e and pinning force.
    ///
    /// According to Thiele's equation:
    /// (G x v) - alpha * D * v + F_STT + F_pin = 0
    /// where F_STT_x = B_STT * J_e with B_STT = (hbar * pi * eta * R_sk) / (2 * e).
    pub fn compute_drift_velocity(&self, current_density_j: f64) -> (f64, f64) {
        let (g_z, d_xx, _) = self.compute_thiele_parameters();
        let alpha = self.params.alpha;
        let r_sk = (self.compute_effective_diameter_nm() * 0.5) * 1e-9;

        // Spin Hall / STT driving force on the skyrmion:
        let eta = 0.45;
        let b_stt = (super::llgs_engine::HBAR * PI * eta * r_sk)
            / (2.0 * super::llgs_engine::ELEMENTARY_CHARGE);
        let f_drive_x = b_stt * current_density_j;

        let (f_pin_x, f_pin_y) = self.compute_pinning_force();

        let fx = f_drive_x + f_pin_x;
        let fy = f_pin_y;

        // Solve 2x2 linear system for (v_x, v_y):
        // [ -alpha*D    -G_z    ] [ v_x ] = [ -fx ]
        // [  G_z       -alpha*D ] [ v_y ]   [ -fy ]
        // Det = (alpha*D)^2 + G_z^2
        let a = alpha * d_xx;
        let det = a * a + g_z * g_z;
        if det < 1e-40 {
            return (0.0, 0.0);
        }

        let vx = (a * fx - g_z * fy) / det;
        let vy = (g_z * fx + a * fy) / det;

        (vx, vy)
    }
}
