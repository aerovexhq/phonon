use phonon_core::CircuitGraph;
use phonon_models::diode::DiodeModel;
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};

#[test]
fn test_diode_forward_bias_sweep() {
    // Circuit:
    // Vin ("in", "0") -> R1 ("in", "anode", 1k) -> D1 ("anode", "0")
    let temp_k = 300.0;
    let r_val = 1_000.0; // 1 kOhm
    let is = 1e-14; // 10 fA
    let n = 1.0;

    let diode_model = DiodeModel {
        is,
        n,
        ..Default::default()
    };

    // Sweep Vin from 0.2V to 2.0V
    for v_in_step in [0.2, 0.5, 0.7, 1.0, 1.5, 2.0] {
        let mut graph = CircuitGraph::new();
        graph
            .add_voltage_source("V1", "in", "0", v_in_step)
            .unwrap();
        graph.add_resistor("R1", "in", "anode", r_val).unwrap();
        graph.add_diode("D1", "anode", "0").unwrap();

        let mut ctx = ModelContext::new();
        ctx.temperature_kelvin = temp_k;
        ctx.set_diode_model("D1", diode_model);

        let opts = NewtonOptions::default();
        let sol = solve_dc_non_linear(&graph, &ctx, &opts).expect("Non-linear solve must converge");

        let v_anode = sol.node_voltage_by_name(&graph, "anode").unwrap();
        let current_resistor = (v_in_step - v_anode) / r_val;
        let current_diode = diode_model.evaluate(v_anode, temp_k).i_d;

        // Verify KCL: current through resistor must equal diode current within solver tolerances
        let diff = (current_resistor - current_diode).abs();
        assert!(
            diff < 1e-6 || diff / current_resistor < 1e-3,
            "At Vin={v_in_step}: V_anode={v_anode}, I_R={current_resistor}, I_D={current_diode}"
        );

        // Anode voltage must stay within realistic silicon diode range [0.1V, 0.9V]
        assert!(v_anode > 0.1 && v_anode < 0.9);
    }
}

#[test]
fn test_diode_reverse_bias() {
    // Reverse bias: Vin = 5V, Diode cathode at "cath", anode at "0"
    // Vin ("in", "0") = 5V -> R1 ("in", "cath", 1k) -> D1 ("0", "cath")
    let temp_k = 300.0;
    let r_val = 1_000.0;
    let is = 1e-14;

    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "in", "0", 5.0).unwrap();
    graph.add_resistor("R1", "in", "cath", r_val).unwrap();
    // Diode anode = "0", cathode = "cath" (reverse biased)
    graph.add_diode("D1", "0", "cath").unwrap();

    let mut ctx = ModelContext::new();
    ctx.temperature_kelvin = temp_k;
    let diode_model = DiodeModel {
        is,
        bv: 100.0,
        ..Default::default()
    };
    ctx.set_diode_model("D1", diode_model);

    let opts = NewtonOptions::default();
    let sol = solve_dc_non_linear(&graph, &ctx, &opts).expect("Reverse bias solve must converge");

    let v_cath = sol.node_voltage_by_name(&graph, "cath").unwrap();
    // Reverse current is negligible (~1e-14 A), so drop across 1k resistor is ~0, V_cath ~ 5.0V
    assert!((v_cath - 5.0).abs() < 1e-5);
}
