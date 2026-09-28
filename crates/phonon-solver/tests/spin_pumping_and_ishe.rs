//! Integration Tests for Spin Pumping & Inverse Spin Hall Effect (ISHE).

use approx::assert_relative_eq;
use phonon_models::cavity_spintronics::{MagnonCavityCoupling, YigMaterial, YigPtInterface};
use phonon_solver::cavity_spintronics::{CoupledLlgCavitySolver, CoupledLlgConfig};
use std::f64::consts::PI;

#[test]
fn test_yig_pt_interface_damping_and_resistance() {
    let yig = YigMaterial::default();
    let interface = YigPtInterface::default();

    let delta_alpha = interface.damping_enhancement(&yig);
    assert!(
        delta_alpha > 1.0e-3 && delta_alpha < 3.0e-3,
        "Delta alpha should be ~2.1e-3, got {}",
        delta_alpha
    );

    let total_alpha = interface.total_damping(&yig);
    assert_relative_eq!(
        total_alpha,
        yig.intrinsic_damping + delta_alpha,
        epsilon = 1e-9
    );

    let r_pt = interface.pt_resistance_ohms();
    // R = L / (sigma * w * d) = 2e-3 / (4e6 * 1e-3 * 10e-9) = 2e-3 / 4e-5 = 50 Ohms
    assert_relative_eq!(r_pt, 50.0, epsilon = 0.01);
}

#[test]
fn test_analytical_spin_pumping_and_ishe_voltage() {
    let interface = YigPtInterface::default();
    let freq_hz = 10.0e9;
    let omega = 2.0 * PI * freq_hz;
    let cone_angle_deg = 0.57; // ~0.01 rad
    let cone_angle_rad = cone_angle_deg * PI / 180.0;

    let js_dc = interface.spin_current_density_dc(omega, cone_angle_rad);
    assert!(js_dc > 0.0);

    let js_charge = interface.charge_current_density_equivalent(js_dc);
    assert!(
        js_charge > 1.0e5,
        "Charge current density should be > 1e5 A/m2, got {}",
        js_charge
    );

    let v_ishe = interface.ishe_voltage_volts(omega, cone_angle_rad);
    assert!(
        v_ishe > 1.0e-6,
        "ISHE voltage should be > 1.0 uV, got {} V",
        v_ishe
    );
    assert!(
        v_ishe < 50.0e-6,
        "ISHE voltage should be realistic (< 50 uV), got {} V",
        v_ishe
    );
}

#[test]
fn test_coupled_llg_cavity_time_domain_solver() {
    let coupling = MagnonCavityCoupling::default();
    let interface = YigPtInterface::default();
    let solver = CoupledLlgCavitySolver::new();

    let config = CoupledLlgConfig {
        duration_seconds: 20.0e-9, // 20 ns
        dt_seconds: 5.0e-12,       // 5 ps
        drive_freq_hz: 10.0e9,
        drive_amplitude_t: 12.0e-6, // 12 uT drive
    };

    let res = solver.solve(&coupling, &interface, &config);

    assert_eq!(res.time_points.len(), 4000);
    assert!(
        res.steady_state_cone_angle_deg > 0.05,
        "Cone angle: {}",
        res.steady_state_cone_angle_deg
    );
    assert!(
        res.dc_ishe_voltage_volts > 1.0e-6,
        "V_ISHE: {} V",
        res.dc_ishe_voltage_volts
    );

    // Verify intracavity photon accumulation
    let final_photons = *res.cavity_photons.last().unwrap();
    assert!(final_photons > 0.0);
}
