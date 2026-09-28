//! 3D Asset Mesh Geometry, Submeshes & Physical Mass Properties
//!
//! Formulates 3D triangular mesh structures, vertex attributes, axis-aligned bounding boxes (AABB),
//! automated surface area, enclosed volume via divergence theorem, center of mass,
//! and 3x3 rotational moment of inertia tensors.

use crate::em::Vector3D;

/// Vertex data structure with Cartesian coordinate, surface normal, and texture UV.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex3D {
    pub position: Vector3D,
    pub normal: Vector3D,
    pub uv: (f64, f64),
}

impl Vertex3D {
    pub const fn new(position: Vector3D, normal: Vector3D, uv: (f64, f64)) -> Self {
        Self {
            position,
            normal,
            uv,
        }
    }
}

/// Triangular face referencing three vertex indices and an assigned material ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triangle3D {
    pub indices: [usize; 3],
    pub material_id: u32,
}

impl Triangle3D {
    pub const fn new(i0: usize, i1: usize, i2: usize, material_id: u32) -> Self {
        Self {
            indices: [i0, i1, i2],
            material_id,
        }
    }
}

/// Submesh partition grouping triangles sharing a common material ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submesh {
    pub name: String,
    pub material_id: u32,
    pub first_triangle: usize,
    pub num_triangles: usize,
}

/// 3D Axis-Aligned Bounding Box (AABB).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb3D {
    pub min: Vector3D,
    pub max: Vector3D,
}

impl Aabb3D {
    pub const EMPTY: Self = Self {
        min: Vector3D::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
        max: Vector3D::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
    };

    pub const fn new(min: Vector3D, max: Vector3D) -> Self {
        Self { min, max }
    }

    pub fn include_point(&mut self, p: Vector3D) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.min.z = self.min.z.min(p.z);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
        self.max.z = self.max.z.max(p.z);
    }

    pub fn center(&self) -> Vector3D {
        Vector3D::new(
            0.5 * (self.min.x + self.max.x),
            0.5 * (self.min.y + self.max.y),
            0.5 * (self.min.z + self.max.z),
        )
    }

    pub fn extents(&self) -> Vector3D {
        Vector3D::new(
            (self.max.x - self.min.x).max(0.0),
            (self.max.y - self.min.y).max(0.0),
            (self.max.z - self.min.z).max(0.0),
        )
    }

    /// Fast Kay-Kajiya slab ray-AABB intersection test.
    pub fn intersects_ray(&self, origin: Vector3D, dir: Vector3D) -> Option<(f64, f64)> {
        let inv_dx = if dir.x.abs() > 1e-12 {
            1.0 / dir.x
        } else {
            f64::INFINITY
        };
        let inv_dy = if dir.y.abs() > 1e-12 {
            1.0 / dir.y
        } else {
            f64::INFINITY
        };
        let inv_dz = if dir.z.abs() > 1e-12 {
            1.0 / dir.z
        } else {
            f64::INFINITY
        };

        let t1 = (self.min.x - origin.x) * inv_dx;
        let t2 = (self.max.x - origin.x) * inv_dx;
        let t3 = (self.min.y - origin.y) * inv_dy;
        let t4 = (self.max.y - origin.y) * inv_dy;
        let t5 = (self.min.z - origin.z) * inv_dz;
        let t6 = (self.max.z - origin.z) * inv_dz;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax >= tmin.max(0.0) {
            Some((tmin, tmax))
        } else {
            None
        }
    }
}

/// Complete 3D Asset Triangle Mesh with Material Subpartitions.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh3D {
    pub name: String,
    pub vertices: Vec<Vertex3D>,
    pub triangles: Vec<Triangle3D>,
    pub submeshes: Vec<Submesh>,
}

impl Mesh3D {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vertices: Vec::new(),
            triangles: Vec::new(),
            submeshes: Vec::new(),
        }
    }

    /// Computes the overall Axis-Aligned Bounding Box (AABB) of the mesh.
    pub fn compute_aabb(&self) -> Aabb3D {
        let mut aabb = Aabb3D::EMPTY;
        for v in &self.vertices {
            aabb.include_point(v.position);
        }
        aabb
    }

    /// Computes total surface area of all triangular faces in m^2.
    pub fn compute_surface_area(&self) -> f64 {
        let mut area = 0.0;
        for tri in &self.triangles {
            let p0 = self.vertices[tri.indices[0]].position;
            let p1 = self.vertices[tri.indices[1]].position;
            let p2 = self.vertices[tri.indices[2]].position;
            let edge1 = p1 - p0;
            let edge2 = p2 - p0;
            area += 0.5 * edge1.cross(&edge2).norm();
        }
        area
    }

    /// Computes enclosed solid volume in m^3 using the divergence theorem:
    /// V = 1/6 * sum( p0 . (p1 x p2) )
    pub fn compute_volume(&self) -> f64 {
        let mut vol = 0.0;
        for tri in &self.triangles {
            let p0 = self.vertices[tri.indices[0]].position;
            let p1 = self.vertices[tri.indices[1]].position;
            let p2 = self.vertices[tri.indices[2]].position;
            vol += p0.dot(&p1.cross(&p2));
        }
        (vol / 6.0).abs()
    }

    /// Computes 3D Center of Mass (centroid) in meters.
    pub fn compute_center_of_mass(&self) -> Vector3D {
        let mut total_vol = 0.0;
        let mut weighted_cm = Vector3D::ZERO;

        for tri in &self.triangles {
            let p0 = self.vertices[tri.indices[0]].position;
            let p1 = self.vertices[tri.indices[1]].position;
            let p2 = self.vertices[tri.indices[2]].position;

            let tetra_vol = p0.dot(&p1.cross(&p2)) / 6.0;
            total_vol += tetra_vol;
            let tetra_cm = Vector3D::new(
                0.25 * (p0.x + p1.x + p2.x),
                0.25 * (p0.y + p1.y + p2.y),
                0.25 * (p0.z + p1.z + p2.z),
            );
            weighted_cm = weighted_cm
                + Vector3D::new(
                    tetra_vol * tetra_cm.x,
                    tetra_vol * tetra_cm.y,
                    tetra_vol * tetra_cm.z,
                );
        }

        if total_vol.abs() > 1e-12 {
            Vector3D::new(
                weighted_cm.x / total_vol,
                weighted_cm.y / total_vol,
                weighted_cm.z / total_vol,
            )
        } else {
            self.compute_aabb().center()
        }
    }

    /// Computes 3x3 Moment of Inertia tensor matrix [Ixx, Ixy, Ixz, Iyx, Iyy, Iyz, Izx, Izy, Izz]
    /// around the center of mass in kg*m^2 for uniform density in kg/m^3.
    pub fn compute_inertia_tensor(&self, density_kg_m3: f64) -> [f64; 9] {
        let cm = self.compute_center_of_mass();
        let mut ixx = 0.0;
        let mut iyy = 0.0;
        let mut izz = 0.0;
        let mut ixy = 0.0;
        let mut ixz = 0.0;
        let mut iyz = 0.0;

        for tri in &self.triangles {
            let p0 = self.vertices[tri.indices[0]].position - cm;
            let p1 = self.vertices[tri.indices[1]].position - cm;
            let p2 = self.vertices[tri.indices[2]].position - cm;

            let det = p0.dot(&p1.cross(&p2));
            let v = det / 6.0;

            // Mirtich canonical tetrahedron integration:
            let x_sq =
                (p0.x * p0.x + p1.x * p1.x + p2.x * p2.x + p0.x * p1.x + p1.x * p2.x + p2.x * p0.x)
                    / 10.0;
            let y_sq =
                (p0.y * p0.y + p1.y * p1.y + p2.y * p2.y + p0.y * p1.y + p1.y * p2.y + p2.y * p0.y)
                    / 10.0;
            let z_sq =
                (p0.z * p0.z + p1.z * p1.z + p2.z * p2.z + p0.z * p1.z + p1.z * p2.z + p2.z * p0.z)
                    / 10.0;

            let xy = (2.0 * p0.x * p0.y
                + 2.0 * p1.x * p1.y
                + 2.0 * p2.x * p2.y
                + p0.x * p1.y
                + p1.x * p0.y
                + p1.x * p2.y
                + p2.x * p1.y
                + p2.x * p0.y
                + p0.x * p2.y)
                / 20.0;
            let xz = (2.0 * p0.x * p0.z
                + 2.0 * p1.x * p1.z
                + 2.0 * p2.x * p2.z
                + p0.x * p1.z
                + p1.x * p0.z
                + p1.x * p2.z
                + p2.x * p1.z
                + p2.x * p0.z
                + p0.x * p2.z)
                / 20.0;
            let yz = (2.0 * p0.y * p0.z
                + 2.0 * p1.y * p1.z
                + 2.0 * p2.y * p2.z
                + p0.y * p1.z
                + p1.y * p0.z
                + p1.y * p2.z
                + p2.y * p1.z
                + p2.y * p0.z
                + p0.y * p2.z)
                / 20.0;

            ixx += density_kg_m3 * v * (y_sq + z_sq);
            iyy += density_kg_m3 * v * (x_sq + z_sq);
            izz += density_kg_m3 * v * (x_sq + y_sq);
            ixy -= density_kg_m3 * v * xy;
            ixz -= density_kg_m3 * v * xz;
            iyz -= density_kg_m3 * v * yz;
        }

        [
            ixx.abs(),
            ixy,
            ixz,
            ixy,
            iyy.abs(),
            iyz,
            ixz,
            iyz,
            izz.abs(),
        ]
    }

    /// Fast Möller-Trumbore ray-triangle intersection algorithm:
    /// Returns (distance t, barycentric u, barycentric v) if hit.
    pub fn ray_intersect_triangle(
        p0: Vector3D,
        p1: Vector3D,
        p2: Vector3D,
        origin: Vector3D,
        dir: Vector3D,
    ) -> Option<(f64, f64, f64)> {
        let edge1 = p1 - p0;
        let edge2 = p2 - p0;
        let h = dir.cross(&edge2);
        let a = edge1.dot(&h);

        if a.abs() < 1e-12 {
            return None; // Ray is parallel to triangle plane
        }

        let f = 1.0 / a;
        let s = origin - p0;
        let u = f * s.dot(&h);
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = s.cross(&edge1);
        let v = f * dir.dot(&q);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(&q);
        if t > 1e-6 {
            Some((t, u, v))
        } else {
            None
        }
    }
}
