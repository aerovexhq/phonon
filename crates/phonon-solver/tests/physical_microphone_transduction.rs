//! Integration Tests for Physical Microphone Transducer Models

use phonon_models::acoustic::{
    CondenserMicrophone, MicrophonePolarPattern, PiezoelectricMicrophone,
};
use phonon_models::em::Vector3D;
use std::f64::consts::PI;

#[test]
fn test_condenser_microphone_displacement_capacitance_and_voltage() {
    let pos = Vector3D::new(0.0, 0.0, 1.5);
    let axis = Vector3D::new(1.0, 0.0, 0.0); // Facing +X
    let mic = CondenserMicrophone::new_studio_capsule(pos, axis, MicrophonePolarPattern::Cardioid);

    // Mechanical parameters:
    let f_res = mic.resonant_frequency_hz();
    // f0 = sqrt(12000 / 1e-6) / (2*pi) = sqrt(1.2e10) / (2*pi) ~ 109544.5 / 6.28318 ~ 17434 Hz
    assert!(
        (f_res - 17434.0).abs() < 10.0,
        "Expected ~17434 Hz resonance, got {:.1}",
        f_res
    );

    let c0 = mic.rest_capacitance_f();
    // C0 = eps0 * A / d0 = 8.854e-12 * 1.2e-4 / 20e-6 ~ 5.31e-11 F ~ 53.1 pF
    assert!(
        (c0 - 5.31e-11).abs() < 1.0e-12,
        "Expected ~53.1 pF, got {:.2e}",
        c0
    );

    // Transduce 1 Pa acoustic wave at 1 kHz (94 dB SPL, reference 1 Pa acoustic pressure):
    let inc_dir = Vector3D::new(1.0, 0.0, 0.0); // Exactly on-axis (0 deg)
    let sig = mic.transduce(inc_dir, 1.0, 1000.0);

    assert_eq!(sig.polar_directivity_factor, 1.0);
    assert!(sig.diaphragm_displacement_m > 0.0);
    assert!(sig.voltage_v > 0.0);
    // Typical studio mic sensitivity is around 10-35 mV/Pa:
    let v_mv = sig.voltage_v * 1000.0;
    assert!(
        v_mv > 5.0 && v_mv < 50.0,
        "Expected realistic mV output, got {:.3} mV",
        v_mv
    );
}

#[test]
fn test_microphone_polar_directivity_patterns() {
    let pos = Vector3D::new(0.0, 0.0, 1.5);
    let axis = Vector3D::new(1.0, 0.0, 0.0); // Facing +X

    let omni =
        CondenserMicrophone::new_studio_capsule(pos, axis, MicrophonePolarPattern::Omnidirectional);
    let cardioid =
        CondenserMicrophone::new_studio_capsule(pos, axis, MicrophonePolarPattern::Cardioid);
    let fig8 = CondenserMicrophone::new_studio_capsule(pos, axis, MicrophonePolarPattern::Figure8);

    // On-axis (+X, 0 deg)
    assert_eq!(omni.polar_pattern.directivity(0.0), 1.0);
    assert_eq!(cardioid.polar_pattern.directivity(0.0), 1.0);
    assert_eq!(fig8.polar_pattern.directivity(0.0), 1.0);

    // Side-axis (+Y, 90 deg)
    assert_eq!(omni.polar_pattern.directivity(PI / 2.0), 1.0);
    assert!((cardioid.polar_pattern.directivity(PI / 2.0) - 0.5).abs() < 1e-4);
    assert!(fig8.polar_pattern.directivity(PI / 2.0) < 1e-4); // Zero pickup at 90 deg

    // Rear-axis (-X, 180 deg)
    assert_eq!(omni.polar_pattern.directivity(PI), 1.0);
    assert!(cardioid.polar_pattern.directivity(PI) < 1e-4); // Complete null at 180 deg
    assert!((fig8.polar_pattern.directivity(PI) - 1.0).abs() < 1e-4); // Full pickup at 180 deg
}

#[test]
fn test_piezoelectric_microphone_stress_voltage_transduction() {
    let pos = Vector3D::new(0.0, 0.0, 0.0);
    let axis = Vector3D::new(0.0, 0.0, 1.0);
    let piezo = PiezoelectricMicrophone::new_pzt5a(pos, axis);

    assert_eq!(piezo.crystal_thickness_m, 1.0e-3);
    assert_eq!(piezo.g33_coefficient, 0.025);
    assert!(piezo.internal_capacitance_f > 1.0e-9); // ~1.5 nF

    // Transduce 100 Pa (high acoustic pressure wave):
    // V = g33 * P * th = 0.025 * 100 * 0.001 = 0.0025 V = 2.5 mV
    let v_out = piezo.transduce(Vector3D::new(0.0, 0.0, 1.0), 100.0);
    assert!(
        (v_out - 0.0025).abs() < 1e-5,
        "Expected ~2.5 mV, got {:.6} V",
        v_out
    );
}

#[test]
fn test_condenser_microphone_frequency_response_sweep() {
    let mic = CondenserMicrophone::new_studio_capsule(
        Vector3D::ZERO,
        Vector3D::new(1.0, 0.0, 0.0),
        MicrophonePolarPattern::Omnidirectional,
    );

    // Below resonance (100 Hz vs 1 kHz): flat displacement
    let x_100 = mic.diaphragm_displacement_amplitude(1.0, 100.0);
    let x_1000 = mic.diaphragm_displacement_amplitude(1.0, 1000.0);
    assert!(
        (x_100 - x_1000).abs() / x_100 < 0.05,
        "Low-frequency displacement response should be flat"
    );

    // At resonance (~17.4 kHz): resonant peak
    let x_res = mic.diaphragm_displacement_amplitude(1.0, mic.resonant_frequency_hz());
    assert!(
        x_res > x_100,
        "Resonant response should exceed quasi-static response"
    );

    // Well above resonance (50 kHz): steep roll-off (1/f^2 mass-controlled)
    let x_50k = mic.diaphragm_displacement_amplitude(1.0, 50_000.0);
    assert!(
        x_50k < x_100 * 0.2,
        "High-frequency response should roll off steeply"
    );
}
