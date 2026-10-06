#![deny(unsafe_code)]

use phonon_solver::skyrmion_reservoir::{
    EffectiveField, LlgsParams, MagneticSkyrmionTexture, PinningSite, SkyrmionGridParams,
    SpinTorqueOscillator, SpintronicReservoir, SpintronicReservoirParams, Vector3,
};

#[test]
fn test_llgs_norm_preservation() {
    let params = LlgsParams::default();
    let initial_m = Vector3::new(0.6, 0.8, 0.0).normalize();
    let m_p = Vector3::new(0.0, 0.0, 1.0);
    let field = EffectiveField::new(Vector3::new(0.0, 0.0, 0.2)); // 0.2 Tesla external field

    let mut stto = SpinTorqueOscillator::new(params, initial_m, m_p, field);
    stto.current_density = 5.0e11; // 5e11 A/m^2 drive

    let dt = 1.0e-13; // 100 fs time step
    let num_steps = 200;

    for _ in 0..num_steps {
        stto.step_rk4(dt);
        let norm = stto.m.norm();
        assert!(
            (norm - 1.0).abs() < 1e-6,
            "Magnetization unit norm violated: norm = {}",
            norm
        );
    }
}

#[test]
fn test_stto_microwave_precession() {
    let params = LlgsParams {
        alpha: 0.01,
        ..Default::default()
    };
    let initial_m = Vector3::new(0.8, 0.0, 0.6).normalize();
    let m_p = Vector3::new(0.0, 0.0, 1.0);
    let field = EffectiveField::new(Vector3::new(0.0, 0.0, 0.15));

    let mut stto = SpinTorqueOscillator::new(params, initial_m, m_p, field);
    stto.current_density = 8.0e11;

    let dt = 5.0e-13;
    let freq_ghz = stto.estimate_precession_frequency(dt, 200);

    assert!(
        freq_ghz > 0.5 && freq_ghz < 50.0,
        "STTO precession frequency out of microwave band: {} GHz",
        freq_ghz
    );
}

#[test]
fn test_topological_charge_quantization() {
    let params = SkyrmionGridParams {
        nx: 32,
        ny: 32,
        cell_size_m: 1.5e-9,
        ..Default::default()
    };

    // 1. Uniform Ferromagnet
    let fm_texture = MagneticSkyrmionTexture::new_ferromagnetic(params.clone());
    let q_fm = fm_texture.compute_topological_charge();
    assert!(
        q_fm.abs() < 1e-5,
        "Ferromagnetic background should have Q = 0, got {}",
        q_fm
    );

    // 2. Chiral Néel Skyrmion
    let mut skyrmion_texture = MagneticSkyrmionTexture::new_ferromagnetic(params);
    skyrmion_texture.initialize_neel_skyrmion(15.5, 15.5, 7.0, 2.5, 0.0);
    let q_sk = skyrmion_texture.compute_topological_charge();

    // Winding number should be close to -1.0 (within discretization tolerance on 32x32)
    assert!(
        (q_sk.abs() - 1.0).abs() < 0.12,
        "Topological charge Q not quantized to +-1: Q = {}",
        q_sk
    );

    let diam_nm = skyrmion_texture.compute_effective_diameter_nm();
    assert!(
        diam_nm > 5.0 && diam_nm < 40.0,
        "Calculated skyrmion diameter out of physical bounds: {} nm",
        diam_nm
    );
}

#[test]
fn test_thiele_skyrmion_hall_angle() {
    let params = SkyrmionGridParams {
        nx: 32,
        ny: 32,
        ..Default::default()
    };
    let mut texture = MagneticSkyrmionTexture::new_ferromagnetic(params);
    texture.initialize_neel_skyrmion(15.5, 15.5, 6.5, 2.0, 0.0);

    let (g_z, d_xx, hall_angle_deg) = texture.compute_thiele_parameters();

    assert!(g_z.abs() > 1e-25, "Gyrovector G_z should be non-zero");
    assert!(d_xx > 0.0, "Dissipation D_xx must be strictly positive");
    assert!(
        hall_angle_deg.abs() > 1.0 && hall_angle_deg.abs() <= 90.0,
        "Skyrmion Hall angle out of physical range: {} deg",
        hall_angle_deg
    );

    // Test drift velocity with non-zero driving current
    let (vx, vy) = texture.compute_drift_velocity(1.0e11);
    assert!(
        vx.abs() > 0.0 || vy.abs() > 0.0,
        "Skyrmion drift velocity should be non-zero under driving current"
    );
}

#[test]
fn test_artificial_pinning_potential() {
    let site = PinningSite::new(20.0e-9, 20.0e-9, 4.0e-9, 1.6e-19); // 1.0 eV well depth

    // At the center of the potential well, restoring force should vanish
    let (fx_center, fy_center) = site.force_at(20.0e-9, 20.0e-9);
    assert!(
        fx_center.abs() < 1e-25 && fy_center.abs() < 1e-25,
        "Force at well center should be zero"
    );

    // At distance dx = +2.0 nm, restoring force should be negative (attracting toward well center)
    let (fx_displaced, _) = site.force_at(22.0e-9, 20.0e-9);
    assert!(
        fx_displaced < 0.0,
        "Restoring force should point toward well center: fx = {}",
        fx_displaced
    );

    // Energy at center should equal -U_0 = -1.6e-19 J
    let energy_center = site.energy_at(20.0e-9, 20.0e-9);
    assert!(
        (energy_center - (-1.6e-19)).abs() < 1e-25,
        "Energy well depth mismatch: {}",
        energy_center
    );
}

#[test]
fn test_spintronic_reservoir_narma10() {
    let reservoir_params = SpintronicReservoirParams {
        num_virtual_nodes: 35,
        feedback_coupling: 0.60,
        leaking_rate: 0.50,
        input_scale: 1.10,
        bias: 0.10,
        ridge_lambda: 1e-5,
        washout_steps: 25,
    };
    let mut reservoir = SpintronicReservoir::new(reservoir_params);

    let (inputs, targets) = SpintronicReservoir::generate_narma10_dataset(600);

    let train_len = 450;
    let train_inputs = &inputs[0..train_len];
    let train_targets = &targets[0..train_len];

    let train_result = reservoir.train_readout(train_inputs, train_targets);
    assert!(train_result.is_ok(), "Reservoir training should succeed");

    let train_nmse = train_result.unwrap();
    assert!(
        train_nmse < 0.35,
        "NARMA-10 training NMSE too high: {}",
        train_nmse
    );

    // Evaluate on test split
    let all_preds = reservoir.predict(&inputs);
    let test_nmse = SpintronicReservoir::calculate_nmse(
        &targets[train_len..],
        &all_preds[train_len..],
    );
    assert!(
        test_nmse < 0.38,
        "NARMA-10 test NMSE too high: {}",
        test_nmse
    );
}

#[test]
fn test_spintronic_reservoir_sine_to_square() {
    let reservoir_params = SpintronicReservoirParams {
        num_virtual_nodes: 25,
        feedback_coupling: 0.70,
        leaking_rate: 0.50,
        input_scale: 1.50,
        bias: 0.05,
        ridge_lambda: 1e-6,
        washout_steps: 20,
    };
    let mut reservoir = SpintronicReservoir::new(reservoir_params);

    let (inputs, targets) = SpintronicReservoir::generate_sine_to_square_dataset(200);

    let train_res = reservoir.train_readout(&inputs[0..140], &targets[0..140]);
    assert!(train_res.is_ok());

    let all_preds = reservoir.predict(&inputs);
    let r2 = SpintronicReservoir::calculate_r_squared(&targets[140..], &all_preds[140..]);

    assert!(
        r2 > 0.85,
        "Sine-to-square conversion R^2 too low: {}",
        r2
    );
}

#[test]
fn test_reservoir_memory_capacity() {
    let reservoir_params = SpintronicReservoirParams {
        num_virtual_nodes: 30,
        feedback_coupling: 0.75,
        leaking_rate: 0.50,
        input_scale: 1.0,
        bias: 0.1,
        ridge_lambda: 1e-5,
        washout_steps: 20,
    };
    let mut reservoir = SpintronicReservoir::new(reservoir_params);

    let capacity = reservoir.evaluate_memory_capacity(120, 8);
    assert!(
        capacity > 1.2,
        "Short-term memory capacity too low: {}",
        capacity
    );
}
