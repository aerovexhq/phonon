//! Integration Tests for 3D Vector Electromagnetic Waves & Dielectric Materials.

use approx::assert_relative_eq;
use phonon_models::em::{
    DielectricWall, EmWaveSource, KnifeEdgeObstacle, Polarization, RfDielectricMaterial, Vector3D,
    INTRINSIC_IMPEDANCE_VACUUM,
};

#[test]
fn test_vector_em_wave_free_space_propagation() {
    let freq = 2.4e9; // 2.4 GHz
    let tx_power = 0.1; // 100 mW (20 dBm)
    let tx_gain = 2.15; // 3.3 dBi dipole

    let tx = EmWaveSource::new(
        freq,
        tx_power,
        tx_gain,
        Polarization::LinearVertical,
        Vector3D::ZERO,
        Vector3D::new(1.0, 0.0, 0.0),
    );

    // Wavelength lambda = c / f = 299792458 / 2.4e9 ~ 0.12491 m
    let lambda = tx.wavelength();
    assert_relative_eq!(lambda, 0.1249135, epsilon = 1e-4);

    // EIRP = 0.1 * 2.15 = 0.215 W ~ 23.32 dBm
    assert_relative_eq!(tx.eirp_watts(), 0.215, epsilon = 1e-6);
    assert_relative_eq!(tx.eirp_dbm(), 23.32438, epsilon = 1e-3);

    // Poynting flux at 1 m and 10 m
    let s_1m = tx.poynting_flux_at_distance(1.0);
    let s_10m = tx.poynting_flux_at_distance(10.0);

    // S(10m) should be exactly 100x smaller than S(1m) (inverse-square law)
    assert_relative_eq!(s_1m / s_10m, 100.0, epsilon = 1e-6);

    // Electric field amplitude at 1 m: E_rms = sqrt(eta_0 * S) = sqrt(376.73 * 0.215 / (4 * pi)) ~ 2.54 V/m
    let e_field = tx.electric_field_at(&Vector3D::new(1.0, 0.0, 0.0));
    let e_mag = e_field.magnitude();
    let expected_e = (INTRINSIC_IMPEDANCE_VACUUM * s_1m).sqrt();
    assert_relative_eq!(e_mag, expected_e, epsilon = 1e-3);

    // Free space path loss at 100 m: FSPL = 20*log10(4*pi*100 / 0.12491) ~ 80.05 dB
    let fspl_100m = tx.free_space_path_loss_db(100.0);
    assert_relative_eq!(fspl_100m, 80.05, epsilon = 0.1);
}

#[test]
fn test_polarization_mismatch_loss() {
    let horiz = Polarization::LinearHorizontal;
    let vert = Polarization::LinearVertical;
    let slant_45 = Polarization::LinearSlant(std::f64::consts::FRAC_PI_4);
    let rhcp = Polarization::CircularRight;
    let lhcp = Polarization::CircularLeft;

    // Matched linear
    assert_eq!(horiz.mismatch_factor(&horiz), 1.0);
    assert_eq!(vert.mismatch_factor(&vert), 1.0);

    // Orthogonal linear (0.0 factor)
    assert_eq!(horiz.mismatch_factor(&vert), 0.0);
    assert_eq!(vert.mismatch_factor(&horiz), 0.0);

    // 45 degree slant: cos^2(45) = 0.5 (-3.01 dB)
    assert_relative_eq!(horiz.mismatch_factor(&slant_45), 0.5, epsilon = 1e-6);
    assert_relative_eq!(vert.mismatch_factor(&slant_45), 0.5, epsilon = 1e-6);

    // Circular matched vs cross
    assert_eq!(rhcp.mismatch_factor(&rhcp), 1.0);
    assert_eq!(rhcp.mismatch_factor(&lhcp), 0.0);

    // Circular to linear: 0.5 (-3 dB)
    assert_eq!(rhcp.mismatch_factor(&vert), 0.5);
    assert_eq!(lhcp.mismatch_factor(&horiz), 0.5);
}

#[test]
fn test_complex_dielectrics_and_fresnel() {
    let concrete = RfDielectricMaterial::concrete();
    let copper = RfDielectricMaterial::metal_copper();

    let freq = 2.45e9;

    // Concrete propagation constant
    let (alpha_c, beta_c) = concrete.propagation_constant(freq);
    assert!(alpha_c > 0.0);
    assert!(beta_c > 0.0);

    // Copper skin depth at 2.45 GHz: delta = sqrt(2 / (omega * mu * sigma)) ~ 1.33 um
    let skin_copper = copper.skin_depth(freq);
    assert_relative_eq!(skin_copper * 1e6, 1.336, epsilon = 0.05);

    // Normal incidence reflection on concrete (eps_r = 4.5): R = (1 - sqrt(4.5)) / (1 + sqrt(4.5)) ~ -0.3585
    let air = RfDielectricMaterial::air();
    let fresnel = phonon_models::em::FresnelCoefficients::calculate(&air, &concrete, 0.0, freq);

    // Power reflection ~ 0.3585^2 ~ 0.1285 (12.85%)
    assert_relative_eq!(fresnel.power_reflection_te, 0.1285, epsilon = 0.02);
    assert_relative_eq!(
        fresnel.power_transmission_te,
        1.0 - fresnel.power_reflection_te,
        epsilon = 1e-6
    );
}

#[test]
fn test_dielectric_wall_ray_intersection_and_attenuation() {
    let wall = DielectricWall::new(
        Vector3D::new(5.0, 0.0, 0.0), // Wall centered at x=5
        Vector3D::new(1.0, 0.0, 0.0), // Normal along X axis
        0.20,                         // 20 cm thick
        10.0,
        5.0,
        RfDielectricMaterial::concrete(),
    );

    let ray_origin = Vector3D::new(0.0, 0.0, 0.0);
    let ray_target = Vector3D::new(10.0, 0.0, 0.0);

    // Ray passing through center of wall
    let hit = wall.intersects_segment(&ray_origin, &ray_target);
    assert!(hit.is_some());
    let h = hit.unwrap();
    assert_relative_eq!(h.distance_m, 5.0, epsilon = 1e-6);
    assert_relative_eq!(h.point.x, 5.0, epsilon = 1e-6);
    assert_relative_eq!(h.incident_angle_rad, 0.0, epsilon = 1e-6);

    // Attenuation through 20 cm concrete at 2.45 GHz should be >= 5 dB
    let att_db = wall.attenuation_db(h.incident_angle_rad, 2.45e9);
    assert!(
        att_db >= 5.0,
        "Concrete wall attenuation should be >= 5 dB, got {att_db}"
    );

    // Ray missing the wall
    let ray_miss = Vector3D::new(10.0, 20.0, 0.0);
    assert!(wall.intersects_segment(&ray_origin, &ray_miss).is_none());
}

#[test]
fn test_knife_edge_diffraction() {
    let _lambda = 0.125; // ~2.4 GHz

    // Clear line of sight (nu <= -0.7) -> 0 dB loss
    let loss_clear = KnifeEdgeObstacle::diffraction_loss_db(-1.0);
    assert_eq!(loss_clear, 0.0);

    // Grazing incidence at edge (nu = 0.0) -> approx 6 dB loss
    let loss_grazing = KnifeEdgeObstacle::diffraction_loss_db(0.0);
    assert_relative_eq!(loss_grazing, 6.0, epsilon = 0.5);

    // Deep obstruction (nu = 2.0) -> approx 18 dB loss
    let loss_deep = KnifeEdgeObstacle::diffraction_loss_db(2.0);
    assert!(loss_deep > 15.0);
}
