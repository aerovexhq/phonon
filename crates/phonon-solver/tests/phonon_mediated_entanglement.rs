#![deny(unsafe_code)]

//! Integration Tests for Phonon-Mediated Remote Qubit Entanglement & SAW Beam Splitters.

use approx::assert_relative_eq;
use phonon_models::quantum_acoustic::{
    SawBeamSplitter, SawQubitCoupling, TransmonQubit, VirtualPhononBus,
};
use phonon_solver::quantum_acoustic::{evaluate_saw_beam_splitter, PhononEntanglementSolver};

#[test]
fn test_virtual_phonon_exchange_coupling_and_bell_state_fidelity() {
    let f_cavity = 4.5e9;
    let v_zpf = 3.5e-6;
    let kappa = 2.0 * std::f64::consts::PI * 100.0e3;
    let beta = 0.015;

    // Both qubits detuned by +100 MHz from cavity (4.6 GHz)
    let delta = 100.0e6;
    let f_qubit = f_cavity + delta;
    let qubit1 = TransmonQubit::from_frequency_and_anharmonicity(f_qubit, -250e6, 30.0e-6, 50.0e-6);
    let qubit2 = TransmonQubit::from_frequency_and_anharmonicity(f_qubit, -250e6, 30.0e-6, 50.0e-6);

    let coup1 = SawQubitCoupling::new(qubit1, f_cavity, kappa, beta, v_zpf);
    let coup2 = SawQubitCoupling::new(qubit2, f_cavity, kappa, beta, v_zpf);
    let bus = VirtualPhononBus::new(coup1, coup2);

    let j_mhz = bus.effective_exchange_coupling_hz().abs() / 1e6;
    assert!(
        (0.1..=5.0).contains(&j_mhz),
        "Expected virtual exchange coupling J_eff/2pi in 0.1-5 MHz, got {} MHz",
        j_mhz
    );

    let t_bell = bus.bell_state_duration_seconds();
    assert!(
        t_bell > 1e-9 && t_bell < 1e-6,
        "Bell state generation time should be in nanosecond regime, got {} s",
        t_bell
    );

    let solver = PhononEntanglementSolver::new(bus);
    let (f_bell, concurrence, rho_final) = solver.simulate_bell_state_generation(100);

    // Verify Bell state fidelity exceeds 95%
    assert!(
        f_bell >= 0.95,
        "Phonon-mediated Bell state fidelity must exceed 95%, got {:.4}",
        f_bell
    );

    // Verify high entanglement concurrence
    assert!(
        concurrence >= 0.90,
        "Concurrence must exceed 0.90, got {:.4}",
        concurrence
    );

    let f_ana = solver.analytical_bell_fidelity();
    assert_relative_eq!(f_bell, f_ana, epsilon = 0.05);

    // Verify trace is 1.0
    let mut tr = 0.0;
    for i in 0..4 {
        tr += rho_final.get(i, i).re;
    }
    assert_relative_eq!(tr, 1.0, epsilon = 1e-6);
}

#[test]
fn test_saw_directional_beam_splitter_and_hom_interference() {
    // 50:50 beam splitter condition: C_bs * L_bs = pi / 4
    let l_bs = 100.0e-6; // 100 um
    let c_bs = std::f64::consts::PI / (4.0 * l_bs);

    let beam_splitter = SawBeamSplitter::new(l_bs, c_bs);

    let report = evaluate_saw_beam_splitter(&beam_splitter);

    // Power transmission and reflection should both be 50%
    assert_relative_eq!(report.transmission, 0.5, epsilon = 1e-6);
    assert_relative_eq!(report.reflection, 0.5, epsilon = 1e-6);

    // Two-phonon coincidence probability should vanish (Hong-Ou-Mandel bunching)
    assert_relative_eq!(report.coincidence_probability, 0.0, epsilon = 1e-6);

    // HOM visibility should be 100%
    assert_relative_eq!(report.hom_visibility, 1.0, epsilon = 1e-6);

    // Bunched two-phonon state amplitude |2, 0> / |0, 2> is 1 / sqrt(2) ~ 0.7071
    assert_relative_eq!(
        report.bunched_amplitude,
        1.0 / 2.0f64.sqrt(),
        epsilon = 1e-6
    );
}
