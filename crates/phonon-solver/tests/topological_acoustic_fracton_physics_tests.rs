#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for topological acoustic
//! fracton dynamics and sub-system symmetry-protected phononic multipole routers.

use phonon_models::topological_acoustic_fracton::TopologicalAcousticFractonParams;
use phonon_solver::topological_acoustic_fracton::TopologicalAcousticFractonSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = TopologicalAcousticFractonParams::new(
        0.02,   // below 0.10
        25.0,   // below 50.0 nm
        0.5,    // below 1.0 GHz
        0.5,    // below 1.0
        0.2,    // below 1.0 mK
        2.0,    // below 4.0 layers
        0.1,    // below 0.5 meV
        0.2,    // below 0.5 um
    );
    assert_eq!(underflow.higher_rank_gauge_coupling_g, 0.10);
    assert_eq!(underflow.sub_dimensional_lattice_constant_nm, 50.0);
    assert_eq!(underflow.acoustic_phonon_frequency_ghz, 1.0);
    assert_eq!(underflow.multipole_moment_order, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.sub_system_layer_count, 4.0);
    assert_eq!(underflow.fracton_pinning_potential_mev, 0.5);
    assert_eq!(underflow.inter_router_separation_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = TopologicalAcousticFractonParams::new(
        8.0,    // above 5.0
        800.0,  // above 500.0 nm
        22.0,   // above 15.0 GHz
        6.0,    // above 4.0
        120.0,  // above 50.0 mK
        90.0,   // above 64.0 layers
        35.0,   // above 20.0 meV
        18.0,   // above 12.0 um
    );
    assert_eq!(overflow.higher_rank_gauge_coupling_g, 5.0);
    assert_eq!(overflow.sub_dimensional_lattice_constant_nm, 500.0);
    assert_eq!(overflow.acoustic_phonon_frequency_ghz, 15.0);
    assert_eq!(overflow.multipole_moment_order, 4.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.sub_system_layer_count, 64.0);
    assert_eq!(overflow.fracton_pinning_potential_mev, 20.0);
    assert_eq!(overflow.inter_router_separation_um, 12.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = TopologicalAcousticFractonParams::default();
    let solver = TopologicalAcousticFractonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.fracton_confinement_fidelity >= 0.9970,
        "Fracton confinement fidelity must be >= 0.9970, got {:.6}",
        metrics.fracton_confinement_fidelity
    );
    assert!(
        metrics.sub_dimensional_edge_channel_isolation_db >= 50.0,
        "Sub-dimensional edge channel isolation must be >= 50.0 dB, got {:.2} dB",
        metrics.sub_dimensional_edge_channel_isolation_db
    );
    assert!(
        metrics.multipole_charge_conservation_error <= 1.0e-5,
        "Multipole charge conservation error must be <= 1.0e-5, got {:.4e}",
        metrics.multipole_charge_conservation_error
    );
    assert!(
        metrics.fracton_diffusion_dephasing_rate_hz <= 25.0,
        "Fracton diffusion dephasing rate must be <= 25.0 Hz, got {:.2} Hz",
        metrics.fracton_diffusion_dephasing_rate_hz
    );
    assert!(
        metrics.sub_system_boundary_mode_purity >= 0.990,
        "Sub-system boundary mode purity must be >= 0.990, got {:.6}",
        metrics.sub_system_boundary_mode_purity
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_gauge_coupling_scaling() {
    let low_g = TopologicalAcousticFractonParams {
        higher_rank_gauge_coupling_g: 0.20,
        ..TopologicalAcousticFractonParams::default()
    };
    let high_g = TopologicalAcousticFractonParams {
        higher_rank_gauge_coupling_g: 4.50,
        ..TopologicalAcousticFractonParams::default()
    };

    let solver_low = TopologicalAcousticFractonSolver::new(low_g);
    let solver_high = TopologicalAcousticFractonSolver::new(high_g);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.fracton_confinement_fidelity > m_low.fracton_confinement_fidelity,
        "Higher gauge coupling must enhance fracton confinement fidelity"
    );
    assert!(
        m_high.sub_dimensional_edge_channel_isolation_db > m_low.sub_dimensional_edge_channel_isolation_db,
        "Higher gauge coupling must improve channel isolation"
    );
    assert!(
        m_high.multipole_charge_conservation_error < m_low.multipole_charge_conservation_error,
        "Higher gauge coupling must reduce multipole conservation error"
    );
    assert!(
        m_high.fracton_diffusion_dephasing_rate_hz < m_low.fracton_diffusion_dephasing_rate_hz,
        "Higher gauge coupling must suppress diffusion dephasing rate"
    );
    assert!(
        m_high.sub_system_boundary_mode_purity > m_low.sub_system_boundary_mode_purity,
        "Higher gauge coupling must improve boundary mode purity"
    );
}

#[test]
fn test_sub_dimensional_lattice_constant_scaling() {
    let compact_lattice = TopologicalAcousticFractonParams {
        sub_dimensional_lattice_constant_nm: 75.0,
        ..TopologicalAcousticFractonParams::default()
    };
    let sparse_lattice = TopologicalAcousticFractonParams {
        sub_dimensional_lattice_constant_nm: 450.0,
        ..TopologicalAcousticFractonParams::default()
    };

    let solver_compact = TopologicalAcousticFractonSolver::new(compact_lattice);
    let solver_sparse = TopologicalAcousticFractonSolver::new(sparse_lattice);

    let m_compact = solver_compact.evaluate_metrics();
    let m_sparse = solver_sparse.evaluate_metrics();

    assert!(
        m_compact.sub_dimensional_edge_channel_isolation_db > m_sparse.sub_dimensional_edge_channel_isolation_db,
        "Compact lattice constant must yield higher sub-dimensional channel isolation"
    );
    assert!(
        m_compact.fracton_confinement_fidelity > m_sparse.fracton_confinement_fidelity,
        "Compact lattice constant must yield higher confinement fidelity"
    );
    assert!(
        m_compact.multipole_charge_conservation_error < m_sparse.multipole_charge_conservation_error,
        "Compact lattice constant must reduce multipole conservation error"
    );
    assert!(
        m_compact.fracton_diffusion_dephasing_rate_hz < m_sparse.fracton_diffusion_dephasing_rate_hz,
        "Compact lattice constant must suppress diffusion dephasing"
    );
}

#[test]
fn test_frequency_scaling() {
    let low_freq = TopologicalAcousticFractonParams {
        acoustic_phonon_frequency_ghz: 1.5,
        ..TopologicalAcousticFractonParams::default()
    };
    let high_freq = TopologicalAcousticFractonParams {
        acoustic_phonon_frequency_ghz: 12.0,
        ..TopologicalAcousticFractonParams::default()
    };

    let solver_low = TopologicalAcousticFractonSolver::new(low_freq);
    let solver_high = TopologicalAcousticFractonSolver::new(high_freq);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.sub_dimensional_edge_channel_isolation_db > m_low.sub_dimensional_edge_channel_isolation_db,
        "Higher acoustic frequency must enhance edge channel isolation"
    );
    assert!(
        m_high.sub_system_boundary_mode_purity > m_low.sub_system_boundary_mode_purity,
        "Higher acoustic frequency must enhance boundary mode purity"
    );
    assert!(
        m_high.fracton_diffusion_dephasing_rate_hz < m_low.fracton_diffusion_dephasing_rate_hz,
        "Higher acoustic frequency must reduce dephasing rate"
    );
}

#[test]
fn test_multipole_moment_order_scaling() {
    let dipole = TopologicalAcousticFractonParams {
        multipole_moment_order: 1.0,
        ..TopologicalAcousticFractonParams::default()
    };
    let quadrupole = TopologicalAcousticFractonParams {
        multipole_moment_order: 2.0,
        ..TopologicalAcousticFractonParams::default()
    };
    let hexadecapole = TopologicalAcousticFractonParams {
        multipole_moment_order: 4.0,
        ..TopologicalAcousticFractonParams::default()
    };

    let solver_dipole = TopologicalAcousticFractonSolver::new(dipole);
    let solver_quad = TopologicalAcousticFractonSolver::new(quadrupole);
    let solver_hex = TopologicalAcousticFractonSolver::new(hexadecapole);

    let m_dipole = solver_dipole.evaluate_metrics();
    let m_quad = solver_quad.evaluate_metrics();
    let m_hex = solver_hex.evaluate_metrics();

    assert!(
        m_quad.multipole_charge_conservation_error < m_dipole.multipole_charge_conservation_error,
        "Quadrupole order must suppress multipole charge conservation error over dipole"
    );
    assert!(
        m_hex.multipole_charge_conservation_error < m_quad.multipole_charge_conservation_error,
        "Higher-order multipole constraints must further suppress conservation error"
    );
    assert!(
        m_hex.fracton_confinement_fidelity > m_dipole.fracton_confinement_fidelity,
        "Higher-order multipoles must enhance fracton confinement fidelity"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let ultra_cryo = TopologicalAcousticFractonParams {
        cryogenic_temperature_mk: 2.0,
        ..TopologicalAcousticFractonParams::default()
    };
    let warm_cryo = TopologicalAcousticFractonParams {
        cryogenic_temperature_mk: 45.0,
        ..TopologicalAcousticFractonParams::default()
    };

    let solver_ultra = TopologicalAcousticFractonSolver::new(ultra_cryo);
    let solver_warm = TopologicalAcousticFractonSolver::new(warm_cryo);

    let m_ultra = solver_ultra.evaluate_metrics();
    let m_warm = solver_warm.evaluate_metrics();

    assert!(
        m_ultra.fracton_diffusion_dephasing_rate_hz < m_warm.fracton_diffusion_dephasing_rate_hz,
        "Lower cryogenic temperature must suppress thermal dephasing rate"
    );
    assert!(
        m_ultra.fracton_confinement_fidelity > m_warm.fracton_confinement_fidelity,
        "Lower temperature must enhance fracton confinement fidelity"
    );
    assert!(
        m_ultra.multipole_charge_conservation_error < m_warm.multipole_charge_conservation_error,
        "Lower temperature must minimize multipole charge conservation error"
    );
    assert!(
        m_ultra.sub_system_boundary_mode_purity > m_warm.sub_system_boundary_mode_purity,
        "Lower temperature must improve boundary mode purity"
    );
}

#[test]
fn test_sub_system_layer_count_scaling() {
    let thin_stack = TopologicalAcousticFractonParams {
        sub_system_layer_count: 6.0,
        ..TopologicalAcousticFractonParams::default()
    };
    let thick_stack = TopologicalAcousticFractonParams {
        sub_system_layer_count: 50.0,
        ..TopologicalAcousticFractonParams::default()
    };

    let solver_thin = TopologicalAcousticFractonSolver::new(thin_stack);
    let solver_thick = TopologicalAcousticFractonSolver::new(thick_stack);

    let m_thin = solver_thin.evaluate_metrics();
    let m_thick = solver_thick.evaluate_metrics();

    assert!(
        m_thick.sub_system_boundary_mode_purity > m_thin.sub_system_boundary_mode_purity,
        "Higher layer count must increase sub-system boundary mode purity"
    );
    assert!(
        m_thick.sub_dimensional_edge_channel_isolation_db > m_thin.sub_dimensional_edge_channel_isolation_db,
        "Higher layer count must enhance edge channel isolation"
    );
    assert!(
        m_thick.fracton_confinement_fidelity > m_thin.fracton_confinement_fidelity,
        "Higher layer count must enhance fracton confinement fidelity"
    );
}

#[test]
fn test_fracton_pinning_potential_scaling() {
    let weak_pinning = TopologicalAcousticFractonParams {
        fracton_pinning_potential_mev: 1.0,
        ..TopologicalAcousticFractonParams::default()
    };
    let strong_pinning = TopologicalAcousticFractonParams {
        fracton_pinning_potential_mev: 18.0,
        ..TopologicalAcousticFractonParams::default()
    };

    let solver_weak = TopologicalAcousticFractonSolver::new(weak_pinning);
    let solver_strong = TopologicalAcousticFractonSolver::new(strong_pinning);

    let m_weak = solver_weak.evaluate_metrics();
    let m_strong = solver_strong.evaluate_metrics();

    assert!(
        m_strong.fracton_confinement_fidelity > m_weak.fracton_confinement_fidelity,
        "Strong pinning potential must significantly enhance confinement fidelity"
    );
    assert!(
        m_strong.fracton_diffusion_dephasing_rate_hz < m_weak.fracton_diffusion_dephasing_rate_hz,
        "Strong pinning potential must suppress diffusion dephasing"
    );
    assert!(
        m_strong.multipole_charge_conservation_error < m_weak.multipole_charge_conservation_error,
        "Strong pinning potential must reduce charge conservation violation"
    );
}

#[test]
fn test_inter_router_separation_scaling() {
    let close_routers = TopologicalAcousticFractonParams {
        inter_router_separation_um: 1.0,
        ..TopologicalAcousticFractonParams::default()
    };
    let distant_routers = TopologicalAcousticFractonParams {
        inter_router_separation_um: 10.0,
        ..TopologicalAcousticFractonParams::default()
    };

    let solver_close = TopologicalAcousticFractonSolver::new(close_routers);
    let solver_distant = TopologicalAcousticFractonSolver::new(distant_routers);

    let m_close = solver_close.evaluate_metrics();
    let m_distant = solver_distant.evaluate_metrics();

    assert!(
        m_distant.sub_dimensional_edge_channel_isolation_db > m_close.sub_dimensional_edge_channel_isolation_db,
        "Greater inter-router separation must enhance sub-dimensional channel isolation"
    );
    assert!(
        m_distant.multipole_charge_conservation_error < m_close.multipole_charge_conservation_error,
        "Greater separation must reduce cross-talk induced conservation error"
    );
}
