#![deny(unsafe_code)]

use phonon_solver::{
    ChiralDislocationConduit, OctupoleDislocationParams, OctupoleDislocationRouter,
    OctupoleLattice, OctupoleMetamaterialParams, ScrewDislocationParams,
};

#[test]
fn test_quantized_octupole_moment() {
    // Topological regime: gamma < lambda
    let topo_params = OctupoleMetamaterialParams {
        intracell_coupling_gamma_mhz: 2.0,
        intercell_coupling_lambda_mhz: 8.0,
        bare_frequency_ghz: 1.0,
        lattice_constant_a_mm: 5.0,
        grid_cells_n: 4,
    };
    let topo_lattice = OctupoleLattice::new(topo_params);
    let o_xyz_topo = topo_lattice.quantized_octupole_moment();
    assert!((o_xyz_topo - 0.5).abs() < 1e-6, "Octupole moment in topological phase must be 0.5, got {}", o_xyz_topo);

    // Trivial regime: gamma > lambda
    let trivial_params = OctupoleMetamaterialParams {
        intracell_coupling_gamma_mhz: 8.0,
        intercell_coupling_lambda_mhz: 2.0,
        bare_frequency_ghz: 1.0,
        lattice_constant_a_mm: 5.0,
        grid_cells_n: 4,
    };
    let trivial_lattice = OctupoleLattice::new(trivial_params);
    let o_xyz_trivial = trivial_lattice.quantized_octupole_moment();
    assert!((o_xyz_trivial - 0.0).abs() < 1e-6, "Octupole moment in trivial phase must be 0.0, got {}", o_xyz_trivial);
}

#[test]
fn test_corner_state_confinement() {
    let lattice = OctupoleLattice::new(OctupoleMetamaterialParams::default());
    let corners = lattice.solve_corner_modes();
    assert_eq!(corners.len(), 8, "Expected 8 localized corner modes in 3D octupole lattice");

    for corner in &corners {
        assert!(
            corner.confinement_ratio >= 0.85,
            "Corner mode at {} had confinement {} < 0.85",
            corner.corner_index,
            corner.confinement_ratio
        );
        assert!(
            (corner.energy_detuning_mhz).abs() < 1e-2,
            "Corner mode energy detuning {} must be near zero",
            corner.energy_detuning_mhz
        );
    }
}

#[test]
fn test_bulk_bandgap() {
    let params = OctupoleMetamaterialParams {
        intracell_coupling_gamma_mhz: 2.0,
        intercell_coupling_lambda_mhz: 8.0,
        bare_frequency_ghz: 1.0,
        ..Default::default()
    };
    let lattice = OctupoleLattice::new(params);
    let gap_ghz = lattice.bulk_bandgap_ghz();
    // 2 * |8.0 - 2.0| MHz = 12.0 MHz = 0.012 GHz
    let expected_gap_ghz = 2.0 * (8.0 - 2.0) * 1e-3;
    assert!(
        (gap_ghz - expected_gap_ghz).abs() < 1e-5,
        "Bulk bandgap expected {} GHz, got {} GHz",
        expected_gap_ghz,
        gap_ghz
    );
}

#[test]
fn test_dislocation_dispersion_and_burgers_vector() {
    let conduit = ChiralDislocationConduit::new(ScrewDislocationParams::default());
    let bz = conduit.burgers_vector_norm();
    assert!(bz > 0.0, "Burgers vector norm must be positive, got {}", bz);
    assert!((bz - 4.0).abs() < 1e-4, "Expected Burgers vector bz = 4.0 mm, got {}", bz);

    let modes = conduit.compute_dislocation_dispersion(31, 1.0);
    assert_eq!(modes.len(), 31);
    for mode in &modes {
        assert!(
            mode.confinement_ratio >= 0.80,
            "Dislocation mode at kz = {} had confinement {} < 0.80",
            mode.wavenumber_kz,
            mode.confinement_ratio
        );
    }
}

#[test]
fn test_obstacle_defect_immunity() {
    let mut conduit = ChiralDislocationConduit::new(ScrewDislocationParams::default());

    // Clean conduit
    conduit.params.obstacle_defect_enabled = false;
    let (t_clean, il_clean_db) = conduit.evaluate_transmission();
    assert!(t_clean >= 0.990, "Clean transmission should be >= 99%");
    assert!(il_clean_db <= 0.05, "Clean IL should be <= 0.05 dB");

    // Defect conduit with obstacle
    conduit.params.obstacle_defect_enabled = true;
    let (t_defect, il_defect_db) = conduit.evaluate_transmission();
    assert!(
        t_defect >= 0.985,
        "Defect transmission must remain >= 98.5% due to topological protection, got {}",
        t_defect
    );
    assert!(
        il_defect_db <= 0.10,
        "Defect insertion loss must be <= 0.10 dB, got {}",
        il_defect_db
    );
}

#[test]
fn test_full_physics_audit() {
    let system = OctupoleDislocationRouter::new(OctupoleDislocationParams::default());
    let audit = system.audit_octupole_dislocation();

    assert!(audit.hamiltonian_hermitian_passed, "Hamiltonian hermiticity check failed");
    assert!(audit.quantized_octupole_moment_passed, "Quantized octupole moment check failed");
    assert!(audit.bulk_bandgap_passed, "Bulk bandgap check failed");
    assert!(audit.corner_state_confinement_passed, "Corner state confinement check failed");
    assert!(audit.burgers_vector_quantized_passed, "Burgers vector check failed");
    assert!(audit.dislocation_dispersion_gapless_passed, "Dislocation dispersion check failed");
    assert!(audit.dislocation_transmission_obstacle_passed, "Dislocation obstacle transmission check failed");
    assert!(audit.forward_insertion_loss_passed, "Forward insertion loss check failed");
    assert!(audit.cross_port_isolation_passed, "Cross-port isolation check failed");
    assert!(audit.vortex_oam_purity_passed, "Vortex OAM purity check failed");
    assert_eq!(audit.total_pass_score, 10, "Total pass score must be 10/10");
    assert!(audit.all_passed, "all_passed must be true");
}
