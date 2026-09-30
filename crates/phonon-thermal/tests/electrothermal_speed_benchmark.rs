#![deny(unsafe_code)]

use std::time::Instant;
use phonon_core::CircuitGraph;
use phonon_models::mosfet::MosfetModel;
use phonon_solver::mna::{ModelContext, NewtonOptions};
use phonon_thermal::cauer::CauerNetwork;
use phonon_thermal::monolithic::{solve_electrothermal_dc, ElectroThermalBinding};

#[test]
fn benchmark_electrothermal_speed() {
    println!("\n=== ELECTRO-THERMAL TRANSISTOR SPEED BENCHMARK ===");
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("VGG", "gate", "0", 3.3).unwrap();
    graph.add_voltage_source("VDD", "vdd", "0", 10.0).unwrap();
    graph.add_resistor("RD", "vdd", "drain", 100.0).unwrap();
    graph.add_mosfet("M1", "drain", "gate", "0", "0").unwrap();

    let mosfet_model = MosfetModel {
        w: 50e-6,
        l: 0.35e-6,
        vth0: 0.8,
        temp_coeff_mu: 1.5,
        ..Default::default()
    };

    let ambient_k = 300.0;
    let mut initial_ctx = ModelContext::new();
    initial_ctx.temperature_kelvin = ambient_k;
    initial_ctx.set_mosfet_model("M1", mosfet_model);

    let newton_opts = NewtonOptions::default();

    let mut cauer = CauerNetwork::new();
    cauer.add_stage("Junction_Die", 20.0, 1e-4);
    cauer.add_stage("Die_Case", 30.0, 1e-3);
    cauer.add_stage("Case_Ambient", 50.0, 1e-2);

    let binding = ElectroThermalBinding {
        component_name: "M1".to_string(),
        cauer,
        ambient_k,
    };

    let cycles = 2_000;
    let start = Instant::now();
    for _ in 0..cycles {
        let _ = solve_electrothermal_dc(&graph, &[binding.clone()], &initial_ctx, &newton_opts, 50, 1e-3).unwrap();
    }
    let elapsed = start.elapsed();
    let per_solve_us = elapsed.as_micros() as f64 / cycles as f64;
    let throughput = cycles as f64 / elapsed.as_secs_f64();
    println!(
        "Layer 5 (Coupled Electro-Thermal Monolithic Steady-State Solve): {:.2} us/solve ({:.1} k-solves/sec)",
        per_solve_us,
        throughput / 1e3
    );
    println!("===================================================\n");
}
