//! Integration tests for relativistic Néel vector domain wall dynamics,
//! sub-picosecond synaptic memristor plasticity, and non-reciprocal magnon diodes.

use phonon_models::afm_spintronics::{AfmMaterialParams, NeelDomainWallParams};
use phonon_solver::afm_spintronics::NeelDomainWallSolver;

#[test]
fn test_neel_domain_wall_velocity_exceeds_5000_m_s() {
    let afm = AfmMaterialParams::default();
    let dw_params = NeelDomainWallParams::new(3.0e-9, 10.0e-9, 0.30);
    let solver = NeelDomainWallSolver::new(afm, dw_params);

    // Current density 6.0e11 A/m^2
    let v_dw = solver.solve_dw_velocity(6.0e11);

    assert!(
        v_dw > 5000.0,
        "Néel domain wall velocity must exceed 5000 m/s, got {:.1} m/s",
        v_dw
    );
    assert!(
        v_dw < afm.exchange_velocity_m_s(),
        "Relativistic velocity must not exceed exchange spin-wave limit ({:.1} m/s), got {:.1} m/s",
        afm.exchange_velocity_m_s(),
        v_dw
    );
}

#[test]
fn test_sub_picosecond_synaptic_transit_time() {
    let afm = AfmMaterialParams::default();
    // 5.0 nm synaptic nanogap track
    let dw_params = NeelDomainWallParams::new(3.0e-9, 5.0e-9, 0.35);
    let solver = NeelDomainWallSolver::new(afm, dw_params);

    // Fast programming pulse at 6.0e11 A/m^2
    let tau_ps = solver.solve_transit_time_ps(6.0e11);

    assert!(
        tau_ps < 1.0,
        "Synaptic transit time across 5 nm track must be sub-picosecond (< 1.0 ps), got {:.3} ps",
        tau_ps
    );
    assert!(
        tau_ps > 0.05,
        "Transit time must remain physically non-zero, got {:.3} ps",
        tau_ps
    );
}

#[test]
fn test_synaptic_memristive_plastic_updates() {
    let afm = AfmMaterialParams::default();
    let dw_params = NeelDomainWallParams::new(3.0e-9, 10.0e-9, 0.30);
    let solver = NeelDomainWallSolver::new(afm, dw_params);

    // Initial position x = 0.20
    let pos_0 = 0.20;
    // Apply a forward 0.2 ps programming pulse at 5.0e11 A/m^2
    let (pos_1, m1) = solver.apply_programming_pulse(5.0e11, 0.2, pos_0);

    assert!(
        pos_1 > pos_0,
        "Positive current pulse must potentiate synapse: pos_0={:.3}, pos_1={:.3}",
        pos_0,
        pos_1
    );
    assert!(
        m1.conductance_s > dw_params.min_conductance_s,
        "Conductance must increase above G_min"
    );

    // Apply a reverse current pulse to depress the synapse
    let (pos_2, m2) = solver.apply_programming_pulse(-5.0e11, 0.2, pos_1);
    assert!(
        pos_2 < pos_1,
        "Negative current pulse must depress synapse: pos_1={:.3}, pos_2={:.3}",
        pos_1,
        pos_2
    );
    assert!(
        m2.conductance_s < m1.conductance_s,
        "Conductance must decrease after negative programming pulse"
    );
}

#[test]
fn test_magnon_diode_rectification_exceeds_15_db() {
    let afm = AfmMaterialParams::default();
    let dw_params = NeelDomainWallParams::default();
    let solver = NeelDomainWallSolver::new(afm, dw_params);

    let rec_db = solver.solve_magnon_diode_rectification();

    assert!(
        rec_db >= 15.0,
        "Magnon diode rectification must exceed 15.0 dB, got {:.2} dB",
        rec_db
    );
    assert!(
        rec_db < 40.0,
        "Rectification ratio must be physically plausible (< 40 dB), got {:.2} dB",
        rec_db
    );
}
