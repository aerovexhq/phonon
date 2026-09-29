//! Integration tests for chiral quantum valley acoustic cavities,
//! valley polarization contrast, and waveguide multiplexer routing.

use phonon_models::valley_acoustic::{
    StrainGaugeParams, ValleyAcousticRouter, ValleyCavityParams, ValleyIndex,
};
use phonon_solver::valley_acoustic::{ValleyCavitySolver, ValleyRouterSolver};

#[test]
fn test_valley_cavity_confinement_and_purcell_enhancement() {
    let gauge = StrainGaugeParams::new(1.0e-9, 3000.0, 3.5, 8.0e6);
    let cavity = ValleyCavityParams::new(50.0e-9, 20.0e-9, 10_000.0, gauge);
    let solver = ValleyCavitySolver::new(cavity);

    let mode_k = solver.solve_mode(ValleyIndex::ValleyK);
    let mode_kp = solver.solve_mode(ValleyIndex::ValleyKPrime);

    assert!(
        mode_k.valley_contrast_db >= 20.0,
        "Valley polarization contrast must be >= 20 dB, got {:.2} dB",
        mode_k.valley_contrast_db
    );
    assert!(
        mode_k.purcell_factor > 10.0,
        "Resonant Purcell factor must exceed 10.0, got {:.2}",
        mode_k.purcell_factor
    );
    assert!(
        mode_k.purcell_factor / mode_kp.purcell_factor > 50.0,
        "Purcell enhancement contrast must exceed 50x"
    );
}

#[test]
fn test_chiral_valley_waveguide_multiplexing_and_corner_transmission() {
    let router = ValleyAcousticRouter::new(0.95, 60.0, 25.0);
    let solver = ValleyRouterSolver::new(router);

    let res_k = solver.solve_routing(ValleyIndex::ValleyK);
    assert!(
        res_k.port_k_transmission >= 0.90,
        "Port K transmission must be >= 0.90, got {:.4}",
        res_k.port_k_transmission
    );
    assert!(
        res_k.port_kprime_transmission < 0.01,
        "Port K' cross-talk transmission must be < 0.01"
    );
    assert!(
        res_k.isolation_contrast_db >= 20.0,
        "Isolation contrast must be >= 20 dB, got {:.2} dB",
        res_k.isolation_contrast_db
    );

    let res_kp = solver.solve_routing(ValleyIndex::ValleyKPrime);
    assert!(
        res_kp.port_kprime_transmission >= 0.90,
        "Port K' transmission must be >= 0.90, got {:.4}",
        res_kp.port_kprime_transmission
    );
    assert!(
        res_kp.port_k_transmission < 0.01,
        "Port K cross-talk transmission must be < 0.01"
    );

    // Verify sharp 120-degree bend transmission immunity
    let router_120 = ValleyAcousticRouter::new(0.95, 120.0, 25.0);
    let solver_120 = ValleyRouterSolver::new(router_120);
    let corner_t = solver_120.solve_corner_transmission();
    assert!(
        corner_t >= 0.90,
        "Corner bend transmission at 120 deg must be >= 90%, got {:.4}",
        corner_t
    );
}
