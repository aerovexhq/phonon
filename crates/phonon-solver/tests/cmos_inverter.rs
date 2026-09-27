use phonon_core::CircuitGraph;
use phonon_models::mosfet::{MosfetModel, MosfetType};
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};

#[test]
fn test_cmos_inverter_transfer_curve() {
    let vdd_val = 1.8;

    let nmos_model = MosfetModel {
        mos_type: MosfetType::Nmos,
        w: 10e-6,
        l: 0.18e-6,
        vth0: 0.7,
        ..Default::default()
    };

    let pmos_model = MosfetModel {
        mos_type: MosfetType::Pmos,
        w: 20e-6, // wider to balance mobility
        l: 0.18e-6,
        vth0: 0.7,
        mu0: 0.025,
        ..Default::default()
    };

    // Sweep Vin from 0.0 to 1.8 V
    let vin_steps = [0.0, 0.3, 0.6, 0.9, 1.2, 1.5, 1.8];
    let mut vout_results = Vec::new();

    for &vin in &vin_steps {
        let mut graph = CircuitGraph::new();
        graph
            .add_voltage_source("VDD", "vdd", "0", vdd_val)
            .unwrap();
        graph.add_voltage_source("VIN", "in", "0", vin).unwrap();

        // PMOS pull-up: drain="out", gate="in", source="vdd", bulk="vdd"
        graph.add_mosfet("M_P", "out", "in", "vdd", "vdd").unwrap();
        // NMOS pull-down: drain="out", gate="in", source="0", bulk="0"
        graph.add_mosfet("M_N", "out", "in", "0", "0").unwrap();

        // High impedance load to ground to ensure well-conditioned node even if both are weakly off
        graph.add_resistor("RLOAD", "out", "0", 1e8).unwrap();

        let mut ctx = ModelContext::new();
        ctx.set_mosfet_model("M_P", pmos_model);
        ctx.set_mosfet_model("M_N", nmos_model);

        let opts = NewtonOptions::default();
        let sol = solve_dc_non_linear(&graph, &ctx, &opts)
            .unwrap_or_else(|e| panic!("CMOS inverter solve failed at Vin={vin}: {e:?}"));

        let vout = sol.node_voltage_by_name(&graph, "out").unwrap();
        vout_results.push((vin, vout));
    }

    // Check Logic High: Vin = 0.0 -> Vout ~ 1.8 V
    let (_, vout_low_in) = vout_results[0];
    assert!(
        vout_low_in > 1.7,
        "Logic high output must be > 1.7V, got {vout_low_in}"
    );

    // Check Logic Low: Vin = 1.8 -> Vout ~ 0.0 V
    let (_, vout_high_in) = vout_results[vout_results.len() - 1];
    assert!(
        vout_high_in < 0.1,
        "Logic low output must be < 0.1V, got {vout_high_in}"
    );

    // Check monotonic decreasing characteristic
    for i in 1..vout_results.len() {
        assert!(
            vout_results[i].1 <= vout_results[i - 1].1 + 1e-4,
            "VTC must be monotonically decreasing: Vin={}, Vout={}; prev Vin={}, prev Vout={}",
            vout_results[i].0,
            vout_results[i].1,
            vout_results[i - 1].0,
            vout_results[i - 1].1
        );
    }

    // Midpoint switching threshold check: around Vin = 0.9V, Vout should be in transition
    let (_, vout_mid) = vout_results[3]; // Vin = 0.9V
    assert!(
        vout_mid > 0.2 && vout_mid < 1.6,
        "Midpoint voltage must be in transition region, got {vout_mid}"
    );
}
