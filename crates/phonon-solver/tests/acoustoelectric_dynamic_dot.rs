use phonon_models::acoustoelectric::{DynamicQuantumDot, PiezoelectricSawParams, SplitGateChannel};
use phonon_solver::acoustoelectric::TdseAcoustoelectricSolver;

#[test]
fn test_piezoelectric_dynamic_dot_confinement() {
    let saw = PiezoelectricSawParams::gaas_standard_3ghz();
    let channel = SplitGateChannel::new(-0.50, -0.80, 1.2e-6);
    let dot = DynamicQuantumDot::new(saw, channel);

    let omega_conf = dot.confinement_frequency_rad_s();
    assert!(
        omega_conf > 1.0e11,
        "Dynamic dot confinement frequency must be high (~100 GHz): got {}",
        omega_conf
    );

    let e_ground = dot.ground_state_energy_j();
    assert!(
        e_ground > 0.0,
        "Ground state energy must be strictly positive"
    );

    let charging_e = dot.charging_energy_j();
    assert!(
        charging_e > 1.0e-22,
        "Charging energy E_C must be substantial for Coulomb blockade"
    );

    let p_esc = dot.escape_probability();
    assert!(
        p_esc < 1.0e-4,
        "Escape probability must be below 1e-4: got {}",
        p_esc
    );
}

#[test]
fn test_tdse_wavepacket_propagation_and_norm_conservation() {
    let saw = PiezoelectricSawParams::gaas_standard_3ghz();
    let channel = SplitGateChannel::new(-0.50, -0.80, 1.0e-6);
    let dot = DynamicQuantumDot::new(saw, channel);
    let solver = TdseAcoustoelectricSolver::new(dot);

    let total_time = 0.5e-10; // 50 ps
    let num_steps = 100;
    let res = solver.solve(total_time, num_steps);

    assert!(
        res.max_norm_error < 1.0e-6,
        "TDSE unitary propagation must conserve norm to within 1e-6: got {}",
        res.max_norm_error
    );

    assert!(
        (res.effective_velocity_m_s - 2865.0).abs() < 1000.0,
        "Wavepacket should drift at approximately SAW sound velocity: got {}",
        res.effective_velocity_m_s
    );
}

#[test]
fn test_linbo3_hybrid_confinement() {
    let saw = PiezoelectricSawParams::linbo3_hybrid_1_5ghz();
    let channel = SplitGateChannel::new(-0.45, -0.75, 1.5e-6);
    let dot = DynamicQuantumDot::new(saw, channel);

    assert!(
        dot.saw_params.coupling_coefficient_k2 > 0.04,
        "LiNbO3 must have high electromechanical coupling K^2"
    );
    assert!(
        dot.quantization_error() < 1.0e-4,
        "LiNbO3 dynamic dot must exhibit sub-1e-4 quantization error: got {}",
        dot.quantization_error()
    );
}
