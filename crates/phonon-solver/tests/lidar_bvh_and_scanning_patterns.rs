//! Integration Test: BVH Raycasting Accelerator & Multi-Architecture Scanning Patterns

use phonon_models::em::Vector3D;
use phonon_models::lidar::LidarScannerConfig;
use phonon_solver::lidar::{Aabb, BvhPrimitive, BvhTree};

#[test]
fn test_aabb_ray_intersection() {
    let aabb = Aabb::new(Vector3D::new(-1.0, -1.0, 5.0), Vector3D::new(1.0, 1.0, 7.0));

    // 1. Direct hit along Z-axis:
    let hit_t = aabb.intersect_ray(
        Vector3D::new(0.0, 0.0, 0.0),
        Vector3D::new(0.0, 0.0, 1.0),
        100.0,
    );
    assert_eq!(hit_t, Some(5.0));

    // 2. Ray pointing away (miss):
    let miss_t = aabb.intersect_ray(
        Vector3D::new(0.0, 0.0, 0.0),
        Vector3D::new(0.0, 0.0, -1.0),
        100.0,
    );
    assert!(miss_t.is_none());

    // 3. Ray passing to the side (miss):
    let side_miss = aabb.intersect_ray(
        Vector3D::new(0.0, 0.0, 0.0),
        Vector3D::new(1.0, 0.0, 1.0).normalize(),
        100.0,
    );
    assert!(side_miss.is_none());
}

#[test]
fn test_bvh_tree_hierarchical_construction_and_raycast() {
    let mut primitives = Vec::new();

    // Add 10 spheres along a line:
    for i in 1..=10 {
        primitives.push(BvhPrimitive::Sphere {
            center: Vector3D::new(0.0, 0.0, i as f64 * 3.0),
            radius: 0.5,
            albedo: 0.70,
            transmission: 0.0,
        });
    }

    let bvh = BvhTree::new(primitives);
    assert!(
        bvh.nodes.len() > 1,
        "BVH should have constructed hierarchical nodes"
    );

    // Raycast towards first sphere at z = 3.0 (radius 0.5 => front at z = 2.5):
    let hit = bvh.raycast(
        Vector3D::new(0.0, 0.0, 0.0),
        Vector3D::new(0.0, 0.0, 1.0),
        100.0,
    );
    assert!(hit.is_some());
    let h = hit.unwrap();
    assert!(
        (h.distance - 2.5).abs() < 1e-4,
        "Expected hit at 2.5 m, got {}",
        h.distance
    );
}

#[test]
fn test_bvh_multi_hit_transmission_for_multi_echo() {
    let primitives = vec![
        // Translucent foliage screen at z = 5.0 m (transmission = 0.5):
        BvhPrimitive::Box {
            min: Vector3D::new(-2.0, -2.0, 4.9),
            max: Vector3D::new(2.0, 2.0, 5.1),
            albedo: 0.40,
            transmission: 0.50,
        },
        // Solid wall at z = 15.0 m (opaque):
        BvhPrimitive::Box {
            min: Vector3D::new(-5.0, -5.0, 14.9),
            max: Vector3D::new(5.0, 5.0, 15.1),
            albedo: 0.60,
            transmission: 0.0,
        },
    ];

    let bvh = BvhTree::new(primitives);
    let hits = bvh.raycast_multi_hits(
        Vector3D::new(0.0, 0.0, 0.0),
        Vector3D::new(0.0, 0.0, 1.0),
        50.0,
        3,
    );

    assert_eq!(
        hits.len(),
        2,
        "Ray should penetrate translucent screen and hit solid wall behind"
    );
    assert!(
        (hits[0].distance - 4.9).abs() < 0.1,
        "First echo at ~4.9 m, got {}",
        hits[0].distance
    );
    assert!(
        (hits[1].distance - 14.9).abs() < 0.2,
        "Second echo at ~14.9 m, got {}",
        hits[1].distance
    );
}

#[test]
fn test_scanning_architecture_ray_generation() {
    let pos = Vector3D::new(0.0, 0.0, 2.0);

    // 1. Mechanical spinning 360-degree LiDAR:
    let config_spin = LidarScannerConfig::new_automotive_spinning_32ch(pos);
    let rays_spin = config_spin.generate_scan_rays();
    assert_eq!(rays_spin.len(), config_spin.total_points_per_frame());
    assert!(rays_spin.len() >= 32 * 900);

    // 2. MEMS Solid-State LiDAR:
    let config_mems = LidarScannerConfig::new_aerospace_mems_1550nm(pos);
    let rays_mems = config_mems.generate_scan_rays();
    assert_eq!(rays_mems.len(), 300 * 64);

    // 3. Flash LiDAR Array:
    let config_flash = LidarScannerConfig::new_flash_lidar(pos);
    let rays_flash = config_flash.generate_scan_rays();
    assert_eq!(rays_flash.len(), 64 * 48);
}
