#![deny(unsafe_code)]

//! Integration Tests for Quantum Acoustic Cavity Resonators, IDTs & Transmon Qubit Coupling.

use approx::assert_relative_eq;
use phonon_models::quantum_acoustic::{
    BraggAcousticMirror, InterdigitalTransducer, SawCavity, SawQubitCoupling, SawSubstrateMaterial,
    TransmonQubit,
};
use phonon_solver::quantum_acoustic::CqaMasterEquationSolver;

#[test]
fn test_piezoelectric_idt_radiation_and_efficiency() {
    let f0 = 4.5e9; // 4.5 GHz
    let n_pairs = 30;
    let aperture = 40e-6; // 40 um
    let r0 = 50.0;

    let idt = InterdigitalTransducer::new(
        SawSubstrateMaterial::LiNbO3_128YX,
        f0,
        n_pairs,
        aperture,
        r0,
    );

    // Acoustic velocity on 128-YX LiNbO3 is 3980 m/s
    let lambda0 = idt.center_wavelength();
    assert_relative_eq!(lambda0, 3980.0 / 4.5e9, epsilon = 1e-12);
    assert_relative_eq!(idt.finger_pitch(), lambda0 / 2.0, epsilon = 1e-12);
    assert_relative_eq!(idt.finger_width(), lambda0 / 4.0, epsilon = 1e-12);

    // Radiation conductance at resonance should be positive and maximal
    let ga_res = idt.radiation_conductance(f0);
    assert!(
        ga_res > 0.001,
        "Radiation conductance {} S is too small",
        ga_res
    );

    // Radiation susceptance at resonance should cross zero
    let ba_res = idt.radiation_susceptance(f0);
    assert!(
        ba_res.abs() < 1e-6,
        "Acoustic susceptance at resonance should be ~0, got {}",
        ba_res
    );

    // Conversion efficiency at resonance
    let eta = idt.conversion_efficiency(f0);
    assert!(
        eta > 0.05 && eta <= 1.0,
        "Expected realistic IDT conversion efficiency in (0.05, 1.0], got {}",
        eta
    );
}

#[test]
fn test_bragg_mirror_and_saw_cavity_quantum_fluctuations() {
    let f0 = 4.5e9;
    let n_strips = 400;
    let rs = 0.015;
    let lambda0 = 3980.0 / f0;

    let mirror = BraggAcousticMirror::new(n_strips, rs, lambda0);
    let r_power = mirror.power_reflectivity();
    assert!(
        r_power >= 0.999,
        "Mirror power reflectivity must exceed 99.9%, got {}",
        r_power
    );

    let l_pen = mirror.penetration_depth();
    assert!(l_pen > 0.0 && l_pen < 100e-6);

    let idt = InterdigitalTransducer::new(SawSubstrateMaterial::LiNbO3_128YX, f0, 30, 40e-6, 50.0);

    let cavity = SawCavity::new(
        SawSubstrateMaterial::LiNbO3_128YX,
        300e-6,
        40e-6,
        idt,
        mirror,
        1.0e5, // Q_int = 100,000
    );

    let l_eff = cavity.effective_length();
    assert!(l_eff > 300e-6);

    let fsr = cavity.free_spectral_range();
    assert!(
        fsr > 1.0e6 && fsr < 20.0e6,
        "FSR should be in MHz range, got {} Hz",
        fsr
    );

    let q_l = cavity.loaded_q();
    assert!(q_l >= 1.0e4, "Loaded Q should exceed 10,000, got {}", q_l);

    let kappa = cavity.phonon_decay_rate(f0);
    let t1_ph = cavity.phonon_lifetime(f0);
    assert_relative_eq!(t1_ph, 1.0 / kappa, epsilon = 1e-15);
    assert!(
        t1_ph >= 1.0e-6,
        "Phonon storage lifetime should exceed 1 us, got {} s",
        t1_ph
    );

    // Zero-point fluctuations
    let x_zpf = cavity.zero_point_displacement(f0);
    assert!(
        x_zpf > 1e-19 && x_zpf < 1e-15,
        "Zero-point displacement {} m is outside physical expectation",
        x_zpf
    );

    let v_zpf = cavity.zero_point_voltage(f0);
    assert!(
        v_zpf > 1e-7 && v_zpf < 1e-4,
        "Zero-point voltage {} V is outside physical expectation",
        v_zpf
    );
}

#[test]
fn test_cqa_strong_coupling_and_swap_fock_state_fidelity() {
    let f0 = 4.5e9;
    let v_zpf = 3.5e-6; // 3.5 uV vacuum fluctuation
    let kappa = 2.0 * std::f64::consts::PI * 100.0e3; // kappa / 2pi = 100 kHz (Q ~ 45,000)

    // Transmon qubit at resonance f_q = 4.5 GHz with T1 = 25 us, T_phi = 40 us
    let qubit = TransmonQubit::from_frequency_and_anharmonicity(f0, -250e6, 25.0e-6, 40.0e-6);
    let beta = 0.015;

    let coupling = SawQubitCoupling::new(qubit, f0, kappa, beta, v_zpf);

    let g_mhz = coupling.coupling_rate_hz() / 1e6;
    assert!(
        (1.0..=20.0).contains(&g_mhz),
        "Coupling rate g/2pi should be in 1-20 MHz range, got {} MHz",
        g_mhz
    );

    let cooperativity = coupling.cooperativity();
    assert!(
        cooperativity > 10.0,
        "Cooperativity must be >> 1 for cQAD strong coupling, got {}",
        cooperativity
    );
    assert!(coupling.is_strong_coupling());

    let rabi_split = coupling.vacuum_rabi_splitting_hz() / 1e6;
    assert_relative_eq!(rabi_split, 2.0 * g_mhz, epsilon = 1e-9);

    // Master equation simulation of SWAP gate
    let solver = CqaMasterEquationSolver::new(coupling, 1.0e4);
    let (f_swap, rho_final) = solver.simulate_swap_gate(100);

    // Verify density matrix validity
    let tr = rho_final.trace().re;
    assert_relative_eq!(tr, 1.0, epsilon = 1e-6);
    assert!(rho_final.purity() >= 0.90);

    // Verify Fock state |1> generation fidelity exceeds 95%
    assert!(
        f_swap >= 0.95,
        "Resonant Fock state |1> fidelity must exceed 95%, got {:.4}",
        f_swap
    );

    let f_ana = solver.analytical_swap_fidelity();
    assert_relative_eq!(f_swap, f_ana, epsilon = 0.05);
}
