//! High-Performance Multi-Physics Spatial Acceleration BVH
//!
//! Provides hierarchical Bounding Volume Hierarchy (BVH) spatial indexing supporting:
//! 1. Optical/LiDAR raycasting: surface hits, BRDF properties, normal vectors.
//! 2. RF electromagnetic transmission: complex dielectric boundary reflections and wall penetration loss.
//! 3. Acoustic ray reflection: acoustic impedance mismatch reflection and absorption.

use phonon_models::assets::{Aabb3D, MaterialLibrary, Mesh3D};
use phonon_models::em::Vector3D;

/// World-space geometric triangle with assigned multi-physics material ID.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldTriangle {
    pub p0: Vector3D,
    pub p1: Vector3D,
    pub p2: Vector3D,
    pub normal: Vector3D,
    pub material_id: u32,
}

impl WorldTriangle {
    pub fn new(p0: Vector3D, p1: Vector3D, p2: Vector3D, material_id: u32) -> Self {
        let edge1 = p1 - p0;
        let edge2 = p2 - p0;
        let normal = edge1.cross(&edge2).normalize();
        Self {
            p0,
            p1,
            p2,
            normal,
            material_id,
        }
    }

    pub fn aabb(&self) -> Aabb3D {
        let mut aabb = Aabb3D::EMPTY;
        aabb.include_point(self.p0);
        aabb.include_point(self.p1);
        aabb.include_point(self.p2);
        aabb
    }

    pub fn centroid(&self) -> Vector3D {
        Vector3D::new(
            (self.p0.x + self.p1.x + self.p2.x) / 3.0,
            (self.p0.y + self.p1.y + self.p2.y) / 3.0,
            (self.p0.z + self.p1.z + self.p2.z) / 3.0,
        )
    }

    pub fn intersect_ray(&self, origin: Vector3D, dir: Vector3D) -> Option<f64> {
        Mesh3D::ray_intersect_triangle(self.p0, self.p1, self.p2, origin, dir).map(|(t, _, _)| t)
    }
}

/// Optical / LiDAR surface interaction result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpticalRayHit {
    /// Distance from ray origin in meters.
    pub distance_m: f64,
    /// Exact 3D Cartesian hit position in meters.
    pub hit_point: Vector3D,
    /// Surface unit normal vector.
    pub normal: Vector3D,
    /// Material database ID.
    pub material_id: u32,
    /// Diffuse surface albedo.
    pub diffuse_albedo: f64,
    /// Specular surface albedo.
    pub specular_albedo: f64,
    /// Surface roughness parameter.
    pub roughness: f64,
}

/// RF electromagnetic transmission evaluation through obstacles.
#[derive(Debug, Clone, PartialEq)]
pub struct RfTransmissionResult {
    /// True if ray intersected any obstacle.
    pub has_obstruction: bool,
    /// Total cumulative penetration loss in dB.
    pub total_attenuation_db: f64,
    /// Total reflected power fraction across interfaces.
    pub reflection_power_loss_db: f64,
    /// Multi-physics material IDs penetrated.
    pub materials_penetrated: Vec<u32>,
}

/// Acoustic ray reflection and transmission result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticRayHit {
    /// Distance from acoustic source to boundary in meters.
    pub distance_m: f64,
    /// 3D hit position.
    pub hit_point: Vector3D,
    /// Boundary surface normal.
    pub normal: Vector3D,
    /// Material ID.
    pub material_id: u32,
    /// Acoustic pressure reflection coefficient R = (Z2 - Z1)/(Z2 + Z1).
    pub reflection_coefficient: f64,
    /// Wall transmission loss in dB.
    pub transmission_loss_db: f64,
}

/// Node in the multi-physics BVH spatial acceleration tree.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiPhysicsBvhNode {
    pub aabb: Aabb3D,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub triangle_indices: Vec<usize>,
}

/// Hierarchical Multi-Physics Bounding Volume Hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiPhysicsBvh {
    pub triangles: Vec<WorldTriangle>,
    pub nodes: Vec<MultiPhysicsBvhNode>,
    pub root_idx: usize,
}

impl MultiPhysicsBvh {
    /// Builds a multi-physics BVH accelerator over a list of world-space triangles.
    pub fn build(triangles: Vec<WorldTriangle>) -> Self {
        if triangles.is_empty() {
            return Self {
                triangles: Vec::new(),
                nodes: vec![MultiPhysicsBvhNode {
                    aabb: Aabb3D::EMPTY,
                    left: None,
                    right: None,
                    triangle_indices: Vec::new(),
                }],
                root_idx: 0,
            };
        }

        let mut indices: Vec<usize> = (0..triangles.len()).collect();
        let mut nodes = Vec::new();
        let root_idx = Self::build_recursive(&triangles, &mut indices, &mut nodes, 0);

        Self {
            triangles,
            nodes,
            root_idx,
        }
    }

    fn build_recursive(
        triangles: &[WorldTriangle],
        indices: &mut [usize],
        nodes: &mut Vec<MultiPhysicsBvhNode>,
        depth: usize,
    ) -> usize {
        let mut aabb = Aabb3D::EMPTY;
        for &idx in indices.iter() {
            let tri_aabb = triangles[idx].aabb();
            aabb.include_point(tri_aabb.min);
            aabb.include_point(tri_aabb.max);
        }

        let node_idx = nodes.len();
        nodes.push(MultiPhysicsBvhNode {
            aabb,
            left: None,
            right: None,
            triangle_indices: Vec::new(),
        });

        // Leaf condition: <= 4 triangles or depth >= 20
        if indices.len() <= 4 || depth >= 20 {
            nodes[node_idx].triangle_indices = indices.to_vec();
            return node_idx;
        }

        // Choose split axis by largest extents:
        let extents = aabb.extents();
        let axis = if extents.x >= extents.y && extents.x >= extents.z {
            0
        } else if extents.y >= extents.x && extents.y >= extents.z {
            1
        } else {
            2
        };

        // Sort indices by centroid along chosen axis:
        indices.sort_by(|&a, &b| {
            let ca = triangles[a].centroid();
            let cb = triangles[b].centroid();
            let va = match axis {
                0 => ca.x,
                1 => ca.y,
                _ => ca.z,
            };
            let vb = match axis {
                0 => cb.x,
                1 => cb.y,
                _ => cb.z,
            };
            va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = indices.len() / 2;
        let (left_indices, right_indices) = indices.split_at_mut(mid);

        let left_child = Self::build_recursive(triangles, left_indices, nodes, depth + 1);
        let right_child = Self::build_recursive(triangles, right_indices, nodes, depth + 1);

        nodes[node_idx].left = Some(left_child);
        nodes[node_idx].right = Some(right_child);

        node_idx
    }

    /// Queries the nearest optical surface hit along a ray.
    pub fn raycast_optical(
        &self,
        origin: Vector3D,
        dir: Vector3D,
        max_dist: f64,
        library: &MaterialLibrary,
    ) -> Option<OpticalRayHit> {
        let mut closest_t = max_dist;
        let mut hit_tri_idx: Option<usize> = None;

        let mut stack = Vec::with_capacity(32);
        stack.push(self.root_idx);

        while let Some(node_idx) = stack.pop() {
            let node = &self.nodes[node_idx];

            if let Some((tmin, _)) = node.aabb.intersects_ray(origin, dir) {
                if tmin > closest_t {
                    continue;
                }

                if node.left.is_none() && node.right.is_none() {
                    for &tri_idx in &node.triangle_indices {
                        if let Some(t) = self.triangles[tri_idx].intersect_ray(origin, dir) {
                            if t < closest_t {
                                closest_t = t;
                                hit_tri_idx = Some(tri_idx);
                            }
                        }
                    }
                } else {
                    if let Some(r) = node.right {
                        stack.push(r);
                    }
                    if let Some(l) = node.left {
                        stack.push(l);
                    }
                }
            }
        }

        hit_tri_idx.map(|idx| {
            let tri = &self.triangles[idx];
            let hit_point = origin + dir * closest_t;
            let mat = library.get_by_id(tri.material_id);
            let (diffuse, specular, roughness) = if let Some(m) = mat {
                (
                    m.optical.diffuse_albedo,
                    m.optical.specular_albedo,
                    m.optical.roughness,
                )
            } else {
                (0.5, 0.2, 0.3)
            };

            OpticalRayHit {
                distance_m: closest_t,
                hit_point,
                normal: tri.normal,
                material_id: tri.material_id,
                diffuse_albedo: diffuse,
                specular_albedo: specular,
                roughness,
            }
        })
    }

    /// Evaluates RF electromagnetic wave transmission through all obstacles along a line of sight.
    pub fn raycast_rf_transmission(
        &self,
        origin: Vector3D,
        dir: Vector3D,
        max_dist: f64,
        freq_hz: f64,
        library: &MaterialLibrary,
    ) -> RfTransmissionResult {
        let mut hits = Vec::new();

        let mut stack = Vec::with_capacity(32);
        stack.push(self.root_idx);

        while let Some(node_idx) = stack.pop() {
            let node = &self.nodes[node_idx];

            if let Some((tmin, _)) = node.aabb.intersects_ray(origin, dir) {
                if tmin > max_dist {
                    continue;
                }

                if node.left.is_none() && node.right.is_none() {
                    for &tri_idx in &node.triangle_indices {
                        if let Some(t) = self.triangles[tri_idx].intersect_ray(origin, dir) {
                            if t <= max_dist {
                                hits.push((t, tri_idx));
                            }
                        }
                    }
                } else {
                    if let Some(r) = node.right {
                        stack.push(r);
                    }
                    if let Some(l) = node.left {
                        stack.push(l);
                    }
                }
            }
        }

        if hits.is_empty() {
            return RfTransmissionResult {
                has_obstruction: false,
                total_attenuation_db: 0.0,
                reflection_power_loss_db: 0.0,
                materials_penetrated: Vec::new(),
            };
        }

        // Sort hits by distance along ray:
        hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut total_att_db = 0.0;
        let mut reflection_loss_db = 0.0;
        let mut mats = Vec::new();

        for (_t, tri_idx) in hits {
            let tri = &self.triangles[tri_idx];
            mats.push(tri.material_id);

            if let Some(mat) = library.get_by_id(tri.material_id) {
                let r_power = mat.em.normal_power_reflection(freq_hz);
                let t_power = (1.0 - r_power).max(1e-6);
                let if_loss_db = -10.0 * t_power.log10();
                reflection_loss_db += if_loss_db;

                // Skin depth / attenuation in lossy materials:
                let skin_d = mat.em.skin_depth_m(freq_hz);
                let nominal_thickness = 0.10; // 10 cm nominal thickness
                if skin_d.is_finite() && skin_d > 0.0 {
                    let att_nepers = nominal_thickness / skin_d;
                    let att_db = att_nepers * 8.686;
                    total_att_db += att_db;
                } else {
                    // Dielectric attenuation:
                    let (_, eps_imag) = mat.em.complex_permittivity_at_freq(freq_hz);
                    let diel_loss_db = eps_imag * 5.0;
                    total_att_db += diel_loss_db;
                }
            } else {
                total_att_db += 3.0; // Default 3 dB loss
            }
        }

        RfTransmissionResult {
            has_obstruction: true,
            total_attenuation_db: total_att_db,
            reflection_power_loss_db: reflection_loss_db,
            materials_penetrated: mats,
        }
    }

    /// Queries the nearest acoustic reflecting boundary along a sound ray.
    pub fn raycast_acoustic(
        &self,
        origin: Vector3D,
        dir: Vector3D,
        max_dist: f64,
        library: &MaterialLibrary,
    ) -> Option<AcousticRayHit> {
        let air = library.get_by_name("Air STP")?;
        let hit = self.raycast_optical(origin, dir, max_dist, library)?;

        let mat = library.get_by_id(hit.material_id)?;
        let r_coeff = air.acoustic.reflection_coefficient(&mat.acoustic);
        let t_power = (1.0 - r_coeff * r_coeff).max(1e-6);
        let trans_loss_db = -10.0 * t_power.log10();

        Some(AcousticRayHit {
            distance_m: hit.distance_m,
            hit_point: hit.hit_point,
            normal: hit.normal,
            material_id: hit.material_id,
            reflection_coefficient: r_coeff,
            transmission_loss_db: trans_loss_db,
        })
    }
}
