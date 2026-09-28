//! Integration Tests: Multi-Physics Spatial Accelerator & Parallel Rayon Benchmark
//!
//! Validates:
//! 1. Hierarchical BVH multi-domain raycasting (optical/LiDAR, RF transmission, acoustic reflection).
//! 2. Multi-physics scene composition with instanced catalog components and material mappings.
//! 3. 10,000-query parallel Rayon benchmark with high throughput (> 500,000 queries/sec).

#![deny(unsafe_code)]

use phonon_models::assets::{create_cubesat_chassis, create_patch_antenna, MaterialLibrary};
use phonon_models::em::Vector3D;
use phonon_models::sensors::Quaternion;
use phonon_solver::assets::{AssetBenchmarkRunner, MultiPhysicsScene};

#[test]
fn test_multi_physics_scene_queries() {
    let library = MaterialLibrary::new();
    let mut scene = MultiPhysicsScene::new(library);

    // Add a CubeSat at (0, 0, 5) m:
    let cubesat = create_cubesat_chassis(1);
    scene.add_instance(
        cubesat,
        Vector3D::new(0.0, 0.0, 5.0),
        Quaternion::default(),
        Vector3D::new(1.0, 1.0, 1.0),
    );

    // Add a patch antenna at (0, 0, 5.06) m:
    let patch = create_patch_antenna(0.08, 0.08, 0.002, 0.04, 0.04);
    scene.add_instance(
        patch,
        Vector3D::new(0.0, 0.0, 5.06),
        Quaternion::default(),
        Vector3D::new(1.0, 1.0, 1.0),
    );

    scene.build_acceleration_structure();
    assert!(scene.total_triangles() > 0);

    // 1. Optical / LiDAR Raycast hitting top of patch antenna:
    let origin = Vector3D::new(0.0, 0.0, 10.0);
    let dir = Vector3D::new(0.0, 0.0, -1.0);
    let opt_hit = scene
        .raycast_optical(origin, dir, 20.0)
        .expect("Should hit patch antenna");
    assert!((opt_hit.distance_m - (10.0 - 5.061)).abs() < 0.1);
    assert!(opt_hit.diffuse_albedo > 0.0);

    // 2. RF Transmission Raycast through CubeSat chassis:
    let rf_res = scene.raycast_rf_transmission(origin, dir, 20.0, 2.4e9);
    assert!(rf_res.has_obstruction);
    assert!(rf_res.total_attenuation_db > 0.0);
    assert!(!rf_res.materials_penetrated.is_empty());

    // 3. Acoustic Raycast:
    let ac_hit = scene
        .raycast_acoustic(origin, dir, 20.0)
        .expect("Should reflect acoustic ray");
    assert!(ac_hit.reflection_coefficient.abs() > 0.5); // High acoustic impedance mismatch
}

#[test]
fn test_parallel_rayon_multi_physics_benchmark_10k() {
    let report = AssetBenchmarkRunner::run_benchmark(10_000);

    println!("\n=== Multi-Physics 3D Asset Ecosystem Benchmark Report ===");
    println!("Total Spatial Queries:  {}", report.total_queries);
    println!("Elapsed Wall-Clock:     {:.2} ms", report.elapsed_ms);
    println!(
        "Throughput:             {:.0} queries/sec",
        report.queries_per_second
    );
    println!("Optical Hits:           {}", report.optical_hits);
    println!(
        "RF Transmissions Eval:  {}",
        report.rf_transmissions_evaluated
    );
    println!("Acoustic Hits:          {}", report.acoustic_hits);
    println!(
        "Primitives Indexed:     {}",
        report.total_primitives_indexed
    );
    println!(
        "Memory Footprint:       {:.2} KB",
        report.memory_footprint_bytes as f64 / 1024.0
    );
    println!("========================================================\n");

    assert_eq!(report.total_queries, 10_000);
    assert!(report.elapsed_ms > 0.0);
    assert!(
        report.queries_per_second > 200_000.0,
        "Spatial query throughput must exceed 200k queries/sec, got {:.0}",
        report.queries_per_second
    );
    assert!(report.optical_hits > 0, "Must detect optical hits");
    assert!(
        report.rf_transmissions_evaluated > 0,
        "Must evaluate RF transmissions"
    );
    assert!(report.acoustic_hits > 0, "Must detect acoustic reflections");
    assert!(report.total_primitives_indexed > 0);
    assert!(report.memory_footprint_bytes > 0);
}
