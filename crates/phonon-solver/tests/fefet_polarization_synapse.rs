//! Integration tests for Ferroelectric FET (FeFET) HZO synaptic transistor:
//! Landau-Khalatnikov polarization switching dynamics, threshold voltage shift,
//! and analog multi-level synaptic conductance modulation.

use phonon_models::memristor::fefet::FerroelectricFetModel;

#[test]
fn test_fefet_polarization_switching_hysteresis() {
    let fefet = FerroelectricFetModel::hzo_10nm();
    let mut p = -fefet.remnant_polarization_c_m2; // Start negatively polarized

    // Step positive pulse across gate (exceeding coercive field E_c ~ 1.2 MV/cm)
    // t_fe = 10 nm -> V_c ~ 1.2 V
    let dt = 1e-9; // 1 ns
    let v_gate_set = 2.5; // Strong positive writing field

    for _ in 0..100 {
        p = fefet.step_rk4(p, v_gate_set, dt);
    }

    assert!(
        p > 0.8 * fefet.remnant_polarization_c_m2,
        "Positive gate pulse must switch polarization to +Pr: got P = {} C/m^2",
        p
    );

    // Negative RESET pulse
    let v_gate_reset = -2.5;
    for _ in 0..100 {
        p = fefet.step_rk4(p, v_gate_reset, dt);
    }
    assert!(
        p < -0.8 * fefet.remnant_polarization_c_m2,
        "Negative gate pulse must switch polarization back to -Pr: got P = {} C/m^2",
        p
    );
}

#[test]
fn test_fefet_threshold_voltage_shift() {
    let fefet = FerroelectricFetModel::hzo_10nm();
    let pr = fefet.remnant_polarization_c_m2;

    let vth_pos = fefet.threshold_voltage(pr);
    let vth_neg = fefet.threshold_voltage(-pr);

    // Positive remnant polarization lowers the threshold voltage (increasing conductance)
    assert!(
        vth_pos < fefet.base_vth_volts,
        "Positive polarization must decrease threshold voltage: got {} V vs nominal {} V",
        vth_pos,
        fefet.base_vth_volts
    );

    // Negative remnant polarization raises the threshold voltage
    assert!(
        vth_neg > fefet.base_vth_volts,
        "Negative polarization must increase threshold voltage: got {} V vs nominal {} V",
        vth_neg,
        fefet.base_vth_volts
    );

    let delta_vth_total = vth_neg - vth_pos;
    assert!(
        delta_vth_total > 0.5,
        "Memory window (delta Vth) must be > 0.5 V, got {} V",
        delta_vth_total
    );
}

#[test]
fn test_fefet_synaptic_multi_level_conductance() {
    let fefet = FerroelectricFetModel::hzo_10nm();
    let n_levels = 8;
    let mut conductances = Vec::with_capacity(n_levels);

    // Read gate voltage at V_read = 0.8 V
    let v_read = 0.8;

    for i in 0..n_levels {
        let frac = (i as f64) / ((n_levels - 1) as f64);
        // Polarization fraction from -Pr to +Pr
        let p = -fefet.remnant_polarization_c_m2 + 2.0 * frac * fefet.remnant_polarization_c_m2;
        let g = fefet.read_conductance(p, v_read);
        conductances.push(g);
    }

    // Conductance must be strictly monotonically non-decreasing with polarization
    for i in 1..n_levels {
        assert!(
            conductances[i] >= conductances[i - 1],
            "Multi-level conductance must increase monotonically with polarization: level {} = {} S, level {} = {} S",
            i - 1,
            conductances[i - 1],
            i,
            conductances[i]
        );
    }

    // Dynamic range check: G_max / G_min
    let dynamic_range = conductances[n_levels - 1] / conductances[0].max(1e-15);
    assert!(
        dynamic_range > 10.0,
        "FeFET synaptic dynamic range must exceed 10x, got {}",
        dynamic_range
    );
}
