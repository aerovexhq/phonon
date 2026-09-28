//! Integration Test: LiDAR Time-of-Flight Point Cloud Synthesis & 10,000-Point Rayon Benchmark

use phonon_models::em::Vector3D;
use phonon_models::lidar::{
    AtmosphericCondition, LaserPulseConfig, LidarScannerConfig, ScanningArchitecture,
};
use phonon_solver::lidar::{BvhPrimitive, BvhTree, LidarBenchmarkRunner, TofLidarEngine};

#[test]
fn test_tof_pointcloud_generation_and_precision() {
    // Flat target wall at x = 20.0 m:
    let primitives = vec![BvhPrimitive::Box {
        min: Vector3D::new(20.0, -10.0, -10.0),
        max: Vector3D::new(20.2, 10.0, 10.0),
        albedo: 0.80,
        transmission: 0.0,
    }];
    let bvh = BvhTree::new(primitives);

    let scanner = LidarScannerConfig {
        laser: LaserPulseConfig::new_automotive_905nm(),
        architecture: ScanningArchitecture::MemsSolidState {
            fov_horizontal_deg: 20.0,
            fov_vertical_deg: 10.0,
            horizontal_samples: 30,
            vertical_lines: 10,
            refresh_rate_hz: 10.0,
        },
        min_range_m: 0.5,
        max_range_m: 100.0,
        position: Vector3D::new(0.0, 0.0, 0.0),
        forward: Vector3D::new(1.0, 0.0, 0.0),
        up: Vector3D::new(0.0, 0.0, 1.0),
    };

    let engine = TofLidarEngine::new(scanner, AtmosphericCondition::ClearAir, bvh);
    let cloud = engine.scan_point_cloud(0, 42);

    assert!(
        !cloud.is_empty(),
        "Point cloud should contain synthesized points"
    );
    assert_eq!(cloud.len(), 300);

    // Verify range accuracy for center point:
    let center_pt = cloud
        .points
        .iter()
        .find(|p| p.channel_ring == 5 && p.azimuth_rad.abs() < 0.05);
    assert!(center_pt.is_some());
    let pt = center_pt.unwrap();
    assert!(
        (pt.range_m - 20.0).abs() < 0.05,
        "Measured range should be ~20.0 m (+-5 cm), got {}",
        pt.range_m
    );
    assert!(
        pt.snr_db > 20.0,
        "SNR at 20m on 80% albedo target should be high: {}",
        pt.snr_db
    );
}

#[test]
fn test_multi_echo_pulse_detection_in_point_cloud() {
    let primitives = vec![
        // Translucent canopy at x = 10.0 m (40% albedo, 60% transmission):
        BvhPrimitive::Box {
            min: Vector3D::new(9.9, -5.0, -5.0),
            max: Vector3D::new(10.1, 5.0, 5.0),
            albedo: 0.40,
            transmission: 0.60,
        },
        // Solid building at x = 25.0 m:
        BvhPrimitive::Box {
            min: Vector3D::new(24.9, -10.0, -10.0),
            max: Vector3D::new(25.1, 10.0, 10.0),
            albedo: 0.70,
            transmission: 0.0,
        },
    ];
    let bvh = BvhTree::new(primitives);

    let scanner = LidarScannerConfig {
        laser: LaserPulseConfig::new_aerospace_1550nm(),
        architecture: ScanningArchitecture::FlashLidar {
            fov_horizontal_deg: 10.0,
            fov_vertical_deg: 10.0,
            resolution_h: 10,
            resolution_v: 10,
            frame_rate_hz: 20.0,
        },
        min_range_m: 0.5,
        max_range_m: 100.0,
        position: Vector3D::new(0.0, 0.0, 0.0),
        forward: Vector3D::new(1.0, 0.0, 0.0),
        up: Vector3D::new(0.0, 0.0, 1.0),
    };

    let engine =
        TofLidarEngine::new(scanner, AtmosphericCondition::ClearAir, bvh).with_multi_echo(true, 3);
    let cloud = engine.scan_point_cloud(0, 101);

    let multi_echo_count = cloud.points.iter().filter(|p| p.echo_index > 0).count();
    assert!(
        multi_echo_count > 0,
        "Multi-echo detection must detect secondary pulse return through translucent canopy"
    );
}

#[test]
fn test_parallel_rayon_lidar_benchmark_runner() {
    let report = LidarBenchmarkRunner::run_benchmark(6);

    println!("\n=== LiDAR Time-of-Flight Perception Benchmark Report ===");
    println!("Total Scans: {}", report.total_scans);
    println!(
        "Total Points Synthesized: {}",
        report.total_points_synthesized
    );
    println!("Elapsed Time: {:.2} ms", report.elapsed_ms);
    println!("Throughput: {:.2} points/sec", report.points_per_second);
    println!("Frame Rate: {:.2} scans/sec", report.scans_per_second);
    println!("Range Measurement RMSE: {:.2} mm", report.range_rmse_mm);
    println!("Mean Target SNR: {:.2} dB", report.mean_snr_db);
    println!(
        "Multi-Echo Detection Verified: {}",
        report.multi_echo_detected
    );
    println!(
        "Fog Backscatter Clutter Detected: {}",
        report.fog_backscatter_clutter_detected
    );
    println!("=======================================================\n");

    assert!(
        report.total_points_synthesized >= 10_000,
        "Should synthesize >= 10,000 points"
    );
    assert!(
        report.points_per_second > 50_000.0,
        "Throughput should be > 50,000 points/sec, got {}",
        report.points_per_second
    );
    assert!(
        report.range_rmse_mm <= 25.0,
        "Ranging RMSE should be <= 25 mm, got {} mm",
        report.range_rmse_mm
    );
    assert!(
        report.multi_echo_detected,
        "Multi-echo returns must be detected"
    );
    assert!(
        report.fog_backscatter_clutter_detected,
        "Fog backscatter clutter must be detected"
    );
}
