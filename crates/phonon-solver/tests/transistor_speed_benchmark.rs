#![deny(unsafe_code)]

use std::time::Instant;
use phonon_core::constants::T_REF;
use phonon_core::CircuitGraph;
use phonon_models::bjt::BjtModel;
use phonon_models::cryogenic::cryo_mosfet::CryoMosfetModel;
use phonon_models::mosfet::MosfetModel;
use phonon_models::optimization::{evaluate_transistor_fitness, TransistorGenome};
use phonon_models::simd::mosfet_simd::{batch_evaluate_nmos_simd, MosfetBatchOutput};
use phonon_models::tcad::TcadDeviceBuilder;
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::optimization::{EngineConfig, InverseDesignEngine};

#[test]
fn benchmark_transistor_abstraction_layers() {
    println!("\n=== TRANSISTOR ABSTRACTION LAYERS SPEED BENCHMARK ===");

    // ------------------------------------------------------------------------
    // Layer 1: TCAD Microscopic Drift-Diffusion Synthesis & Solve
    // ------------------------------------------------------------------------
    let nmos_tcad = TcadDeviceBuilder::new_mosfet("M_tcad_nmos")
        .length(300.0e-9)
        .cross_section_area(1.0e-12)
        .oxide_thickness(3.0e-9)
        .p_doping(1.0e23)
        .n_doping(1.0e26)
        .mesh_points(60)
        .build();

    let tcad_cycles = 10_000;
    let start_tcad = Instant::now();
    for i in 0..tcad_cycles {
        let v_gs = 0.2 + (i % 10) as f64 * 0.1;
        let _ = nmos_tcad.evaluate_mosfet(0.5, v_gs, 0.0, T_REF);
    }
    let elapsed_tcad = start_tcad.elapsed();
    let tcad_per_eval_ns = elapsed_tcad.as_nanos() as f64 / tcad_cycles as f64;
    let tcad_throughput = tcad_cycles as f64 / elapsed_tcad.as_secs_f64();
    println!(
        "Layer 1 (TCAD 1D Mesh Drift-Diffusion): {:.2} ns/eval ({:.1} k-evals/sec)",
        tcad_per_eval_ns,
        tcad_throughput / 1e3
    );

    // ------------------------------------------------------------------------
    // Layer 2: Inverse Design (Single Genome Fitness & Full NSGA-II Cycle)
    // ------------------------------------------------------------------------
    let genome = TransistorGenome::n2_gaa_nanosheet_preset();
    let genome_cycles = 100_000;
    let start_genome = Instant::now();
    for _ in 0..genome_cycles {
        let _ = evaluate_transistor_fitness(&genome);
    }
    let elapsed_genome = start_genome.elapsed();
    let genome_per_eval_ns = elapsed_genome.as_nanos() as f64 / genome_cycles as f64;
    let genome_throughput = genome_cycles as f64 / elapsed_genome.as_secs_f64();
    println!(
        "Layer 2a (Inverse Design Single Genome Fitness): {:.2} ns/eval ({:.1} k-evals/sec)",
        genome_per_eval_ns,
        genome_throughput / 1e3
    );

    let mut engine_cfg = EngineConfig::default();
    engine_cfg.nsga2_config.population_size = 36;
    engine_cfg.nsga2_config.max_generations = 5;
    engine_cfg.enable_adjoint_refinement = true;
    engine_cfg.adjoint_steps = 3;
    let engine = InverseDesignEngine::new(engine_cfg);
    let start_opt = Instant::now();
    let _ = engine.run_optimization(5, 42);
    let elapsed_opt = start_opt.elapsed();
    println!(
        "Layer 2b (Full NSGA-II + Adjoint 36-pop 5-gen Optimization): {:.2} ms/run",
        elapsed_opt.as_secs_f64() * 1000.0
    );

    // ------------------------------------------------------------------------
    // Layer 3: Compact Analytical MOSFET (Ward-Dutton Charge Conservation)
    // ------------------------------------------------------------------------
    let mos_compact = MosfetModel::default();
    let compact_cycles = 1_000_000;
    let start_compact = Instant::now();
    for i in 0..compact_cycles {
        let v_gs = 0.5 + (i % 20) as f64 * 0.05;
        let v_ds = 0.1 + (i % 15) as f64 * 0.1;
        let _ = mos_compact.evaluate(v_ds, v_gs, 0.0, 0.0, T_REF);
    }
    let elapsed_compact = start_compact.elapsed();
    let compact_per_eval_ns = elapsed_compact.as_nanos() as f64 / compact_cycles as f64;
    let compact_throughput = compact_cycles as f64 / elapsed_compact.as_secs_f64();
    println!(
        "Layer 3a (Compact MOSFET with Ward-Dutton Charges & Jacobians): {:.2} ns/eval ({:.2} M-evals/sec)",
        compact_per_eval_ns,
        compact_throughput / 1e6
    );

    // Compact Gummel-Poon BJT
    let bjt_compact = BjtModel::default();
    let bjt_cycles = 1_000_000;
    let start_bjt = Instant::now();
    for i in 0..bjt_cycles {
        let v_b = 0.6 + (i % 20) as f64 * 0.01;
        let v_c = 1.0 + (i % 10) as f64 * 0.2;
        let _ = bjt_compact.evaluate(v_c, v_b, 0.0, T_REF);
    }
    let elapsed_bjt = start_bjt.elapsed();
    let bjt_per_eval_ns = elapsed_bjt.as_nanos() as f64 / bjt_cycles as f64;
    let bjt_throughput = bjt_cycles as f64 / elapsed_bjt.as_secs_f64();
    println!(
        "Layer 3b (Compact Gummel-Poon BJT): {:.2} ns/eval ({:.2} M-evals/sec)",
        bjt_per_eval_ns,
        bjt_throughput / 1e6
    );

    // Full MNA Non-Linear Circuit DC Solve (Newton-Raphson convergence)
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("VGG", "gate", "0", 1.8).unwrap();
    graph.add_voltage_source("VDD", "vdd", "0", 1.8).unwrap();
    graph.add_resistor("RD", "vdd", "drain", 1000.0).unwrap();
    graph.add_mosfet("M1", "drain", "gate", "0", "0").unwrap();
    let mut ctx = ModelContext::new();
    ctx.set_mosfet_model("M1", MosfetModel::default());
    let opts = NewtonOptions::default();

    let mna_cycles = 50_000;
    let start_mna = Instant::now();
    for _ in 0..mna_cycles {
        let _ = solve_dc_non_linear(&graph, &ctx, &opts).unwrap();
    }
    let elapsed_mna = start_mna.elapsed();
    let mna_per_solve_us = elapsed_mna.as_micros() as f64 / mna_cycles as f64;
    let mna_throughput = mna_cycles as f64 / elapsed_mna.as_secs_f64();
    println!(
        "Layer 3c (Full MNA Circuit Newton-Raphson DC Solve): {:.2} us/solve ({:.1} k-solves/sec)",
        mna_per_solve_us,
        mna_throughput / 1e3
    );

    // ------------------------------------------------------------------------
    // Layer 4: Cryogenic Cryo-CMOS (Freeze-out + Central-Difference Jacobians)
    // ------------------------------------------------------------------------
    let cryo_mos = CryoMosfetModel::default();
    let cryo_cycles = 500_000;
    let start_cryo = Instant::now();
    for i in 0..cryo_cycles {
        let v_gs = 0.3 + (i % 20) as f64 * 0.05;
        let v_ds = 0.2 + (i % 10) as f64 * 0.1;
        let _ = cryo_mos.evaluate(v_gs, v_ds, 4.2);
    }
    let elapsed_cryo = start_cryo.elapsed();
    let cryo_per_eval_ns = elapsed_cryo.as_nanos() as f64 / cryo_cycles as f64;
    let cryo_throughput = cryo_cycles as f64 / elapsed_cryo.as_secs_f64();
    println!(
        "Layer 4 (Cryo-CMOS 4.2K with Freeze-Out & Central-Diff Jacobians): {:.2} ns/eval ({:.2} M-evals/sec)",
        cryo_per_eval_ns,
        cryo_throughput / 1e6
    );

    // ------------------------------------------------------------------------
    // Layer 6: Vectorized SIMD Batch Evaluation (4-lane chunked)
    // ------------------------------------------------------------------------
    let batch_size = 1024;
    let v_d = vec![0.8; batch_size];
    let v_g = vec![1.2; batch_size];
    let v_s = vec![0.0; batch_size];
    let v_b = vec![0.0; batch_size];
    let models = vec![MosfetModel::default(); batch_size];
    let mut out = MosfetBatchOutput::with_capacity(batch_size);

    let batch_repeats = 10_000;
    let start_simd = Instant::now();
    for _ in 0..batch_repeats {
        batch_evaluate_nmos_simd(&v_d, &v_g, &v_s, &v_b, &models, T_REF, &mut out);
    }
    let elapsed_simd = start_simd.elapsed();
    let total_simd_evals = batch_size * batch_repeats;
    let simd_per_eval_ns = elapsed_simd.as_nanos() as f64 / total_simd_evals as f64;
    let simd_throughput = total_simd_evals as f64 / elapsed_simd.as_secs_f64();
    println!(
        "Layer 6 (Vectorized SIMD Batch 1024-device Evaluation): {:.2} ns/transistor ({:.2} M-evals/sec)",
        simd_per_eval_ns,
        simd_throughput / 1e6
    );
    println!("====================================================\n");
}
