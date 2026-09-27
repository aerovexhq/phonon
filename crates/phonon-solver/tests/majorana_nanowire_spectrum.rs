//! Integration tests for Majorana Nanowire tight-binding BdG spectrum,
//! topological phase transitions, boundary localization, and quantized zero-bias conductance.

use approx::assert_relative_eq;
use phonon_models::topological::{
    BdGHamiltonian, MajoranaNanowire, NanowireParams, TunnelingConductanceModel,
    QUANTUM_CONDUCTANCE,
};

#[test]
fn test_bdg_particle_hole_symmetry() {
    let params = NanowireParams {
        num_sites: 16,
        chemical_potential_mev: 0.2,
        zeeman_mev: 1.0,
        ..Default::default()
    };

    let bdg = BdGHamiltonian::new(params);
    let solution = bdg.solve();

    let n = solution.eigenvalues.len();
    assert_eq!(n, 4 * 16);

    // Particle-hole symmetry guarantees that for every eigenvalue E, there exists -E
    for i in 0..(n / 2) {
        let neg_e = solution.eigenvalues[i];
        let pos_e = solution.eigenvalues[n - 1 - i];
        assert_relative_eq!(neg_e, -pos_e, epsilon = 1e-7);
    }
}

#[test]
fn test_topological_phase_transition_and_zero_modes() {
    // 1. Trivial regime: V_Z = 0.2 meV < sqrt(Delta^2 + mu^2) = sqrt(0.5^2 + 0^2) = 0.5 meV
    let trivial_params = NanowireParams {
        num_sites: 20,
        pairing_delta_mev: 0.5,
        chemical_potential_mev: 0.0,
        zeeman_mev: 0.2,
        ..Default::default()
    };
    assert!(!trivial_params.is_topological());

    let trivial_mzm = MajoranaNanowire::solve(trivial_params);
    // In trivial regime, lowest positive energy is gapped (near Delta - V_Z = 0.3 meV)
    assert!(
        trivial_mzm.zero_mode_energy_mev > 0.15,
        "Trivial phase must be bulk-gapped without zero modes: got {} meV",
        trivial_mzm.zero_mode_energy_mev
    );

    // 2. Topological regime: V_Z = 1.2 meV > 0.5 meV
    let topo_params = NanowireParams {
        num_sites: 36,
        pairing_delta_mev: 0.5,
        chemical_potential_mev: 0.0,
        zeeman_mev: 1.2,
        rashba_mev_nm: 40.0,
        ..Default::default()
    };
    assert!(topo_params.is_topological());

    let topo_mzm = MajoranaNanowire::solve(topo_params);
    println!("MZM 36 sites energy: {} meV", topo_mzm.zero_mode_energy_mev);
    // In topological regime, lowest positive energy is substantially suppressed below the bulk gap
    assert!(
        topo_mzm.zero_mode_energy_mev < 0.10,
        "Topological phase must host near-zero Majorana bound states: got {} meV",
        topo_mzm.zero_mode_energy_mev
    );
}

#[test]
fn test_majorana_spatial_boundary_localization() {
    let params = NanowireParams {
        num_sites: 36,
        pairing_delta_mev: 0.5,
        zeeman_mev: 1.2,
        rashba_mev_nm: 40.0,
        ..Default::default()
    };

    let mzm = MajoranaNanowire::solve(params);

    // End fraction across 12 sites (1/3 of the wire on each side)
    let (g1_left, g2_right) = mzm.boundary_localization_fractions(12);
    println!(
        "MZM boundary fractions: left={}, right={}",
        g1_left, g2_right
    );

    // Majoranas must be exponentially localized at opposite boundaries
    assert!(
        g1_left > 0.70,
        "gamma_1 must be strongly localized at the left end: fraction = {}",
        g1_left
    );
    assert!(
        g2_right > 0.70,
        "gamma_2 must be strongly localized at the right end: fraction = {}",
        g2_right
    );

    // Also verify that gamma_1 has almost no presence at the far right and vice-versa
    let g1_right: f64 = mzm.gamma1_density[24..36].iter().sum();
    let g2_left: f64 = mzm.gamma2_density[0..12].iter().sum();
    assert!(g1_right < 0.15, "gamma_1 must have small right-end tail");
    assert!(g2_left < 0.15, "gamma_2 must have small left-end tail");
}

#[test]
fn test_quantized_zero_bias_conductance_peak() {
    let model = TunnelingConductanceModel {
        tunnel_coupling_mev: 0.05,
        dissipation_mev: 0.0,
        majorana_energy_mev: 0.0,
        temperature_k: 0.0001, // Ultra-low temperature limit
        background_gn: 0.0,
    };

    // Zero-bias conductance must be quantized at exactly 2e^2/h
    let g_zero = model.zero_bias_conductance();
    assert_relative_eq!(
        g_zero,
        QUANTUM_CONDUCTANCE,
        epsilon = 1e-4 * QUANTUM_CONDUCTANCE
    );

    // Off-resonance conductance (e.g. at V = 0.5 mV) must be significantly lower
    let g_off = model.differential_conductance(0.5e-3);
    assert!(
        g_off < 0.1 * QUANTUM_CONDUCTANCE,
        "Off-resonance conductance must drop: g_off = {}, G0 = {}",
        g_off,
        QUANTUM_CONDUCTANCE
    );

    // Finite temperature broadening test: at T = 50 mK, peak broadens and decreases
    let warm_model = TunnelingConductanceModel {
        temperature_k: 0.05, // 50 mK
        ..model
    };
    let g_warm = warm_model.zero_bias_conductance();
    assert!(
        g_warm < g_zero,
        "Thermal broadening must reduce zero-bias peak height: {} vs {}",
        g_warm,
        g_zero
    );
}
