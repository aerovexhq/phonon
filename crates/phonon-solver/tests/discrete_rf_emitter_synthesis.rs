//! Integration Tests for First-Principles RF Emitter Synthesis & Transceiver Transduction

use approx::assert_relative_eq;
use phonon_core::constants::SPEED_OF_LIGHT;
use phonon_models::em::{
    DiscreteTransmitter, OscillatorType, PhysicalAntenna, RfPowerAmplifier, Vector3D,
};
use phonon_solver::em::RfTransceiverSolver;
use std::f64::consts::PI;

#[test]
fn test_lc_tank_oscillator_synthesis() {
    let l = 10.0e-9; // 10 nH
    let c = 1.0e-12; // 1 pF
    let r_s = 0.5; // 0.5 Ohm ESR
    let osc = OscillatorType::LCTank {
        inductance_h: l,
        capacitance_f: c,
        series_resistance_ohms: r_s,
        peak_voltage_v: 1.5,
    };

    // f0 = 1 / (2 * pi * sqrt(LC)) ~ 1.5915 GHz
    let expected_f0 = 1.0 / (2.0 * PI * (l * c).sqrt());
    assert_relative_eq!(osc.oscillation_frequency_hz(), expected_f0, epsilon = 1e3);

    // Q = omega0 * L / R_s ~ 200
    let expected_q = (2.0 * PI * expected_f0 * l) / r_s;
    assert_relative_eq!(osc.quality_factor(), expected_q, epsilon = 0.5);

    // Power output should be positive
    assert!(osc.output_power_watts() > 0.0);
}

#[test]
fn test_colpitts_oscillator_barkhausen_and_harmonics() {
    let osc = OscillatorType::Colpitts {
        inductance_h: 2.108e-9,
        c1_f: 4.0e-12,
        c2_f: 4.0e-12,
        transconductance_s: 0.05, // 50 mS
        load_resistance_ohms: 50.0,
        bias_current_a: 0.02,
    };

    // Oscillation frequency should be near 2.45 GHz
    let f0 = osc.oscillation_frequency_hz();
    assert!(f0 > 2.4e9 && f0 < 2.5e9, "Colpitts f0 was {f0} Hz");

    // Barkhausen criterion: loop gain A_loop = g_m * R_L * (C1/C2) = 0.05 * 50 * 1 = 2.5 >= 1.0
    assert!(osc.verifies_barkhausen_criterion());

    // Phase noise at 100 kHz offset should be very low (< -90 dBc/Hz)
    let pn_100k = osc.phase_noise_dbc_per_hz(100.0e3);
    assert!(pn_100k < -80.0, "Phase noise was {pn_100k} dBc/Hz");
}

#[test]
fn test_quartz_crystal_bvd_resonator() {
    let f_target = 433.92e6;
    let omega = 2.0 * PI * f_target;
    let lm = 15.0e-3; // 15 mH
    let cm = 1.0 / (omega * omega * lm);
    let osc = OscillatorType::Crystal {
        motional_inductance_h: lm,
        motional_capacitance_f: cm,
        motional_resistance_ohms: 20.0,
        shunt_capacitance_f: 4.0e-12,
        drive_level_watts: 0.001,
    };

    // Series resonance should match 433.92 MHz
    let fs = osc.oscillation_frequency_hz();
    assert_relative_eq!(fs, f_target, epsilon = 100.0);

    // Anti-resonance parallel frequency should be slightly higher than series
    let fp = osc
        .anti_resonance_frequency_hz()
        .expect("Crystal should have fp");
    assert!(
        fp > fs,
        "Parallel anti-resonance {fp} must exceed series {fs}"
    );

    // Extremely high Quality factor (> 100,000)
    let q = osc.quality_factor();
    assert!(q > 100_000.0, "Crystal Q was {q}");

    // Phase noise should be exceptionally clean (< -110 dBc/Hz at 10 kHz)
    let pn_10k = osc.phase_noise_dbc_per_hz(10.0e3);
    assert!(pn_10k < -100.0, "Crystal phase noise was {pn_10k} dBc/Hz");
}

#[test]
fn test_rf_power_amplifier_compression_and_pae() {
    let pa = RfPowerAmplifier::wifi_class_ab();

    // Low power input (-10 dBm) -> linear amplification with 24 dB gain -> ~14 dBm
    let (pout_dbm_lin, _) = pa.amplify(-10.0);
    assert_relative_eq!(pout_dbm_lin, 14.0, epsilon = 0.5);

    // High power input (+10 dBm) -> compression towards Psat (25 dBm)
    let (pout_dbm_sat, pout_watts) = pa.amplify(10.0);
    assert!(pout_dbm_sat <= pa.psat_dbm);
    assert!(pout_dbm_sat > pa.p1db_dbm);

    // Power Added Efficiency
    let pae = pa.evaluate_pae(pout_watts, 0.01);
    assert!(pae > 0.0 && pae <= pa.max_pae);
}

#[test]
fn test_discrete_transmitter_and_friis_transceiver_link() {
    let tx_pos = Vector3D::ZERO;
    let mut transmitter = DiscreteTransmitter::wifi_2_4ghz_patch("WiFi_AP", tx_pos);
    transmitter.antenna = PhysicalAntenna::half_wave_dipole("TX_Dipole", 2.45e9);

    // Receiver placed 50 meters away along +X axis
    let rx_pos = Vector3D::new(50.0, 0.0, 0.0);
    let rx_antenna = PhysicalAntenna::half_wave_dipole("RX_Dipole", 2.45e9);

    let solver = RfTransceiverSolver::new();
    let link = solver.solve_transceiver_link(
        &transmitter,
        &rx_antenna,
        rx_pos,
        rx_antenna.radiation_resistance_ohms() + rx_antenna.loss_resistance_ohms(), // Matched load
        3.5,                                                                        // 3.5 dB LNA NF
        20.0e6, // 20 MHz channel
    );

    assert_relative_eq!(link.distance_m, 50.0, epsilon = 1e-4);

    // Propagation delay tau = 50 m / c ~ 166.78 ns
    let expected_delay = 50.0 / SPEED_OF_LIGHT;
    assert_relative_eq!(link.propagation_delay_s, expected_delay, epsilon = 1e-12);

    // Power received should match analytical Friis transmission formula to within < 0.2%
    assert!(
        link.friis_discrepancy_ratio < 0.002,
        "Friis discrepancy was {:.6}%, P_rx={:.6e} W, P_friis={:.6e} W",
        link.friis_discrepancy_ratio * 100.0,
        link.received_power_watts,
        link.analytical_friis_power_watts,
    );

    // Received power at 50 m for ~100 mW TX should be around -45 to -65 dBm
    assert!(
        link.received_power_dbm > -75.0 && link.received_power_dbm < -30.0,
        "Received power was {:.2} dBm",
        link.received_power_dbm
    );

    // SNR should be strongly positive (> 20 dB)
    assert!(link.snr_db > 20.0, "SNR was {:.2} dB", link.snr_db);

    // Induced open-circuit voltage should be in millivolts
    assert!(link.induced_open_circuit_voltage_v > 0.0);
}

#[test]
fn test_transient_rf_pulse_burst_propagation() {
    let mut transmitter = DiscreteTransmitter::wifi_2_4ghz_patch("TX", Vector3D::ZERO);
    transmitter.antenna = PhysicalAntenna::half_wave_dipole("TX_Dipole", 2.45e9);

    let rx_pos = Vector3D::new(30.0, 0.0, 0.0); // 30 meters
    let rx_antenna = PhysicalAntenna::half_wave_dipole("RX_Dipole", 2.45e9);

    let solver = RfTransceiverSolver::new();
    let pulse_duration = 100.0e-9; // 100 ns burst
    let num_samples = 500;

    let burst = solver.simulate_transient_burst(
        &transmitter,
        &rx_antenna,
        rx_pos,
        pulse_duration,
        num_samples,
    );

    assert_eq!(burst.time_points_s.len(), num_samples);
    assert_eq!(burst.tx_current_waveform_a.len(), num_samples);
    assert_eq!(burst.rx_voltage_waveform_v.len(), num_samples);

    // Expected delay ~ 30 m / c = 100 ns
    let expected_delay = 30.0 / SPEED_OF_LIGHT;
    assert_relative_eq!(burst.propagation_delay_s, expected_delay, epsilon = 1e-11);

    // Peak received voltage should match circuit model
    assert!(burst.peak_received_voltage_v > 0.0);
}
