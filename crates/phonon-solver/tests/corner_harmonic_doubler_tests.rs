#![deny(unsafe_code)]

//! Comprehensive test suite for Phase 399: Corner-Induced Acoustic Second-Harmonic Doubler & Router.
//!
//! Verifies:
//! 1. 0D corner state mid-gap energy (E approx 0) and spatial confinement >= 85%.
//! 2. Hierarchical corner-to-edge modal overlap integral I_corner_edge >= 0.80.
//! 3. Steady-state second-harmonic conversion efficiency >= 30.0% under resonant cavity enhancement.
//! 4. Spurious harmonic spectral purity suppression >= 30.0 dB.
//! 5. Backscattering-immune sharp corner transmission T_corner >= 95.0% with return loss <= -25.0 dB.
//! 6. Multi-port beam steering isolation >= 25.0 dB and defect immunity T_defect >= 0.95 * T_clean.

use phonon_solver::corner_harmonic_doubler::{
    CornerBendAngle, CornerCouplingParams, CornerToEdgeLattice, CornerTopologicalRouter,
    DoublerParams, NonlinearFrequencyDoubler, RouterParams, RouterTargetPort,
};

#[test]
fn test_corner_state_mid_gap_energy_and_confinement() {
    let params = CornerCouplingParams {
        f_0_ghz: 1.0,
        gamma_mhz: 2.0,
        lambda_mhz: 8.0,
        nx: 6,
        ny: 6,
        chi_2: 0.08,
        a_mm: 5.0,
        disorder_w: 0.0,
    };

    assert!(params.is_topological_soti(), "Lattice must be in SOTI regime when gamma < lambda");
    assert_eq!(params.bulk_bandgap_mhz(), 12.0, "Bulk bandgap must be 2 * |8.0 - 2.0| = 12.0 MHz");

    let lattice = CornerToEdgeLattice::new(params);
    let result = lattice.solve();

    assert!(result.is_topological, "Result must indicate topological phase");
    assert!(
        result.corner_energy_mhz.abs() < 0.05 * result.bulk_bandgap_mhz,
        "Corner state energy |E| = {} MHz must be mid-gap (|E| < 0.05 * Delta_bulk = {} MHz)",
        result.corner_energy_mhz,
        0.05 * result.bulk_bandgap_mhz
    );

    assert!(
        result.corner_confinement >= 0.85,
        "Corner state spatial confinement must be >= 85%, got {:.2}%",
        result.corner_confinement * 100.0
    );

    for (i, cs) in result.corner_states.iter().enumerate() {
        assert!(
            cs.confinement_ratio >= 0.85,
            "Corner state {} ({:?}) confinement must be >= 85%, got {:.2}%",
            i,
            cs.corner_id,
            cs.confinement_ratio * 100.0
        );
    }
}

#[test]
fn test_hierarchical_corner_to_edge_overlap_integral() {
    let params = CornerCouplingParams::default();
    let lattice = CornerToEdgeLattice::new(params);
    let result = lattice.solve();

    assert!(
        result.hierarchical_overlap_integral >= 0.80,
        "Hierarchical corner-to-edge modal overlap integral must be >= 0.80, got {:.4}",
        result.hierarchical_overlap_integral
    );

    // Also test fast evaluation
    let fast_result = lattice.solve_fast();
    assert!(
        fast_result.hierarchical_overlap_integral >= 0.80,
        "Fast overlap integral must be >= 0.80, got {:.4}",
        fast_result.hierarchical_overlap_integral
    );
}

#[test]
fn test_steady_state_second_harmonic_conversion_efficiency() {
    let params = DoublerParams {
        p_in_mw: 50.0,
        f_1_ghz: 1.0,
        f_2_ghz: 2.0,
        q_corner: 5000.0,
        g_shg_mhz: 12.0,
        alpha_db_cm: 0.15,
        waveguide_length_mm: 10.0,
    };

    let doubler = NonlinearFrequencyDoubler::new(params);
    let ss = doubler.solve_steady_state();

    assert!(
        ss.efficiency >= 0.30,
        "Second-harmonic conversion efficiency must be >= 30.0%, got {:.2}% (P_out = {:.2} mW from P_in = {:.2} mW)",
        ss.efficiency * 100.0,
        ss.p_out_mw,
        ss.p_in_mw
    );
    assert_eq!(ss.efficiency_pct, ss.efficiency * 100.0);

    // Test transient trajectory converges towards steady state
    let transient = doubler.solve_transient(100.0, 50);
    assert!(!transient.is_empty(), "Transient trajectory must have samples");
    let last = transient.last().unwrap();
    assert!(
        (last.p_shg_mw - ss.p_out_mw).abs() < 3.0,
        "Transient end power ({:.2} mW) must converge near steady-state output ({:.2} mW)",
        last.p_shg_mw,
        ss.p_out_mw
    );
}

#[test]
fn test_spurious_harmonic_rejection() {
    let params = DoublerParams::default();
    let doubler = NonlinearFrequencyDoubler::new(params);
    let ss = doubler.solve_steady_state();

    assert!(
        ss.spectral_purity_db >= 30.0,
        "Spurious harmonic suppression spectral purity must be >= 30.0 dB, got {:.2} dB",
        ss.spectral_purity_db
    );

    let spectrum = doubler.harmonic_spectrum();
    assert_eq!(spectrum.len(), 4, "Spectrum should report 4 harmonics");

    // Second harmonic carrier (2*omega_1) must be 0.0 dB reference
    let carrier = spectrum.iter().find(|p| p.order == 2).unwrap();
    assert_eq!(carrier.relative_power_db, 0.0);

    // Fundamental leak (omega_1) must be suppressed by >= 30 dB
    let fundamental = spectrum.iter().find(|p| p.order == 1).unwrap();
    assert!(
        fundamental.relative_power_db <= -30.0,
        "Fundamental leak relative power must be <= -30.0 dB, got {:.2} dB",
        fundamental.relative_power_db
    );

    // Third harmonic (3*omega_1) must be suppressed by >= 30 dB
    let third = spectrum.iter().find(|p| p.order == 3).unwrap();
    assert!(
        third.relative_power_db <= -30.0,
        "Third harmonic relative power must be <= -30.0 dB, got {:.2} dB",
        third.relative_power_db
    );
}

#[test]
fn test_backscattering_immune_corner_transmission_and_return_loss() {
    let params = RouterParams {
        bend_angle: CornerBendAngle::Deg90,
        has_defect: false,
        target_port: RouterTargetPort::Port2Deflected,
        frequency_ghz: 2.0,
        waveguide_width_mm: 3.5,
    };

    let router = CornerTopologicalRouter::new(params);
    let s_mat = router.solve_scattering_matrix();

    assert!(
        s_mat.s21_transmission_linear >= 0.95,
        "Sharp 90-degree corner transmission T_corner must be >= 95.0%, got {:.2}%",
        s_mat.s21_transmission_linear * 100.0
    );

    assert!(
        s_mat.insertion_loss_db <= 0.22,
        "Insertion loss must be <= 0.22 dB, got {:.3} dB",
        s_mat.insertion_loss_db
    );

    assert!(
        s_mat.s11_return_loss_db <= -25.0,
        "Return loss |S_11|^2 must be <= -25.0 dB (backscattering immunity), got {:.2} dB",
        s_mat.s11_return_loss_db
    );

    assert!(
        s_mat.s11_reflection_linear <= 0.003162,
        "Power reflection factor must be <= 0.316%, got {:.4}%",
        s_mat.s11_reflection_linear * 100.0
    );
}

#[test]
fn test_multi_port_beam_steering_and_defect_immunity() {
    let mut params = RouterParams::default();
    params.target_port = RouterTargetPort::Port2Deflected;
    params.has_defect = false;

    let router_clean = CornerTopologicalRouter::new(params.clone());
    let s_clean = router_clean.solve_scattering_matrix();

    assert!(
        s_clean.cross_port_isolation_db >= 25.0,
        "Cross-port isolation must be >= 25.0 dB, got {:.2} dB",
        s_clean.cross_port_isolation_db
    );

    // Verify target port is Port 2
    let p2 = s_clean.ports.iter().find(|p| p.port_id == 2).unwrap();
    assert!(p2.is_target);
    assert!(p2.transmission_linear >= 0.95);

    // Verify non-target ports Port 1 and Port 3 are isolated
    let p1 = s_clean.ports.iter().find(|p| p.port_id == 1).unwrap();
    assert!(!p1.is_target);
    assert!(p1.isolation_db >= 25.0);

    let p3 = s_clean.ports.iter().find(|p| p.port_id == 3).unwrap();
    assert!(!p3.is_target);
    assert!(p3.isolation_db >= 25.0);

    // Test defect immunity with vacancy obstacle
    params.has_defect = true;
    let router_defect = CornerTopologicalRouter::new(params);
    let s_defect = router_defect.solve_scattering_matrix();

    assert!(
        s_defect.defect_immunity_ratio >= 0.95,
        "Defect immunity ratio T_defect / T_clean must be >= 0.95, got {:.4} (T_defect = {:.2}%, T_clean = {:.2}%)",
        s_defect.defect_immunity_ratio,
        s_defect.defect_transmission_linear * 100.0,
        s_clean.clean_transmission_linear * 100.0
    );
    assert!(
        s_defect.s21_transmission_linear >= 0.95 * s_clean.s21_transmission_linear,
        "Defect transmission must be >= 95% of clean transmission"
    );
}
