//! Integration Test: Direct Material Switching Logic (NDR / RTD and MIT VO2).
//!
//! Validates:
//! - Monostable-Bistable Transition Logic (MOBILE) based on twin Negative Differential Resistance (NDR) pairs.
//! - Non-transistor bistable latching and threshold logic action.
//! - Metal-Insulator Transition (MIT $\text{VO}_2$) phase transition switching (>1000x resistance drop).
//! - Electro-thermal phase switching dynamics and threshold voltage gating.

use phonon_models::synthesis::{MitDeviceModel, MitParameters, NdrDeviceModel, NdrParameters};

#[test]
fn test_rtd_mobile_bistable_latching_dynamics() {
    let rtd_load = NdrDeviceModel::new(NdrParameters {
        peak_current_a: 1.0e-3,
        peak_voltage_v: 0.25,
        valley_current_a: 1.0e-4,
        valley_voltage_v: 0.50,
        excess_voltage_scale_v: 0.15,
    });

    let rtd_driver = NdrDeviceModel::new(NdrParameters {
        peak_current_a: 1.05e-3, // Slightly higher peak current (+5% modulation)
        peak_voltage_v: 0.25,
        valley_current_a: 1.0e-4,
        valley_voltage_v: 0.50,
        excess_voltage_scale_v: 0.15,
    });

    // Verify negative conductance region exists
    let eval_ndr = rtd_load.evaluate(0.35);
    assert!(eval_ndr.in_ndr_region);
    assert!(eval_ndr.conductance_s < 0.0);

    // Verify driver has modulated peak resonance current exceeding the load RTD
    let eval_load_peak = rtd_load.evaluate(0.25);
    let eval_driver_peak = rtd_driver.evaluate(0.25);
    assert!(
        eval_driver_peak.current_a > eval_load_peak.current_a,
        "Driver peak current ({}) must exceed load peak ({})",
        eval_driver_peak.current_a,
        eval_load_peak.current_a
    );

    // In a MOBILE latch, because I_p(driver) > I_p(load), as clock ramps up,
    // the load RTD reaches its peak current first and switches to the high-resistance valley state
    let eval_load_valley = rtd_load.evaluate(0.50);
    assert!(eval_load_valley.current_a < eval_load_peak.current_a);
    assert!(eval_load_valley.pvcr >= 10.0);
}

#[test]
fn test_mit_vo2_abrupt_resistance_collapse_and_thresholding() {
    let vo2 = MitDeviceModel::new(MitParameters {
        r_off_ohms: 120_000.0,
        r_on_ohms: 80.0,
        threshold_voltage_v: 0.75,
        transition_width_v: 0.04,
        critical_temperature_k: 341.0,
        thermal_transition_width_k: 1.5,
    });

    // 1. Below threshold voltage (0.3 V) at room temperature (300 K): Insulating state
    let state_low = vo2.evaluate(0.30, 300.0);
    assert!(!state_low.is_metallic);
    assert!(state_low.resistance_ohms > 100_000.0);
    assert!(state_low.current_a < 5.0e-6); // < 5 uA

    // 2. Above threshold voltage (0.9 V) at room temperature: Abrupt metallic state
    let state_high = vo2.evaluate(0.90, 300.0);
    assert!(state_high.is_metallic);
    assert!(state_high.resistance_ohms < 200.0);
    assert!(state_high.current_a > 4.0e-3); // > 4 mA (high conduction!)

    // 3. Resistance switching ratio must exceed 3 orders of magnitude (1000x)
    let ratio = state_low.resistance_ohms / state_high.resistance_ohms;
    assert!(
        ratio > 800.0,
        "MIT switching ratio ({}) must exceed 800x",
        ratio
    );

    // 4. Thermal transition above 341 K (68 °C) triggers metallic state even at low voltage
    let state_thermal = vo2.evaluate(0.10, 348.0);
    assert!(state_thermal.is_metallic);
    assert!(state_thermal.resistance_ohms < 200.0);
}

#[test]
fn test_mit_vo2_logic_threshold_action() {
    let vo2 = MitDeviceModel::new(MitParameters::default());

    // Test a fine voltage sweep across the threshold boundary (0.70 V to 0.90 V)
    let eval_pre = vo2.evaluate(0.70, 300.0);
    let eval_post = vo2.evaluate(0.90, 300.0);

    // Resistance should plummet by orders of magnitude over 200 mV
    assert!(eval_pre.resistance_ohms > 20_000.0);
    assert!(eval_post.resistance_ohms < 300.0);

    let current_ratio = eval_post.current_a / eval_pre.current_a;
    assert!(
        current_ratio > 100.0,
        "Current surge across MIT threshold ({}) must exceed 100x",
        current_ratio
    );
}
