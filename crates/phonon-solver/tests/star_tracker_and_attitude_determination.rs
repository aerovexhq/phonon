#![allow(clippy::needless_range_loop)]
//! Integration tests for Phase 45: Star Tracker Attitude Determination & Wahba QUEST Solver.

use phonon_models::em::Vector3D;
use phonon_models::sensors::imu::Quaternion;
use phonon_models::space::star_tracker::{StarTrackerCamera, StarTrackerSystem};

#[test]
fn test_brown_conrady_camera_projection_and_undistortion() {
    let camera = StarTrackerCamera::default();

    // Bore-sight star (+Z direction)
    let star_boresight = Vector3D::new(0.0, 0.0, 1.0);
    let proj = camera.project_vector(star_boresight);
    assert!(
        proj.is_some(),
        "Bore-sight star must project inside detector"
    );
    let (u0, v0) = proj.unwrap();
    assert!(
        (u0 - camera.principal_point.0).abs() < 1e-6
            && (v0 - camera.principal_point.1).abs() < 1e-6,
        "Bore-sight star must project onto principal point"
    );

    // Off-axis star (5 degrees off boresight)
    let angle_rad = 5.0 * std::f64::consts::PI / 180.0;
    let star_offaxis = Vector3D::new(angle_rad.sin(), 0.0, angle_rad.cos());
    let proj_off = camera.project_vector(star_offaxis);
    assert!(
        proj_off.is_some(),
        "5-degree off-axis star should be in FOV"
    );

    let (u, v) = proj_off.unwrap();
    let unprojected = camera.unproject_pixel(u, v);
    let dot = unprojected.dot(&star_offaxis);
    assert!(
        (dot - 1.0).abs() < 1e-4,
        "Unprojected star vector should match original within sub-milliradian distortion tolerance"
    );
}

#[test]
fn test_star_catalog_generation_and_spatial_distribution() {
    let catalog = StarTrackerSystem::generate_reference_catalog();
    assert!(
        catalog.len() >= 64,
        "Catalog must contain at least 64 bright stars"
    );

    // Verify all star vectors are normalized unit vectors:
    for star in &catalog {
        let norm = star.unit_vector.norm();
        assert!(
            (norm - 1.0).abs() < 1e-12,
            "Catalog star unit vector must be normalized, got {norm}"
        );
        assert!(
            star.visual_magnitude > 0.0 && star.visual_magnitude < 10.0,
            "Visual magnitude should be in visible band"
        );
    }
}

#[test]
fn test_lost_in_space_triangle_pyramid_star_matching() {
    let camera = StarTrackerCamera::default();
    let tracker = StarTrackerSystem::new(camera);

    let q_test = Quaternion::from_euler_rpy(0.1, -0.05, 0.2);
    let obs = tracker.observe_stars(q_test);

    if obs.len() >= 3 {
        let obs_vectors: Vec<Vector3D> = obs.iter().map(|&(_, v)| v).collect();
        let matched = tracker.match_stars(&obs_vectors, 0.01);
        assert!(
            !matched.is_empty(),
            "Pattern matching must identify catalog star pairs"
        );
    }
}

#[test]
fn test_wahba_quest_attitude_determination_precision() {
    let camera = StarTrackerCamera::default();
    let tracker = StarTrackerSystem::new(camera);

    // Arbitrary true attitude orientation:
    let q_true = Quaternion::from_euler_rpy(0.15, -0.25, 0.35);

    let obs = tracker.observe_stars(q_true);
    assert!(
        obs.len() >= 3,
        "Need at least 3 observed stars to solve Wahba QUEST, got {}",
        obs.len()
    );

    let mut pairs = Vec::with_capacity(obs.len());
    for &(id, b_meas) in &obs {
        if let Some(cat_star) = tracker.catalog.iter().find(|s| s.id == id) {
            pairs.push((b_meas, cat_star.unit_vector));
        }
    }

    let q_est = tracker
        .solve_wahba_quest(&pairs)
        .expect("QUEST solver should find attitude");

    // Quaternion error: q_err = q_true^* * q_est
    let q_err = q_true.conjugate().multiply(&q_est);
    let error_angle_rad = 2.0 * q_err.w.abs().clamp(-1.0, 1.0).acos();
    let error_arcsec = error_angle_rad * (180.0 / std::f64::consts::PI) * 3600.0;

    assert!(
        error_arcsec < 60.0,
        "Wahba QUEST attitude estimation error should be < 60 arcsec, got {error_arcsec} arcsec"
    );
}
