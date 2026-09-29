//! Integration tests for Axion Dark Matter Haloscope Quantum Readout.

use phonon_models::kitwpa::{AxionModel, HaloscopeCavity, KitwpaTransmissionLine};
use phonon_solver::kitwpa::HaloscopeReadoutSolver;

#[test]
fn test_axion_conversion_and_quantum_noise() {
    let cavity = HaloscopeCavity::default();
    let axion = AxionModel::default();

    // Check converted axion frequency ~ 8.0 GHz
    let f_a = axion.axion_frequency_hz();
    assert!(
        (f_a / 1e9 - 8.0).abs() < 0.1,
        "Converted frequency should be ~8.0 GHz, got {:.3} GHz",
        f_a / 1e9
    );

    // Check Sikivie converted power ~ 2.5e-23 W (-196 dBm)
    let p_a = axion.converted_signal_power_watts(&cavity);
    assert!(
        (p_a - 2.5e-23).abs() < 1e-25,
        "Converted power should be ~2.5e-23 W, got {:e}",
        p_a
    );

    // Caves added noise quanta at high gain (G >= 100 / 20 dB): N_add -> 0.5
    let n_add = axion.caves_added_noise_quanta(100.0);
    assert!(
        (0.495..=0.505).contains(&n_add),
        "N_add must approach 0.5, got {:.4}",
        n_add
    );

    // Added noise temperature at 8 GHz: T_add = h f / k_B * 0.5 ~ 0.191 K (191 mK)
    let t_add = axion.quantum_noise_temperature_k(f_a, 100.0);
    assert!(
        (0.18..0.20).contains(&t_add),
        "T_add should be ~190 mK, got {:.3} K",
        t_add
    );
}

#[test]
fn test_haloscope_readout_solver_scan_speedup() {
    let cavity = HaloscopeCavity::default();
    let axion = AxionModel::default();
    let line = KitwpaTransmissionLine::default();
    let solver = HaloscopeReadoutSolver::new();

    let readout = solver.analyze_readout(&cavity, &axion, &line, 2.5, 5.0);

    // Gain must exceed 20 dB
    assert!(readout.kitwpa_power_gain_db >= 20.0);

    // Added noise quanta <= 0.505
    assert!(readout.added_noise_quanta <= 0.505);

    // System temperature with KITWPA ~ 50 mK + 191 mK = 241 mK
    assert!(readout.kitwpa_system_temp_k < 0.30);

    // System temperature with HEMT ~ 50 mK + 2.5 K = 2.55 K
    assert!(readout.hemt_system_temp_k > 2.50);

    // Scan rate speedup = (T_hemt / T_kitwpa)^2 > 100x
    assert!(
        readout.scan_rate_speedup > 100.0,
        "Scan speedup must exceed 100x, got {:.1}x",
        readout.scan_rate_speedup
    );

    // Converted power in dBm around -196 dBm
    assert!((-210.0..-180.0).contains(&readout.converted_power_dbm));
}
