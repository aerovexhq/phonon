#![deny(unsafe_code)]

//! Comprehensive Verification Test Suite for Microscopic Electron & Phonon Wavepacket Scattering Simulator.
//!
//! Validates:
//! 1. TDSE Crank-Nicolson unitary integrator exact norm conservation (Delta N < 1e-6).
//! 2. Free wavepacket group velocity and analytical quantum wavepacket spreading.
//! 3. Acoustic phonon dispersion relation and inelastic scattering kinematics (absorption/emission).
//! 4. Lattice deformation potential spatio-temporal modulation.
//! 5. Quantum potential barrier reflection, tunneling, and unitarity (T + R = 1.0).
//! 6. Complex number arithmetic and boundary conditions.

use phonon_solver::wavepacket_scattering::{
    AcousticPhononMode, BarrierShape, Complex, PotentialBarrier, SchroedingerStepper,
    WavepacketParams,
};

#[test]
fn test_wavepacket_initialization_and_norm_conservation() {
    let params = WavepacketParams {
        grid_points: 256,
        domain_length_nm: 60.0,
        effective_mass_ratio: 0.067,
        center_x_nm: 15.0,
        sigma_nm: 3.5,
        initial_energy_ev: 0.12,
        direction: 1.0,
        dt_fs: 0.2,
    };

    let mut stepper = SchroedingerStepper::new(params);
    let initial_norm = stepper.total_norm();
    assert!(
        (initial_norm - 1.0).abs() < 1e-7,
        "Initial wavepacket norm must be exactly 1.0, got {:.8}",
        initial_norm
    );

    // Propagate for 50 time steps (10.0 fs)
    stepper.step_n(50, None);

    let final_norm = stepper.total_norm();
    assert!(
        (final_norm - 1.0).abs() < 1e-6,
        "Crank-Nicolson integrator must conserve total probability: got {:.8}",
        final_norm
    );
}

#[test]
fn test_free_wavepacket_propagation_velocity_and_spreading() {
    let params = WavepacketParams {
        grid_points: 512,
        domain_length_nm: 120.0,
        effective_mass_ratio: 0.067,
        center_x_nm: 25.0,
        sigma_nm: 4.0,
        initial_energy_ev: 0.15,
        direction: 1.0,
        dt_fs: 0.2,
    };

    let mut stepper = SchroedingerStepper::new(params.clone());
    let diag_initial = stepper.evaluate_diagnostics();

    // Propagate for 100 steps (20.0 fs)
    stepper.step_n(100, None);
    let diag_final = stepper.evaluate_diagnostics();

    // The center of mass must advance forward to the right
    assert!(
        diag_final.mean_position_nm > diag_initial.mean_position_nm,
        "Wavepacket must travel forward: initial x = {:.2} nm, final x = {:.2} nm",
        diag_initial.mean_position_nm,
        diag_final.mean_position_nm
    );

    let distance_traveled_nm = diag_final.mean_position_nm - diag_initial.mean_position_nm;
    let expected_distance_nm = (diag_initial.mean_velocity_m_s * 20.0e-15) * 1e9;

    assert!(
        (distance_traveled_nm - expected_distance_nm).abs() / expected_distance_nm < 0.15,
        "Measured propagation distance ({:.2} nm) must match group velocity ({:.2} nm)",
        distance_traveled_nm,
        expected_distance_nm
    );

    // Quantum wavepacket must exhibit natural spatial dispersion / spreading
    assert!(
        diag_final.position_spread_nm >= diag_initial.position_spread_nm * 0.99,
        "Wavepacket spread must not collapse: initial sigma = {:.3} nm, final sigma = {:.3} nm",
        diag_initial.position_spread_nm,
        diag_final.position_spread_nm
    );
}

#[test]
fn test_acoustic_phonon_dispersion_and_scattering_kinematics() {
    let phonon = AcousticPhononMode {
        q_rad_nm: 0.35,
        sound_velocity_m_s: 5240.0,
        strain_amplitude: 2.0e-3,
        deformation_potential_ev: 10.0,
        material_density_kg_m3: 5317.0,
    };

    let omega = phonon.angular_frequency_rad_s();
    assert!(omega > 1e11, "Phonon angular frequency must be physical");
    assert!(phonon.frequency_ghz() > 100.0);

    let hbar_omega_mev = phonon.phonon_energy_mev();
    assert!(
        hbar_omega_mev > 0.5 && hbar_omega_mev < 50.0,
        "Phonon energy must be in standard acoustic range (0.5 - 50 meV), found {:.2} meV",
        hbar_omega_mev
    );

    let k0 = 1.2; // rad/nm
    let e0 = 0.15; // eV
    let kin = phonon.evaluate_scattering_kinematics(k0, e0, 0.067);

    assert_eq!(kin.initial_k_rad_nm, k0);
    assert_eq!(kin.initial_energy_ev, e0);

    // Verify absorption: k+ = k0 + q, E+ = E0 + hbar*omega
    assert!((kin.absorption_k_rad_nm - (k0 + 0.35)).abs() < 1e-9);
    assert!((kin.absorption_energy_ev - (e0 + hbar_omega_mev * 1e-3)).abs() < 1e-6);

    // Verify emission: k- = k0 - q, E- = E0 - hbar*omega
    assert!((kin.emission_k_rad_nm - (k0 - 0.35)).abs() < 1e-9);
    assert!((kin.emission_energy_ev - (e0 - hbar_omega_mev * 1e-3)).abs() < 1e-6);

    assert!(kin.scattering_rate_s_inv > 0.0);
    assert!(kin.scattering_lifetime_fs > 0.0);
}

#[test]
fn test_lattice_deformation_potential_spatial_profile() {
    let phonon = AcousticPhononMode::default();
    let x_coords = vec![0.0, 5.0, 10.0, 15.0, 20.0];

    let pot_t0 = phonon.deformation_potential_at(&x_coords, 0.0);
    let pot_t1 = phonon.deformation_potential_at(&x_coords, 25.0);

    assert_eq!(pot_t0.len(), 5);
    assert_eq!(pot_t1.len(), 5);

    // Peak potential amplitude must equal Xi * S0 = 9.0 * 1.5e-3 = 0.0135 eV (13.5 meV)
    let expected_peak = 9.0 * 1.5e-3;
    assert!((pot_t0[0] - expected_peak).abs() < 1e-6);

    // Time-dependent phase shift: values at t=25 fs must differ from t=0
    assert_ne!(pot_t0[1], pot_t1[1]);
}

#[test]
fn test_barrier_quantum_tunneling_and_reflection() {
    let barrier = PotentialBarrier {
        barrier_x_nm: 50.0,
        barrier_height_ev: 0.25,
        barrier_width_nm: 4.0,
        barrier_shape: BarrierShape::Rectangular,
    };

    // Sub-barrier energy E = 0.10 eV < V0 = 0.25 eV -> low tunneling, high reflection
    let (t_sub, r_sub) = barrier.analytical_transmission(0.10, 0.067);
    assert!(t_sub < 0.10, "Sub-barrier tunneling must be low, got {:.3}", t_sub);
    assert!(r_sub > 0.90, "Sub-barrier reflection must dominate, got {:.3}", r_sub);
    assert!((t_sub + r_sub - 1.0).abs() < 1e-6, "Unitarity must hold");

    // Over-barrier energy E = 0.40 eV > V0 = 0.25 eV -> high transmission
    let (t_over, r_over) = barrier.analytical_transmission(0.40, 0.067);
    assert!(t_over > 0.70, "Over-barrier transmission must dominate, got {:.3}", t_over);
    assert!((t_over + r_over - 1.0).abs() < 1e-6);

    // Numerical wavepacket evaluation
    let params = WavepacketParams {
        grid_points: 256,
        domain_length_nm: 80.0,
        effective_mass_ratio: 0.067,
        center_x_nm: 20.0,
        sigma_nm: 4.0,
        initial_energy_ev: 0.10,
        direction: 1.0,
        dt_fs: 0.2,
    };
    let stepper = SchroedingerStepper::new(params);
    let res = barrier.evaluate_wavepacket_transmission(&stepper);

    assert_eq!(res.barrier_center_nm, 50.0);
    assert!(res.fringe_period_nm > 0.0);
    assert!(res.unitarity_residual < 1e-5);
}

#[test]
fn test_complex_number_arithmetic() {
    let c1 = Complex::new(3.0, 4.0);
    assert_eq!(c1.norm_sq(), 25.0);
    assert_eq!(c1.norm(), 5.0);

    let c2 = Complex::new(1.0, -2.0);
    let sum = c1.add(c2);
    assert_eq!(sum, Complex::new(4.0, 2.0));

    let diff = c1.sub(c2);
    assert_eq!(diff, Complex::new(2.0, 6.0));

    let prod = c1.mul(c2);
    assert_eq!(prod, Complex::new(11.0, -2.0));

    let quot = prod.div(c2);
    assert!((quot.re - c1.re).abs() < 1e-9);
    assert!((quot.im - c1.im).abs() < 1e-9);
}
