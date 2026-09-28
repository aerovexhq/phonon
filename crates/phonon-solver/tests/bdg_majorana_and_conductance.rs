#![deny(unsafe_code)]

use phonon_models::superconducting_spintronics::{
    FermionParity, FourMajoranaQubit, TopologicalNanowireParams,
};
use phonon_solver::superconducting_spintronics::{BdgHamiltonianSolver, QUANTUM_CONDUCTANCE_G0};

#[test]
fn test_bdg_topological_phase_transition() {
    let solver = BdgHamiltonianSolver::new(100);

    // Trivial regime: E_Z < sqrt(Delta^2 + mu^2)
    let mut params_trivial = TopologicalNanowireParams::inas_al_standard();
    params_trivial.zeeman_energy_mev = 0.1; // Below Delta_0 = 0.25 meV
    let sol_trivial = solver.solve(&params_trivial);

    assert!(!sol_trivial.is_topological);
    assert_eq!(sol_trivial.topological_gap_mev, 0.0);
    assert!(sol_trivial.zero_bias_conductance_normalized < 0.1);

    // Topological regime: E_Z > sqrt(Delta^2 + mu^2)
    let params_topo = TopologicalNanowireParams::inas_al_standard(); // E_Z = 0.75 meV > 0.25 meV
    let sol_topo = solver.solve(&params_topo);

    assert!(sol_topo.is_topological);
    assert!(sol_topo.topological_gap_mev > 0.05);
    // Quantized zero-bias peak ~ 1.0 (2e^2/h)
    assert!(
        sol_topo.zero_bias_conductance_normalized > 0.99,
        "Quantized conductance should be ~ 1.0, got {}",
        sol_topo.zero_bias_conductance_normalized
    );
    assert!(sol_topo.localization_ratio > 10.0);
}

#[test]
fn test_differential_conductance_spectrum() {
    let solver = BdgHamiltonianSolver::new(100);
    let params = TopologicalNanowireParams::inas_al_standard();

    let v_bias = vec![-0.1, -0.05, 0.0, 0.05, 0.1];
    let spectrum = solver.conductance_spectrum(&params, &v_bias);

    assert_eq!(spectrum.len(), 5);
    // Peak at V = 0
    let g_zero = spectrum[2];
    assert!(
        (g_zero - QUANTUM_CONDUCTANCE_G0).abs() < 0.01 * QUANTUM_CONDUCTANCE_G0,
        "Zero-bias conductance peak should be 2e^2/h = {}, got {}",
        QUANTUM_CONDUCTANCE_G0,
        g_zero
    );

    // Sub-gap points are lower than the resonant peak
    assert!(spectrum[1] < g_zero);
    assert!(spectrum[3] < g_zero);
}

#[test]
fn test_four_majorana_qubit_braiding() {
    let params = TopologicalNanowireParams::inas_al_standard();
    let mut qubit = FourMajoranaQubit::new_zero(params);

    // Initial state |0_L>
    assert_eq!(qubit.alpha, (1.0, 0.0));
    assert_eq!(qubit.beta, (0.0, 0.0));

    // B12 braid: Phase gate S (alpha -> alpha, beta -> i*beta)
    qubit.apply_braid_12();
    assert!((qubit.alpha.0 - 1.0).abs() < 1e-6);

    // B23 braid: Hadamard-like superposition gate
    qubit.apply_braid_23();
    let inv_sqrt2 = 1.0 / std::f64::consts::SQRT_2;
    assert!((qubit.alpha.0 - inv_sqrt2).abs() < 1e-4);
    assert!((qubit.beta.1 + inv_sqrt2).abs() < 1e-4);

    // Diabatic error for 10 ns adiabatic braiding
    let eps_diab = qubit.diabatic_error(1.0e-8);
    assert!(
        eps_diab < 1e-4,
        "Diabatic excitation probability should be < 1e-4, got {}",
        eps_diab
    );
}

#[test]
fn test_fermion_parity_operations() {
    let mut parity = FermionParity::Even;
    assert_eq!(parity.sign_value(), 1.0);

    parity = parity.flip();
    assert_eq!(parity, FermionParity::Odd);
    assert_eq!(parity.sign_value(), -1.0);

    parity = parity.flip();
    assert_eq!(parity, FermionParity::Even);
}
