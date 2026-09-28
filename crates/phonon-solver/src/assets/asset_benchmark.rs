//! 10,000-Query Parallel Rayon Multi-Physics Spatial Benchmark Runner
//!
//! Evaluates cross-domain spatial query performance (optical/LiDAR, RF transmission, acoustic reflection)
//! over instanced 3D asset geometries and 100+ physical materials.

use crate::assets::asset_cache::MultiPhysicsScene;
use phonon_models::assets::{
    create_cubesat_chassis, create_dipole_antenna, create_finned_heatsink, create_patch_antenna,
    create_quadrotor_frame, create_tactile_landing_gear, MaterialLibrary,
};
use phonon_models::em::Vector3D;
use phonon_models::sensors::Quaternion;
use rayon::prelude::*;
use std::time::Instant;

/// Performance report summarizing multi-physics spatial query throughput and validation.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetBenchmarkReport {
    /// Total number of multi-physics spatial queries evaluated.
    pub total_queries: usize,
    /// Total elapsed wall-clock time in milliseconds.
    pub elapsed_ms: f64,
    /// Spatial query throughput in queries per second.
    pub queries_per_second: f64,
    /// Total optical / LiDAR surface hits detected.
    pub optical_hits: usize,
    /// Total RF electromagnetic transmission paths evaluated through obstacles.
    pub rf_transmissions_evaluated: usize,
    /// Total acoustic surface reflections detected.
    pub acoustic_hits: usize,
    /// Total geometric triangles indexed in the spatial BVH accelerator.
    pub total_primitives_indexed: usize,
    /// Approximate memory footprint of the scene geometry in bytes.
    pub memory_footprint_bytes: usize,
}

/// Parallel Rayon Multi-Physics Asset Benchmark Runner.
pub struct AssetBenchmarkRunner;

impl AssetBenchmarkRunner {
    /// Constructs a standardized multi-asset urban and aerospace test scene.
    pub fn create_benchmark_scene() -> MultiPhysicsScene {
        let library = MaterialLibrary::new();
        let mut scene = MultiPhysicsScene::new(library);

        // 1. Drone Carbon-Fiber Airframe at (0, 0, 5) m:
        let drone_frame = create_quadrotor_frame(0.45);
        scene.add_instance(
            drone_frame,
            Vector3D::new(0.0, 0.0, 5.0),
            Quaternion::default(),
            Vector3D::new(1.0, 1.0, 1.0),
        );

        // 2. Microstrip Patch Antenna mounted on drone at (0, 0, 5.02) m:
        let patch = create_patch_antenna(0.06, 0.06, 0.0016, 0.03, 0.03);
        scene.add_instance(
            patch,
            Vector3D::new(0.0, 0.0, 5.02),
            Quaternion::default(),
            Vector3D::new(1.0, 1.0, 1.0),
        );

        // 3. Finned Aluminum Heat Sink under drone avionics:
        let heatsink = create_finned_heatsink(0.05, 0.05, 0.005, 0.015, 6);
        scene.add_instance(
            heatsink,
            Vector3D::new(0.0, 0.0, 4.95),
            Quaternion::from_euler_rpy(std::f64::consts::PI, 0.0, 0.0),
            Vector3D::new(1.0, 1.0, 1.0),
        );

        // 4. Four Tactile Landing Gear Struts at (+/-0.15, +/-0.15, 4.8):
        for &(sx, sy) in &[(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
            let leg = create_tactile_landing_gear(0.12, 0.015);
            scene.add_instance(
                leg,
                Vector3D::new(sx * 0.15, sy * 0.15, 4.88),
                Quaternion::from_euler_rpy(std::f64::consts::PI, 0.0, 0.0),
                Vector3D::new(1.0, 1.0, 1.0),
            );
        }

        // 5. Aerospace CubeSat 3U Chassis at (10, 5, 20) m:
        let cubesat = create_cubesat_chassis(3);
        scene.add_instance(
            cubesat,
            Vector3D::new(10.0, 5.0, 20.0),
            Quaternion::from_euler_rpy(0.2, 0.4, 0.1),
            Vector3D::new(1.0, 1.0, 1.0),
        );

        // 6. Terrestrial RF Dipole Antenna at (-8, -4, 2) m:
        let dipole = create_dipole_antenna(0.50, 0.004, 12);
        scene.add_instance(
            dipole,
            Vector3D::new(-8.0, -4.0, 2.0),
            Quaternion::default(),
            Vector3D::new(1.0, 1.0, 1.0),
        );

        scene.build_acceleration_structure();
        scene
    }

    /// Runs the parallel Rayon benchmark across `num_queries` multi-physics spatial queries.
    pub fn run_benchmark(num_queries: usize) -> AssetBenchmarkReport {
        let scene = Self::create_benchmark_scene();
        let total_prims = scene.total_triangles();

        // Memory footprint estimation:
        let vert_bytes = scene
            .instances
            .iter()
            .map(|inst| inst.mesh.vertices.len() * 48)
            .sum::<usize>();
        let tri_bytes = scene
            .instances
            .iter()
            .map(|inst| inst.mesh.triangles.len() * 32)
            .sum::<usize>();
        let bvh_bytes = scene
            .bvh
            .as_ref()
            .map(|b| b.nodes.len() * 64 + b.triangles.len() * 80)
            .unwrap_or(0);
        let memory_bytes = vert_bytes + tri_bytes + bvh_bytes;

        let start = Instant::now();

        // Run queries in parallel across Rayon worker threads:
        let query_results: Vec<(usize, usize, usize)> = (0..num_queries)
            .into_par_iter()
            .map(|idx| {
                let angle = (idx as f64) * 0.037;
                let radius = 15.0;
                let origin = Vector3D::new(
                    radius * angle.cos(),
                    radius * angle.sin(),
                    5.0 + 3.0 * (angle * 0.5).sin(),
                );
                // Target center of drone or cubesat:
                let target = if idx % 2 == 0 {
                    Vector3D::new(0.0, 0.0, 5.0)
                } else {
                    Vector3D::new(10.0, 5.0, 20.0)
                };
                let dir = (target - origin).normalize();

                let mut opt_hit = 0;
                let mut rf_eval = 0;
                let mut ac_hit = 0;

                match idx % 3 {
                    0 => {
                        // Optical / LiDAR query:
                        if scene.raycast_optical(origin, dir, 50.0).is_some() {
                            opt_hit = 1;
                        }
                    }
                    1 => {
                        // RF Electromagnetic Transmission query:
                        let rf_res = scene.raycast_rf_transmission(origin, dir, 50.0, 2.4e9);
                        if rf_res.has_obstruction {
                            rf_eval = 1;
                        }
                    }
                    _ => {
                        // Acoustic wave ray query:
                        if scene.raycast_acoustic(origin, dir, 50.0).is_some() {
                            ac_hit = 1;
                        }
                    }
                }

                (opt_hit, rf_eval, ac_hit)
            })
            .collect();

        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;

        let mut total_opt = 0;
        let mut total_rf = 0;
        let mut total_ac = 0;

        for (opt, rf, ac) in query_results {
            total_opt += opt;
            total_rf += rf;
            total_ac += ac;
        }

        let queries_per_sec = (num_queries as f64) / (elapsed_ms / 1000.0).max(1e-6);

        AssetBenchmarkReport {
            total_queries: num_queries,
            elapsed_ms,
            queries_per_second: queries_per_sec,
            optical_hits: total_opt,
            rf_transmissions_evaluated: total_rf,
            acoustic_hits: total_ac,
            total_primitives_indexed: total_prims,
            memory_footprint_bytes: memory_bytes,
        }
    }
}
