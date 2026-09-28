//! High-Performance Bounding Volume Hierarchy (BVH) Raycasting Accelerator
//!
//! Provides spatial acceleration for high-throughput pulsed laser raycasting,
//! compatible with Aerovex AABB geometry and supporting multi-hit transmission
//! for multi-echo LiDAR returns.

use phonon_models::em::Vector3D;

/// 3D Axis-Aligned Bounding Box (AABB).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: Vector3D,
    pub max: Vector3D,
}

impl Aabb {
    pub fn new(min: Vector3D, max: Vector3D) -> Self {
        Self { min, max }
    }

    /// Creates an empty unbounded AABB.
    pub fn empty() -> Self {
        Self {
            min: Vector3D::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Vector3D::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    /// Expands the AABB to enclose a 3D point.
    pub fn expand_point(&mut self, p: Vector3D) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.min.z = self.min.z.min(p.z);

        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
        self.max.z = self.max.z.max(p.z);
    }

    /// Merges another AABB into this one.
    pub fn merge(&self, other: &Aabb) -> Self {
        Self {
            min: Vector3D::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vector3D::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    /// Center centroid of the bounding box.
    pub fn center(&self) -> Vector3D {
        (self.min + self.max) * 0.5
    }

    /// Surface area of the bounding box (for SAH heuristics).
    pub fn surface_area(&self) -> f64 {
        let d = self.max - self.min;
        let dx = d.x.max(0.0);
        let dy = d.y.max(0.0);
        let dz = d.z.max(0.0);
        2.0 * (dx * dy + dy * dz + dz * dx)
    }

    /// Ray-AABB intersection test using Kay-Kajiya slab method.
    pub fn intersect_ray(&self, origin: Vector3D, dir: Vector3D, max_t: f64) -> Option<f64> {
        let mut tmin = 0.0f64;
        let mut tmax = max_t;

        let inv_dx = if dir.x.abs() > 1e-9 { 1.0 / dir.x } else { 1e9 };
        let mut tx0 = (self.min.x - origin.x) * inv_dx;
        let mut tx1 = (self.max.x - origin.x) * inv_dx;
        if inv_dx < 0.0 {
            std::mem::swap(&mut tx0, &mut tx1);
        }
        tmin = tmin.max(tx0);
        tmax = tmax.min(tx1);
        if tmax < tmin {
            return None;
        }

        let inv_dy = if dir.y.abs() > 1e-9 { 1.0 / dir.y } else { 1e9 };
        let mut ty0 = (self.min.y - origin.y) * inv_dy;
        let mut ty1 = (self.max.y - origin.y) * inv_dy;
        if inv_dy < 0.0 {
            std::mem::swap(&mut ty0, &mut ty1);
        }
        tmin = tmin.max(ty0);
        tmax = tmax.min(ty1);
        if tmax < tmin {
            return None;
        }

        let inv_dz = if dir.z.abs() > 1e-9 { 1.0 / dir.z } else { 1e9 };
        let mut tz0 = (self.min.z - origin.z) * inv_dz;
        let mut tz1 = (self.max.z - origin.z) * inv_dz;
        if inv_dz < 0.0 {
            std::mem::swap(&mut tz0, &mut tz1);
        }
        tmin = tmin.max(tz0);
        tmax = tmax.min(tz1);
        if tmax < tmin {
            return None;
        }

        Some(tmin)
    }
}

/// Geometric surface primitive stored in the BVH.
#[derive(Debug, Clone, PartialEq)]
pub enum BvhPrimitive {
    Box {
        min: Vector3D,
        max: Vector3D,
        albedo: f64,
        transmission: f64,
    },
    Sphere {
        center: Vector3D,
        radius: f64,
        albedo: f64,
        transmission: f64,
    },
    Plane {
        point: Vector3D,
        normal: Vector3D,
        albedo: f64,
    },
}

impl BvhPrimitive {
    /// Computes the bounding box of the primitive.
    pub fn bounding_box(&self) -> Aabb {
        match self {
            Self::Box { min, max, .. } => Aabb::new(*min, *max),
            Self::Sphere { center, radius, .. } => {
                let r = *radius;
                Aabb::new(
                    Vector3D::new(center.x - r, center.y - r, center.z - r),
                    Vector3D::new(center.x + r, center.y + r, center.z + r),
                )
            }
            Self::Plane { point, .. } => {
                // Large finite slice for planar elements:
                let p = *point;
                let extent = 500.0;
                Aabb::new(
                    Vector3D::new(p.x - extent, p.y - extent, p.z - extent),
                    Vector3D::new(p.x + extent, p.y + extent, p.z + extent),
                )
            }
        }
    }

    /// Tests ray intersection with the primitive.
    pub fn intersect(&self, origin: Vector3D, dir: Vector3D, max_t: f64) -> Option<BvhHit> {
        match self {
            Self::Box {
                min,
                max,
                albedo,
                transmission,
            } => {
                let aabb = Aabb::new(*min, *max);
                let t_entry = aabb.intersect_ray(origin, dir, max_t)?;
                if t_entry <= 1e-4 {
                    return None;
                }
                let hit_p = origin + dir * t_entry;
                // Calculate normal on box face:
                let eps = 1e-4;
                let normal = if (hit_p.x - min.x).abs() < eps {
                    Vector3D::new(-1.0, 0.0, 0.0)
                } else if (hit_p.x - max.x).abs() < eps {
                    Vector3D::new(1.0, 0.0, 0.0)
                } else if (hit_p.y - min.y).abs() < eps {
                    Vector3D::new(0.0, -1.0, 0.0)
                } else if (hit_p.y - max.y).abs() < eps {
                    Vector3D::new(0.0, 1.0, 0.0)
                } else if (hit_p.z - min.z).abs() < eps {
                    Vector3D::new(0.0, 0.0, -1.0)
                } else {
                    Vector3D::new(0.0, 0.0, 1.0)
                };

                Some(BvhHit {
                    distance: t_entry,
                    hit_point: hit_p,
                    normal,
                    albedo: *albedo,
                    transmission: *transmission,
                })
            }
            Self::Sphere {
                center,
                radius,
                albedo,
                transmission,
            } => {
                let oc = origin - *center;
                let b = 2.0 * oc.dot(&dir);
                let c = oc.dot(&oc) - radius * radius;
                let disc = b * b - 4.0 * c;
                if disc < 0.0 {
                    return None;
                }
                let sqrt_disc = disc.sqrt();
                let t1 = (-b - sqrt_disc) / 2.0;
                let t2 = (-b + sqrt_disc) / 2.0;
                let t = if t1 > 1e-4 && t1 < max_t {
                    t1
                } else if t2 > 1e-4 && t2 < max_t {
                    t2
                } else {
                    return None;
                };

                let hit_p = origin + dir * t;
                let normal = (hit_p - *center).normalize();
                Some(BvhHit {
                    distance: t,
                    hit_point: hit_p,
                    normal,
                    albedo: *albedo,
                    transmission: *transmission,
                })
            }
            Self::Plane {
                point,
                normal,
                albedo,
            } => {
                let norm = normal.normalize();
                let denom = norm.dot(&dir);
                if denom.abs() < 1e-6 {
                    return None;
                }
                let t = (*point - origin).dot(&norm) / denom;
                if t <= 1e-4 || t >= max_t {
                    return None;
                }
                let hit_p = origin + dir * t;
                let face_normal = if denom < 0.0 { norm } else { -norm };
                Some(BvhHit {
                    distance: t,
                    hit_point: hit_p,
                    normal: face_normal,
                    albedo: *albedo,
                    transmission: 0.0,
                })
            }
        }
    }
}

/// Ray-surface intersection record.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BvhHit {
    pub distance: f64,
    pub hit_point: Vector3D,
    pub normal: Vector3D,
    pub albedo: f64,
    pub transmission: f64,
}

/// Internal Node in the Bounding Volume Hierarchy tree.
#[derive(Debug, Clone, PartialEq)]
pub struct BvhNode {
    pub aabb: Aabb,
    pub left_child: Option<usize>,
    pub right_child: Option<usize>,
    pub primitive_indices: Vec<usize>,
}

/// High-throughput Bounding Volume Hierarchy (BVH) spatial accelerator tree.
#[derive(Debug, Clone, PartialEq)]
pub struct BvhTree {
    pub primitives: Vec<BvhPrimitive>,
    pub nodes: Vec<BvhNode>,
}

impl Default for BvhTree {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl BvhTree {
    /// Constructs a hierarchical BVH tree from a list of primitives.
    pub fn new(primitives: Vec<BvhPrimitive>) -> Self {
        let mut tree = Self {
            primitives,
            nodes: Vec::new(),
        };

        if !tree.primitives.is_empty() {
            let mut indices: Vec<usize> = (0..tree.primitives.len()).collect();
            tree.build_recursive(&mut indices);
        }

        tree
    }

    fn build_recursive(&mut self, indices: &mut [usize]) -> usize {
        let mut bounds = Aabb::empty();
        for &idx in indices.iter() {
            let b = self.primitives[idx].bounding_box();
            bounds = bounds.merge(&b);
        }

        let node_idx = self.nodes.len();
        self.nodes.push(BvhNode {
            aabb: bounds,
            left_child: None,
            right_child: None,
            primitive_indices: Vec::new(),
        });

        // Leaf threshold:
        if indices.len() <= 4 {
            self.nodes[node_idx].primitive_indices = indices.to_vec();
            return node_idx;
        }

        // Determine widest dimension:
        let extent = bounds.max - bounds.min;
        let axis = if extent.x >= extent.y && extent.x >= extent.z {
            0
        } else if extent.y >= extent.z {
            1
        } else {
            2
        };

        // Sort indices along chosen axis by centroid:
        indices.sort_by(|&a, &b| {
            let ca = self.primitives[a].bounding_box().center();
            let cb = self.primitives[b].bounding_box().center();
            let val_a = if axis == 0 {
                ca.x
            } else if axis == 1 {
                ca.y
            } else {
                ca.z
            };
            let val_b = if axis == 0 {
                cb.x
            } else if axis == 1 {
                cb.y
            } else {
                cb.z
            };
            val_a
                .partial_cmp(&val_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = indices.len() / 2;
        let (left_slice, right_slice) = indices.split_at_mut(mid);

        let left_child = self.build_recursive(left_slice);
        let right_child = self.build_recursive(right_slice);

        self.nodes[node_idx].left_child = Some(left_child);
        self.nodes[node_idx].right_child = Some(right_child);

        node_idx
    }

    /// Casts a ray through the BVH tree, returning the closest surface hit within max_range.
    pub fn raycast(&self, origin: Vector3D, dir: Vector3D, max_range: f64) -> Option<BvhHit> {
        if self.nodes.is_empty() {
            return None;
        }

        let mut closest_hit: Option<BvhHit> = None;
        let mut closest_dist = max_range;
        let mut stack = Vec::with_capacity(32);
        stack.push(0usize);

        while let Some(node_idx) = stack.pop() {
            let node = &self.nodes[node_idx];
            if let Some(t_box) = node.aabb.intersect_ray(origin, dir, closest_dist) {
                if t_box >= closest_dist {
                    continue;
                }

                if !node.primitive_indices.is_empty() {
                    // Leaf node: intersect primitives
                    for &p_idx in &node.primitive_indices {
                        if let Some(hit) =
                            self.primitives[p_idx].intersect(origin, dir, closest_dist)
                        {
                            if hit.distance < closest_dist {
                                closest_dist = hit.distance;
                                closest_hit = Some(hit);
                            }
                        }
                    }
                } else {
                    if let Some(left) = node.left_child {
                        stack.push(left);
                    }
                    if let Some(right) = node.right_child {
                        stack.push(right);
                    }
                }
            }
        }

        closest_hit
    }

    /// Traces a ray through the scene collecting multiple transmission hits (for multi-echo return).
    pub fn raycast_multi_hits(
        &self,
        origin: Vector3D,
        dir: Vector3D,
        max_range: f64,
        max_echoes: usize,
    ) -> Vec<BvhHit> {
        let mut hits = Vec::with_capacity(max_echoes);
        let mut current_origin = origin;
        let mut remaining_range = max_range;
        let mut accumulated_dist = 0.0;

        for _ in 0..max_echoes {
            if let Some(hit) = self.raycast(current_origin, dir, remaining_range) {
                let actual_hit = BvhHit {
                    distance: accumulated_dist + hit.distance,
                    hit_point: hit.hit_point,
                    normal: hit.normal,
                    albedo: hit.albedo,
                    transmission: hit.transmission,
                };
                hits.push(actual_hit);

                if hit.transmission <= 1e-4 {
                    // Opaque surface stops optical penetration:
                    break;
                }

                // Advance ray past the translucent/foliage boundary:
                let advance = hit.distance + 0.05;
                if advance >= remaining_range {
                    break;
                }
                current_origin = current_origin + dir * advance;
                accumulated_dist += advance;
                remaining_range -= advance;
            } else {
                break;
            }
        }

        hits
    }
}
