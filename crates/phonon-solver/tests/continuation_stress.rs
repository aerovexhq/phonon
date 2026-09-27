use phonon_core::CircuitGraph;
use phonon_models::bjt::BjtModel;
use phonon_models::diode::DiodeModel;
use phonon_models::mosfet::MosfetModel;
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};

#[test]
fn test_source_stepping_continuation() {
    // Highly stiff multi-diode string: 5 series diodes driven by a 10V step
    // with a tiny series resistance (10 Ohm)
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "in", "0", 10.0).unwrap();
    graph.add_resistor("R1", "in", "d1", 10.0).unwrap();
    graph.add_diode("D1", "d1", "d2").unwrap();
    graph.add_diode("D2", "d2", "d3").unwrap();
    graph.add_diode("D3", "d3", "d4").unwrap();
    graph.add_diode("D4", "d4", "d5").unwrap();
    graph.add_diode("D5", "d5", "0").unwrap();

    let mut ctx = ModelContext::new();
    let diode_model = DiodeModel::default();
    ctx.set_diode_model("D1", diode_model);
    ctx.set_diode_model("D2", diode_model);
    ctx.set_diode_model("D3", diode_model);
    ctx.set_diode_model("D4", diode_model);
    ctx.set_diode_model("D5", diode_model);

    // Solve with source stepping enabled
    let opts = NewtonOptions {
        max_iters: 50,
        enable_gmin_stepping: true,
        enable_source_stepping: true,
        ..Default::default()
    };

    let sol = solve_dc_non_linear(&graph, &ctx, &opts)
        .expect("Continuation solver must converge on stiff 5-diode string");

    let v_d1 = sol.node_voltage_by_name(&graph, "d1").unwrap();
    let v_d5 = sol.node_voltage_by_name(&graph, "d5").unwrap();

    // 5 forward-biased diodes: total drop should be around 5 * 0.7V = ~3.5V to 4.5V
    assert!(
        v_d1 > 3.0 && v_d1 < 5.0,
        "Total diode string voltage v_d1={v_d1}"
    );
    assert!(v_d5 > 0.6 && v_d5 < 1.0, "Single diode drop v_d5={v_d5}");
}

#[test]
fn test_gmin_stepping_continuation() {
    // High-gain BJT circuit with gmin stepping explicitly enabled
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("VCC", "vcc", "0", 15.0).unwrap();
    graph.add_voltage_source("VIN", "in", "0", 1.5).unwrap();
    graph.add_resistor("RB", "in", "base", 220_000.0).unwrap();
    graph.add_resistor("RC", "vcc", "coll", 2_200.0).unwrap();
    graph.add_bjt("Q1", "coll", "base", "0").unwrap();

    let mut ctx = ModelContext::new();
    ctx.set_bjt_model("Q1", BjtModel::default());

    let opts = NewtonOptions {
        max_iters: 30,
        enable_gmin_stepping: true,
        enable_source_stepping: false,
        ..Default::default()
    };

    let sol = solve_dc_non_linear(&graph, &ctx, &opts)
        .expect("Gmin stepping must converge on BJT amplifier");

    let v_coll = sol.node_voltage_by_name(&graph, "coll").unwrap();
    assert!(v_coll > 0.0 && v_coll < 15.0, "v_coll={v_coll}");
}

#[test]
fn test_non_linear_singularity_diagnosis() {
    // Circuit with a floating MOSFET terminal (bulk disconnected)
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("VDD", "vdd", "0", 1.8).unwrap();
    graph
        .add_mosfet("M1", "vdd", "vdd", "0", "floating_bulk")
        .unwrap();

    let mut ctx = ModelContext::new();
    ctx.set_mosfet_model("M1", MosfetModel::default());

    let opts = NewtonOptions {
        max_iters: 10,
        enable_gmin_stepping: false,
        enable_source_stepping: false,
        ..Default::default()
    };

    let res = solve_dc_non_linear(&graph, &ctx, &opts);
    assert!(
        res.is_err(),
        "Circuit with floating node must fail LU factorization"
    );

    if let Err(phonon_solver::SolverError::SingularMatrix {
        entity_diagnostic, ..
    }) = res
    {
        assert!(
            entity_diagnostic.contains("floating_bulk"),
            "Diagnostic must identify the offending node: {entity_diagnostic}"
        );
        assert!(
            entity_diagnostic.contains("M1"),
            "Diagnostic must identify the connected MOSFET M1: {entity_diagnostic}"
        );
    } else {
        panic!("Expected SingularMatrix error with diagnostics, got: {res:?}");
    }
}
