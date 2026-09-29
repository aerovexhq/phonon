//! Automated unit and physical validation tests for quantum cavity acoustomechanical
//! squeezing and backaction evasion.

use phonon_models::quantum_cavity_acoustomechanics::AcoustomechanicalSqueezingParams;
use phonon_solver::quantum_cavity_acoustomechanics::QuantumCavityAcoustomechanicalSolver;

#[test]
fn test_ponderomotive_squeezing_db() {
    let params = AcoustomechanicalSqueezingParams::default();
    let solver = QuantumCavityAcoustomechanicalSolver::new(params);
    let squeezing_db = solver.compute_ponderomotive_squeezing_db();

    // Target ponderomotive squeezing >= 10.0 dB
    assert!(
        squeezing_db >= 10.0,
        "Ponderomotive squeezing must be >= 10.0 dB, got {:.2} dB",
        squeezing_db
    );
    assert!(
        squeezing_db <= 25.0,
        "Ponderomotive squeezing exceeds unphysical limits, got {:.2} dB",
        squeezing_db
    );
}

#[test]
fn test_qnd_measurement_fidelity() {
    let params = AcoustomechanicalSqueezingParams::default();
    let solver = QuantumCavityAcoustomechanicalSolver::new(params);
    let fidelity = solver.compute_qnd_measurement_fidelity();

    // Target QND fidelity >= 98.0%
    assert!(
        fidelity >= 0.980,
        "QND measurement fidelity must be >= 98.0%, got {:.4}%",
        fidelity * 100.0
    );
    assert!(
        fidelity <= 1.0,
        "QND fidelity cannot exceed unity, got {:.6}",
        fidelity
    );
}

#[test]
fn test_mechanical_decoherence_rate_hz() {
    let params = AcoustomechanicalSqueezingParams::default();
    let solver = QuantumCavityAcoustomechanicalSolver::new(params);
    let rate_hz = solver.compute_mechanical_decoherence_rate_hz();

    // Target mechanical decoherence rate <= 10.0 Hz
    assert!(
        rate_hz <= 10.0,
        "Mechanical decoherence rate must be <= 10.0 Hz, got {:.3} Hz",
        rate_hz
    );
    assert!(
        rate_hz > 0.0,
        "Decoherence rate must be positive, got {:.3} Hz",
        rate_hz
    );
}

#[test]
fn test_intracavity_photon_number() {
    let params = AcoustomechanicalSqueezingParams::default();
    let solver = QuantumCavityAcoustomechanicalSolver::new(params);
    let n_c = solver.compute_intracavity_photon_number();

    // Target intracavity photon number >= 5.0e5
    assert!(
        n_c >= 5.0e5,
        "Intracavity photon number must be >= 5.0e5, got {:.2e}",
        n_c
    );
}

#[test]
fn test_backaction_evasion_purity() {
    let params = AcoustomechanicalSqueezingParams::default();
    let solver = QuantumCavityAcoustomechanicalSolver::new(params);
    let purity = solver.compute_backaction_evasion_purity();

    // Target backaction evasion purity >= 95.0%
    assert!(
        purity >= 0.950,
        "Backaction evasion purity must be >= 95.0%, got {:.4}%",
        purity * 100.0
    );
    assert!(
        purity <= 1.0,
        "BAE purity cannot exceed unity, got {:.6}",
        purity
    );
}

#[test]
fn test_full_acoustomechanical_physical_compliance() {
    let params = AcoustomechanicalSqueezingParams::default();
    let solver = QuantumCavityAcoustomechanicalSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default acoustomechanical squeezing system must satisfy all physical compliance thresholds: {:?}",
        metrics
    );
    assert!(metrics.ponderomotive_squeezing_db >= 10.0);
    assert!(metrics.qnd_measurement_fidelity >= 0.980);
    assert!(metrics.mechanical_decoherence_rate_hz <= 10.0);
    assert!(metrics.intracavity_photon_number >= 5.0e5);
    assert!(metrics.backaction_evasion_purity >= 0.950);
}
