use phonon_models::chiral_phonon::{
    ChiralPhononMaterialParams, HeatFlowDirection, HoneycombChiralLattice, TopologicalPhononDiode,
};
use phonon_solver::chiral_phonon::{NegfPhononSolver, ThermalHallBoltzmannSolver};

#[test]
fn test_topological_thermal_diode_rectification() {
    let diode = TopologicalPhononDiode::new(250.0, 100.0, 4800.0);
    let solver = NegfPhononSolver::new(diode);

    let t_hot = 300.0;
    let t_cold = 50.0;
    let bend_angle = 90.0;

    let result = solver.solve(t_hot, t_cold, bend_angle);

    // Forward heat flux must exceed reverse heat flux
    assert!(
        result.heat_current_forward_w > result.heat_current_reverse_w,
        "Forward heat flux ({}) must exceed reverse flux ({})",
        result.heat_current_forward_w,
        result.heat_current_reverse_w
    );

    // Rectification ratio must exceed 10x
    assert!(
        result.rectification_ratio >= 10.0,
        "Rectification ratio R = {} must be >= 10x",
        result.rectification_ratio
    );

    // Verify backscattering immunity around corner bend
    assert!(
        result.corner_bend_transmission >= 0.90,
        "Corner bend transmission T_bend = {} must be >= 90%",
        result.corner_bend_transmission
    );
    assert!(
        result.corner_backscattering_prob <= 0.10,
        "Corner backscattering probability R_back = {} must be <= 10%",
        result.corner_backscattering_prob
    );
}

#[test]
fn test_thermal_hall_effect_boltzmann() {
    let params = ChiralPhononMaterialParams::fe2mo3o8_standard();
    let lattice = HoneycombChiralLattice::new(params);
    let hall_solver = ThermalHallBoltzmannSolver::new(lattice);

    let result = hall_solver.solve(150.0, 12);

    assert!(
        result.kappa_xx_w_per_m_k > 0.0,
        "Longitudinal thermal conductivity kappa_xx must be positive: {}",
        result.kappa_xx_w_per_m_k
    );
    assert!(
        result.kappa_xy_w_per_m_k.abs() > 0.0,
        "Thermal Hall conductivity kappa_xy must be non-zero due to acoustic Berry curvature: {}",
        result.kappa_xy_w_per_m_k
    );
    assert!(
        result.acoustic_chern_number == 1,
        "Acoustic Chern number must be C = 1"
    );
}

#[test]
fn test_landauer_heat_current_temperature_dependence() {
    let diode = TopologicalPhononDiode::new(200.0, 80.0, 5200.0);
    let solver = NegfPhononSolver::new(diode);

    // As delta T increases, heat current must monotonically increase
    let j_small_dt = solver.compute_heat_current(110.0, 100.0, HeatFlowDirection::Forward);
    let j_large_dt = solver.compute_heat_current(150.0, 100.0, HeatFlowDirection::Forward);

    assert!(
        j_large_dt > j_small_dt,
        "Heat flux must increase with larger Delta T: {} vs {}",
        j_large_dt,
        j_small_dt
    );
}
