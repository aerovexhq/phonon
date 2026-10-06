#![deny(unsafe_code)]

//! Automated Verification Suite for Atmospheric Neutron Spallation Cascade & DO-254 DAL-A Co-Simulator.

use std::time::Instant;
use phonon_solver::atmospheric_neutron::{
    AtmosphericNeutronCoSimulator, AtmosphericNeutronModel, Do254DalLevel, FlightAltitude,
    LightningIndirectSimulator, LightningSeverityLevel, LightningWaveformKind,
    MitigationArchitecture, ProtectionClampDevice, SiliconDeviceParams, SiliconReactionChannel,
    SiliconSpallationEngine, SolarModulation,
};

#[test]
fn test_altitude_flux_scaling_and_geomagnetic_cutoff() {
    let sea_level = AtmosphericNeutronModel::new(FlightAltitude::SeaLevel, 45.0, SolarModulation::SolarModerate);
    assert!((sea_level.altitude_acceleration_factor() - 1.0).abs() < 1e-4);

    let fl300 = AtmosphericNeutronModel::new(FlightAltitude::FL300, 45.0, SolarModulation::SolarModerate);
    let fl350 = AtmosphericNeutronModel::new(FlightAltitude::FL350, 45.0, SolarModulation::SolarModerate);
    let fl390 = AtmosphericNeutronModel::new(FlightAltitude::FL390, 45.0, SolarModulation::SolarModerate);
    let fl430 = AtmosphericNeutronModel::new(FlightAltitude::FL430, 45.0, SolarModulation::SolarModerate);

    let a300 = fl300.altitude_acceleration_factor();
    let a350 = fl350.altitude_acceleration_factor();
    let a390 = fl390.altitude_acceleration_factor();
    let a430 = fl430.altitude_acceleration_factor();

    assert!(a300 > 80.0, "FL300 accel should be > 80x, got {}", a300);
    assert!(a350 > a300, "FL350 should exceed FL300");
    assert!(a390 > a350, "FL390 should exceed FL350");
    assert!(a430 > a390, "FL430 should exceed FL390");
    assert!(a430 > 500.0, "FL430 accel should exceed 500x, got {}", a430);

    // Geomagnetic cutoff rigidity
    let equator = AtmosphericNeutronModel::new(FlightAltitude::FL390, 0.0, SolarModulation::SolarModerate);
    let polar = AtmosphericNeutronModel::new(FlightAltitude::FL390, 75.0, SolarModulation::SolarModerate);

    assert!(equator.cutoff_rigidity_gv() > 14.0, "Equatorial rigidity ~14.9 GV");
    assert!(polar.cutoff_rigidity_gv() < 1.0, "Polar rigidity < 1.0 GV");
    assert!(
        polar.total_flux_acceleration_factor() > equator.total_flux_acceleration_factor(),
        "Polar cosmic flux must be higher than equatorial"
    );
}

#[test]
fn test_differential_neutron_spectrum_jesd89a() {
    let model = AtmosphericNeutronModel::new(FlightAltitude::FL390, 45.0, SolarModulation::SolarModerate);

    let flux_2mev = model.differential_flux(2.0);
    let flux_100mev = model.differential_flux(100.0);
    let flux_1000mev = model.differential_flux(1000.0);

    assert!(flux_2mev > 0.0);
    assert!(flux_100mev > 0.0);
    assert!(flux_1000mev > 0.0);
    assert!(flux_2mev > flux_1000mev, "Low energy flux should exceed relativistic tail");

    let flux_hr = model.integrated_flux_per_hour();
    assert!(flux_hr > 1000.0 && flux_hr < 50_000.0, "Integrated flux at FL390 in expected range: {}", flux_hr);

    let samples = model.sample_spectrum(40);
    assert_eq!(samples.len(), 40);
    for (e, f) in &samples {
        assert!(*e >= 1.0 && *e <= 10_000.0);
        assert!(*f > 0.0 && f.is_finite());
    }
}

#[test]
fn test_silicon_spallation_recoil_kinematics() {
    let alpha_chan = SiliconReactionChannel::AlphaRecoil;
    let proton_chan = SiliconReactionChannel::ProtonRecoil;
    let deep_chan = SiliconReactionChannel::DeepSpallation;

    // Below threshold
    assert_eq!(alpha_chan.cross_section_mb(1.0), 0.0);
    assert_eq!(proton_chan.cross_section_mb(2.0), 0.0);
    assert_eq!(deep_chan.cross_section_mb(10.0), 0.0);

    // Above threshold
    assert!(alpha_chan.cross_section_mb(14.0) > 100.0);
    assert!(proton_chan.cross_section_mb(17.0) > 150.0);
    assert!(deep_chan.cross_section_mb(100.0) > 300.0);

    // Recoil energy bounds
    let e_rec_alpha = alpha_chan.max_recoil_energy_mev(20.0);
    assert!(e_rec_alpha > 1.0 && e_rec_alpha < 10.0);
    let e_rec_deep = deep_chan.max_recoil_energy_mev(100.0);
    assert!(e_rec_deep > 5.0 && e_rec_deep <= 35.0);
}

#[test]
fn test_do254_dala_mitigation_and_ser_certification() {
    let env = AtmosphericNeutronModel::new(FlightAltitude::FL390, 50.0, SolarModulation::SolarMinimum);
    let device = SiliconDeviceParams::default();

    // Simplex unmitigated
    let simplex_engine = SiliconSpallationEngine::new(
        env.clone(),
        device.clone(),
        MitigationArchitecture::Simplex,
        Do254DalLevel::DalA,
    );

    let raw_fit = simplex_engine.raw_chip_ser_fit();
    assert!(raw_fit > 1000.0, "Raw chip SER at FL390 should exceed 1000 FIT: {}", raw_fit);
    let raw_rate = simplex_engine.raw_failure_rate_per_hour();
    assert!(raw_rate > 1.0e-6, "Simplex unmitigated rate exceeds DAL-A limit");
    assert!(!simplex_engine.is_dal_compliant(), "Simplex should fail DO-254 DAL-A");

    // TMR mitigated
    let tmr_engine = SiliconSpallationEngine::new(
        env.clone(),
        device.clone(),
        MitigationArchitecture::TripleModularRedundancy,
        Do254DalLevel::DalA,
    );

    let tmr_rate = tmr_engine.mitigated_failure_rate_per_hour();
    assert!(tmr_rate < 1.0e-9, "TMR must reduce failure rate below 1e-9 / hr: {}", tmr_rate);
    assert!(tmr_engine.is_dal_compliant(), "TMR must satisfy DO-254 DAL-A certification");
    assert!(tmr_engine.dal_safety_margin_db() > 10.0, "Safety margin should exceed 10 dB");

    // Lockstep with scrubbing
    let lockstep_engine = SiliconSpallationEngine::new(
        env,
        device,
        MitigationArchitecture::LockstepWithScrubbing { scrub_interval_s: 0.05 },
        Do254DalLevel::DalA,
    );
    assert!(lockstep_engine.is_dal_compliant(), "Lockstep with 50ms scrubbing must satisfy DAL-A");
}

#[test]
fn test_do160g_lightning_pin_injection_and_thermal_clamping() {
    let sim = LightningIndirectSimulator::new(
        LightningWaveformKind::Waveform4,
        LightningSeverityLevel::Level5,
        ProtectionClampDevice::TvsDiode { v_br: 12.0, r_dyn: 0.035 },
    );

    let v_peak = sim.peak_clamping_voltage_v();
    let i_peak = sim.peak_surge_current_a();

    // 1600V open-circuit clamped to low voltage rail
    assert!(v_peak < 80.0, "TVS must clamp 1600V to safe rail, got {}", v_peak);
    assert!(i_peak > 1000.0, "Level 5 surge current should exceed 1000 A: {}", i_peak);

    let samples = sim.simulate_transient(300.0, 100);
    assert_eq!(samples.len(), 100);

    let peak_temp = sim.peak_junction_temperature_deg_c();
    assert!(peak_temp > 70.0, "Junction temp must rise above ambient");
    assert!(peak_temp < 350.0, "Peak temp must remain below silicon failure threshold: {}", peak_temp);
    assert!(sim.thermal_failure_margin_deg_c() > 0.0, "Thermal headroom must be positive");
}

#[test]
fn test_fast_initialization_cold_boot_latency() {
    let start = Instant::now();
    let mut sim = AtmosphericNeutronCoSimulator::new_fast();
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 5, "Cold boot must complete in < 5 ms, took {:?}", elapsed);

    let report = sim.latest_report();
    assert!(report.is_dal_a_compliant);
    assert!(report.altitude_acceleration > 100.0);

    // Recompute
    sim.neutron_model.altitude = FlightAltitude::FL430;
    let updated = sim.recompute();
    assert!(updated.altitude_acceleration > 450.0);
}
