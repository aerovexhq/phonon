#![deny(unsafe_code)]

//! Test suite for Topological Acoustic Higher-Order Skyrmion-Lattice Beam Deflector & Chiral Router.

use phonon_solver::skyrmion_deflector::{
    AcousticPseudoSpin, DeflectorParams, SkyrmionDeflectorEngine, SkyrmionProfileKind,
    SkyrmionTexture, SkyrmionTextureParams,
};

#[test]
fn test_skyrmion_texture_and_charge_quantization() {
    // 1. Neel Skyrmion (vorticity = 1)
    let params_neel = SkyrmionTextureParams {
        domain_size_mm: 100.0,
        radius_mm: 20.0,
        wall_width_mm: 8.0,
        helicity_rad: 0.0,
        core_polarity: -1,
        vorticity: 1,
        kind: SkyrmionProfileKind::Neel,
    };
    let tex_neel = SkyrmionTexture::new(params_neel, 48);
    let q_neel = tex_neel.total_topological_charge();
    assert!((q_neel - 1.0).abs() < 0.15, "Neel skyrmion charge must be close to 1.0, got: {}", q_neel);

    // 2. Higher-Order Skyrmion (vorticity = 2)
    let params_ho = SkyrmionTextureParams::higher_order(20.0);
    let tex_ho = SkyrmionTexture::new(params_ho, 48);
    let q_ho = tex_ho.total_topological_charge();
    assert!((q_ho - 2.0).abs() < 0.25, "Higher-order skyrmion charge must be close to 2.0, got: {}", q_ho);

    // 3. Antiskyrmion (vorticity = -1)
    let params_anti = SkyrmionTextureParams::antiskyrmion(20.0);
    let tex_anti = SkyrmionTexture::new(params_anti, 48);
    let q_anti = tex_anti.total_topological_charge();
    assert!((q_anti - (-1.0)).abs() < 0.15, "Antiskyrmion charge must be close to -1.0, got: {}", q_anti);

    // 4. Trivial Ferromagnet (vorticity = 0)
    let params_triv = SkyrmionTextureParams::trivial();
    let tex_triv = SkyrmionTexture::new(params_triv, 32);
    let q_triv = tex_triv.total_topological_charge();
    assert_eq!(q_triv, 0.0, "Trivial texture must have exact zero charge");
}

#[test]
fn test_effective_synthetic_gauge_field() {
    let params = SkyrmionTextureParams::default();
    let tex = SkyrmionTexture::new(params, 40);

    // The effective field B_eff at the center / core region should be non-zero
    let mid = tex.grid_n / 2;
    let b_center = tex.effective_gauge_field_at(mid, mid);
    // Near outer boundary it should be nearly zero
    let b_boundary = tex.effective_gauge_field_at(1, 1);

    assert!(b_center.abs() > 0.001, "Center synthetic magnetic field should be non-zero, got: {}", b_center);
    assert!(b_boundary.abs() < b_center.abs() * 0.1, "Boundary field should be much smaller than core");
}

#[test]
fn test_anomalous_acoustic_hall_beam_deflection() {
    let deflector_params = DeflectorParams::default();
    let engine = SkyrmionDeflectorEngine::new(deflector_params);

    // Spin-Up deflection
    let beam_up = engine.compute_beam_deflection(AcousticPseudoSpin::SpinUp);
    assert!(beam_up.deflection_angle_deg > 5.0, "SpinUp should deflect upward, got: {}", beam_up.deflection_angle_deg);
    assert!(beam_up.transmission_efficiency >= 0.90);
    assert_eq!(beam_up.trajectory_points.len(), 40);

    // Spin-Down deflection
    let beam_down = engine.compute_beam_deflection(AcousticPseudoSpin::SpinDown);
    assert!(beam_down.deflection_angle_deg < -5.0, "SpinDown should deflect downward, got: {}", beam_down.deflection_angle_deg);
    assert!((beam_up.deflection_angle_deg + beam_down.deflection_angle_deg).abs() < 1e-6, "Deflection must be symmetric");

    // Unpolarized / zero pseudo-spin
    let beam_unpol = engine.compute_beam_deflection(AcousticPseudoSpin::Unpolarized);
    assert_eq!(beam_unpol.deflection_angle_deg, 0.0, "Unpolarized beam must remain undeflected");
}

#[test]
fn test_higher_order_skyrmion_deflection_scaling() {
    // Standard skyrmion (N_sk = 1)
    let params_std = DeflectorParams::default();
    let engine_std = SkyrmionDeflectorEngine::new(params_std);
    let beam_std = engine_std.compute_beam_deflection(AcousticPseudoSpin::SpinUp);

    // Higher-order skyrmion (N_sk = 2)
    let mut params_ho = DeflectorParams::default();
    params_ho.texture_params = SkyrmionTextureParams::higher_order(18.0);
    let engine_ho = SkyrmionDeflectorEngine::new(params_ho);
    let beam_ho = engine_ho.compute_beam_deflection(AcousticPseudoSpin::SpinUp);

    assert!(
        beam_ho.deflection_angle_deg > beam_std.deflection_angle_deg * 1.5,
        "Higher-order skyrmion should roughly double deflection angle: std={}, ho={}",
        beam_std.deflection_angle_deg,
        beam_ho.deflection_angle_deg
    );
}

#[test]
fn test_skyrmion_router_s_parameters_and_isolation() {
    let engine = SkyrmionDeflectorEngine::new(DeflectorParams::default());
    let m = &engine.metrics;

    // Insertion loss S21 >= -1.0 dB
    assert!(m.s21_spin_up_db >= -1.0, "S21 insertion loss too high: {}", m.s21_spin_up_db);
    // Port 3 cross-talk isolation <= -28.0 dB
    assert!(m.s31_spin_up_isolation_db <= -28.0, "Isolation must be at least 28 dB, got: {}", m.s31_spin_up_isolation_db);
    // Return loss <= -20.0 dB
    assert!(m.s11_reflection_db <= -20.0, "Return loss should be <= -20 dB, got: {}", m.s11_reflection_db);
    // Purity >= 98.0%
    assert!(m.polarization_purity_percent >= 98.0, "Polarization purity must be >= 98%, got: {}", m.polarization_purity_percent);
}

#[test]
fn test_defect_immunity() {
    let mut params_defect = DeflectorParams::default();
    params_defect.defect_active = true;
    params_defect.defect_attenuation_factor = 0.94;

    let engine = SkyrmionDeflectorEngine::new(params_defect);
    let beam = engine.compute_beam_deflection(AcousticPseudoSpin::SpinUp);

    assert!(beam.transmission_efficiency >= 0.85, "Transmission with defect should remain high");
    assert_eq!(engine.metrics.defect_immunity_retention, 0.94);
}
