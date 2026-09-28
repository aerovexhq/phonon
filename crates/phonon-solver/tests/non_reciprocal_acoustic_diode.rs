use phonon_models::phononic_topological::{AcousticCirculatorParams, SpatioTemporalModulator};
use phonon_solver::phononic_topological::{AcousticCirculatorSolver, NonReciprocalDiodeSolver};
use std::f64::consts::PI;

#[test]
fn test_spatio_temporal_stiffness_modulation() {
    let modulator = SpatioTemporalModulator::default();
    let k0 = modulator.base_bulk_modulus_pa;

    // Verify dynamic modulation bounds
    let k_min =
        modulator.dynamic_bulk_modulus(0.0, PI / (2.0 * modulator.modulation_frequency_rad_per_s));
    let k_max = modulator.dynamic_bulk_modulus(0.0, 0.0);
    assert!((k_max - k0 * (1.0 + modulator.modulation_depth)).abs() < 1e-6);
    assert!((k_min - k0).abs() < 1e-6);

    // Forward vs reverse phase mismatch
    let omega = 2.0 * PI * 4000.0;
    let dk_f = modulator.phase_mismatch_forward(omega);
    let dk_r = modulator.phase_mismatch_reverse(omega);
    assert!(
        dk_f.abs() < 1e-6,
        "Forward phase mismatch should be near zero for sonic modulation"
    );
    assert!(dk_r.abs() > 50.0, "Reverse phase mismatch should be large");
}

#[test]
fn test_phonon_diode_non_reciprocity_and_isolation() {
    let modulator = SpatioTemporalModulator::default();
    let solver = NonReciprocalDiodeSolver::new(modulator);

    let omega = 2.0 * PI * 4000.0;
    let tf = solver.modulator.forward_transmission(omega);
    let tr = solver.modulator.reverse_transmission(omega);
    let iso = solver.modulator.isolation_db(omega);
    let il = solver.modulator.insertion_loss_db(omega);

    // Forward transmission high, reverse transmission strongly suppressed
    assert!(
        tf >= 0.85,
        "Forward transmission was {tf}, expected >= 0.85"
    );
    assert!(
        tr <= 0.01,
        "Reverse transmission was {tr}, expected <= 0.01"
    );

    // Non-reciprocal diode isolation must exceed 20 dB
    assert!(
        iso >= 20.0,
        "Diode isolation was {iso} dB, expected >= 20 dB"
    );
    assert!(il < 1.0, "Insertion loss was {il} dB, expected < 1.0 dB");

    // Spectral response sweep
    let spectrum = solver.solve_spectral_response(3000.0, 5000.0, 21);
    assert_eq!(spectrum.len(), 21);
    let peak = solver.peak_isolation(3000.0, 5000.0);
    assert!(peak.isolation_db >= 20.0);
}

#[test]
fn test_acoustic_circulator_3port_scattering() {
    let circulator = AcousticCirculatorParams::default();
    let solver = AcousticCirculatorSolver::new(circulator.clone());

    let w0 = circulator.unperturbed_resonance_rad_per_s();
    let (w_plus, w_minus) = circulator.resonance_frequencies_rad_per_s();
    assert!(w_plus > w0);
    assert!(w_minus < w0);
    assert!((w_plus - w_minus - circulator.mode_splitting_rad_per_s()).abs() < 1e-9);

    // 3-port S-matrix at center resonance frequency
    let s = solver.solve_circulator_s_parameters(w0);

    let s11 = s[0][0]; // Port 1 reflection
    let s21 = s[1][0]; // Port 1 -> Port 2 transmission
    let s31 = s[2][0]; // Port 1 -> Port 3 reverse isolation

    assert!(
        s21 >= 0.80,
        "Forward transmission |S21|^2 was {s21}, expected >= 0.80"
    );
    assert!(
        s31 <= 0.01,
        "Reverse isolation |S31|^2 was {s31}, expected <= 0.01"
    );
    assert!(
        s11 <= 0.20,
        "Reflection |S11|^2 was {s11}, expected <= 0.20"
    );

    let iso_db = circulator.circulator_isolation_db(w0);
    assert!(
        iso_db >= 20.0,
        "Circulator isolation was {iso_db} dB, expected >= 20 dB"
    );

    // Cyclical permutation symmetry: S21 == S32 == S13
    let s32 = s[2][1];
    let s13 = s[0][2];
    assert!((s21 - s32).abs() < 1e-6);
    assert!((s21 - s13).abs() < 1e-6);

    assert!(solver.verify_circulation());
}
