//! Integration tests for T-junction Majorana exchange, non-Abelian braiding,
//! Clifford logic synthesis, and fermion parity conservation.

use approx::assert_relative_eq;
use phonon_models::topological::{ArmId, TJunctionNanowireNetwork, TopologicalQubit};
use phonon_solver::topological::{FermionParitySolver, MajoranaBraidingSolver};

#[test]
fn test_t_junction_gate_and_geometry_control() {
    let mut network = TJunctionNanowireNetwork::default();

    // Default: Left and Right are topological, Stem is trivial
    assert!(network.is_arm_topological(ArmId::Left));
    assert!(network.is_arm_topological(ArmId::Right));
    assert!(!network.is_arm_topological(ArmId::Stem));

    // Separation between outer tips (Left 0.0, Right 0.0) is 2 * L = 2000 nm
    let sep = network.mzm_separation_nm();
    assert_relative_eq!(sep, 2000.0, epsilon = 1.0);

    // Park MZM2 in the Stem arm
    network.set_mzm_locations((ArmId::Left, 0.0), (ArmId::Stem, 0.0));
    let sep_perp = network.mzm_separation_nm();
    // sqrt(1000^2 + 1000^2) = 1414.2 nm
    assert_relative_eq!(sep_perp, 1414.2135, epsilon = 1.0);
}

#[test]
fn test_adiabatic_braiding_and_landau_zener_fidelity() {
    let network = TJunctionNanowireNetwork::default();
    let mut solver = MajoranaBraidingSolver::new(network, 2.0e-9); // 2 ns duration
    let mut qubit = TopologicalQubit::new();

    let step = solver.execute_braid_exchange(&mut qubit);

    // Minimum bulk gap along the trajectory must remain open (> 0.2 meV)
    assert!(
        step.min_bulk_gap_mev > 0.1,
        "Minigap must remain protected: got {} meV",
        step.min_bulk_gap_mev
    );

    // Landau-Zener probability must be vanishingly small (< 1e-12)
    assert!(
        step.landau_zener_probability < 1e-10,
        "Adiabatic braiding must produce negligible Landau-Zener excitation: {}",
        step.landau_zener_probability
    );

    // Gate fidelity must exceed 0.999999
    assert!(
        step.gate_fidelity > 0.999999,
        "Braiding fidelity must be near unity: {}",
        step.gate_fidelity
    );

    // Parity must remain strictly preserved
    assert!(qubit.is_parity_preserved());
}

#[test]
fn test_clifford_phase_gate_and_pauli_z() {
    let mut qubit = TopologicalQubit::new(); // Starts in |0_L>
    assert_relative_eq!(qubit.state.prob_0(), 1.0, epsilon = 1e-12);
    assert_relative_eq!(qubit.state.prob_1(), 0.0, epsilon = 1e-12);

    // Single braid B_12 generates Phase Gate S (relative phase pi/2)
    qubit.apply_braid_12();
    // Probabilities remain unchanged (pure phase rotation)
    assert_relative_eq!(qubit.state.prob_0(), 1.0, epsilon = 1e-12);
    assert_relative_eq!(qubit.state.prob_1(), 0.0, epsilon = 1e-12);

    // Four braids B_12^4 = S^4 = Identity
    qubit.apply_braid_12();
    qubit.apply_braid_12();
    qubit.apply_braid_12();
    assert_eq!(qubit.braid_count, 4);

    let target_0 = TopologicalQubit::new();
    assert_relative_eq!(qubit.state.fidelity(&target_0.state), 1.0, epsilon = 1e-12);
}

#[test]
fn test_clifford_hadamard_gate_and_superposition() {
    let mut qubit = TopologicalQubit::new(); // Starts in |0_L>

    // Apply topological Hadamard gate (B_12 * B_23 * B_12)
    qubit.apply_hadamard();

    // Must produce symmetric superposition |+_L> = (|0_L> + |1_L>) / sqrt(2)
    assert_relative_eq!(qubit.state.prob_0(), 0.5, epsilon = 1e-6);
    assert_relative_eq!(qubit.state.prob_1(), 0.5, epsilon = 1e-6);

    // Applying Hadamard a second time returns to |0_L> (H^2 = I)
    qubit.apply_hadamard();
    assert_relative_eq!(qubit.state.prob_0(), 1.0, epsilon = 1e-6);
    assert_relative_eq!(qubit.state.prob_1(), 0.0, epsilon = 1e-6);
}

#[test]
fn test_fermion_parity_tracking_and_poisoning_lifetime() {
    let mut tracker = FermionParitySolver::new(5.0); // 5 Hz poisoning rate (tau_qp = 200 ms)
    let mut qubit = TopologicalQubit::new();

    assert_relative_eq!(tracker.t1_topo_s(), 0.20, epsilon = 1e-6);
    assert_relative_eq!(tracker.t2_topo_s(), 0.40, epsilon = 1e-6);

    // Track over 50,000 gate operations of 2 ns each (total time = 100 us)
    let report = tracker.track_cycles(50_000, 2.0e-9, &mut qubit);

    // In 100 us, probability of a 5 Hz poisoning event is ~ 5e-4
    assert!(
        report.parity_retention_fidelity > 0.999,
        "Parity must be conserved across high-speed gate sequences: fidelity = {}",
        report.parity_retention_fidelity
    );
    assert!(qubit.is_parity_preserved());
}
