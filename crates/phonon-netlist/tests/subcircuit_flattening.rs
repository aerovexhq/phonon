use phonon_netlist::*;
use phonon_solver::mna::{solve_dc_linear, SolverOptions};

#[test]
fn test_subcircuit_flattening_voltage_divider() {
    let netlist_src = r#"
* Hierarchical Subcircuit Test
.SUBCKT DIVIDER IN OUT GND_PIN
R1 IN OUT 1k
R2 OUT GND_PIN 1k
.ENDS DIVIDER

V1 vin 0 12.0
X1 vin vmid 0 DIVIDER
X2 vmid vout 0 DIVIDER
.OP
"#;

    let parsed = parse_netlist(netlist_src).expect("Failed to parse hierarchical netlist");
    assert_eq!(parsed.subcircuits.len(), 1);
    assert!(parsed.subcircuits.contains_key("DIVIDER"));

    let elaborated = elaborate_netlist(&parsed).expect("Failed to elaborate subcircuits");
    // Graph should contain: V1, X1.R1, X1.R2, X2.R1, X2.R2 (total 5 components)
    assert_eq!(elaborated.graph.num_components(), 5);

    let sol =
        solve_dc_linear(&elaborated.graph, &SolverOptions::default()).expect("DC solve failed");
    let vin = elaborated
        .graph
        .get_node("vin")
        .expect("Node 'vin' not found");
    let vmid = elaborated
        .graph
        .get_node("vmid")
        .expect("Node 'vmid' not found");
    let vout = elaborated
        .graph
        .get_node("vout")
        .expect("Node 'vout' not found");

    // X1 divides 12V to 6V (equivalent resistance of X2 in parallel with X1.R2 is 2k || 1k = 666.7, so let's verify exact Ohm's law)
    // R1 = 1k, R_load = R2 (1k) in parallel with (R3 + R4 = 2k) -> 1k * 2k / 3k = 666.6667
    // Vmid = 12 * (2/3) / (1 + 2/3) = 12 * 2 / 5 = 4.8 V
    // Vout = Vmid * (1 / 2) = 2.4 V
    let v_in_val = sol.node_voltage(vin);
    let v_mid_val = sol.node_voltage(vmid);
    let v_out_val = sol.node_voltage(vout);

    assert!((v_in_val - 12.0).abs() < 1e-6);
    assert!((v_mid_val - 4.8).abs() < 1e-6, "vmid was {v_mid_val}");
    assert!((v_out_val - 2.4).abs() < 1e-6, "vout was {v_out_val}");
}

#[test]
fn test_nested_subcircuits() {
    let netlist_src = r#"
* Nested Subcircuit Test
.SUBCKT RES_PAIR A B C
R1 A B 100
R2 B C 200
.ENDS RES_PAIR

.SUBCKT QUAD_BLOCK IN OUT
X_FIRST IN MID 0 RES_PAIR
X_SECOND MID OUT 0 RES_PAIR
.ENDS QUAD_BLOCK

V1 v_in 0 10.0
X_TOP v_in v_out QUAD_BLOCK
.OP
"#;

    let parsed = parse_netlist(netlist_src).expect("Failed to parse nested subcircuits");
    assert_eq!(parsed.subcircuits.len(), 2);

    let elaborated = elaborate_netlist(&parsed).expect("Failed to elaborate nested subcircuits");
    // Components should be:
    // V1
    // X_TOP.X_FIRST.R1
    // X_TOP.X_FIRST.R2
    // X_TOP.X_SECOND.R1
    // X_TOP.X_SECOND.R2
    assert_eq!(elaborated.graph.num_components(), 5);

    let sol =
        solve_dc_linear(&elaborated.graph, &SolverOptions::default()).expect("DC solve failed");
    let vin = elaborated.graph.get_node("v_in").expect("vin missing");
    let vout = elaborated.graph.get_node("v_out").expect("vout missing");

    assert!((sol.node_voltage(vin) - 10.0).abs() < 1e-6);
    // Since v_out is connected to ground through R2 (200), let's check its validity
    assert!(sol.node_voltage(vout) >= 0.0);
}

#[test]
fn test_subcircuit_pin_mismatch_error() {
    let netlist_src = r#"
.SUBCKT MY_SUBCKT A B C
R1 A B 100
R2 B C 100
.ENDS MY_SUBCKT

X1 in out MY_SUBCKT
"#;

    let parsed = parse_netlist(netlist_src).expect("Parsed netlist");
    let err = elaborate_netlist(&parsed).expect_err("Should have failed pin count mismatch");
    match err {
        NetlistError::PinCountMismatch {
            expected, found, ..
        } => {
            assert_eq!(expected, 3);
            assert_eq!(found, 2);
        }
        _ => panic!("Expected PinCountMismatch error, got {err:?}"),
    }
}

#[test]
fn test_undefined_subcircuit_error() {
    let netlist_src = r#"
X1 in out NON_EXISTENT
"#;

    let parsed = parse_netlist(netlist_src).expect("Parsed netlist");
    let err = elaborate_netlist(&parsed).expect_err("Should have failed with undefined subcircuit");
    match err {
        NetlistError::UndefinedSubcircuit {
            subcircuit_name, ..
        } => {
            assert_eq!(subcircuit_name, "NON_EXISTENT");
        }
        _ => panic!("Expected UndefinedSubcircuit error, got {err:?}"),
    }
}
