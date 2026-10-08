#![deny(unsafe_code)]

//! Integration Test Suite for Floquet Higher-Order Corner Magneto-Phonon Isolator & Circulator.
//!
//! Validates:
//! 1. Floquet synthetic rotation-induced Coriolis gauge field (B_synth >= 10 T) & 0D corner confinement (>= 85%).
//! 2. Magneto-phonon acoustomagnonic avoided crossing gap (>= 40 MHz) & non-reciprocal dispersion.
//! 3. 4-port cyclic circulator scattering matrix, return loss (>= 22 dB) & cyclic symmetry (dev <= 0.05 dB).
//! 4. Sharp 90-degree corner defect immunity (ratio >= 0.95), IDT transduction (>= 28%) & quantum noise (n_add <= 0.08).
//! 5. 10-point rigorous physics audit full 10/10 PASS verification.

use phonon_solver::floquet_corner_isolator::{
    FloquetCornerIsolatorProcessor, FloquetCornerParams, FloquetCornerSolver,
    FloquetTransducerParams, FourPortCirculatorParams, FourPortCirculatorSolver,
    MagnetoPhononParams, MagnetoPhononSolver, MicrowaveAcousticTransducerSolver,
};

#[test]
fn test_floquet_corner_modes_and_synthetic_gauge_field() {
    let params = FloquetCornerParams::default();
    let solver = FloquetCornerSolver::new(params);

    let m = solver.evaluate_metrics();
    assert!(
        m.synthetic_magnetic_field_tesla >= 10.0,
        "Synthetic Coriolis magnetic field must be >= 10.0 T, got {:.2} T",
        m.synthetic_magnetic_field_tesla
    );
    assert!(
        m.bulk_topological_gap_mhz >= 10.0,
        "Floquet dynamic bulk gap must be >= 10.0 MHz, got {:.2} MHz",
        m.bulk_topological_gap_mhz
    );
    assert!(
        m.corner_confinement_ratio >= 0.85,
        "0D corner mode spatial confinement must be >= 85%, got {:.3}",
        m.corner_confinement_ratio
    );
    assert_eq!(
        m.quadrupole_moment, 0.5,
        "Quadrupole moment must be quantized to 0.5 in non-trivial SOTI phase"
    );

    // 2D real-space density profile
    let density_pts = solver.compute_spatial_energy_density();
    assert!(!density_pts.is_empty());
    let corner_pts = density_pts.iter().filter(|pt| pt.is_corner_node).count();
    assert_eq!(corner_pts, 4, "Must have exactly 4 physical corner nodes");

    // 1D quasi-energy dispersion
    let dispersion = solver.compute_quasienergy_dispersion(20);
    assert_eq!(dispersion.len(), 60);
    for pt in &dispersion {
        assert!(pt.quasi_energy_upper_mhz >= pt.quasi_energy_lower_mhz);
    }
}

#[test]
fn test_magneto_phonon_coupling_and_nonreciprocal_dispersion() {
    let params = MagnetoPhononParams::default();
    let solver = MagnetoPhononSolver::new(params);

    let m = solver.evaluate_metrics();
    assert!(
        m.polariton_gap_mhz >= 40.0,
        "Avoided crossing polariton gap must be >= 40.0 MHz, got {:.2} MHz",
        m.polariton_gap_mhz
    );
    assert!(
        m.insertion_loss_db <= 0.40,
        "Forward insertion loss must be <= 0.40 dB, got {:.3} dB",
        m.insertion_loss_db
    );
    assert!(
        m.isolation_db >= 36.0,
        "Backward non-reciprocal isolation must be >= 36.0 dB, got {:.2} dB",
        m.isolation_db
    );
    assert!(
        m.group_velocity_forward_kms != m.group_velocity_backward_kms,
        "Forward and backward group velocities must be asymmetric"
    );
    assert!(
        m.nonreciprocal_wavenumber_delta_rad_um > 0.0,
        "Non-reciprocal Delta k must be strictly positive"
    );

    // Multi-frequency dispersion spectrum
    let spectrum = solver.compute_dispersion_spectrum(40);
    assert_eq!(spectrum.len(), 40);
    for pt in &spectrum {
        assert!(pt.forward_transmission_db <= 0.0);
        assert!(pt.backward_transmission_db < -10.0);
    }
}

#[test]
fn test_four_port_circulator_s_matrix_and_cyclic_symmetry() {
    let params = FourPortCirculatorParams::default();
    let solver = FourPortCirculatorSolver::new(params);

    let m = solver.evaluate_metrics();
    assert!(
        m.insertion_loss_db <= 0.40,
        "Forward cyclic insertion loss must be <= 0.40 dB, got {:.3} dB",
        m.insertion_loss_db
    );
    assert!(
        m.backward_isolation_db >= 36.0,
        "Backward adjacent isolation must be >= 36.0 dB, got {:.2} dB",
        m.backward_isolation_db
    );
    assert!(
        m.return_loss_db >= 22.0,
        "Port return loss must be >= 22.0 dB, got {:.2} dB",
        m.return_loss_db
    );
    assert!(
        m.cross_isolation_db >= 38.0,
        "Cross-port diagonal isolation must be >= 38.0 dB, got {:.2} dB",
        m.cross_isolation_db
    );
    assert!(
        m.circulation_bandwidth_3db_mhz >= 120.0,
        "3-dB circulation bandwidth must be >= 120 MHz, got {:.1} MHz",
        m.circulation_bandwidth_3db_mhz
    );
    assert!(
        m.cyclic_symmetry_deviation_db <= 0.05,
        "Cyclic 4-fold permutation deviation must be <= 0.05 dB, got {:.4} dB",
        m.cyclic_symmetry_deviation_db
    );

    // 4x4 linear S-matrix at center frequency
    let s_mat = solver.compute_s_matrix_center_linear();
    // Verify forward transmission row elements
    assert!(s_mat[1][0] >= 0.955, "S_21 forward magnitude must be >= 0.955");
    assert!(s_mat[2][1] >= 0.955, "S_32 forward magnitude must be >= 0.955");
    assert!(s_mat[3][2] >= 0.955, "S_43 forward magnitude must be >= 0.955");
    assert!(s_mat[0][3] >= 0.955, "S_14 forward magnitude must be >= 0.955");

    // Verify backward isolation row elements
    assert!(s_mat[0][1] <= 0.0158, "S_12 backward isolation must be <= 0.0158 (-36 dB)");
    assert!(s_mat[1][2] <= 0.0158, "S_23 backward isolation must be <= 0.0158 (-36 dB)");

    // S-parameter spectrum
    let spectrum = solver.compute_sparameter_spectrum(40);
    assert_eq!(spectrum.len(), 40);
}

#[test]
fn test_corner_defect_immunity_and_transducer_linearity() {
    // Corner defect test
    let clean_params = FourPortCirculatorParams {
        corner_defect_present: false,
        ..Default::default()
    };
    let defect_params = FourPortCirculatorParams {
        corner_defect_present: true,
        ..Default::default()
    };

    let clean_solver = FourPortCirculatorSolver::new(clean_params);
    let defect_solver = FourPortCirculatorSolver::new(defect_params);

    let m_clean = clean_solver.evaluate_metrics();
    let m_defect = defect_solver.evaluate_metrics();

    assert!(
        m_defect.corner_defect_transmission_ratio >= 0.95,
        "Corner defect transmission ratio must be >= 0.95, got {:.4}",
        m_defect.corner_defect_transmission_ratio
    );
    assert!(
        m_clean.insertion_loss_db <= m_defect.insertion_loss_db,
        "Clean insertion loss ({:.3} dB) must be <= defective ({:.3} dB)",
        m_clean.insertion_loss_db,
        m_defect.insertion_loss_db
    );
    assert!(
        m_defect.insertion_loss_db <= 0.40,
        "Even with obstacle defect, IL must remain <= 0.40 dB, got {:.3} dB",
        m_defect.insertion_loss_db
    );

    // Transducer test
    let t_params = FloquetTransducerParams::default();
    let t_solver = MicrowaveAcousticTransducerSolver::new(t_params);

    let t_m = t_solver.evaluate_metrics();
    assert!(
        t_m.transduction_efficiency_pct >= 28.0,
        "Transduction efficiency must be >= 28%, got {:.1}%",
        t_m.transduction_efficiency_pct
    );
    assert!(
        t_m.power_handling_p1db_dbm >= 15.0,
        "1-dB power compression must be >= +15.0 dBm, got {:.1} dBm",
        t_m.power_handling_p1db_dbm
    );
    assert!(
        t_m.added_noise_quanta <= 0.08,
        "Thermal added noise at 20 mK must be <= 0.08 quanta, got {:.5}",
        t_m.added_noise_quanta
    );

    // Power linearity curve
    let p_curve = t_solver.compute_power_linearity_curve(30);
    assert_eq!(p_curve.len(), 30);
    let low_power = &p_curve[0];
    assert!(
        low_power.compression_db < 0.01,
        "At low power (-40 dBm), compression must be negligible"
    );
}

#[test]
fn test_floquet_corner_isolator_physics_audit_10_of_10_pass() {
    let processor = FloquetCornerIsolatorProcessor::default();
    let audit = processor.evaluate_audit();

    let (passed, total) = audit.score();
    assert_eq!(
        (passed, total),
        (10, 10),
        "All 10 physics audit criteria must pass! Audit summary:\n{}",
        audit.summary()
    );
    assert!(audit.all_passed());

    assert!(audit.synthetic_magnetic_field_pass);
    assert!(audit.corner_mode_confinement_pass);
    assert!(audit.magneto_phonon_polariton_gap_pass);
    assert!(audit.forward_insertion_loss_pass);
    assert!(audit.backward_isolation_pass);
    assert!(audit.return_loss_pass);
    assert!(audit.cyclic_symmetry_pass);
    assert!(audit.circulation_bandwidth_pass);
    assert!(audit.corner_defect_immunity_pass);
    assert!(audit.cryogenic_transduction_noise_pass);

    let summary = audit.summary();
    assert!(summary.contains("10/10 PASS"));
}
