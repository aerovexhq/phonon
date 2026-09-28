//! Integration Tests for Floquet Topological Bands & Chiral Edge States.

use approx::assert_relative_eq;
use phonon_core::constants::CONDUCTANCE_QUANTUM;
use phonon_models::floquet::{FloquetGrapheneLattice, FloquetLaserPulse, FloquetPolarization};
use phonon_solver::floquet::FloquetBandSolver;

#[test]
fn test_laser_pulse_parameters_and_magnus_gap() {
    let pulse = FloquetLaserPulse {
        wavelength_m: 3.2e-6,               // 3.2 um mid-IR
        peak_electric_field_v_per_m: 4.0e8, // 0.4 V/nm
        pulse_duration_seconds: 60.0e-15,
    };

    // 3.2 um photon energy ~ 0.387 eV
    let hbar_omega = pulse.photon_energy_ev();
    assert_relative_eq!(hbar_omega, 0.3874, epsilon = 0.01);

    let lattice = FloquetGrapheneLattice::default();
    let a_dim = lattice.dimensionless_drive_amplitude(&pulse);
    assert!(
        a_dim > 0.05 && a_dim < 0.5,
        "Dimensionless drive: {}",
        a_dim
    );

    // Circular polarization (RCP)
    let rcp = FloquetPolarization::rcp();
    let mass_rcp = lattice.floquet_mass_gap_ev(&pulse, &rcp);
    assert!(
        mass_rcp > 0.05,
        "Floquet mass gap should be > 50 meV, got {} eV",
        mass_rcp
    );
    let full_gap = lattice.total_topological_bandgap_ev(&pulse, &rcp);
    assert_relative_eq!(full_gap, 2.0 * mass_rcp, epsilon = 1e-9);

    // Linear polarization preserves zero gap at leading Magnus order
    let lin = FloquetPolarization::linear(0.0);
    let mass_lin = lattice.floquet_mass_gap_ev(&pulse, &lin);
    assert_eq!(mass_lin, 0.0);
}

#[test]
fn test_floquet_chern_number_and_quantized_hall_conductance() {
    let lattice = FloquetGrapheneLattice::default();

    let rcp = FloquetPolarization::rcp();
    assert_eq!(lattice.floquet_chern_number(&rcp), 1);
    let sigma_rcp = lattice.floquet_hall_conductance_si(&rcp);
    assert_relative_eq!(sigma_rcp, CONDUCTANCE_QUANTUM / 2.0, epsilon = 1e-6);

    let lcp = FloquetPolarization::lcp();
    assert_eq!(lattice.floquet_chern_number(&lcp), -1);
    let sigma_lcp = lattice.floquet_hall_conductance_si(&lcp);
    assert_relative_eq!(sigma_lcp, -CONDUCTANCE_QUANTUM / 2.0, epsilon = 1e-6);

    let lin = FloquetPolarization::linear(0.5);
    assert_eq!(lattice.floquet_chern_number(&lin), 0);
    assert_eq!(lattice.floquet_hall_conductance_si(&lin), 0.0);
}

#[test]
fn test_floquet_band_solver_and_edge_states() {
    let lattice = FloquetGrapheneLattice::default();
    let pulse = FloquetLaserPulse {
        wavelength_m: 3.2e-6,
        peak_electric_field_v_per_m: 4.0e8,
        pulse_duration_seconds: 60.0e-15,
    };
    let rcp = FloquetPolarization::rcp();
    let solver = FloquetBandSolver::new();

    // High symmetry path
    let path = solver.compute_high_symmetry_path(&lattice, &pulse, &rcp, 30);
    assert_eq!(path.len(), 90);

    let min_gap = FloquetBandSolver::min_bandgap_ev(&path);
    let expected_gap = lattice.total_topological_bandgap_ev(&pulse, &rcp);
    assert!(min_gap > 0.1, "Min gap: {}", min_gap);
    assert_relative_eq!(min_gap, expected_gap, epsilon = 0.05);

    // Nanoribbon with chiral edge states
    let ribbon = solver.compute_nanoribbon_edge_states(&lattice, &pulse, &rcp, 8, 41);
    assert_eq!(ribbon.len(), 41);
}
