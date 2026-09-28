//! Integration Test: Optical Camera Intrinsics, Brown-Conrady Distortion & CoC Blur

use phonon_models::em::Vector3D;
use phonon_models::optics::{CameraIntrinsics, OpticalCamera};

#[test]
fn test_camera_intrinsics_fov_and_pinhole_projection() {
    // 1920x1080 resolution, 35mm lens, 36x20.25mm sensor (approx 16:9 full frame):
    let intrinsics = CameraIntrinsics::new(1920, 1080, 35.0, 36.0, 20.25);

    let hfov_deg = intrinsics.horizontal_fov_rad().to_degrees();
    let vfov_deg = intrinsics.vertical_fov_rad().to_degrees();

    assert!(
        hfov_deg > 50.0 && hfov_deg < 65.0,
        "Expected ~54 deg HFOV, got {}",
        hfov_deg
    );
    assert!(
        vfov_deg > 30.0 && vfov_deg < 40.0,
        "Expected ~32 deg VFOV, got {}",
        vfov_deg
    );

    // Center point directly in front:
    let p_center = Vector3D::new(0.0, 0.0, 5.0);
    let (u, v) = intrinsics
        .project_to_pixel(p_center)
        .expect("Projection failed");
    assert!(
        (u - intrinsics.cx).abs() < 1e-4,
        "Center X projection offset: {}",
        u
    );
    assert!(
        (v - intrinsics.cy).abs() < 1e-4,
        "Center Y projection offset: {}",
        v
    );

    // Point behind camera should return None:
    let p_behind = Vector3D::new(0.0, 0.0, -1.0);
    assert!(intrinsics.project_to_pixel(p_behind).is_none());
}

#[test]
fn test_brown_conrady_radial_distortion_and_unprojection() {
    let mut intrinsics = CameraIntrinsics::new(1920, 1080, 24.0, 36.0, 20.25);
    // Add barrel distortion (negative k1):
    intrinsics = intrinsics.with_distortion(-0.15, 0.02, 0.0, 0.001, -0.001);

    // Off-center point:
    let p_test = Vector3D::new(1.2, 0.8, 4.0);
    let (u_dist, v_dist) = intrinsics
        .project_to_pixel(p_test)
        .expect("Projection failed");

    // Unproject pixel back to ray:
    let ray = intrinsics.unproject_to_ray(u_dist, v_dist);

    // Normalized ideal direction:
    let expected_dir = p_test.normalize();
    let dot = ray.dot(&expected_dir);
    assert!(
        dot > 0.999,
        "Unprojected ray direction dot product {} should be close to 1.0",
        dot
    );
}

#[test]
fn test_depth_of_field_circle_of_confusion() {
    let intrinsics = CameraIntrinsics::new(1920, 1080, 50.0, 36.0, 20.25);
    let focus_dist = 3.0; // focused at 3 meters
    let f_number = 2.8;
    let focal_len = 0.050; // 50 mm

    // At focus plane (3.0 m), CoC should be practically zero:
    let coc_in_focus = intrinsics.circle_of_confusion_m(3.0, focus_dist, f_number, focal_len);
    assert!(
        coc_in_focus < 1e-9,
        "CoC at focus distance should be 0, got {}",
        coc_in_focus
    );

    // At close distance (1.0 m, foreground out-of-focus):
    let coc_foreground = intrinsics.circle_of_confusion_m(1.0, focus_dist, f_number, focal_len);
    assert!(
        coc_foreground > 1e-5,
        "Foreground CoC should be significant: {}",
        coc_foreground
    );

    // At background distance (10.0 m, background out-of-focus):
    let coc_background = intrinsics.circle_of_confusion_m(10.0, focus_dist, f_number, focal_len);
    assert!(
        coc_background > 1e-5,
        "Background CoC should be significant: {}",
        coc_background
    );
}

#[test]
fn test_optical_camera_world_to_camera_transform() {
    let cam = OpticalCamera::new_standard_1080p(
        Vector3D::new(10.0, 5.0, 2.0),
        Vector3D::new(0.0, 0.0, 1.0),
        Vector3D::new(0.0, 1.0, 0.0),
    );

    let p_world = Vector3D::new(12.0, 4.0, 7.0);
    let p_cam = cam.world_to_camera(p_world);

    // Z_cam should be forward distance: 7.0 - 2.0 = 5.0
    assert!((p_cam.z - 5.0).abs() < 1e-4, "Camera Z offset: {}", p_cam.z);
}
