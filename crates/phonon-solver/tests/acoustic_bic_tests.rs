#![deny(unsafe_code)]

use phonon_solver::acoustic_bic::{
    BicKind, BicLatticeParams, BicLatticeSolver, CavityVortexEngine, CavityVortexParams,
};

#[test]
fn test_symmetry_protected_bic_at_gamma() {
    let params = BicLatticeParams {
        bic_kind: BicKind::SymmetryProtectedGamma,
        asymmetry_parameter: 0.0,
        intrinsic_loss_hz: 0.2,
        resonance_freq_hz: 4000.0,
        ..Default::default()
    };
    let solver = BicLatticeSolver::new(params);

    // Singularity must be at Gamma (0, 0)
    assert_eq!(solver.bic_singularity_pos, (0.0, 0.0));

    // Topological charge q must be +1
    assert_eq!(
        solver.calculated_topological_charge, 1,
        "Symmetry-protected BIC must carry topological charge q = +1"
    );

    // Quality factor at Gamma for alpha = 0 is limited only by intrinsic loss:
    // Q = f0 / (2 * gamma_nr) = 4000 / (2 * 0.2) = 10,000
    assert!(
        solver.theoretical_q_bic >= 9900.0,
        "Theoretical Q at BIC must be >= 9900, got {:.1}",
        solver.theoretical_q_bic
    );
    assert_eq!(solver.current_q_factor, solver.theoretical_q_bic);

    // Find center point in grid: gamma_rad must be strictly zero (or < 1e-6)
    let center_pt = solver
        .polarization_field
        .iter()
        .min_by(|a, b| {
            let da = a.kx_norm * a.kx_norm + a.ky_norm * a.ky_norm;
            let db = b.kx_norm * b.kx_norm + b.ky_norm * b.ky_norm;
            da.partial_cmp(&db).unwrap()
        })
        .expect("Grid must not be empty");

    assert!(
        center_pt.radiative_linewidth_hz < 1e-4,
        "Radiative linewidth at Gamma must vanish, got {:.6} Hz",
        center_pt.radiative_linewidth_hz
    );
}

#[test]
fn test_friedrich_wintgen_off_gamma_bic() {
    let params = BicLatticeParams {
        bic_kind: BicKind::FriedrichWintgenOffGamma,
        asymmetry_parameter: 0.0,
        intrinsic_loss_hz: 0.2,
        ..Default::default()
    };
    let solver = BicLatticeSolver::new(params);

    // Singularity must be off-Gamma
    assert_eq!(solver.bic_singularity_pos, (0.35, 0.0));

    // Topological charge for Friedrich-Wintgen interference BIC is q = -1
    assert_eq!(
        solver.calculated_topological_charge, -1,
        "Friedrich-Wintgen BIC must carry topological charge q = -1"
    );
}

#[test]
fn test_higher_order_vortex_charge() {
    let params = BicLatticeParams {
        bic_kind: BicKind::HigherOrderVortex,
        asymmetry_parameter: 0.0,
        ..Default::default()
    };
    let solver = BicLatticeSolver::new(params);

    // Topological charge for higher-order rotation symmetry is q = +2
    assert_eq!(
        solver.calculated_topological_charge, 2,
        "Higher-order vortex BIC must carry topological charge q = +2"
    );
}

#[test]
fn test_quasi_bic_inverse_quadratic_scaling() {
    let mut params = BicLatticeParams {
        bic_kind: BicKind::SymmetryProtectedGamma,
        radiative_coupling_coeff_hz: 200.0,
        intrinsic_loss_hz: 0.001, // Minimal loss to observe radiative scaling clearly
        resonance_freq_hz: 4000.0,
        ..Default::default()
    };

    // Calculate Q for alpha = 0.02
    params.asymmetry_parameter = 0.02;
    let solver_small = BicLatticeSolver::new(params.clone());
    let q_small = solver_small.current_q_factor;

    // Calculate Q for alpha = 0.10
    params.asymmetry_parameter = 0.10;
    let solver_large = BicLatticeSolver::new(params.clone());
    let q_large = solver_large.current_q_factor;

    // Scaling ratio should be approx (0.10 / 0.02)^2 = 5^2 = 25
    let scaling_ratio = q_small / q_large;
    assert!(
        scaling_ratio >= 18.0 && scaling_ratio <= 27.0,
        "Quasi-BIC Q must scale inversely with alpha^2 (~25x), got {:.2}x",
        scaling_ratio
    );
}

#[test]
fn test_fano_resonance_transmission_and_vortex_profile() {
    let params = CavityVortexParams {
        lattice_params: BicLatticeParams {
            bic_kind: BicKind::SymmetryProtectedGamma,
            asymmetry_parameter: 0.05,
            resonance_freq_hz: 4000.0,
            intrinsic_loss_hz: 0.5,
            radiative_coupling_coeff_hz: 250.0,
            ..Default::default()
        },
        fano_asymmetry_q: -2.0,
        frequency_span_hz: 40.0,
        ..Default::default()
    };
    let engine = CavityVortexEngine::new(params);

    // Check peak Q factor and intra-cavity energy enhancement
    assert!(
        engine.metrics.peak_q_factor >= 1500.0,
        "Loaded Q must be >= 1500, got {:.1}",
        engine.metrics.peak_q_factor
    );
    assert!(
        engine.metrics.peak_field_enhancement >= 1000.0,
        "Cavity energy enhancement must be >= 1000x, got {:.1}x",
        engine.metrics.peak_field_enhancement
    );

    // Transmission spectrum must have sharp Fano dip
    assert!(
        engine.metrics.min_transmission_dip_db <= -10.0,
        "Fano transmission dip must be <= -10 dB, got {:.2} dB",
        engine.metrics.min_transmission_dip_db
    );

    // Vortex near field: check donut profile and central null
    let near_field = &engine.near_field_grid;
    assert!(!near_field.is_empty());

    // Center null: points near r -> 0 have small amplitude
    let core_point = near_field
        .iter()
        .min_by(|a, b| a.radius_mm.partial_cmp(&b.radius_mm).unwrap())
        .unwrap();
    let outer_point = near_field
        .iter()
        .max_by(|a, b| a.pressure_amplitude.partial_cmp(&b.pressure_amplitude).unwrap())
        .unwrap();

    assert!(
        core_point.pressure_amplitude < outer_point.pressure_amplitude * 0.4,
        "Vortex core amplitude must be suppressed relative to peak donut ring"
    );

    // OAM mode purity must be >= 95%
    assert!(
        engine.metrics.oam_mode_purity_pct >= 95.0,
        "OAM purity must be >= 95%, got {:.2}%",
        engine.metrics.oam_mode_purity_pct
    );
}
