//! Circuit co-simulation test using trained Neural Surrogate models in monolithic MNA solve
//! with full Kirchhoff's Current Law (KCL) verification.

use phonon_core::CircuitGraph;
use phonon_models::surrogate::{
    ActivationFunction, DenseLayer, MultilayerPerceptron, NeuralSurrogateCompanion,
};
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};

#[test]
fn test_neural_surrogate_common_source_amplifier_kcl() {
    let mut graph = CircuitGraph::new();

    // DC Power Supply: VDD = 2.5V connected between "vdd" and "0"
    graph.add_voltage_source("VDD", "vdd", "0", 2.5).unwrap();

    // Input Bias Voltage: VIN = 1.2V connected between "vin" and "0"
    graph.add_voltage_source("VIN", "vin", "0", 1.2).unwrap();

    // Pull-up Drain Resistor: RD = 1.0 kOhm between "vdd" and "out"
    graph.add_resistor("RD", "vdd", "out", 1000.0).unwrap();

    // Neural surrogate transistor M1: drain="out", gate="vin", source="0", bulk="0"
    graph
        .add_neural_surrogate("M1", &["out", "vin", "0", "0"], "nmos_surrogate")
        .unwrap();

    // Construct a well-conditioned surrogate MLP representing an NMOS transistor
    // Input: [V_DS, V_GS, V_BS] -> Output: [I_DS]
    // Layer 1: 3 -> 4
    let w1 = vec![0.5, 1.2, 0.0, 0.3, 0.8, 0.0, 0.2, 0.5, 0.0, 0.1, 0.2, 0.0];
    let b1 = vec![-0.4, -0.3, -0.2, -0.1];
    let layer1 = DenseLayer::new(3, 4, w1, b1, ActivationFunction::SiLU);

    // Layer 2: 4 -> 1 (scaled to ~mA currents)
    let w2 = vec![1.5e-3, 1.0e-3, 0.8e-3, 0.5e-3];
    let b2 = vec![1e-5];
    let layer2 = DenseLayer::new(4, 1, w2, b2, ActivationFunction::Linear);

    let mlp = MultilayerPerceptron::new(vec![layer1, layer2]);
    let surrogate = NeuralSurrogateCompanion::new_transistor("nmos_surrogate", mlp);

    let mut context = ModelContext::new();
    context.set_neural_surrogate("nmos_surrogate", surrogate);

    let newton_opts = NewtonOptions {
        reltol: 1e-4,
        vntol: 1e-6,
        abstol: 1e-12,
        max_iters: 100,
        enable_gmin_stepping: true,
        enable_source_stepping: true,
    };

    let solution = solve_dc_non_linear(&graph, &context, &newton_opts)
        .expect("Non-linear solver with neural surrogate must converge");

    let out_node = graph.get_node("out").unwrap();
    let v_out = solution.node_voltage(out_node);
    let vdd_node = graph.get_node("vdd").unwrap();
    let v_vdd = solution.node_voltage(vdd_node);
    let vin_node = graph.get_node("vin").unwrap();
    let v_vin = solution.node_voltage(vin_node);

    // Verify sensible operating point
    assert!((v_vdd - 2.5).abs() < 1e-6, "VDD must be 2.5V");
    assert!((v_vin - 1.2).abs() < 1e-6, "VIN must be 1.2V");
    assert!(
        v_out > 0.0 && v_out < 2.5,
        "V(out) must be between 0 and VDD, got {v_out}"
    );

    // Verify Kirchhoff's Current Law across all nodes including the neural surrogate
    let empty_caps = std::collections::HashMap::new();
    let kcl_report = phonon_solver::verification::verify_kcl_dynamic(
        &graph,
        &solution.node_voltages,
        &solution.branch_currents,
        &empty_caps,
        Some(&context),
        1e-3,
        1e-6,
    );
    assert!(
        kcl_report.is_valid,
        "KCL verification failed: max_residual={}, worst_node={:?}",
        kcl_report.max_residual, kcl_report.worst_node
    );
}

#[test]
fn test_neural_surrogate_two_terminal_diode_circuit() {
    let mut graph = CircuitGraph::new();

    // DC Power Supply: VIN = 3.0V
    graph.add_voltage_source("VIN", "vin", "0", 3.0).unwrap();

    // Current-limiting Resistor: R1 = 500 Ohm
    graph.add_resistor("R1", "vin", "vd", 500.0).unwrap();

    // Neural surrogate diode D1: anode="vd", cathode="0"
    graph
        .add_neural_surrogate("D1", &["vd", "0"], "diode_surrogate")
        .unwrap();

    // Construct 1-input (V_D) -> 1-output (I_D) diode surrogate
    let w1 = vec![2.0, 3.0];
    let b1 = vec![-1.0, -1.8];
    let layer1 = DenseLayer::new(1, 2, w1, b1, ActivationFunction::SiLU);

    let w2 = vec![5e-3, 8e-3];
    let b2 = vec![1e-6];
    let layer2 = DenseLayer::new(2, 1, w2, b2, ActivationFunction::Linear);

    let mlp = MultilayerPerceptron::new(vec![layer1, layer2]);
    let surrogate = NeuralSurrogateCompanion::new_diode("diode_surrogate", mlp);

    let mut context = ModelContext::new();
    context.set_neural_surrogate("diode_surrogate", surrogate);

    let newton_opts = NewtonOptions::default();

    let solution = solve_dc_non_linear(&graph, &context, &newton_opts)
        .expect("Diode surrogate solve must converge");

    let vd_node = graph.get_node("vd").unwrap();
    let v_d = solution.node_voltage(vd_node);
    assert!(
        v_d > 0.0 && v_d < 3.0,
        "Diode junction voltage must be bounded between 0 and 3V, got {v_d}"
    );

    let empty_caps = std::collections::HashMap::new();
    let kcl_report = phonon_solver::verification::verify_kcl_dynamic(
        &graph,
        &solution.node_voltages,
        &solution.branch_currents,
        &empty_caps,
        Some(&context),
        1e-3,
        1e-6,
    );
    assert!(
        kcl_report.is_valid,
        "KCL verification failed for neural diode: max_residual={}",
        kcl_report.max_residual
    );
}
