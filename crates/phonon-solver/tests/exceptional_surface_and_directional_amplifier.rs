use phonon_models::non_hermitian_skin::{
    AcousticExceptionalSurface, ExceptionalSurfaceParams, NhseLatticeParams,
};
use phonon_solver::non_hermitian_skin::DirectionalAmplifierSolver;

#[test]
fn test_acoustic_exceptional_surface_coalescence() {
    let params = ExceptionalSurfaceParams::standard_cavity_pair();
    let surface = AcousticExceptionalSurface::new(params);

    let kappa_crit = params.critical_coupling();
    assert_eq!(kappa_crit, 50.0);

    // Exactly at critical coupling with zero detuning: on Exceptional Surface
    let state_on_es = surface.evaluate_state(0.0, kappa_crit);
    assert!(
        state_on_es.is_on_surface,
        "System must be on the Exceptional Surface at critical coupling"
    );
    assert!(
        state_on_es.eigenvalue_splitting_rad_s < 1.0,
        "Eigenvalue splitting must coalesce on ES: got {}",
        state_on_es.eigenvalue_splitting_rad_s
    );
    assert_eq!(
        state_on_es.phase_rigidity, 0.0,
        "Phase rigidity must vanish on the Exceptional Surface"
    );
    assert!(
        state_on_es.petermann_factor >= 1e4,
        "Petermann factor must diverge on the Exceptional Surface"
    );

    // Far from critical coupling: outside ES
    let state_off_es = surface.evaluate_state(20.0, 100.0);
    assert!(
        !state_off_es.is_on_surface,
        "System must be off the Exceptional Surface when detuned"
    );
    assert!(state_off_es.phase_rigidity > 0.5);
}

#[test]
fn test_topological_directional_amplifier_contrast() {
    let params = NhseLatticeParams::standard_chain();
    let solver = DirectionalAmplifierSolver::new(params);
    let res = solver.solve();

    assert!(
        res.forward_gain_db > 30.0,
        "Forward gain must exceed 30 dB: got {} dB",
        res.forward_gain_db
    );

    assert!(
        res.reverse_isolation_db > 30.0,
        "Reverse isolation must exceed 30 dB: got {} dB",
        res.reverse_isolation_db
    );

    // Directional amplification contrast must exceed 30 dB
    assert!(
        res.directional_contrast_db >= 30.0,
        "Directional contrast must exceed 30 dB: got {} dB",
        res.directional_contrast_db
    );

    // Sensor array SNR enhancement
    assert!(
        res.snr_enhancement_db > 15.0,
        "Directed sensor array SNR boost must exceed 15 dB: got {} dB",
        res.snr_enhancement_db
    );
}
