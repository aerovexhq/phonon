//! Integration Tests for High-Harmonic Generation (HHG) & Carrier Dynamics.

use approx::assert_relative_eq;
use phonon_models::floquet::{
    FloquetGrapheneLattice, FloquetLaserPulse, HhgCutoffModel, SemiconductorBlochParams,
};
use phonon_solver::floquet::HhgSpectraSolver;

#[test]
fn test_hhg_cutoff_model_and_ponderomotive_scaling() {
    let pulse = FloquetLaserPulse {
        wavelength_m: 3.2e-6,
        peak_electric_field_v_per_m: 4.0e8,
        pulse_duration_seconds: 60.0e-15,
    };

    let model = HhgCutoffModel::default();
    let up_ev = model.ponderomotive_energy_ev(&pulse);
    assert!(up_ev > 0.35, "Up should be > 0.35 eV, got {} eV", up_ev);

    let bandgap = 0.3; // 300 meV
    let e_cut = model.cutoff_energy_ev(bandgap, &pulse);
    assert_relative_eq!(e_cut, bandgap + 3.17 * up_ev, epsilon = 1e-9);

    let cutoff_order = model.cutoff_harmonic_order(bandgap, &pulse);
    assert!(
        cutoff_order >= 4,
        "Cutoff order should be >= 4, got {}",
        cutoff_order
    );
}

#[test]
fn test_sbe_hhg_spectrum_and_harmonics() {
    let lattice = FloquetGrapheneLattice::default();
    let pulse = FloquetLaserPulse {
        wavelength_m: 3.2e-6,
        peak_electric_field_v_per_m: 3.5e8,
        pulse_duration_seconds: 50.0e-15,
    };
    let sbe_params = SemiconductorBlochParams::default();

    let solver = HhgSpectraSolver::new();
    let res = solver.solve(&lattice, &pulse, &sbe_params, 4, 100);

    assert!(!res.harmonic_orders.is_empty());
    assert_eq!(res.harmonic_orders.len(), res.power_db.len());

    // Fundamental harmonic (H1) should have high power (normalized to ~0 dB)
    let powers = res.harmonic_powers_db(&[1, 3, 5, 7]);
    assert_eq!(powers.len(), 4);
    assert_relative_eq!(powers[0].1, 0.0, epsilon = 3.0); // H1 near 0 dB

    // Non-perturbative high harmonics should be present
    assert!(
        res.detected_cutoff_order >= 5,
        "Cutoff order: {}",
        res.detected_cutoff_order
    );
}
