use phonon_netlist::*;
use phonon_solver::mna::{solve_dc_linear, solve_dc_non_linear, NewtonOptions, SolverOptions};

#[test]
fn test_parse_and_solve_resistor_divider() {
    let netlist_src = r#"
* Simple Voltage Divider
V1 in 0 10.0
R1 in out 1k
R2 out 0 1k
.OP
.END
"#;

    let parsed = parse_netlist(netlist_src).expect("Failed to parse netlist");
    assert_eq!(parsed.title, "Simple Voltage Divider");
    assert_eq!(parsed.components.len(), 3);
    assert_eq!(parsed.directives, vec![Directive::Op]);

    let elaborated = elaborate_netlist(&parsed).expect("Failed to elaborate netlist");
    assert_eq!(elaborated.plan, SimulationPlan::Op);

    let sol = solve_dc_linear(&elaborated.graph, &SolverOptions::default())
        .expect("DC linear solve failed");
    let out_node = elaborated
        .graph
        .get_node("out")
        .expect("Node 'out' not found");
    let in_node = elaborated
        .graph
        .get_node("in")
        .expect("Node 'in' not found");

    assert!((sol.node_voltage(in_node) - 10.0).abs() < 1e-9);
    assert!((sol.node_voltage(out_node) - 5.0).abs() < 1e-9);
}

#[test]
fn test_parse_and_solve_diode_circuit() {
    let netlist_src = r#"
Diode Test Circuit
V1 in 0 5.0
R1 in n1 1k
D1 n1 0 D1N4148
.MODEL D1N4148 D (IS=1e-14 RS=0.5 N=1.0)
.OP
"#;

    let parsed = parse_netlist(netlist_src).expect("Failed to parse diode netlist");
    assert_eq!(parsed.models.len(), 1);
    assert!(parsed.models.contains_key("D1N4148"));

    let elaborated = elaborate_netlist(&parsed).expect("Failed to elaborate diode netlist");
    let sol = solve_dc_non_linear(
        &elaborated.graph,
        &elaborated.model_ctx,
        &NewtonOptions::default(),
    )
    .expect("Non-linear DC solve failed");

    let n1_node = elaborated
        .graph
        .get_node("n1")
        .expect("Node 'n1' not found");
    let v_d = sol.node_voltage(n1_node);

    // Diode drop should be roughly ~0.65V - 0.75V
    assert!(v_d > 0.6 && v_d < 0.8, "Diode voltage was {v_d}");
}

#[test]
fn test_parse_dc_sweep_directive() {
    let netlist_src = r#"
DC Sweep Test
V1 in 0 0.0
R1 in 0 100
.DC V1 0.0 5.0 0.5
"#;

    let parsed = parse_netlist(netlist_src).expect("Failed to parse DC sweep");
    let elaborated = elaborate_netlist(&parsed).expect("Elaboration failed");

    assert_eq!(
        elaborated.plan,
        SimulationPlan::Dc {
            source_name: "V1".to_string(),
            start: 0.0,
            stop: 5.0,
            step: 0.5,
        }
    );
}

#[test]
fn test_parse_engineering_units() {
    let netlist_src = r#"
Engineering Units
R1 in 1 10k
R2 1 2 2.2Meg
C1 2 3 100p
C2 3 4 10u
L1 4 0 1m
"#;

    let parsed = parse_netlist(netlist_src).expect("Failed to parse units");
    match &parsed.components[0] {
        ComponentAst::Resistor { resistance, .. } => assert_eq!(*resistance, 10e3),
        _ => panic!("Expected resistor"),
    }
    match &parsed.components[1] {
        ComponentAst::Resistor { resistance, .. } => assert_eq!(*resistance, 2.2e6),
        _ => panic!("Expected resistor"),
    }
    match &parsed.components[2] {
        ComponentAst::Capacitor { capacitance, .. } => assert_eq!(*capacitance, 100e-12),
        _ => panic!("Expected capacitor"),
    }
    match &parsed.components[3] {
        ComponentAst::Capacitor { capacitance, .. } => {
            assert!((*capacitance - 10e-6).abs() < 1e-15)
        }
        _ => panic!("Expected capacitor"),
    }
    match &parsed.components[4] {
        ComponentAst::Inductor { inductance, .. } => assert_eq!(*inductance, 1e-3),
        _ => panic!("Expected inductor"),
    }
}

#[test]
fn test_parse_transmission_line() {
    let netlist_src = r#"
* Transmission Line Test
T1 in 0 out 0 Z0=50 TD=1n
T2 in 0 out2 0 75 2n
.TRAN 0.1n 10n
"#;

    let parsed = parse_netlist(netlist_src).expect("Failed to parse tline");
    assert_eq!(parsed.components.len(), 2);

    match &parsed.components[0] {
        ComponentAst::TransmissionLine {
            name,
            in_pos,
            in_neg,
            out_pos,
            out_neg,
            z0,
            td,
        } => {
            assert_eq!(name, "T1");
            assert_eq!(in_pos, "in");
            assert_eq!(in_neg, "0");
            assert_eq!(out_pos, "out");
            assert_eq!(out_neg, "0");
            assert_eq!(*z0, 50.0);
            assert_eq!(*td, 1e-9);
        }
        _ => panic!("Expected transmission line"),
    }

    match &parsed.components[1] {
        ComponentAst::TransmissionLine { z0, td, .. } => {
            assert_eq!(*z0, 75.0);
            assert_eq!(*td, 2e-9);
        }
        _ => panic!("Expected transmission line"),
    }

    let elaborated = elaborate_netlist(&parsed).expect("Elaboration failed");
    assert_eq!(elaborated.graph.components().len(), 2);
    match elaborated.plan {
        SimulationPlan::Tran { tstep, tstop } => {
            assert!((tstep - 1e-10).abs() < 1e-15);
            assert!((tstop - 1e-8).abs() < 1e-15);
        }
        _ => panic!("Expected TRAN plan"),
    }
}
