//! Integration tests for skyrmionic logic gates, acoustic circulators,
//! and energy dissipation per bit shift.

use phonon_models::skyrmion_phonon_drag::SkyrmionPhononParams;
use phonon_solver::skyrmion_phonon_drag::SkyrmionPhononSolver;

#[test]
fn test_logic_and_circulator_bounds() {
    let params = SkyrmionPhononParams::default();
    let solver = SkyrmionPhononSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.logic_switching_contrast_db >= 25.0,
        "Logic switching contrast {} dB must be >= 25.0 dB",
        metrics.logic_switching_contrast_db
    );
    assert!(
        metrics.circulator_isolation_db >= 20.0,
        "Circulator isolation {} dB must be >= 20.0 dB",
        metrics.circulator_isolation_db
    );
    assert!(
        metrics.energy_dissipation_per_bit_fj < 1.0,
        "Energy per bit {} fJ must be strictly sub-femtojoule (< 1.0 fJ)",
        metrics.energy_dissipation_per_bit_fj
    );
    assert!(
        metrics.energy_dissipation_per_bit_fj > 0.0,
        "Energy per bit must be positive"
    );
}

#[test]
fn test_frequency_circulator_isolation_scaling() {
    let p_low_freq = SkyrmionPhononParams {
        saw_frequency_ghz: 1.5,
        ..Default::default()
    };
    let p_high_freq = SkyrmionPhononParams {
        saw_frequency_ghz: 4.0,
        ..Default::default()
    };

    let solver_low = SkyrmionPhononSolver::new(p_low_freq);
    let solver_high = SkyrmionPhononSolver::new(p_high_freq);

    let iso_low = solver_low.compute_circulator_isolation_db();
    let iso_high = solver_high.compute_circulator_isolation_db();

    assert!(
        iso_high > iso_low,
        "Higher SAW frequency should enhance circulator isolation (high={} vs low={})",
        iso_high,
        iso_low
    );
    assert!(iso_low >= 20.0);
    assert!(iso_high >= 20.0);
}

#[test]
fn test_thermal_stability_temperature_scaling() {
    let p_cryo = SkyrmionPhononParams {
        temperature_k: 20.0,
        ..Default::default()
    };
    let p_room = SkyrmionPhononParams {
        temperature_k: 300.0,
        ..Default::default()
    };

    let solver_cryo = SkyrmionPhononSolver::new(p_cryo);
    let solver_room = SkyrmionPhononSolver::new(p_room);

    let stab_cryo = solver_cryo.compute_topological_stability_factor();
    let stab_room = solver_room.compute_topological_stability_factor();

    assert!(
        stab_cryo > stab_room,
        "Cryogenic temperature must yield higher thermal stability factor"
    );
    assert!(stab_room >= 1.0);
}
