#![deny(unsafe_code)]

use phonon_solver::non_hermitian_skin::{
    AcousticFunnelParams, AcousticFunnelSolver, HatanoNelsonParams, NonHermitianSkinSolver,
};

#[test]
fn test_point_gap_topological_winding() {
    // Non-reciprocal regime with g = 0.45
    let mut params = HatanoNelsonParams::default();
    params.asymmetry_g = 0.45;
    let solver = NonHermitianSkinSolver::new(params.clone());

    assert_eq!(
        solver.topology.winding_number, 1,
        "Non-reciprocal Hatano-Nelson lattice must have point-gap winding number W = 1, got {}",
        solver.topology.winding_number
    );

    // Reciprocal Hermitian limit with g = 0.0
    params.asymmetry_g = 0.0;
    let solver_sym = NonHermitianSkinSolver::new(params);

    assert_eq!(
        solver_sym.topology.winding_number, 0,
        "Reciprocal lattice must have zero winding number W = 0, got {}",
        solver_sym.topology.winding_number
    );
}

#[test]
fn test_gbz_radius_and_skin_depth() {
    let params = HatanoNelsonParams {
        asymmetry_g: 0.50,
        ..Default::default()
    };

    let expected_gbz = (-0.50_f64).exp();
    let actual_gbz = params.gbz_radius();
    assert!(
        (actual_gbz - expected_gbz).abs() < 1e-6,
        "GBZ radius must be exp(-g) = {}, got {}",
        expected_gbz,
        actual_gbz
    );

    let expected_xi = 1.0 / 0.50; // 2.0 sites
    let actual_xi = params.skin_localization_length();
    assert!(
        (actual_xi - expected_xi).abs() < 1e-6,
        "Skin depth must be 1 / g = {}, got {}",
        expected_xi,
        actual_xi
    );
}

#[test]
fn test_non_hermitian_skin_eigenstate_accumulation() {
    let mut params = HatanoNelsonParams::default();
    params.chain_length = 30;
    params.asymmetry_g = 0.55;

    let solver = NonHermitianSkinSolver::new(params.clone());
    let frac = solver.boundary_localization_fraction();

    assert!(
        frac >= 0.85,
        "Under open boundaries, skin effect must localize >= 85% of eigenstate probability at right edge, got {:.2}%",
        frac * 100.0
    );

    // Symmetric case g = 0.0
    params.asymmetry_g = 0.0;
    let solver_sym = NonHermitianSkinSolver::new(params);
    let frac_sym = solver_sym.boundary_localization_fraction();

    assert!(
        frac_sym < 0.35,
        "Symmetric lattice should have uniform/sine distribution with < 35% at boundary, got {:.2}%",
        frac_sym * 100.0
    );
}

#[test]
fn test_directional_acoustic_funnel_isolation() {
    let mut params = AcousticFunnelParams::default();
    params.lattice.chain_length = 35;
    params.lattice.asymmetry_g = 0.45;

    let solver = AcousticFunnelSolver::new(params);
    let s_params = &solver.s_parameters;

    assert!(
        s_params.non_reciprocal_isolation_db >= 35.0,
        "Acoustic funnel must exhibit non-reciprocal isolation >= 35.0 dB, got {:.2} dB",
        s_params.non_reciprocal_isolation_db
    );

    assert!(
        s_params.s21_forward_db >= -1.0,
        "Forward transmission S21 must be >= -1.0 dB, got {:.2} dB",
        s_params.s21_forward_db
    );

    assert!(
        s_params.funnel_accumulation_efficiency >= 0.85,
        "Funnel accumulation efficiency must be >= 85%, got {:.2}%",
        s_params.funnel_accumulation_efficiency * 100.0
    );
}

#[test]
fn test_ultrasensitive_ep_n_perturbation_sensor() {
    let mut params = AcousticFunnelParams::default();
    params.lattice.chain_length = 30;
    params.lattice.asymmetry_g = 0.50;
    params.analyte_mass_pg = 1.0; // 1 pg

    let solver = AcousticFunnelSolver::new(params);
    let s_params = &solver.s_parameters;

    assert!(
        s_params.sensor_frequency_shift_hz > 1.0,
        "Sensor frequency shift for 1 pg analyte must be > 1.0 Hz, got {:.2} Hz",
        s_params.sensor_frequency_shift_hz
    );

    assert!(
        s_params.sensitivity_enhancement_factor >= 1000.0,
        "Sensitivity enhancement factor must be >= 1000x due to EP_N non-Hermitian scaling, got {:.1}x",
        s_params.sensitivity_enhancement_factor
    );
}
