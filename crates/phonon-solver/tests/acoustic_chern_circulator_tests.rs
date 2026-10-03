#![deny(unsafe_code)]

//! Comprehensive test suite for Phase 343: Acoustic Chern Insulator & 3-Port Chiral Circulator Engine.
//!
//! Verifies:
//! - Time-reversal symmetry breaking under spinning fluid angular velocity Omega > 0.
//! - Quantized topological Chern number C = +1 and non-zero bandgap Delta_topo > 0.
//! - 3-port cyclic scattering matrix non-reciprocity (S_21 >= 0.94, S_31 <= -35.0 dB, S_11 <= -25.0 dB).
//! - Backscattering-immune edge wave routing around sharp 90-deg corner, 120-deg bend, and vacancy defects (T_defect >= 0.95 * T_clean).

use phonon_solver::acoustic_chern_circulator::{
    ChernLatticeParams, CirculatorPort, ObstacleKind, ThreePortCirculator,
};

#[test]
fn test_time_reversal_symmetry_breaking() {
    // 1. Static fluid (Omega = 0 rad/s): Time-reversal symmetry preserved
    let params_trs = ChernLatticeParams {
        omega_rad_s: 0.0,
        ..ChernLatticeParams::default()
    };

    assert!(!params_trs.is_time_reversal_broken());
    assert_eq!(params_trs.compute_chern_number(), 0);
    assert_eq!(params_trs.compute_topological_gap_khz(), 0.0);
    assert_eq!(params_trs.effective_topological_mass(), 0.0);
    assert_eq!(params_trs.fluid_circulation_m2_s(), 0.0);

    let circulator_trs = ThreePortCirculator::new(params_trs);
    let s_trs = circulator_trs.compute_scattering_matrix();
    // Reciprocal: forward and backward transmissions are equal
    assert_eq!(s_trs.s21, s_trs.s12);
    assert_eq!(s_trs.s31, s_trs.s13);
    assert!((s_trs.s21 - s_trs.s31).abs() < 1.0e-6);

    // 2. Spinning fluid (Omega = 1200 rad/s > 0): Time-reversal symmetry broken
    let params_broken = ChernLatticeParams::default();
    assert!(params_broken.is_time_reversal_broken());
    assert!(params_broken.fluid_circulation_m2_s() > 0.0);
    assert!(params_broken.effective_topological_mass() > 0.0);
    assert_eq!(params_broken.compute_chern_number(), 1);
    assert!(params_broken.compute_topological_gap_khz() > 0.0);

    let circulator_broken = ThreePortCirculator::new(params_broken);
    let s_broken = circulator_broken.compute_scattering_matrix();

    // Giant non-reciprocal contrast: S_21 >> S_12
    assert!(s_broken.s21 > 0.94);
    assert!(s_broken.s12 < 0.02);
    let isolation_db = s_broken.isolation_db(CirculatorPort::Port1, CirculatorPort::Port3);
    assert!(isolation_db >= 35.0);

    // 3. Reversed spinning fluid (Omega = -1200 rad/s < 0): Reverse chirality C = -1
    let params_rev = ChernLatticeParams {
        omega_rad_s: -1200.0,
        ..ChernLatticeParams::default()
    };
    assert!(params_rev.is_time_reversal_broken());
    assert_eq!(params_rev.compute_chern_number(), -1);
    assert!(params_rev.compute_topological_gap_khz() > 0.0);

    let circulator_rev = ThreePortCirculator::new(params_rev);
    let s_rev = circulator_rev.compute_scattering_matrix();
    // Reversed cyclic routing: 1 -> 3 is forward transmission, 1 -> 2 is isolated
    assert!(s_rev.s31 > 0.94, "Reverse chirality must route Port 1 -> Port 3");
    assert!(s_rev.s21 < 0.02, "Reverse chirality must isolate Port 2 from Port 1");
}

#[test]
fn test_topological_chern_number_and_gap() {
    let params = ChernLatticeParams::default();

    // Verify default parameters
    assert_eq!(params.a_0_mm, 20.0);
    assert_eq!(params.c_0_mps, 343.0);
    assert_eq!(params.r_cyl_mm, 4.0);
    assert_eq!(params.omega_rad_s, 1200.0);
    assert_eq!(params.kappa, 0.15);
    assert_eq!(params.f_0_khz, 4.0);

    // Topological Chern number C = +1
    let chern = params.compute_chern_number();
    assert_eq!(chern, 1, "Spinning fluid with Omega > 0 must have Chern number C = +1");

    // Topological bandgap Delta_topo ~ 1.5 kHz near center frequency f_0 ~ 4.0 kHz
    let gap = params.compute_topological_gap_khz();
    assert!(
        (gap - 1.50).abs() < 0.05,
        "Topological bandgap should be ~1.50 kHz: got {} kHz",
        gap
    );

    // Berry curvature evaluation across 2D Brillouin zone
    let berry_k = params.compute_berry_curvature(0.0, 0.0);
    assert!(berry_k.is_finite());

    let grid = params.compute_berry_curvature_grid(16);
    assert_eq!(grid.len(), 256);
    let mut sum_curvature = 0.0;
    for pt in &grid {
        sum_curvature += pt.berry_curvature;
    }
    assert!(sum_curvature > 0.0, "Integrated Berry curvature must be positive for C = +1");

    // Chiral edge mode dispersion
    let edge_mode = params.compute_chiral_edge_mode();
    assert_eq!(edge_mode.chern_number, 1);
    assert!(
        edge_mode.group_velocity_mps > 190.0,
        "Chiral edge group velocity must be positive: got {} m/s",
        edge_mode.group_velocity_mps
    );
    assert_eq!(edge_mode.center_frequency_khz, 4.0);
    assert!(
        edge_mode.decay_length_mm > 20.0 && edge_mode.decay_length_mm < 50.0,
        "Bulk decay length should be ~36 mm: got {} mm",
        edge_mode.decay_length_mm
    );
    assert!(!edge_mode.dispersion_curve.is_empty());
}

#[test]
fn test_three_port_scattering_matrix_cyclic_non_reciprocity() {
    let circulator = ThreePortCirculator::default();
    let s_mat = circulator.compute_scattering_matrix();

    // 1. Port 1 to 2 forward transmission: |S_21| >= 0.94, insertion loss <= 0.5 dB
    assert!(
        s_mat.s21 >= 0.94,
        "Forward transmission |S_21| must be >= 0.94: got {}",
        s_mat.s21
    );
    let il_p1 = s_mat.insertion_loss_db(CirculatorPort::Port1, CirculatorPort::Port2);
    assert!(
        il_p1 <= 0.50,
        "Insertion loss for Port 1 -> 2 must be <= 0.5 dB: got {} dB",
        il_p1
    );

    // 2. Port 1 to 3 isolation: |S_31| <= 0.0178, isolation >= 35.0 dB
    assert!(
        s_mat.s31 <= 0.0178,
        "Isolation transmission |S_31| must be <= 0.0178: got {}",
        s_mat.s31
    );
    let iso_p1 = s_mat.isolation_db(CirculatorPort::Port1, CirculatorPort::Port3);
    assert!(
        iso_p1 >= 35.0,
        "Isolation for Port 1 -> 3 must be >= 35.0 dB: got {} dB",
        iso_p1
    );
    let s31_db = s_mat.s_param_db(3, 1);
    assert!(
        s31_db <= -35.0,
        "S_31 (dB) must be <= -35.0 dB: got {} dB",
        s31_db
    );

    // 3. Port 1 return loss: |S_11| <= 0.056, return loss <= -25.0 dB
    assert!(
        s_mat.s11 <= 0.056,
        "Return loss |S_11| must be <= 0.056: got {}",
        s_mat.s11
    );
    let rl_p1 = s_mat.return_loss_db(CirculatorPort::Port1);
    assert!(
        rl_p1 <= -25.0,
        "Return loss must be <= -25.0 dB: got {} dB",
        rl_p1
    );

    // 4. Cyclic permutation: S_32 approx S_21, S_13 approx S_21 (1 -> 2 -> 3 -> 1)
    assert!(
        (s_mat.s32 - s_mat.s21).abs() < 1.0e-6,
        "S_32 must equal S_21: got {} vs {}",
        s_mat.s32,
        s_mat.s21
    );
    assert!(
        (s_mat.s13 - s_mat.s21).abs() < 1.0e-6,
        "S_13 must equal S_21: got {} vs {}",
        s_mat.s13,
        s_mat.s21
    );
    assert!(
        (s_mat.s12 - s_mat.s31).abs() < 1.0e-6,
        "S_12 must equal S_31: got {} vs {}",
        s_mat.s12,
        s_mat.s31
    );
    assert!(
        (s_mat.s23 - s_mat.s31).abs() < 1.0e-6,
        "S_23 must equal S_31: got {} vs {}",
        s_mat.s23,
        s_mat.s31
    );

    // 5. Cyclic routing from all three ports
    for p in [CirculatorPort::Port1, CirculatorPort::Port2, CirculatorPort::Port3] {
        let out = circulator.output_port_for(p);
        let iso = circulator.isolated_port_for(p);
        let il = s_mat.insertion_loss_db(p, out);
        let is = s_mat.isolation_db(p, iso);
        assert!(il <= 0.50, "Insertion loss from {:?} to {:?} must be <= 0.5 dB", p, out);
        assert!(is >= 35.0, "Isolation from {:?} to {:?} must be >= 35.0 dB", p, iso);
    }

    // 6. S-parameter spectrum sweep over [3.0 kHz, 5.0 kHz]
    let spectrum = circulator.compute_spectrum(3.0, 5.0, 60);
    assert_eq!(spectrum.points.len(), 60);
    assert!(
        spectrum.peak_isolation_db >= 35.0,
        "Peak isolation in spectrum must be >= 35.0 dB: got {} dB",
        spectrum.peak_isolation_db
    );
    assert!(
        spectrum.min_insertion_loss_db <= 0.50,
        "Minimum insertion loss in spectrum must be <= 0.50 dB: got {} dB",
        spectrum.min_insertion_loss_db
    );
}

#[test]
fn test_backscattering_immune_corner_bending() {
    let circulator = ThreePortCirculator::default();

    // 1. Clean boundary baseline
    let res_clean = circulator.evaluate_defect_immunity(ObstacleKind::None);
    assert!(res_clean.t_clean >= 0.95);
    assert!(res_clean.is_immune);
    assert_eq!(res_clean.transmission_percent, 100.0);

    // 2. Sharp 90-degree corner
    let res_90 = circulator.evaluate_defect_immunity(ObstacleKind::SharpCorner90);
    assert!(
        res_90.t_defect >= 0.95 * res_90.t_clean,
        "Transmission around sharp 90-deg corner must be >= 0.95 * T_clean: got {} vs {}",
        res_90.t_defect,
        0.95 * res_90.t_clean
    );
    assert!(
        res_90.transmission_ratio >= 0.95,
        "Backscattering immunity ratio must be >= 0.95: got {}",
        res_90.transmission_ratio
    );
    assert!(
        res_90.backscattering_reflection_db <= -30.0,
        "Backscattering reflection must be <= -30 dB: got {} dB",
        res_90.backscattering_reflection_db
    );
    assert!(res_90.is_immune);

    // 3. Sharp 120-degree bend
    let res_120 = circulator.evaluate_defect_immunity(ObstacleKind::SharpBend120);
    assert!(
        res_120.t_defect >= 0.95 * res_120.t_clean,
        "Transmission around 120-deg bend must be >= 0.95 * T_clean: got {} vs {}",
        res_120.t_defect,
        0.95 * res_120.t_clean
    );
    assert!(res_120.is_immune);

    // 4. Missing lattice site vacancy
    let res_vac = circulator.evaluate_defect_immunity(ObstacleKind::MissingSiteVacancy);
    assert!(
        res_vac.t_defect >= 0.95 * res_vac.t_clean,
        "Transmission around missing site vacancy must be >= 0.95 * T_clean: got {} vs {}",
        res_vac.t_defect,
        0.95 * res_vac.t_clean
    );
    assert!(res_vac.is_immune);

    // 5. Compare with trivial non-topological lattice (Omega = 0)
    let circulator_trivial = ThreePortCirculator::new(ChernLatticeParams {
        omega_rad_s: 0.0,
        ..ChernLatticeParams::default()
    });
    let res_trivial = circulator_trivial.evaluate_defect_immunity(ObstacleKind::SharpCorner90);
    assert!(
        !res_trivial.is_immune,
        "Trivial reciprocal waveguide must suffer strong backscattering at 90-deg corner"
    );
    assert!(
        res_trivial.transmission_ratio < 0.95,
        "Trivial transmission ratio must be < 0.95: got {}",
        res_trivial.transmission_ratio
    );
}
