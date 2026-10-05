#![deny(unsafe_code)]

use phonon_solver::acoustic_skyrmion_router::{
    ChiralDomainWallRouter, DomainWallDefect, SkyrmionLatticeParams, SkyrmionLatticeType,
    ThieleDynamics, TopologicalChargeCalculator, Vector3Field,
};
use std::f64::consts::PI;

#[test]
fn test_single_skyrmion_topological_charge_quantization() {
    let mut params = SkyrmionLatticeParams::default();
    params.lattice_type = SkyrmionLatticeType::SingleSkyrmion;
    params.skyrmion_radius_um = 60.0;
    params.domain_size_um = 300.0;
    params.grid_nx = 100;
    params.grid_ny = 100;
    params.vorticity_m = 1;
    params.helicity_gamma_rad = 0.0; // Neel skyrmion

    let field = Vector3Field::from_params(&params);
    let calc = TopologicalChargeCalculator::new();

    // Verify continuous central-difference integral
    let q_continuous = calc.compute_total_charge(&field, false);
    // Target charge Q = -1.0
    let residual_continuous = (q_continuous.abs() - 1.0).abs();
    assert!(
        residual_continuous < 0.03,
        "Continuous Q = {q_continuous}, residual {residual_continuous} must be < 0.03"
    );

    // Verify discrete solid angle lattice charge
    let q_discrete = calc.compute_lattice_solid_angle_charge(&field, false);
    let residual_discrete = (q_discrete.abs() - 1.0).abs();
    assert!(
        residual_discrete < 0.03,
        "Discrete Q = {q_discrete}, residual {residual_discrete} must be < 0.03"
    );

    // Test antiskyrmion (vorticity m = -1 => target Q = +1.0)
    params.vorticity_m = -1;
    let anti_field = Vector3Field::from_params(&params);
    let q_anti = calc.compute_total_charge(&anti_field, false);
    assert!(
        (q_anti - 1.0).abs() < 0.03,
        "Antiskyrmion Q = {q_anti} must be approximately +1.0 within 0.03"
    );

    // Test Bloch skyrmion (gamma = pi / 2 => target Q = -1.0)
    params.vorticity_m = 1;
    params.helicity_gamma_rad = PI / 2.0;
    let bloch_field = Vector3Field::from_params(&params);
    let q_bloch = calc.compute_total_charge(&bloch_field, false);
    assert!(
        (q_bloch - (-1.0)).abs() < 0.03,
        "Bloch skyrmion Q = {q_bloch} must be approximately -1.0 within 0.03"
    );
}

#[test]
fn test_hexagonal_skyrmion_lattice_generation_and_charges() {
    let mut params = SkyrmionLatticeParams::default();
    params.lattice_type = SkyrmionLatticeType::HexagonalVortexLattice;
    params.lattice_pitch_um = 120.0;
    params.domain_size_um = 400.0;
    params.grid_nx = 80;
    params.grid_ny = 80;
    params.vorticity_m = 1;

    let field = Vector3Field::from_params(&params);
    let calc = TopologicalChargeCalculator::new();

    // Vector field normalization check: |n| = 1.0 across all grid points
    for j in 0..field.ny {
        for i in 0..field.nx {
            let n = field.get(i, j);
            let mag_sq = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
            assert!(
                (mag_sq - 1.0).abs() < 1e-6,
                "Vector field must be normalized to unit length at ({i}, {j}), got {mag_sq}"
            );
        }
    }

    // In a multi-vortex hexagonal crystal, integrated topological charge contains multiple vortices
    let q_total = calc.compute_total_charge(&field, true);
    assert!(
        q_total < -2.0,
        "Hexagonal vortex lattice must carry multiple negative vortex charges, got Q = {q_total}"
    );

    // Test square lattice generation
    params.lattice_type = SkyrmionLatticeType::SquareLattice;
    let sq_field = Vector3Field::from_params(&params);
    let q_sq = calc.compute_total_charge(&sq_field, true);
    assert!(
        q_sq < -1.0,
        "Square lattice must carry periodic vortex charges, got Q = {q_sq}"
    );
}

#[test]
fn test_thiele_equation_transverse_hall_deflection() {
    let mut thiele = ThieleDynamics::default();
    thiele.q_topo = -1.0;
    thiele.force_un = [10.0, 0.0]; // Drive along +X

    let [ux, uy] = thiele.compute_drift_velocity();
    assert!(ux > 0.0, "Drift velocity along driving axis must be positive");
    assert!(
        uy.abs() > 0.0,
        "Gyrotropic coupling must induce non-zero transverse Hall velocity u_y"
    );

    let hall_angle_deg = thiele.compute_hall_angle_deg();
    assert!(
        hall_angle_deg.abs() > 1.0,
        "Topological Hall deflection angle theta_H must be strictly non-zero, got {hall_angle_deg} deg"
    );

    // Opposite topological charge should deflect in the opposite transverse direction
    let mut anti_thiele = thiele;
    anti_thiele.q_topo = 1.0;
    let anti_angle_deg = anti_thiele.compute_hall_angle_deg();
    assert!(
        (anti_angle_deg + hall_angle_deg).abs() < 1e-4,
        "Opposite topological charge must invert Hall angle: {anti_angle_deg} vs {hall_angle_deg}"
    );

    // Verify trajectory generation
    let trajectory = thiele.compute_trajectory([0.0, 0.0], 1e-3, 50);
    assert_eq!(trajectory.len(), 50);
    let last = trajectory.last().unwrap();
    assert!(last.x_um > 0.0);
    assert!(last.y_um.abs() > 0.0);
}

#[test]
fn test_chiral_domain_wall_non_reciprocity() {
    let router = ChiralDomainWallRouter::default();
    let pt = router.compute_s_parameters_at(router.center_freq_khz, DomainWallDefect::None);

    // Forward transmission S_21 >= 0.94 (insertion loss <= 0.5 dB)
    assert!(
        pt.s21_mag >= 0.94,
        "Forward transmission |S_21| = {} must be >= 0.94 (IL = {:.2} dB)",
        pt.s21_mag,
        pt.s21_db
    );
    assert!(
        pt.s21_db <= 0.53,
        "Insertion loss must be <= 0.53 dB, got {:.2} dB",
        pt.s21_db
    );

    // Backward transmission S_12 <= 0.03 (chiral isolation >= 30.0 dB)
    assert!(
        pt.s12_mag <= 0.03,
        "Backward transmission |S_12| = {} must be <= 0.03 (Isolation = {:.2} dB)",
        pt.s12_mag,
        pt.s12_db
    );
    assert!(
        pt.s12_db >= 30.0,
        "Chiral isolation must be >= 30.0 dB, got {:.2} dB",
        pt.s12_db
    );

    // Spectrum calculation check
    let spectrum = router.compute_spectrum(15.0, 35.0, 41);
    assert_eq!(spectrum.points.len(), 41);
    assert!(spectrum.max_isolation_db >= 30.0);
}

#[test]
fn test_defect_immune_routing_around_sharp_corners() {
    let router = ChiralDomainWallRouter::default();

    // 90-degree corner bend
    let res_90 = router.evaluate_defect_immunity(DomainWallDefect::CornerBend90);
    assert!(
        res_90.transmission_ratio >= 0.95,
        "Transmission around 90-deg bend must be >= 95% of clean, got {:.3}",
        res_90.transmission_ratio
    );

    // 120-degree corner bend
    let res_120 = router.evaluate_defect_immunity(DomainWallDefect::CornerBend120);
    assert!(
        res_120.transmission_ratio >= 0.95,
        "Transmission around 120-deg bend must be >= 95% of clean, got {:.3}",
        res_120.transmission_ratio
    );

    // Missing resonator defect
    let res_defect = router.evaluate_defect_immunity(DomainWallDefect::MissingResonator);
    assert!(
        res_defect.transmission_ratio >= 0.95,
        "Transmission past missing resonator must be >= 95% of clean, got {:.3}",
        res_defect.transmission_ratio
    );
}
