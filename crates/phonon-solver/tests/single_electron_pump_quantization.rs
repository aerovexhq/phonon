use phonon_models::acoustoelectric::{
    DynamicQuantumDot, FlyingQubit, FlyingQubitCoupler, PiezoelectricSawParams, SplitGateChannel,
};
use phonon_solver::acoustoelectric::SingleElectronPumpSolver;

#[test]
fn test_single_electron_pump_current_quantization() {
    let saw = PiezoelectricSawParams::gaas_standard_3ghz();
    let channel = SplitGateChannel::new(-0.52, -0.80, 1.0e-6);
    let dot = DynamicQuantumDot::new(saw, channel);
    let solver = SingleElectronPumpSolver::new(dot);

    let result = solver.solve();

    let expected_i = 1.602_176_634e-19 * 3.0e9; // ~ 0.48065 nA
    let diff = (result.pumped_current_a - expected_i).abs();
    assert!(
        diff / expected_i < 1.0e-4,
        "Current must be quantized within 1e-4 precision: got {}, expected {}",
        result.pumped_current_a,
        expected_i
    );

    assert!(
        result.relative_quantization_error < 1.0e-4,
        "Relative quantization error must be < 1e-4: got {}",
        result.relative_quantization_error
    );
}

#[test]
fn test_flying_qubit_spin_transit_and_fidelity() {
    let mut qubit = FlyingQubit::new(2865.0);
    let m = 0.067 * 9.109_383_701_5e-31;

    // Propagate over 1 ns
    qubit.propagate(1.0e-9, m);

    assert!(
        qubit.position_m > 0.0,
        "Qubit position must advance with SAW velocity"
    );

    let norm = qubit.state.norm();
    assert!(
        (norm - 1.0).abs() < 1.0e-10,
        "Spinor state norm must remain strictly 1.0: got {}",
        norm
    );
}

#[test]
fn test_flying_two_qubit_exchange_entanglement() {
    let v_saw = 2865.0;
    let coupler = FlyingQubitCoupler::calibrated_sqrt_swap(1.2e-6, v_saw);

    let (concurrence, bell_fidelity) = coupler.evaluate_entanglement(v_saw);

    assert!(
        concurrence >= 0.90,
        "Flying two-qubit exchange coupler must achieve concurrence >= 0.90: got {}",
        concurrence
    );

    assert!(
        bell_fidelity >= 0.95,
        "Bell state creation fidelity must be >= 95%: got {}",
        bell_fidelity
    );
}
