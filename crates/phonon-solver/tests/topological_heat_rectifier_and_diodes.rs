//! Integration tests for topological acoustic heat rectifiers,
//! thermal diodes, and directional phononic waveguides.

use phonon_models::chiral_spin_seebeck::{
    ChiralSpinSeebeckParams, TopologicalThermalRectifierParams,
};
use phonon_solver::chiral_spin_seebeck::{ChiralSpinSeebeckSolver, TopologicalHeatRectifierSolver};

#[test]
fn test_topological_thermal_rectification_ratio() {
    let params = ChiralSpinSeebeckParams::default();
    let solver = ChiralSpinSeebeckSolver::new(params);
    let metrics = solver.solve();

    // Rectification ratio must satisfy >= 10.0x
    assert!(
        metrics.thermal_rectification_ratio >= 10.0,
        "Thermal rectification ratio must be >= 10.0x, got {:.2}x",
        metrics.thermal_rectification_ratio
    );

    // Directional heat flow comparison
    assert!(
        metrics.forward_heat_current_uw >= metrics.backward_heat_current_uw * 10.0,
        "Forward heat ({:.3} uW) must be at least 10x backward heat ({:.3} uW)",
        metrics.forward_heat_current_uw,
        metrics.backward_heat_current_uw
    );
}

#[test]
fn test_cascaded_topological_thermal_diode() {
    let params = TopologicalThermalRectifierParams {
        diode_stages_count: 6,
        forward_phonon_transmission: 0.90,
        backward_phonon_transmission: 0.12,
        ..Default::default()
    };
    let solver = TopologicalHeatRectifierSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.rectification_ratio >= 100.0,
        "6-stage thermal diode rectification ratio must be >= 100.0x, got {:.2}x",
        metrics.rectification_ratio
    );
    assert!(
        metrics.thermal_contrast_db >= 20.0,
        "Thermal contrast must be >= 20.0 dB, got {:.2} dB",
        metrics.thermal_contrast_db
    );
    assert!(
        metrics.reverse_isolation_db >= 20.0,
        "Reverse isolation must be >= 20.0 dB, got {:.2} dB",
        metrics.reverse_isolation_db
    );
}
