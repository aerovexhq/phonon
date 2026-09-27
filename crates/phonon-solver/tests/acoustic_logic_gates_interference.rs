//! Integration tests for non-linear acoustic wave interference logic gates
//! and Phononic Full Adder operating mechanically without charge transport.

use phonon_models::phononic::{
    AcousticAnd, AcousticInverter, AcousticOr, AcousticWave, AcousticXor, PhononicFullAdder,
};
use phonon_solver::phononic::AcousticLogicSolver;

#[test]
fn test_acoustic_inverter_destructive_interference() {
    let amp = 1.0e-9;
    let freq = 10.0e9;
    let inverter = AcousticInverter::new(amp, freq);

    // 0 -> 1: with null input, bias beam passes through
    assert!(inverter.evaluate_binary(false));

    // 1 -> 0: with '1' input, destructive interference cancels acoustic displacement to 0
    assert!(!inverter.evaluate_binary(true));

    // Check raw wave amplitudes
    let in_1 = AcousticWave::binary_1(amp, freq);
    let out_wave = inverter.evaluate(&in_1);
    // Residual displacement should be near zero (extinction)
    assert!(
        out_wave.amplitude < 1e-12,
        "Residual amplitude = {}",
        out_wave.amplitude
    );
}

#[test]
fn test_acoustic_and_constructive_thresholding() {
    let amp = 1.0e-9;
    let freq = 10.0e9;
    let and_gate = AcousticAnd::new(amp, freq);

    assert!(!and_gate.evaluate_binary(false, false));
    assert!(!and_gate.evaluate_binary(false, true));
    assert!(!and_gate.evaluate_binary(true, false));
    // Only two in-phase coherent waves constructive interference breaches the 2.5 * A0^2 threshold
    assert!(and_gate.evaluate_binary(true, true));
}

#[test]
fn test_acoustic_or_combiner() {
    let amp = 1.0e-9;
    let freq = 10.0e9;
    let or_gate = AcousticOr::new(amp, freq);

    assert!(!or_gate.evaluate_binary(false, false));
    assert!(or_gate.evaluate_binary(false, true));
    assert!(or_gate.evaluate_binary(true, false));
    assert!(or_gate.evaluate_binary(true, true));
}

#[test]
fn test_acoustic_xor_antiphase_interference() {
    let amp = 1.0e-9;
    let freq = 10.0e9;
    let xor_gate = AcousticXor::new(amp, freq);

    assert!(!xor_gate.evaluate_binary(false, false));
    assert!(xor_gate.evaluate_binary(false, true));
    assert!(xor_gate.evaluate_binary(true, false));
    // Two in-phase inputs with one phase shifted by pi cancel to zero
    assert!(!xor_gate.evaluate_binary(true, true));
}

#[test]
fn test_phononic_full_adder_all_8_truth_table_states() {
    let amp = 1.0e-9;
    let freq = 10.0e9;
    let adder = PhononicFullAdder::new(amp, freq);

    // 100% truth table verification across all 8 states
    assert!(
        adder.verify_truth_table(),
        "Phononic Full Adder must achieve 100% truth table fidelity!"
    );

    // Explicit state checks: (A, B, Cin) -> (Sum, Cout)
    assert_eq!(adder.evaluate(false, false, false), (false, false));
    assert_eq!(adder.evaluate(false, false, true), (true, false));
    assert_eq!(adder.evaluate(false, true, false), (true, false));
    assert_eq!(adder.evaluate(false, true, true), (false, true));
    assert_eq!(adder.evaluate(true, false, false), (true, false));
    assert_eq!(adder.evaluate(true, false, true), (false, true));
    assert_eq!(adder.evaluate(true, true, false), (false, true));
    assert_eq!(adder.evaluate(true, true, true), (true, true));

    // Verify zero static leakage power
    assert_eq!(adder.static_leakage_power_watts(), 0.0);
    // Dynamic energy is in the tens of attojoules
    assert!(adder.dynamic_energy_per_op_joules() < 50.0e-18);
}

#[test]
fn test_acoustic_logic_solver_time_domain_wavepackets() {
    let solver = AcousticLogicSolver::default();

    // Verify time-domain transient simulation for Inverter
    let res_0 = solver.verify_inverter(false);
    assert!(res_0.logic_correct);
    assert!(res_0.output_state);

    let res_1 = solver.verify_inverter(true);
    assert!(res_1.logic_correct);
    assert!(!res_1.output_state);
    assert!(
        res_1.extinction_ratio_db > 20.0,
        "Inverter extinction ratio = {} dB",
        res_1.extinction_ratio_db
    );

    // Verify gates via time-domain pulse superposition
    assert!(solver.verify_and(true, true).output_state);
    assert!(!solver.verify_and(true, false).output_state);
    assert!(solver.verify_or(true, false).output_state);
    assert!(!solver.verify_or(false, false).output_state);
    assert!(solver.verify_xor(true, false).output_state);
    assert!(!solver.verify_xor(true, true).output_state);

    // Verify full adder suite
    assert!(solver.verify_full_adder_suite());
}
