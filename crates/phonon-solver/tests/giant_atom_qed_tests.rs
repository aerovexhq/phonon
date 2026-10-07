#![deny(unsafe_code)]

use phonon_solver::giant_atom_qed::{
    GiantAtomParams, GiantAtomProcessor, GiantAtomTopology, MultiAtomSystem, NonMarkovianSolver,
};

#[test]
fn test_superradiant_rate_enhancement() {
    let params = GiantAtomParams::preset_superradiant();
    assert_eq!(params.coupling_points, 2);
    let gamma_super = params.decay_rate_at_mhz(params.atom_freq_ghz);
    let gamma_0 = params.single_point_decay_mhz;

    // For theta = 2*pi, constructive interference yields ~ 4 * gamma_0
    assert!(
        gamma_super >= 3.8 * gamma_0,
        "Superradiant decay rate {} should be >= 3.8 * single point decay {}",
        gamma_super,
        gamma_0
    );
}

#[test]
fn test_subradiant_suppression_and_bic() {
    let params = GiantAtomParams::preset_bound_state();
    assert_eq!(params.coupling_points, 2);
    let gamma_sub = params.decay_rate_at_mhz(params.atom_freq_ghz);
    let gamma_0 = params.single_point_decay_mhz;

    // For theta = pi, destructive interference suppresses waveguide decay to near zero
    assert!(
        gamma_sub <= 0.05 * gamma_0,
        "Subradiant decay rate {} should be <= 0.05 * single point decay {}",
        gamma_sub,
        gamma_0
    );

    // Solve non-Markovian dynamics for BIC state
    let solver = NonMarkovianSolver::new(params);
    let res = solver.solve_dynamics(80.0, 100);

    // Trajectory must retain population in the atom (bound state in continuum)
    assert!(
        res.bound_state_population > 0.05,
        "Bound state population {} should be > 0.05",
        res.bound_state_population
    );
}

#[test]
fn test_non_markovian_delay_dynamics() {
    let mut params = GiantAtomParams::preset_bound_state();
    params.coupling_spacing_um = 6.0;
    let tau_ns = params.delay_time_ns();
    assert!(tau_ns > 0.0);

    let ratio = params.non_markovian_ratio();
    assert!(
        ratio >= 0.02,
        "Non-Markovian delay ratio {} should be >= 0.02",
        ratio
    );

    let solver = NonMarkovianSolver::new(params);
    let res = solver.solve_dynamics(40.0, 120);

    assert!(!res.trajectory.is_empty());
    // Population before delay tau should be captured
    let pre_delay = res
        .trajectory
        .iter()
        .find(|p| p.time_ns >= 0.2 * tau_ns && p.time_ns < tau_ns);
    assert!(pre_delay.is_some());
}

#[test]
fn test_waveguide_scattering_spectrum() {
    let params = GiantAtomParams::preset_superradiant();
    let solver = NonMarkovianSolver::new(params.clone());
    let spec = solver.compute_scattering_spectrum(
        params.atom_freq_ghz - 0.04,
        params.atom_freq_ghz + 0.04,
        80,
    );

    assert_eq!(spec.points.len(), 80);
    // At resonant center, transmission should exhibit a deep notch (reflection dip)
    assert!(
        spec.min_transmission_db <= -15.0,
        "Resonant notch min transmission {} dB should be <= -15 dB",
        spec.min_transmission_db
    );
    // Far from resonance, transmission approaches 0 dB
    assert!(
        spec.max_transmission_db >= -0.5,
        "Max transmission {} dB should be >= -0.5 dB",
        spec.max_transmission_db
    );
}

#[test]
fn test_braided_decoherence_free_coupling() {
    let params = GiantAtomParams::preset_braided_entanglement();
    assert_eq!(params.atom_topology, GiantAtomTopology::Braided);

    let system = MultiAtomSystem::new(params.clone());
    let matrix = system.compute_coupling_matrix();

    // Braided topology allows non-zero exchange coupling with suppressed collective decay
    assert!(
        matrix.exchange_coupling_mhz >= 1.0,
        "Braided exchange coupling {} MHz should be >= 1.0 MHz",
        matrix.exchange_coupling_mhz
    );
    assert!(
        matrix.gamma_ab_mhz <= 0.20 * params.single_point_decay_mhz,
        "Decoherence-free mutual decay {} MHz should be suppressed relative to single decay {}",
        matrix.gamma_ab_mhz,
        params.single_point_decay_mhz
    );
}

#[test]
fn test_bell_state_synthesis_and_concurrence() {
    let params = GiantAtomParams::preset_braided_entanglement();
    let system = MultiAtomSystem::new(params);
    let res = system.simulate_entanglement_generation(100.0, 100);

    assert!(
        res.peak_fidelity >= 0.980,
        "Bell state fidelity {} should be >= 0.980",
        res.peak_fidelity
    );
    assert!(
        res.peak_concurrence >= 0.950,
        "Concurrence {} should be >= 0.950",
        res.peak_concurrence
    );
    assert!(!res.trajectory.is_empty());
}

#[test]
fn test_10_point_physics_audit() {
    let params = GiantAtomParams::preset_braided_entanglement();
    let processor = GiantAtomProcessor::new(params);
    let audit = processor.audit_giant_atom();

    assert!(audit.superradiant_enhancement_pass);
    assert!(audit.subradiant_suppression_pass);
    assert!(audit.bound_state_persistence_pass);
    assert!(audit.non_markovian_delay_pass);
    assert!(audit.transmission_notch_pass);
    assert!(audit.transparency_window_pass);
    assert!(audit.braided_exchange_pass);
    assert!(audit.bell_fidelity_pass);
    assert!(audit.concurrence_pass);
    assert!(audit.phase_coherence_pass);
    assert_eq!(audit.total_pass_count, 10);
    assert!(audit.is_full_pass);
}
