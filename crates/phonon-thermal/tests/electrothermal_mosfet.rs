use phonon_core::CircuitGraph;
use phonon_models::mosfet::MosfetModel;
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_thermal::cauer::CauerNetwork;
use phonon_thermal::monolithic::{solve_electrothermal_dc, ElectroThermalBinding};

#[test]
fn test_coupled_electrothermal_mosfet_self_heating() {
    let mut graph = CircuitGraph::new();
    // VGG = 3.3 V, VDD = 10 V, RD = 100 Ohm
    graph.add_voltage_source("VGG", "gate", "0", 3.3).unwrap();
    graph.add_voltage_source("VDD", "vdd", "0", 10.0).unwrap();
    graph.add_resistor("RD", "vdd", "drain", 100.0).unwrap();
    graph.add_mosfet("M1", "drain", "gate", "0", "0").unwrap();

    let mosfet_model = MosfetModel {
        w: 50e-6, // 50 um wide power cell
        l: 0.35e-6,
        vth0: 0.8,
        temp_coeff_mu: 1.5, // T^-1.5 mobility degradation
        ..Default::default()
    };

    let ambient_k = 300.0;
    let mut initial_ctx = ModelContext::new();
    initial_ctx.temperature_kelvin = ambient_k;
    initial_ctx.set_mosfet_model("M1", mosfet_model);

    let newton_opts = NewtonOptions::default();

    // 1. Solve uncoupled (purely electrical at T = 300 K)
    let cold_sol = solve_dc_non_linear(&graph, &initial_ctx, &newton_opts)
        .expect("Cold electrical solve must converge");
    let v_drain_cold = cold_sol.node_voltage_by_name(&graph, "drain").unwrap();
    let i_ds_cold = (10.0 - v_drain_cold) / 100.0;
    let p_cold = v_drain_cold * i_ds_cold;

    assert!(
        p_cold > 0.05,
        "Power dissipation must be significant: {p_cold} W"
    );

    // 2. Setup Cauer thermal network (R_th,ja = 100 K/W)
    let mut cauer = CauerNetwork::new();
    cauer.add_stage("Junction_Die", 20.0, 1e-4);
    cauer.add_stage("Die_Case", 30.0, 1e-3);
    cauer.add_stage("Case_Ambient", 50.0, 1e-2);
    assert_eq!(cauer.total_thermal_resistance(), 100.0);

    let binding = ElectroThermalBinding {
        component_name: "M1".to_string(),
        cauer,
        ambient_k,
    };

    // 3. Solve coupled electro-thermal steady-state
    let coupled_sol =
        solve_electrothermal_dc(&graph, &[binding], &initial_ctx, &newton_opts, 50, 1e-3)
            .expect("Coupled electro-thermal solve must converge");

    let t_junction = coupled_sol.junction_temperatures["M1"];
    let p_coupled = coupled_sol.power_dissipations["M1"];

    // Junction must heat up significantly
    let delta_t = t_junction - ambient_k;
    assert!(
        delta_t > 5.0,
        "Junction should heat up by at least 5 K, got delta_t={delta_t} K (T_j={t_junction})"
    );

    // Verify thermal equilibrium: delta_T == P_coupled * R_th_total (100 K/W)
    let expected_delta_t = p_coupled * 100.0;
    assert!(
        (delta_t - expected_delta_t).abs() < 0.1,
        "delta_t={delta_t}, expected={expected_delta_t}"
    );

    // 4. Verify negative thermal feedback on drain current
    let v_drain_coupled = coupled_sol.node_voltages[graph.get_node("drain").unwrap().index()];
    let i_ds_coupled = (10.0 - v_drain_coupled) / 100.0;

    // Hot current must be strictly less than cold current due to mobility degradation
    assert!(
        i_ds_coupled < i_ds_cold,
        "Self-heating should decrease drain current: cold={i_ds_cold}, coupled={i_ds_coupled}"
    );
}
