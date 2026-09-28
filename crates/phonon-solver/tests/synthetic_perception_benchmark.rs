//! Integration Test: Synthetic Optical Perception & 10,000-Frame Rayon Benchmark

use phonon_models::em::Vector3D;
use phonon_models::optics::{CameraIntrinsics, CmosPixelConfig, OpticalCamera};
use phonon_solver::optics::{
    AabbBox, CheckerPlane, OffscreenPerceptionEngine, OpticalBenchmarkRunner, OpticalRealismTier,
    SceneObject, Sphere, SyntheticScene,
};

#[test]
fn test_synthetic_scene_ray_intersections() {
    let mut scene = SyntheticScene::new(100.0);

    // Sphere at (0, 0, 5) radius 1
    scene.add_object(SceneObject::Sphere(Sphere::new(
        Vector3D::new(0.0, 0.0, 5.0),
        1.0,
        0.8,
        0.0,
    )));

    // Box at (3, 0, 5)
    scene.add_object(SceneObject::Box(AabbBox::new(
        Vector3D::new(2.5, -0.5, 4.5),
        Vector3D::new(3.5, 0.5, 5.5),
        0.6,
        0.0,
    )));

    // Plane at y = -2
    scene.add_object(SceneObject::Plane(CheckerPlane::new(
        Vector3D::new(0.0, -2.0, 0.0),
        Vector3D::new(0.0, 1.0, 0.0),
        1.0,
        0.9,
        0.1,
    )));

    // 1. Ray towards sphere:
    let ray_sphere =
        scene.intersect_ray(Vector3D::new(0.0, 0.0, 0.0), Vector3D::new(0.0, 0.0, 1.0));
    assert!(ray_sphere.is_some());
    let hit_s = ray_sphere.unwrap();
    assert!(
        (hit_s.distance_t - 4.0).abs() < 1e-4,
        "Sphere front hit distance: {}",
        hit_s.distance_t
    );

    // 2. Ray towards box:
    let dir_box = Vector3D::new(3.0, 0.0, 5.0).normalize();
    let ray_box = scene.intersect_ray(Vector3D::new(0.0, 0.0, 0.0), dir_box);
    assert!(ray_box.is_some());

    // 3. Ray towards plane:
    let dir_plane = Vector3D::new(0.0, -1.0, 2.0).normalize();
    let ray_plane = scene.intersect_ray(Vector3D::new(0.0, 0.0, 0.0), dir_plane);
    assert!(ray_plane.is_some());
}

#[test]
fn test_multi_tier_offscreen_perception_engine() {
    let intrinsics = CameraIntrinsics::new(32, 24, 35.0, 36.0, 27.0);
    let camera = OpticalCamera {
        intrinsics,
        position: Vector3D::new(0.0, 0.0, 0.0),
        forward: Vector3D::new(0.0, 0.0, 1.0),
        up: Vector3D::new(0.0, 1.0, 0.0),
        f_number: 2.8,
        focal_length_m: 0.035,
        focus_distance_m: 5.0,
        exposure_time_s: 0.01667,
        iso_gain: 1.0,
        sensor_config: CmosPixelConfig::new_standard_industrial(),
    };

    let scene = SyntheticScene::new_default_environment(1000.0);

    // Tier 0: Microscopic CMOS APS:
    let engine_t0 = OffscreenPerceptionEngine::new(
        camera.clone(),
        OpticalRealismTier::Tier0MicroscopicCmos,
        scene.clone(),
    );
    let frame_t0 = engine_t0.render_frame(101);
    assert_eq!(frame_t0.width, 32);
    assert_eq!(frame_t0.height, 24);
    assert_eq!(frame_t0.raw_electrons.len(), 32 * 24);
    assert_eq!(frame_t0.image_u8.len(), 32 * 24);

    // Tier 1: Physical Lens Raster:
    let engine_t1 = OffscreenPerceptionEngine::new(
        camera.clone(),
        OpticalRealismTier::Tier1PhysicalLensRaster,
        scene.clone(),
    );
    let frame_t1 = engine_t1.render_frame(202);
    assert_eq!(frame_t1.image_u8.len(), 32 * 24);

    // Tier 2: Accelerated Pinhole:
    let engine_t2 =
        OffscreenPerceptionEngine::new(camera, OpticalRealismTier::Tier2AcceleratedPinhole, scene);
    let frame_t2 = engine_t2.render_frame(303);
    assert_eq!(frame_t2.image_u8.len(), 32 * 24);
}

#[test]
fn test_10000_frame_parallel_optical_benchmark() {
    let report =
        OpticalBenchmarkRunner::run_benchmark(10_000, OpticalRealismTier::Tier0MicroscopicCmos);

    println!("\n=== Optical Perception Benchmark Report ===");
    println!("Total Frames: {}", report.total_frames);
    println!("Elapsed Time: {:.2} ms", report.elapsed_ms);
    println!("FPS Throughput: {:.2} FPS", report.frames_per_second);
    println!("Sensor Dynamic Range: {:.2} dB", report.dynamic_range_db);
    println!("Mean SNR: {:.2} dB", report.mean_snr_db);
    println!(
        "Daylight Saturation Detected: {}",
        report.daylight_saturation_detected
    );
    println!(
        "Low-Light Detection Verified: {}",
        report.low_light_detection_verified
    );
    println!("==========================================\n");

    assert_eq!(report.total_frames, 10_000);
    assert!(
        report.frames_per_second > 1000.0,
        "Throughput should be > 1000 FPS, got {}",
        report.frames_per_second
    );
    assert!(
        report.dynamic_range_db >= 70.0,
        "Sensor DR should be >= 70 dB, got {}",
        report.dynamic_range_db
    );
    assert!(
        report.daylight_saturation_detected,
        "Daylight saturation must be detected"
    );
    assert!(
        report.low_light_detection_verified,
        "Low-light signal detection must be verified"
    );
}
