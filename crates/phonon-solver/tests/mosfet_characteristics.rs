use phonon_core::CircuitGraph;
use phonon_models::mosfet::{MosfetModel, MosfetType};
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};

#[test]
fn test_nmos_output_and_saturation() {
    // NMOS with Source and Bulk at GND
    // Gate at fixed 1.8 V (strong inversion)
    // Drain connected to Vdd (1.8 V) through Rd = 1000 Ohm
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("VGG", "gate", "0", 1.8).unwrap();
    graph.add_voltage_source("VDD", "vdd", "0", 1.8).unwrap();
    graph.add_resistor("RD", "vdd", "drain", 1000.0).unwrap();
    graph.add_mosfet("M1", "drain", "gate", "0", "0").unwrap();

    let mut ctx = ModelContext::new();
    let nmos_model = MosfetModel::default(); // Nmos, Vth0 = 0.7V
    ctx.set_mosfet_model("M1", nmos_model);

    let opts = NewtonOptions::default();
    let sol = solve_dc_non_linear(&graph, &ctx, &opts).expect("NMOS circuit must converge");

    let v_drain = sol.node_voltage_by_name(&graph, "drain").unwrap();
    let v_gate = sol.node_voltage_by_name(&graph, "gate").unwrap();

    assert!((v_gate - 1.8).abs() < 1e-6);
    // Drain voltage should be pulled down significantly by conducting NMOS
    assert!(v_drain < 1.0, "v_drain={v_drain} should be pulled down");
    assert!(v_drain > 0.0, "v_drain={v_drain} must be positive");

    let i_ds = (1.8 - v_drain) / 1000.0;
    // Current should be in milliamp range for W=10um, L=0.18um
    assert!(i_ds > 1e-4 && i_ds < 5e-3, "i_ds={i_ds}");
}

#[test]
fn test_nmos_subthreshold_behavior() {
    // Test that below threshold (Vth0 = 0.7V), drain current drops exponentially with Vgs
    let mut currents = Vec::new();
    let vgs_steps = [0.2, 0.3, 0.4];

    for &vgs in &vgs_steps {
        let mut graph = CircuitGraph::new();
        graph.add_voltage_source("VGS", "gate", "0", vgs).unwrap();
        graph.add_voltage_source("VDD", "vdd", "0", 1.2).unwrap();
        graph.add_resistor("RD", "vdd", "drain", 10_000.0).unwrap();
        graph.add_mosfet("M1", "drain", "gate", "0", "0").unwrap();

        let mut ctx = ModelContext::new();
        ctx.set_mosfet_model("M1", MosfetModel::default());

        let opts = NewtonOptions::default();
        let sol =
            solve_dc_non_linear(&graph, &ctx, &opts).expect("Subthreshold solve must converge");

        let v_drain = sol.node_voltage_by_name(&graph, "drain").unwrap();
        let i_ds = (1.2 - v_drain) / 10_000.0;
        currents.push(i_ds);
    }

    // In subthreshold, current should strictly increase as Vgs increases
    assert!(currents[0] < currents[1]);
    assert!(currents[1] < currents[2]);

    // Ratio of currents for 100 mV step: should be a factor of > 5 (subthreshold exponential slope)
    let ratio1 = currents[1] / currents[0];
    let ratio2 = currents[2] / currents[1];
    assert!(
        ratio1 > 3.0,
        "Subthreshold current ratio {ratio1} must exhibit exponential increase"
    );
    assert!(
        ratio2 > 3.0,
        "Subthreshold current ratio {ratio2} must exhibit exponential increase"
    );
}

#[test]
fn test_pmos_pullup_behavior() {
    // PMOS connected to VDD (1.8V): Source and Bulk at VDD
    // Gate tied to 0V (PMOS strongly ON)
    // Drain tied to GND through 10k resistor
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("VDD", "vdd", "0", 1.8).unwrap();
    graph.add_voltage_source("VG", "gate", "0", 0.0).unwrap();
    graph.add_resistor("RL", "drain", "0", 10_000.0).unwrap();
    // PMOS: drain="drain", gate="gate", source="vdd", bulk="vdd"
    graph
        .add_mosfet("M_P", "drain", "gate", "vdd", "vdd")
        .unwrap();

    let pmos_model = MosfetModel {
        mos_type: MosfetType::Pmos,
        mu0: 0.025, // lower hole mobility
        ..Default::default()
    };

    let mut ctx = ModelContext::new();
    ctx.set_mosfet_model("M_P", pmos_model);

    let opts = NewtonOptions::default();
    let sol = solve_dc_non_linear(&graph, &ctx, &opts).expect("PMOS solve must converge");

    let v_drain = sol.node_voltage_by_name(&graph, "drain").unwrap();
    // PMOS strongly ON should pull drain near VDD
    assert!(
        v_drain > 1.5,
        "v_drain={v_drain} should be pulled up close to 1.8V"
    );
}
