//! Integration tests for SNSPD electro-thermal dynamics, hotspot nucleation, and inductive reset.

use phonon_models::snspd::{HotspotDynamicsModel, NanowireGeometry};
use phonon_solver::snspd::{ElectroThermalConfig, ElectroThermalSolver};

#[test]
fn test_nanowire_superconducting_properties_and_critical_current() {
    let geom = NanowireGeometry::standard_nbn();

    assert_eq!(geom.thickness_nm, 5.0);
    assert_eq!(geom.width_nm, 80.0);
    assert_eq!(geom.length_um, 100.0);
    assert_eq!(geom.critical_temperature_k, 11.5);
    assert_eq!(geom.substrate_temperature_k, 2.0);

    // Number of squares: L / w = 100,000 nm / 80 nm = 1250 squares
    let n_sq = geom.number_of_squares();
    assert!((n_sq - 1250.0).abs() < 1e-6);

    // Normal resistance: 400 Ohms/sq * 1250 = 500,000 Ohms (500 kOhms)
    let r_n = geom.normal_resistance_ohms();
    assert!((r_n - 500_000.0).abs() < 1e-6);

    // Total kinetic inductance: 80 pH/sq * 1250 = 100 nH = 1.0e-7 H
    let l_k = geom.kinetic_inductance_henries();
    assert!((l_k - 100.0e-9).abs() < 1e-12);

    // Inductive reset time: tau_rec = L_k / R_L = 100 nH / 50 Ohms = 2.0 ns
    let tau_rec_s = geom.inductive_reset_time_s();
    assert!((tau_rec_s - 2.0e-9).abs() < 1e-12);

    // Ginzburg-Landau temperature dependence: I_c(T) = I_c0 * [1 - (T/T_c)^2]^(3/2)
    let i_c_0 = geom.critical_current_zero_t_ua; // 25.0 uA
    let i_c_sub = geom.operational_critical_current_ua(); // at 2.0 K
    assert!(i_c_sub > 0.90 * i_c_0 && i_c_sub < i_c_0);

    let i_c_tc = geom.critical_current_at_temp_ua(geom.critical_temperature_k);
    assert_eq!(i_c_tc, 0.0);

    // Superconducting gap: Delta(0) approx 1.764 * k_B * T_c
    let gap_mev = geom.superconducting_gap_zero_t_mev();
    assert!(
        gap_mev > 1.5 && gap_mev < 2.0,
        "Delta(0) should be approx 1.75 meV, got {}",
        gap_mev
    );
}

#[test]
fn test_electrothermal_pulse_generation_and_recovery() {
    let geom = NanowireGeometry::standard_nbn();
    let hotspot = HotspotDynamicsModel::standard_nbn();

    let config = ElectroThermalConfig {
        time_step_ps: 2.0,
        total_time_ps: 6000.0,
        photon_absorption_time_ps: 200.0,
        wavelength_nm: 1550.0,
        bias_ratio: 0.92,
    };

    let solver = ElectroThermalSolver::new(geom, hotspot, config);
    let trace = solver.solve();

    assert!(
        trace.triggered,
        "Photon at 1550 nm should trigger resistive hotspot"
    );
    assert!(
        trace.peak_voltage_mv > 0.5,
        "Peak output voltage should exceed 0.5 mV, got {} mV",
        trace.peak_voltage_mv
    );
    assert!(
        trace.peak_voltage_mv < 1.5,
        "Peak output voltage should be near I_b * R_L approx 1.1 mV, got {} mV",
        trace.peak_voltage_mv
    );

    // Verify sub-50 ps rise time
    assert!(
        trace.rise_time_ps > 0.0 && trace.rise_time_ps < 50.0,
        "Rise time should be sub-50 ps, got {} ps",
        trace.rise_time_ps
    );

    // Verify current diversion: current drops significantly below bias current
    let min_curr = trace
        .current_ua
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let i_b = geom.bias_current_ua(config.bias_ratio);
    assert!(
        min_curr < 0.5 * i_b,
        "Current should divert significantly, min = {} uA, I_b = {} uA",
        min_curr,
        i_b
    );

    // Verify that at the end of simulation, current has largely recovered towards I_b
    let end_curr = *trace.current_ua.last().unwrap();
    assert!(
        end_curr > 0.85 * i_b,
        "Current should recover towards bias, end = {} uA, I_b = {} uA",
        end_curr,
        i_b
    );
}

#[test]
fn test_retrapping_current_and_hysteresis() {
    let geom = NanowireGeometry::standard_nbn();
    let hotspot = HotspotDynamicsModel::standard_nbn();

    let i_c = geom.operational_critical_current_ua();
    let i_r = hotspot.retrapping_current_ua(&geom);

    // Physical hysteresis criterion: I_r < I_c (typically I_r / I_c approx 0.15 - 0.40)
    let ratio = i_r / i_c;
    assert!(
        ratio > 0.15 && ratio < 0.40,
        "Retrapping ratio I_r / I_c should be in [0.15, 0.40], got {}",
        ratio
    );

    // Check domain growth velocity: positive for I > I_r, negative for I < I_r
    let v_grow = hotspot.domain_growth_velocity_m_s(i_c * 0.8, i_r);
    let v_shrink = hotspot.domain_growth_velocity_m_s(i_r * 0.5, i_r);

    assert!(
        v_grow > 0.0,
        "Domain should expand when current exceeds I_r"
    );
    assert!(
        v_shrink < 0.0,
        "Domain should collapse when current is below I_r"
    );
}
