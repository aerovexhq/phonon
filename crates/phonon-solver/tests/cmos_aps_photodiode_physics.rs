//! Integration Test: CMOS APS Photodiode Physics, Quantum Efficiency & Noise Models

use phonon_models::em::ChannelRng;
use phonon_models::optics::{
    silicon_quantum_efficiency, transduce_cmos_pixel, CmosPixelConfig, ShutterType,
    SILICON_CUTOFF_WAVELENGTH_NM,
};

#[test]
fn test_silicon_quantum_efficiency_spectral_response() {
    // UV cutoff (< 320 nm):
    assert_eq!(silicon_quantum_efficiency(300.0), 0.0);
    assert_eq!(silicon_quantum_efficiency(319.0), 0.0);

    // Visible spectrum rising to peak around green (530 - 550 nm):
    let qe_blue = silicon_quantum_efficiency(450.0);
    let qe_green = silicon_quantum_efficiency(530.0);
    let qe_red = silicon_quantum_efficiency(650.0);

    assert!(
        qe_blue > 0.40,
        "Blue QE should be substantial (> 40%), got {}",
        qe_blue
    );
    assert!(
        qe_green > 0.70,
        "Green peak QE should be high (> 70%), got {}",
        qe_green
    );
    assert!(qe_red > 0.50, "Red QE should be > 50%, got {}", qe_red);
    assert!(qe_green >= qe_blue, "Green QE should exceed Blue QE");

    // Near-Infrared decline towards bandgap cutoff:
    let qe_nir = silicon_quantum_efficiency(900.0);
    assert!(
        qe_nir > 0.10,
        "900 nm NIR QE should be > 10%, got {}",
        qe_nir
    );
    assert!(qe_nir < qe_red, "NIR QE should be lower than red QE");

    // Bandgap cutoff (>= 1107 nm):
    assert_eq!(
        silicon_quantum_efficiency(SILICON_CUTOFF_WAVELENGTH_NM),
        0.0
    );
    assert_eq!(silicon_quantum_efficiency(1200.0), 0.0);
}

#[test]
fn test_cmos_pixel_dark_current_temperature_scaling() {
    let config = CmosPixelConfig::new_standard_industrial();
    let exposure_s = 0.050; // 50 ms

    // Room temperature (20 C = 293.15 K):
    let i_dark_room = config.dark_current_electrons(293.15, exposure_s);
    assert!(
        i_dark_room > 0.0,
        "Dark current at room temp should be positive: {}",
        i_dark_room
    );

    // Hot temperature (60 C = 333.15 K):
    let i_dark_hot = config.dark_current_electrons(333.15, exposure_s);
    assert!(
        i_dark_hot > i_dark_room * 5.0,
        "Dark current should increase dramatically with temperature: room={}, hot={}",
        i_dark_room,
        i_dark_hot
    );

    // Freezing temperature (-20 C = 253.15 K):
    let i_dark_cold = config.dark_current_electrons(253.15, exposure_s);
    assert!(
        i_dark_cold < i_dark_room * 0.2,
        "Dark current should be suppressed at cold temperatures: room={}, cold={}",
        i_dark_room,
        i_dark_cold
    );
}

#[test]
fn test_cmos_pixel_transduction_and_saturation() {
    let config = CmosPixelConfig::new_standard_industrial();
    let mut rng = ChannelRng::new(42);

    // 1. Zero incident photons (dark frame):
    let dark_out = transduce_cmos_pixel(&config, 0.0, 550.0, 0.016, 293.15, &mut rng);
    assert!(!dark_out.is_saturated);
    assert!(dark_out.photoelectrons < 1e-6);
    // Digital number should be near black level:
    assert!(
        dark_out.digital_number < 50,
        "Dark level DN too high: {}",
        dark_out.digital_number
    );

    // 2. Nominal indoor illumination:
    let mid_out = transduce_cmos_pixel(&config, 10_000.0, 550.0, 0.016, 293.15, &mut rng);
    assert!(!mid_out.is_saturated);
    assert!(mid_out.collected_electrons > 5000.0);
    assert!(mid_out.collected_electrons < config.full_well_capacity);
    assert!(
        mid_out.snr_db > 30.0,
        "SNR under nominal signal should be > 30 dB, got {}",
        mid_out.snr_db
    );

    // 3. Blinding direct sunlight (saturation):
    let sat_out = transduce_cmos_pixel(&config, 200_000.0, 550.0, 0.016, 293.15, &mut rng);
    assert!(
        sat_out.is_saturated,
        "Pixel should be saturated under high flux"
    );
    assert_eq!(sat_out.collected_electrons, config.full_well_capacity);
    assert_eq!(sat_out.digital_number, config.max_digital_number());
}

#[test]
fn test_cmos_dynamic_range_and_shutter_types() {
    let mut config = CmosPixelConfig::new_standard_industrial();
    let dr = config.dynamic_range_db();
    assert!(
        dr >= 70.0,
        "Dynamic range of 4T CMOS APS should be >= 70 dB, got {}",
        dr
    );

    config.shutter_type = ShutterType::RollingShutter;
    assert_eq!(config.shutter_type, ShutterType::RollingShutter);
}
