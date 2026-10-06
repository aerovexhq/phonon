#![deny(unsafe_code)]

use phonon_solver::pdn_droop::{
    calculate_pdn_impedance_profile, simulate_dynamic_droop, simulate_mitigated_droop,
    ClockStretchParams, DldoControllerParams, LoadStepProfile,
    PdnDroopCoSimulator, PdnNetworkParams,
};

#[test]
fn test_target_impedance_calculation() {
    let mut params = PdnNetworkParams::default();
    params.vrm.v_dd_v = 0.85;
    params.max_voltage_ripple_ratio = 0.05; // 5% = 42.5 mV
    params.load_step_current_a = 800.0;

    let z_target_ohm = params.target_impedance_ohm();
    let expected_z = (0.85 * 0.05) / 800.0;
    assert!((z_target_ohm - expected_z).abs() < 1e-12);

    let z_target_mohm = z_target_ohm * 1e3;
    assert!((z_target_mohm - 0.053125).abs() < 1e-6);
}

#[test]
fn test_decoupling_hierarchy_capacitance_totals() {
    let params = PdnNetworkParams::default();

    let die_cap = params.total_on_die_capacitance_f();
    assert!(die_cap > 1.0e-6, "On-die cap should be > 1 uF with DTC: actual = {die_cap}");

    let pkg_cap = params.total_package_capacitance_f();
    assert!(pkg_cap > 5.0e-5, "Package cap should be > 50 uF: actual = {pkg_cap}");

    let bulk_cap = params.total_bulk_capacitance_f();
    assert!(bulk_cap > 1.0e-3, "Bulk cap should be > 1 mF: actual = {bulk_cap}");
}

#[test]
fn test_multi_decade_impedance_sweep_and_anti_resonances() {
    let params = PdnNetworkParams::default();
    let profile = calculate_pdn_impedance_profile(&params, 10);

    assert!(!profile.points.is_empty());
    assert!(profile.points.first().unwrap().freq_hz <= 1001.0);
    assert!(profile.points.last().unwrap().freq_hz >= 9.9e8);

    // Verify anti-resonance peaks are extracted
    assert!(!profile.anti_resonances.is_empty(), "Should detect anti-resonance peaks");
    for peak in &profile.anti_resonances {
        assert!(peak.freq_hz >= 1e3 && peak.freq_hz <= 1e9);
        assert!(peak.peak_impedance_mohm > 0.0);
    }
}

#[test]
fn test_dynamic_droop_stages_under_high_di_dt() {
    let params = PdnNetworkParams::default();
    let mut load = LoadStepProfile::default();
    load.delta_i_a = 800.0;
    load.t_rise_s = 1.0e-9; // 800 A/ns -> 8e11 A/s
    load.t_step_start_s = 1.0e-8; // 10 ns

    assert!(load.di_dt_a_s() >= 7.9e11);

    let res = simulate_dynamic_droop(&params, &load, 150);

    assert_eq!(res.time_ns.len(), res.v_die_v.len());
    assert!(res.peak_overall_droop_mv > 10.0, "Droop should be substantial under 800A step");

    // 1st droop occurs early due to on-die / package loop
    assert!(res.first_droop.timestamp_s >= load.t_step_start_s);
    assert!(res.first_droop.peak_droop_mv > 0.0);

    // 2nd droop occurs in package / board loop
    assert!(res.second_droop.timestamp_s > res.first_droop.timestamp_s);

    // 3rd droop occurs in VRM response timeframe
    assert!(res.third_droop.timestamp_s >= res.second_droop.timestamp_s);
}

#[test]
fn test_active_dldo_and_clock_stretching_mitigation() {
    let params = PdnNetworkParams::default();
    let load = LoadStepProfile::default();
    let dldo = DldoControllerParams::default();
    let stretch = ClockStretchParams::default();

    let res = simulate_mitigated_droop(&params, &load, &dldo, &stretch, 150);

    assert!(res.report.unmitigated_peak_droop_mv > res.report.mitigated_peak_droop_mv);
    assert!(
        res.report.droop_reduction_pct >= 35.0,
        "DLDO mitigation should reduce peak droop by at least 35%, got {:.2}%",
        res.report.droop_reduction_pct
    );
    assert!(res.report.dldo_peak_current_a > 100.0);
}

#[test]
fn test_co_simulator_instant_cold_boot_and_telemetry() {
    let start = std::time::Instant::now();
    let sim = PdnDroopCoSimulator::new_fast();
    let elapsed = start.elapsed();

    // Verify sub-millisecond cold boot
    assert!(elapsed.as_micros() < 500, "new_fast() took too long: {:?}", elapsed);

    let ir_drop = sim.dc_ir_drop_mv();
    assert!(ir_drop > 0.0 && ir_drop < 500.0);

    let report = sim.generate_telemetry_report();
    assert!(report.target_impedance_mohm > 0.0);
    assert!(report.droop_reduction_pct >= 35.0);
}
