use phonon_core::CircuitGraph;
use phonon_models::diode::DiodeModel;
use phonon_solver::mna::{ModelContext, NewtonOptions};
use phonon_thermal::cauer::CauerNetwork;
use phonon_thermal::monolithic::{solve_electrothermal_dc, ElectroThermalBinding};
use phonon_thermal::relaxation::MultirateElectroThermalSimulator;

#[test]
fn test_relaxation_converges_to_monolithic_steady_state() {
    let mut graph = CircuitGraph::new();
    // Vin = 2.0 V, R1 = 50 Ohm, Diode D1
    graph.add_voltage_source("VIN", "in", "0", 2.0).unwrap();
    graph.add_resistor("R1", "in", "anode", 50.0).unwrap();
    graph.add_diode("D1", "anode", "0").unwrap();

    let ambient_k = 300.0;
    let diode_model = DiodeModel {
        is: 1e-12, // 1 pA
        ..Default::default()
    };

    let mut ctx = ModelContext::new();
    ctx.temperature_kelvin = ambient_k;
    ctx.set_diode_model("D1", diode_model);

    // Cauer network: R_th = 40 K/W, C_th = 0.02 J/K -> tau = 0.8 s
    let mut cauer = CauerNetwork::new();
    cauer.add_stage("Junction_Ambient", 40.0, 0.02);

    let binding = ElectroThermalBinding {
        component_name: "D1".to_string(),
        cauer: cauer.clone(),
        ambient_k,
    };

    let newton_opts = NewtonOptions::default();

    // 1. Solve steady-state monolithically
    let monolithic_sol = solve_electrothermal_dc(&graph, &[binding], &ctx, &newton_opts, 30, 1e-3)
        .expect("Monolithic electro-thermal solve must converge");

    let t_j_steady = monolithic_sol.junction_temperatures["D1"];
    let p_steady = monolithic_sol.power_dissipations["D1"];

    assert!(
        t_j_steady > ambient_k + 0.5,
        "Steady-state temperature must be elevated: {t_j_steady} K"
    );

    // 2. Run multirate waveform relaxation transient simulation
    let sim = MultirateElectroThermalSimulator::new(&graph, "D1", cauer, ambient_k);
    let t_stop = 6.0; // 6 seconds = 7.5 * tau
    let dt_thermal = 0.05; // 50 ms thermal steps

    let trace = sim
        .simulate(t_stop, dt_thermal, &ctx, &newton_opts)
        .expect("Relaxation simulation must succeed");

    assert!(!trace.is_empty());

    // First point must be at ambient
    assert_eq!(trace[0].time_s, 0.0);
    assert_eq!(trace[0].junction_temp_k, ambient_k);

    // Trajectory must be monotonically increasing in temperature
    for i in 1..trace.len() {
        assert!(
            trace[i].junction_temp_k >= trace[i - 1].junction_temp_k - 1e-6,
            "Temperature must rise monotonically"
        );
    }

    // Final point at t=6.0s must match monolithic steady state within 0.5 K
    let last_point = trace.last().unwrap();
    assert!(
        (last_point.junction_temp_k - t_j_steady).abs() < 0.5,
        "Relaxation final T_j ({}) must match monolithic steady state ({})",
        last_point.junction_temp_k,
        t_j_steady
    );

    // Power at end of transient should match steady-state power within 5%
    assert!(
        (last_point.power_watts - p_steady).abs() / p_steady < 0.05,
        "Final power ({}) must match steady-state power ({})",
        last_point.power_watts,
        p_steady
    );
}
