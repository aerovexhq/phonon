use phonon_core::CircuitGraph;
use phonon_models::bjt::BjtModel;
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};

#[test]
fn test_bjt_common_emitter_operating_point() {
    let mut graph = CircuitGraph::new();

    // VCC = 10 V, VBB = 2.0 V
    graph.add_voltage_source("VCC", "vcc", "0", 10.0).unwrap();
    graph.add_voltage_source("VBB", "vbb", "0", 2.0).unwrap();

    // Base bias resistor RB = 100 kOhm
    graph.add_resistor("RB", "vbb", "base", 100_000.0).unwrap();

    // Collector load resistor RC = 1 kOhm
    graph.add_resistor("RC", "vcc", "coll", 1_000.0).unwrap();

    // NPN BJT: collector="coll", base="base", emitter="0"
    graph.add_bjt("Q1", "coll", "base", "0").unwrap();

    let mut ctx = ModelContext::new();
    let bjt_model = BjtModel::default(); // NPN, bf=100, is=1e-16
    ctx.set_bjt_model("Q1", bjt_model);

    let opts = NewtonOptions::default();
    let sol =
        solve_dc_non_linear(&graph, &ctx, &opts).expect("BJT common emitter solve must converge");

    let v_base = sol.node_voltage_by_name(&graph, "base").unwrap();
    let v_coll = sol.node_voltage_by_name(&graph, "coll").unwrap();

    // Verify V_BE is typical forward-biased silicon base-emitter diode (0.65V to 0.85V)
    assert!(
        v_base > 0.65 && v_base < 0.85,
        "V_BE must be around 0.7V, got {v_base}"
    );

    // Compute currents
    let i_b = (2.0 - v_base) / 100_000.0;
    let i_c = (10.0 - v_coll) / 1_000.0;

    assert!(
        i_b > 1e-6 && i_b < 2e-5,
        "I_B must be around 13 uA, got {i_b}"
    );
    assert!(
        i_c > 5e-4 && i_c < 2e-3,
        "I_C must be around 1.3 mA, got {i_c}"
    );

    // Current gain beta = I_C / I_B should be close to model bf (100.0) accounting for Early effect
    let beta_eff = i_c / i_b;
    assert!(
        (beta_eff - 100.0).abs() < 15.0,
        "Effective beta must be close to 100, got {beta_eff}"
    );

    // Transistor must be in Forward-Active mode: V_CE > V_BE
    assert!(
        v_coll > v_base,
        "V_CE ({v_coll}) must be greater than V_BE ({v_base}) for forward active mode"
    );
}

#[test]
fn test_bjt_saturation_mode() {
    // Drive base with high voltage (5V) and low base resistor (10k)
    // with high collector resistor (10k) so it saturates
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("VCC", "vcc", "0", 5.0).unwrap();
    graph.add_voltage_source("VBB", "vbb", "0", 5.0).unwrap();
    graph.add_resistor("RB", "vbb", "base", 10_000.0).unwrap();
    graph.add_resistor("RC", "vcc", "coll", 10_000.0).unwrap();
    graph.add_bjt("Q1", "coll", "base", "0").unwrap();

    let mut ctx = ModelContext::new();
    ctx.set_bjt_model("Q1", BjtModel::default());

    let opts = NewtonOptions::default();
    let sol = solve_dc_non_linear(&graph, &ctx, &opts).expect("BJT saturation solve must converge");

    let v_coll = sol.node_voltage_by_name(&graph, "coll").unwrap();
    let v_base = sol.node_voltage_by_name(&graph, "base").unwrap();

    // In saturation, V_CE drops to V_CE(sat) ~ 0.05V to 0.3V
    assert!(
        v_coll < 0.4,
        "V_CE in saturation must be low (< 0.4V), got {v_coll}"
    );
    assert!(
        v_coll < v_base,
        "V_CE ({v_coll}) must be less than V_BE ({v_base}) in saturation"
    );
}
