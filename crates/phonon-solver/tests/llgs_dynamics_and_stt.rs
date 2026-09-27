//! Integration Test: LLGS Precessional Dynamics, Damping & Spin Torque (STT/SOT).
//!
//! Validates:
//! 1. Norm conservation \(|\vec{m}(t)| \equiv 1.000\) across thousands of RK4 integration steps.
//! 2. Damping towards the net effective field (anisotropy + external bias).
//! 3. Spin-Transfer Torque (STT) driven 180-degree magnetization reversal.

use phonon_models::spintronics::{LlgsConfig, LlgsSolver, MagneticMaterial, Nanomagnet, Vec3};

#[test]
fn test_llgs_long_trajectory_norm_conservation() {
    let mut mat = MagneticMaterial::cofeb();
    mat.alpha = 0.03; // Standard low Gilbert damping

    let mut magnet = Nanomagnet::new_rectangular(
        1,
        Vec3::ZERO,
        60.0e-9,
        30.0e-9,
        3.0e-9,
        mat,
        Vec3::X,
        Vec3::new(0.5, 0.866, 0.0), // 60 degree initial tilt
    );

    let solver = LlgsSolver::new(LlgsConfig {
        dt_s: 1.0e-13, // 100 fs
        ..Default::default()
    });

    let h_bias = Vec3::new(1.5e5, 0.0, 0.0); // 150 kA/m external bias along +X

    // Run 2,000 steps (200 ps)
    for _ in 0..2000 {
        solver.step_rk4(&mut magnet, h_bias, Vec3::ZERO);
        let norm = magnet.m.norm();
        assert!(
            (norm - 1.0).abs() < 1.0e-9,
            "Magnetization norm must remain exactly 1.0 at every step, got {:.12}",
            norm
        );
    }

    // After 200 ps of precession and damping, magnet should be settled along +X
    assert!(
        magnet.m.x > 0.85,
        "Magnet must relax into +X alignment, got m.x = {:.4}",
        magnet.m.x
    );
}

#[test]
fn test_stt_controlled_spin_reversal() {
    let mut mat = MagneticMaterial::cofeb();
    mat.alpha = 0.05;

    let mut magnet = Nanomagnet::new_rectangular(
        1,
        Vec3::ZERO,
        50.0e-9,
        25.0e-9,
        2.5e-9,
        mat,
        Vec3::X,
        Vec3::new(-0.99, 0.14, 0.0), // Initial state along -X (Logic 0)
    );

    // Apply STT spin current polarized along +X to switch to Logic 1
    let solver = LlgsSolver::new(LlgsConfig {
        dt_s: 1.0e-13,
        stt_current_density_a_per_m2: 3.5e12, // Exceeds critical switching threshold J_c
        spin_polarization_p: 0.80,
        polarizing_layer_m: Vec3::X,
        ..Default::default()
    });

    // Run 3000 steps (300 ps)
    for _ in 0..3000 {
        solver.step_rk4(&mut magnet, Vec3::ZERO, Vec3::ZERO);
    }

    assert!(
        magnet.m.x > 0.70,
        "STT torque must reverse magnetization to +X (Logic 1), got m.x = {:.4}",
        magnet.m.x
    );
}
