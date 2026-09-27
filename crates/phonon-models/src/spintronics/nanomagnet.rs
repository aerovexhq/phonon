//! Single-Domain Nanomagnet Models, Demagnetization Tensors, and Magnetostatic Dipole Coupling.
//!
//! Models:
//! 1. Vector magnetization state \(\vec{m} = (m_x, m_y, m_z)\) with norm constraint \(|\vec{m}| = 1\).
//! 2. Shape and crystalline magnetic anisotropy (in-plane and perpendicular PMA).
//! 3. Rectangular prism demagnetization tensor \(\mathbf{N} = \text{diag}(N_x, N_y, N_z)\) where \(N_x + N_y + N_z = 1\).
//! 4. Magnetostatic dipole stray fields:
//!    \[\vec{H}_{\text{dip}}(\vec{r}) = \frac{1}{4\pi r^3} \left[ 3 (\vec{m}_{\text{moment}} \cdot \hat{r}) \hat{r} - \vec{m}_{\text{moment}} \right]\]
//! 5. Pairwise dipolar interaction energy \(E_{\text{dip}} = -\mu_0 \vec{m}_{\text{moment}, 1} \cdot \vec{H}_{\text{dip}, 2\to 1}\).

use phonon_core::MU_0;
use std::f64::consts::PI;

/// A 3D Cartesian vector with standard vector algebra operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[allow(clippy::should_implement_trait)]
impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn norm_sq(self) -> f64 {
        self.dot(self)
    }

    pub fn norm(self) -> f64 {
        self.norm_sq().sqrt()
    }

    pub fn normalize(self) -> Self {
        let n = self.norm();
        if n > 1.0e-30 {
            Self {
                x: self.x / n,
                y: self.y / n,
                z: self.z / n,
            }
        } else {
            Self::X
        }
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn scale(self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}

/// Ferromagnetic material properties for single-domain nanomagnetic computing.
#[derive(Debug, Clone, PartialEq)]
pub struct MagneticMaterial {
    pub name: String,
    /// Saturation magnetization \(M_s\) [A/m]
    pub ms_a_per_m: f64,
    /// Dimensionless Gilbert damping parameter \(\alpha\)
    pub alpha: f64,
    /// Exchange stiffness \(A_{\text{ex}}\) [J/m]
    pub exchange_stiffness_j_per_m: f64,
    /// Uniaxial magnetocrystalline or interfacial anisotropy \(K_u\) [\(\text{J/m}^3\)]
    pub ku_j_per_m3: f64,
    /// Gyromagnetic ratio \(\gamma_0 = \mu_0 \gamma\) [\(\text{rad}/(\text{s}\cdot\text{T})\)]
    pub gamma_0: f64,
}

impl MagneticMaterial {
    /// Cobalt-Iron-Boron (CoFeB) thin film.
    pub fn cofeb() -> Self {
        Self {
            name: "CoFeB".to_string(),
            ms_a_per_m: 1.0e6,                   // 1000 kA/m
            alpha: 0.015,                        // Low Gilbert damping
            exchange_stiffness_j_per_m: 1.5e-11, // 15 pJ/m
            ku_j_per_m3: 6.0e4,                  // 60 kJ/m^3
            gamma_0: 1.76e11,                    // ~1.76e11 rad/(s T)
        }
    }

    /// Permalloy (Ni80Fe20) soft ferromagnet.
    pub fn permalloy() -> Self {
        Self {
            name: "Permalloy".to_string(),
            ms_a_per_m: 8.0e5, // 800 kA/m
            alpha: 0.02,
            exchange_stiffness_j_per_m: 1.3e-11,
            ku_j_per_m3: 1.0e3, // Very low crystalline anisotropy
            gamma_0: 1.76e11,
        }
    }

    /// Perpendicular Magnetic Anisotropy (PMA) Co/Pt multilayer.
    pub fn pma_copt() -> Self {
        Self {
            name: "PMA_CoPt".to_string(),
            ms_a_per_m: 6.0e5,
            alpha: 0.035,
            exchange_stiffness_j_per_m: 1.0e-11,
            ku_j_per_m3: 3.5e5, // Strong PMA ~350 kJ/m^3
            gamma_0: 1.76e11,
        }
    }
}

/// A nanoscale single-domain ferromagnetic island with shape and dipole interactions.
#[derive(Debug, Clone)]
pub struct Nanomagnet {
    pub id: usize,
    /// Center spatial coordinates [m]
    pub position_m: Vec3,
    /// Physical length \(L_x\) [m]
    pub length_m: f64,
    /// Physical width \(L_y\) [m]
    pub width_m: f64,
    /// Physical thickness \(L_z\) [m]
    pub thickness_m: f64,
    /// Normalized magnetization direction \(|\vec{m}| = 1\)
    pub m: Vec3,
    /// Unit vector along the magnetic easy axis
    pub easy_axis: Vec3,
    /// Diagonal demagnetization factors \((N_x, N_y, N_z)\)
    pub demag_tensor: (f64, f64, f64),
    /// Underlying ferromagnetic material
    pub material: MagneticMaterial,
}

impl Nanomagnet {
    /// Creates a rectangular prism nanomagnet with analytical demagnetization factors.
    #[allow(clippy::too_many_arguments)]
    pub fn new_rectangular(
        id: usize,
        position_m: Vec3,
        length_m: f64,
        width_m: f64,
        thickness_m: f64,
        material: MagneticMaterial,
        easy_axis: Vec3,
        initial_m: Vec3,
    ) -> Self {
        let (nx, ny, nz) = Self::approximate_demag_factors(length_m, width_m, thickness_m);
        Self {
            id,
            position_m,
            length_m,
            width_m,
            thickness_m,
            m: initial_m.normalize(),
            easy_axis: easy_axis.normalize(),
            demag_tensor: (nx, ny, nz),
            material,
        }
    }

    /// Computes analytical Osborn/Aharoni demagnetizing factor approximations for a prism.
    pub fn approximate_demag_factors(lx: f64, ly: f64, lz: f64) -> (f64, f64, f64) {
        // Approximate ellipsoid/prism formulas
        let inv_x = 1.0 / lx;
        let inv_y = 1.0 / ly;
        let inv_z = 1.0 / lz;
        let total = inv_x + inv_y + inv_z;

        let nx = inv_x / total;
        let ny = inv_y / total;
        let nz = inv_z / total;

        // Ensure exact trace condition: Nx + Ny + Nz = 1.0
        let sum = nx + ny + nz;
        (nx / sum, ny / sum, nz / sum)
    }

    /// Returns the physical volume \(V = L_x \cdot L_y \cdot L_z\) [\(\text{m}^3\)].
    pub fn volume(&self) -> f64 {
        self.length_m * self.width_m * self.thickness_m
    }

    /// Returns the total magnetic moment vector \(\vec{\mu} = M_s V \vec{m}\) [\(\text{A}\cdot\text{m}^2\) or \(\text{J/T}\)].
    pub fn magnetic_moment(&self) -> Vec3 {
        let mag = self.material.ms_a_per_m * self.volume();
        self.m.scale(mag)
    }

    /// Evaluates the shape demagnetizing field \(\vec{H}_{\text{demag}} = -M_s (N_x m_x, N_y m_y, N_z m_z)\) [A/m].
    pub fn compute_demag_field(&self) -> Vec3 {
        let (nx, ny, nz) = self.demag_tensor;
        let ms = self.material.ms_a_per_m;
        Vec3::new(
            -ms * nx * self.m.x,
            -ms * ny * self.m.y,
            -ms * nz * self.m.z,
        )
    }

    /// Evaluates the uniaxial magnetic anisotropy field \(\vec{H}_{\text{ani}} = \frac{2 K_u}{\mu_0 M_s} (\vec{m} \cdot \hat{u}_k) \hat{u}_k\) [A/m].
    pub fn compute_anisotropy_field(&self) -> Vec3 {
        let ms = self.material.ms_a_per_m;
        let h_k = (2.0 * self.material.ku_j_per_m3) / (MU_0 * ms);
        let proj = self.m.dot(self.easy_axis);
        self.easy_axis.scale(h_k * proj)
    }

    /// Evaluates the magnetostatic dipole stray field produced by this magnet at an external target position \(\vec{r}_{\text{target}}\) [A/m].
    ///
    /// \[\vec{H}_{\text{dip}}(\vec{r}) = \frac{1}{4\pi r^3} \left[ 3 (\vec{\mu} \cdot \hat{r}) \hat{r} - \vec{\mu} \right]\]
    pub fn compute_dipole_field_at(&self, target_pos: Vec3) -> Vec3 {
        let r_vec = target_pos.sub(self.position_m);
        let dist = r_vec.norm();
        if dist < 1.0e-12 {
            return Vec3::ZERO;
        }

        let r_hat = r_vec.scale(1.0 / dist);
        let mu = self.magnetic_moment();
        let proj = mu.dot(r_hat);

        // Term: [3 (mu . r_hat) r_hat - mu]
        let bracket = r_hat.scale(3.0 * proj).sub(mu);
        let factor = 1.0 / (4.0 * PI * dist.powi(3));

        bracket.scale(factor)
    }

    /// Evaluates the mutual dipolar interaction energy \(E_{\text{dip}}\) [Joules] between this magnet and another magnet.
    ///
    /// \[E_{\text{dip}} = -\mu_0 \vec{\mu}_1 \cdot \vec{H}_{\text{dip}, 2\to 1}\]
    pub fn dipole_interaction_energy(&self, other: &Nanomagnet) -> f64 {
        let h_dip = other.compute_dipole_field_at(self.position_m);
        let mu = self.magnetic_moment();
        -MU_0 * mu.dot(h_dip)
    }

    /// Returns the Boolean logic state: true for positive projection along easy axis, false for negative.
    pub fn logic_state(&self) -> bool {
        self.m.dot(self.easy_axis) > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_operations() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, -5.0, 6.0);

        assert_eq!(a.dot(b), 4.0 - 10.0 + 18.0);
        let c = Vec3::X.cross(Vec3::Y);
        assert!((c.x - Vec3::Z.x).abs() < 1e-12);
        assert!((c.y - Vec3::Z.y).abs() < 1e-12);
        assert!((c.z - Vec3::Z.z).abs() < 1e-12);

        let n = a.normalize();
        assert!((n.norm() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_demag_tensor_trace_unity() {
        let (nx, ny, nz) = Nanomagnet::approximate_demag_factors(60.0e-9, 40.0e-9, 2.5e-9);
        let sum = nx + ny + nz;
        assert!((sum - 1.0).abs() < 1e-12, "Demag tensor trace must be 1.0");
        // Thinnest dimension (z) should have largest demag factor
        assert!(nz > nx);
        assert!(nz > ny);
    }

    #[test]
    fn test_side_by_side_antiferromagnetic_dipole_coupling() {
        // Two identical magnets placed side-by-side along the Y axis, with easy axis along X
        let pos1 = Vec3::new(0.0, 0.0, 0.0);
        let pos2 = Vec3::new(0.0, 60.0e-9, 0.0); // 60 nm spacing along Y

        let mat = MagneticMaterial::cofeb();
        let m1 = Nanomagnet::new_rectangular(
            1,
            pos1,
            60.0e-9,
            30.0e-9,
            3.0e-9,
            mat.clone(),
            Vec3::X,
            Vec3::X,
        );

        // Case A: Parallel alignment (both along +X)
        let m2_parallel = Nanomagnet::new_rectangular(
            2,
            pos2,
            60.0e-9,
            30.0e-9,
            3.0e-9,
            mat.clone(),
            Vec3::X,
            Vec3::X,
        );
        let e_parallel = m1.dipole_interaction_energy(&m2_parallel);

        // Case B: Anti-parallel alignment (m1 along +X, m2 along -X)
        let m2_antiparallel = Nanomagnet::new_rectangular(
            2,
            pos2,
            60.0e-9,
            30.0e-9,
            3.0e-9,
            mat,
            Vec3::X,
            Vec3::new(-1.0, 0.0, 0.0),
        );
        let e_antiparallel = m1.dipole_interaction_energy(&m2_antiparallel);

        // For side-by-side magnets, the dipole field is directed opposite to the source magnetization
        // Hence, anti-parallel alignment minimizes energy (E_antiparallel < E_parallel) -> Inversion!
        assert!(
            e_antiparallel < e_parallel,
            "Side-by-side magnets must favor anti-ferromagnetic coupling (Inverter action), E_anti={:.2e} J, E_para={:.2e} J",
            e_antiparallel,
            e_parallel
        );
    }

    #[test]
    fn test_collinear_ferromagnetic_dipole_coupling() {
        // Two identical magnets placed end-to-end (collinear) along the X axis, easy axis along X
        let pos1 = Vec3::new(0.0, 0.0, 0.0);
        let pos2 = Vec3::new(80.0e-9, 0.0, 0.0); // 80 nm spacing along X

        let mat = MagneticMaterial::cofeb();
        let m1 = Nanomagnet::new_rectangular(
            1,
            pos1,
            60.0e-9,
            30.0e-9,
            3.0e-9,
            mat.clone(),
            Vec3::X,
            Vec3::X,
        );

        let m2_parallel = Nanomagnet::new_rectangular(
            2,
            pos2,
            60.0e-9,
            30.0e-9,
            3.0e-9,
            mat.clone(),
            Vec3::X,
            Vec3::X,
        );
        let e_parallel = m1.dipole_interaction_energy(&m2_parallel);

        let m2_antiparallel = Nanomagnet::new_rectangular(
            2,
            pos2,
            60.0e-9,
            30.0e-9,
            3.0e-9,
            mat,
            Vec3::X,
            Vec3::new(-1.0, 0.0, 0.0),
        );
        let e_antiparallel = m1.dipole_interaction_energy(&m2_antiparallel);

        // For collinear end-to-end magnets, parallel alignment minimizes energy (E_parallel < E_antiparallel) -> Wire propagation!
        assert!(
            e_parallel < e_antiparallel,
            "Collinear magnets must favor ferromagnetic coupling (Wire transmission), E_para={:.2e} J, E_anti={:.2e} J",
            e_parallel,
            e_antiparallel
        );
    }
}
