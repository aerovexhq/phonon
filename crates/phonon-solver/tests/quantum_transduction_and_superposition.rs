//! Integration tests for coherent microwave-to-phonon quantum transducers,
//! non-classical mechanical quadrature squeezing, and macroscopic Schrödinger cat state fidelity.

use phonon_models::cavity_magnomechanics::CavityMagnomechanicalParams;
use phonon_solver::cavity_magnomechanics::QuantumTransducerSolver;

#[test]
fn test_quantum_transducer_and_quadrature_squeezing() {
    let params = CavityMagnomechanicalParams {
        cavity_frequency_ghz: 9.0,
        magnon_frequency_ghz: 9.0,
        phonon_frequency_mhz: 20.0,
        photon_damping_mhz: 2.5,
        magnon_damping_mhz: 1.5,
        phonon_damping_hz: 100.0,
        photon_magnon_coupling_mhz: 25.0,
        single_spin_magnetostriction_hz: 0.5,
        coherent_magnon_number: 1.5e11,
        bath_temperature_mk: 20.0,
    };
    let solver = QuantumTransducerSolver::new(params);

    // Mechanical quadrature squeezing must be >= 3.0 dB
    let sq_db = solver.solve_quadrature_squeezing_db();
    assert!(
        sq_db >= 3.0,
        "Quadrature squeezing {:.2} dB must be >= 3.0 dB",
        sq_db
    );

    // Macroscopic cat state Wigner negative volume must be strictly positive
    let w_neg = solver.solve_wigner_negative_volume(1.5);
    assert!(
        w_neg > 0.0,
        "Wigner negative volume {:.4} must be > 0.0",
        w_neg
    );
    assert!(
        w_neg <= 1.0,
        "Wigner negative volume {:.4} should be <= 1.0",
        w_neg
    );

    // Transduction bandwidth should be dynamically broadened into kHz range
    let bw_khz = solver.solve_transduction_bandwidth_khz();
    assert!(
        bw_khz > 1.0,
        "Transduction bandwidth {:.2} kHz should be > 1.0 kHz",
        bw_khz
    );
}
