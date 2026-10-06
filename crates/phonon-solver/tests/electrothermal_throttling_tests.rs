#![deny(unsafe_code)]

use phonon_solver::electrothermal_throttling::*;

#[test]
fn test_self_consistent_thermal_equilibrium() {
    let leak_params = LeakageModelParams::default();
    let dyn_params = DynamicPowerParams::default();
    let v_dd = 1.10;
    let frequency_hz = 4.0e9; // 4.0 GHz
    let clock_duty = 1.0;
    let ambient_temp_c = 25.0;
    let total_r_th = 0.12; // 0.12 K/W for efficient liquid cooling

    let eq = solve_thermal_equilibrium(
        &leak_params,
        &dyn_params,
        v_dd,
        frequency_hz,
        clock_duty,
        ambient_temp_c,
        total_r_th,
    );

    assert!(eq.converged, "Thermal equilibrium solver must converge");
    assert!(
        eq.junction_temp_c > ambient_temp_c,
        "Junction temperature ({:.2} C) must exceed ambient ({:.2} C)",
        eq.junction_temp_c,
        ambient_temp_c
    );
    assert!(
        eq.junction_temp_c < 85.0,
        "Junction temperature ({:.2} C) should be within nominal operating limits under 0.12 K/W",
        eq.junction_temp_c
    );
    assert!(
        eq.dynamic_power_w > 0.0,
        "Dynamic power must be strictly positive"
    );
    assert!(
        eq.leakage_power_w > 0.0,
        "Leakage power must be strictly positive"
    );
    assert_eq!(
        eq.status,
        ThermalStabilityStatus::Stable,
        "System should be thermally stable with S = {:.3}",
        eq.stability_factor
    );
    assert!(
        eq.stability_factor > 0.40,
        "Stability factor ({:.3}) should have healthy margin above 0.40",
        eq.stability_factor
    );
}

#[test]
fn test_thermal_runaway_bifurcation_detection() {
    let leak_params = LeakageModelParams::default();
    let dyn_params = DynamicPowerParams::default();
    let v_dd = 1.25; // High boost voltage
    let frequency_hz = 4.8e9; // 4.8 GHz
    let clock_duty = 1.0;
    let ambient_temp_c = 55.0; // High ambient
    let degraded_r_th = 0.85; // Severely degraded thermal resistance (fan failure or detached sink)

    let eq = solve_thermal_equilibrium(
        &leak_params,
        &dyn_params,
        v_dd,
        frequency_hz,
        clock_duty,
        ambient_temp_c,
        degraded_r_th,
    );

    // With severe R_th = 0.85 K/W and 1.25V at 4.8GHz, system should trigger thermal runaway or marginal instability
    assert!(
        eq.status == ThermalStabilityStatus::Runaway || eq.status == ThermalStabilityStatus::Marginal,
        "Severe cooling degradation must trigger runaway or marginal status, got {:?}",
        eq.status
    );

    // Test bifurcation curve generation
    let curve = generate_bifurcation_curve(
        &leak_params,
        &dyn_params,
        v_dd,
        frequency_hz,
        clock_duty,
        ambient_temp_c,
        degraded_r_th,
        50,
    );

    assert_eq!(curve.temperatures_c.len(), 50);
    assert_eq!(curve.heat_generation_w.len(), 50);
    assert_eq!(curve.heat_dissipation_w.len(), 50);
    assert!(
        curve.bifurcation_temp_c.is_some(),
        "Bifurcation temperature where d(P_leak)/dT >= 1/R_th must be identified"
    );
}

#[test]
fn test_closed_loop_dvfs_thermal_throttling() {
    let mut config = DvfsControllerConfig::default();
    config.target_temp_c = 60.0; // Target 60 C triggers throttling during heavy compute burst
    let leak_params = LeakageModelParams::default();
    let dyn_params = DynamicPowerParams::default();
    let workload = WorkloadProfile {
        baseline_activity: 0.20,
        burst_activity: 0.95,
        burst_start_ms: 5.0,
        burst_end_ms: 35.0,
    };
    let ambient_temp_c = 25.0;
    let total_r_th = 0.18; // 0.18 K/W liquid cooling resistance
    let thermal_mass = 0.015; // 15 mJ/K die thermal mass

    let result = run_closed_loop_transient(
        &config,
        &leak_params,
        &dyn_params,
        &workload,
        ambient_temp_c,
        total_r_th,
        thermal_mass,
        50.0, // 50 ms duration
        0.1,  // 0.1 ms step
    );

    assert!(
        result.thermal_runaway_prevented,
        "Closed-loop DVFS must prevent thermal runaway"
    );
    assert!(
        result.peak_temperature_c < config.shutdown_temp_c,
        "Peak temperature ({:.2} C) must stay below shutdown threshold ({:.2} C)",
        result.peak_temperature_c,
        config.shutdown_temp_c
    );

    // Check that during the burst, throttling reduced frequency from peak boost
    let min_freq_during_run = result
        .frequency_history_ghz
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);

    assert!(
        min_freq_during_run < config.f_max_ghz,
        "DVFS should have throttled frequency below max boost ({:.2} GHz), min observed was {:.2} GHz",
        config.f_max_ghz,
        min_freq_during_run
    );
}

#[test]
fn test_liquid_coldplate_microchannel_performance() {
    let mut params = MicrochannelParams::default();
    params.flow_rate_lpm = 0.5;

    let perf_low = calculate_microchannel_performance(&params);
    assert!(perf_low.channel_count > 50, "Should have dozens of microchannels");
    assert!(perf_low.reynolds_number > 0.0);
    assert!(perf_low.heat_transfer_coeff_w_m2_k > 2000.0);

    // Increase flow rate to 2.0 L/min
    params.flow_rate_lpm = 2.0;
    let perf_high = calculate_microchannel_performance(&params);

    assert!(
        perf_high.total_coldplate_resistance_k_w < perf_low.total_coldplate_resistance_k_w,
        "Higher flow rate must yield lower cold-plate thermal resistance: {:.4} vs {:.4} K/W",
        perf_high.total_coldplate_resistance_k_w,
        perf_low.total_coldplate_resistance_k_w
    );
    assert!(
        perf_high.reynolds_number > perf_low.reynolds_number,
        "Reynolds number must scale with flow rate"
    );
    assert!(
        perf_high.pressure_drop_kpa > perf_low.pressure_drop_kpa,
        "Pressure drop must increase at higher flow rates"
    );
}

#[test]
fn test_two_phase_immersion_and_chf_margin() {
    let params = ImmersionCoolingParams::default();
    let props = params.fluid.properties();

    let chf_w_cm2 = calculate_zuber_chf(&props);
    assert!(
        chf_w_cm2 >= 15.0 && chf_w_cm2 <= 40.0,
        "Zuber CHF for dielectric fluid should be in 15-40 W/cm^2 range, got {:.2} W/cm^2",
        chf_w_cm2
    );

    // Test nucleate boiling at 10 K superheat (T_surf = T_sat + 10 K = 66 C)
    let t_surf = props.boiling_point_c + 10.0;
    let perf = calculate_immersion_performance(&params, t_surf);

    assert!(
        perf.is_nucleate_boiling_active,
        "Nucleate boiling must be active at 10 K superheat"
    );
    assert!(
        perf.current_heat_flux_w_cm2 > 1.0,
        "Heat flux ({:.2} W/cm^2) should be substantial in nucleate boiling",
        perf.current_heat_flux_w_cm2
    );
    assert!(
        perf.chf_margin_percentage > 0.0 && perf.chf_margin_percentage < 100.0,
        "CHF margin ({:.1}%) should be bounded",
        perf.chf_margin_percentage
    );
    assert!(
        perf.boiling_thermal_resistance_k_w < 2.0,
        "Boiling thermal resistance ({:.4} K/W) should be low in nucleate regime",
        perf.boiling_thermal_resistance_k_w
    );

    // Test boiling curve generation
    let curve = generate_boiling_curve(&params, 30);
    assert_eq!(curve.len(), 30);
    assert!(curve.last().unwrap().heat_flux_w_cm2 > curve.first().unwrap().heat_flux_w_cm2);
}

#[test]
fn test_tim_pump_out_aging_and_cold_boot() {
    let mut tim = TimAgingModel::default();
    let r0 = tim.total_resistance_k_w();

    // After 5,000 thermal cycles
    tim.power_cycles = 5000;
    let r5000 = tim.total_resistance_k_w();

    assert!(
        r5000 > r0,
        "TIM thermal resistance after 5000 cycles ({:.4} K/W) must exceed baseline ({:.4} K/W)",
        r5000,
        r0
    );

    let traj = tim.generate_aging_trajectory(10000, 20);
    assert_eq!(traj.len(), 20);
    assert!(traj.last().unwrap().degradation_percentage > 0.0);

    // Test fast cold-boot simulator constructor
    let sim = ElectrothermalCoSimulator::new_fast();
    let telemetry = sim.compute_telemetry();

    assert!(telemetry.junction_temperature_c > 0.0);
    assert!(telemetry.total_power_w > 0.0);
    assert!(telemetry.total_thermal_resistance_k_w > 0.0);
    assert!(
        telemetry.stability_status == ThermalStabilityStatus::Stable
            || telemetry.stability_status == ThermalStabilityStatus::Marginal
    );
}
