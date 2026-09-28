//! Multi-Physics Asset Cache & Scene Composition Engine
//!
//! Manages 3D asset mesh instancing with spatial transformations,
//! world-space geometry flattening, and integrated BVH acceleration generation.

use crate::assets::spatial_index::{
    AcousticRayHit, MultiPhysicsBvh, OpticalRayHit, RfTransmissionResult, WorldTriangle,
};
use phonon_models::assets::{MaterialLibrary, Mesh3D};
use phonon_models::em::Vector3D;
use phonon_models::sensors::Quaternion;

/// Concrete instance of a 3D asset positioned and oriented in world Cartesian space.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshInstance {
    pub mesh: Mesh3D,
    pub position: Vector3D,
    pub orientation: Quaternion,
    pub scale: Vector3D,
}

impl MeshInstance {
    pub fn new(mesh: Mesh3D, position: Vector3D, orientation: Quaternion, scale: Vector3D) -> Self {
        Self {
            mesh,
            position,
            orientation,
            scale,
        }
    }

    /// Transforms local mesh vertices into world space and returns world triangles.
    pub fn to_world_triangles(&self) -> Vec<WorldTriangle> {
        let mut world_tris = Vec::with_capacity(self.mesh.triangles.len());

        let world_verts: Vec<Vector3D> = self
            .mesh
            .vertices
            .iter()
            .map(|v| {
                let scaled = Vector3D::new(
                    v.position.x * self.scale.x,
                    v.position.y * self.scale.y,
                    v.position.z * self.scale.z,
                );
                let rotated = self.orientation.rotate_vector_body_to_world(scaled);
                self.position + rotated
            })
            .collect();

        for tri in &self.mesh.triangles {
            let p0 = world_verts[tri.indices[0]];
            let p1 = world_verts[tri.indices[1]];
            let p2 = world_verts[tri.indices[2]];
            world_tris.push(WorldTriangle::new(p0, p1, p2, tri.material_id));
        }

        world_tris
    }
}

/// Unified multi-physics simulation scene holding instanced 3D assets and BVH accelerator.
#[derive(Debug, Clone)]
pub struct MultiPhysicsScene {
    pub library: MaterialLibrary,
    pub instances: Vec<MeshInstance>,
    pub bvh: Option<MultiPhysicsBvh>,
}

impl MultiPhysicsScene {
    pub fn new(library: MaterialLibrary) -> Self {
        Self {
            library,
            instances: Vec::new(),
            bvh: None,
        }
    }

    /// Adds an instanced 3D asset mesh with position, attitude, and scaling.
    pub fn add_instance(
        &mut self,
        mesh: Mesh3D,
        position: Vector3D,
        orientation: Quaternion,
        scale: Vector3D,
    ) {
        self.instances
            .push(MeshInstance::new(mesh, position, orientation, scale));
        self.bvh = None; // Invalidate cached BVH
    }

    /// Compiles world-space geometry and constructs the multi-physics BVH accelerator.
    pub fn build_acceleration_structure(&mut self) {
        let mut all_triangles = Vec::new();
        for inst in &self.instances {
            all_triangles.extend(inst.to_world_triangles());
        }
        self.bvh = Some(MultiPhysicsBvh::build(all_triangles));
    }

    /// Returns the total number of world-space triangles across all instances.
    pub fn total_triangles(&self) -> usize {
        self.instances
            .iter()
            .map(|inst| inst.mesh.triangles.len())
            .sum()
    }

    /// Queries the nearest optical/LiDAR hit.
    pub fn raycast_optical(
        &self,
        origin: Vector3D,
        dir: Vector3D,
        max_dist: f64,
    ) -> Option<OpticalRayHit> {
        let bvh = self.bvh.as_ref()?;
        bvh.raycast_optical(origin, dir, max_dist, &self.library)
    }

    /// Evaluates RF transmission along line-of-sight.
    pub fn raycast_rf_transmission(
        &self,
        origin: Vector3D,
        dir: Vector3D,
        max_dist: f64,
        freq_hz: f64,
    ) -> RfTransmissionResult {
        if let Some(bvh) = self.bvh.as_ref() {
            bvh.raycast_rf_transmission(origin, dir, max_dist, freq_hz, &self.library)
        } else {
            RfTransmissionResult {
                has_obstruction: false,
                total_attenuation_db: 0.0,
                reflection_power_loss_db: 0.0,
                materials_penetrated: Vec::new(),
            }
        }
    }

    /// Queries nearest acoustic reflecting surface.
    pub fn raycast_acoustic(
        &self,
        origin: Vector3D,
        dir: Vector3D,
        max_dist: f64,
    ) -> Option<AcousticRayHit> {
        let bvh = self.bvh.as_ref()?;
        bvh.raycast_acoustic(origin, dir, max_dist, &self.library)
    }
}
