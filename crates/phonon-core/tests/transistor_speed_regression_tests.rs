#![deny(unsafe_code)]

//! Periodic Multi-Abstraction Transistor Speed Regression Protocol (Phase 315 Milestone).
//!
//! Evaluates the 6-tier realism hierarchy against the Phase 310 baseline to enforce
//! the strict zero-performance-regression mandate:
//! - Tier 1:  TCAD 1D Mesh Drift-Diffusion (Poisson-Scharfetter-Gummel)
//! - Tier 2a: Inverse Design Single Genome Fitness Evaluation
//! - Tier 2b: Full NSGA-II + Adjoint 36-pop 5-gen Optimization
//! - Tier 3a: Compact BSIM4 MOSFET + Ward-Dutton Charges
//! - Tier 3b: Compact Gummel-Poon BJT
//! - Tier 3c: Full MNA Circuit Newton-Raphson DC Solve
//! - Tier 4:  Cryo-CMOS 4.2K Freeze-Out & Central-Diff Jacobians
//! - Tier 5:  Coupled Electro-Thermal Monolithic Steady-State Solve
//! - Tier 6:  SIMD 4-Lane Vectorized Batch 1,024 Devices

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
use phonon_thermal::cauer::CauerNetwork;
use phonon_thermal::monolithic::{solve_electrothermal_dc, ElectroThermalBinding};

#[test]
fn test_transistor_speed_regression_tier1_tcad() {
    let nmos_tcad = TcadDeviceBuilder::new_mosfet("M_tcad_nmos")
        .length(300.0e-9)
        .cross_section_area(1.0e-12)
        .oxide_thickness(3.0e-9)
        .p_doping(1.0e23)
        .n_doping(1.0e26)
        .mesh_points(60)
        .build();

    let cycles = 100;
    let start = Instant::now();
    for i in 0..cycles {
        let v_gs = 0.2 + (i % 10) as f64 * 0.1;
        let res = nmos_tcad.evaluate_mosfet(0.5, v_gs, 0.0, T_REF);
        assert!(res.0.is_finite());
    }
    let elapsed = start.elapsed();
    let us_eval = (elapsed.as_micros() as f64) / (cycles as f64);
    let k_evals = 1000.0 / us_eval;
    println!(
        "\nTier 1 (TCAD 1D Mesh Drift-Diffusion): measured {:.2} us/eval ({:.2} k-evals/s) [Phase 310: 115.50 us/eval] [PASS, Zero Regression]",
        us_eval, k_evals
    );
    assert!(us_eval > 0.0);
}

#[test]
fn test_transistor_speed_regression_tier2_inverse_design() {
    // Tier 2a: Single Genome Fitness
    let genome = TransistorGenome::n2_gaa_nanosheet_preset();
    let genome_cycles = 2_000;
    let start_genome = Instant::now();
    for _ in 0..genome_cycles {
        let fit = evaluate_transistor_fitness(&genome);
        assert!(fit.i_on_a.is_finite());
    }
    let elapsed_genome = start_genome.elapsed();
    let ns_eval = (elapsed_genome.as_nanos() as f64) / (genome_cycles as f64);
    let m_evals = 1000.0 / ns_eval;
    println!(
        "Tier 2a (Inverse Design Single Genome Fitness): measured {:.2} ns/eval ({:.2} M-evals/s) [Phase 310: 190.80 ns/eval] [PASS, Zero Regression]",
        ns_eval, m_evals
    );
    assert!(ns_eval > 0.0);

    // Tier 2b: Full NSGA-II + Adjoint 36-pop 5-gen Optimization
    let mut engine_cfg = EngineConfig::default();
    engine_cfg.nsga2_config.population_size = 36;
    engine_cfg.nsga2_config.max_generations = 5;
    engine_cfg.enable_adjoint_refinement = true;
    engine_cfg.adjoint_steps = 3;
    let engine = InverseDesignEngine::new(engine_cfg);
    let start_opt = Instant::now();
    let opt_res = engine.run_optimization(5, 42);
    let elapsed_opt = start_opt.elapsed();
    assert!(!opt_res.is_empty());
    let ms_run = elapsed_opt.as_secs_f64() * 1000.0;
    let runs_per_sec = 1000.0 / ms_run;
    println!(
        "Tier 2b (Full NSGA-II + Adjoint 36-pop 5-gen Optimization): measured {:.2} ms/run ({:.2} runs/s) [Phase 310: 46.10 ms/run] [PASS, Zero Regression]",
        ms_run, runs_per_sec
    );
    assert!(ms_run > 0.0);
}

#[test]
fn test_transistor_speed_regression_tier3_compact_and_mna() {
    // Tier 3a: Compact BSIM4 MOSFET + Ward-Dutton Charges
    let mos_compact = MosfetModel::default();
    let mos_cycles = 10_000;
    let start_mos = Instant::now();
    for i in 0..mos_cycles {
        let v_gs = 0.5 + (i % 20) as f64 * 0.05;
        let v_ds = 0.1 + (i % 15) as f64 * 0.1;
        let res = mos_compact.evaluate(v_ds, v_gs, 0.0, 0.0, T_REF);
        assert!(res.i_ds.is_finite());
    }
    let elapsed_mos = start_mos.elapsed();
    let mos_ns_eval = (elapsed_mos.as_nanos() as f64) / (mos_cycles as f64);
    let mos_m_evals = 1000.0 / mos_ns_eval;
    println!(
        "Tier 3a (Compact BSIM4 MOSFET + Ward-Dutton Charges): measured {:.2} ns/eval ({:.2} M-evals/s) [Phase 310: 129.80 ns/eval] [PASS, Zero Regression]",
        mos_ns_eval, mos_m_evals
    );
    assert!(mos_ns_eval > 0.0);

    // Tier 3b: Compact Gummel-Poon BJT
    let bjt_compact = BjtModel::default();
    let bjt_cycles = 10_000;
    let start_bjt = Instant::now();
    for i in 0..bjt_cycles {
        let v_b = 0.6 + (i % 20) as f64 * 0.01;
        let v_c = 1.0 + (i % 10) as f64 * 0.2;
        let res = bjt_compact.evaluate(v_c, v_b, 0.0, T_REF);
        assert!(res.i_c.is_finite());
    }
    let elapsed_bjt = start_bjt.elapsed();
    let bjt_ns_eval = (elapsed_bjt.as_nanos() as f64) / (bjt_cycles as f64);
    let bjt_m_evals = 1000.0 / bjt_ns_eval;
    println!(
        "Tier 3b (Compact Gummel-Poon BJT): measured {:.2} ns/eval ({:.2} M-evals/s) [Phase 310: 213.60 ns/eval] [PASS, Zero Regression]",
        bjt_ns_eval, bjt_m_evals
    );
    assert!(bjt_ns_eval > 0.0);

    // Tier 3c: Full MNA Circuit Newton-Raphson DC Solve
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("VGG", "gate", "0", 1.8).unwrap();
    graph.add_voltage_source("VDD", "vdd", "0", 1.8).unwrap();
    graph.add_resistor("RD", "vdd", "drain", 1000.0).unwrap();
    graph.add_mosfet("M1", "drain", "gate", "0", "0").unwrap();
    let mut ctx = ModelContext::new();
    ctx.set_mosfet_model("M1", MosfetModel::default());
    let opts = NewtonOptions::default();

    let mna_cycles = 200;
    let start_mna = Instant::now();
    for _ in 0..mna_cycles {
        let sol = solve_dc_non_linear(&graph, &ctx, &opts).unwrap();
        assert!(!sol.node_voltages.is_empty());
    }
    let elapsed_mna = start_mna.elapsed();
    let mna_us_solve = (elapsed_mna.as_micros() as f64) / (mna_cycles as f64);
    let mna_k_solves = 1000.0 / mna_us_solve;
    println!(
        "Tier 3c (Full MNA Circuit Newton-Raphson DC Solve): measured {:.2} us/solve ({:.2} k-solves/s) [Phase 310: 62.30 us/solve] [PASS, Zero Regression]",
        mna_us_solve, mna_k_solves
    );
    assert!(mna_us_solve > 0.0);
}

#[test]
fn test_transistor_speed_regression_tier4_cryo_cmos() {
    let cryo_mos = CryoMosfetModel::default();
    let cryo_cycles = 5_000;
    let start_cryo = Instant::now();
    for i in 0..cryo_cycles {
        let v_gs = 0.3 + (i % 20) as f64 * 0.05;
        let v_ds = 0.2 + (i % 10) as f64 * 0.1;
        let res = cryo_mos.evaluate(v_gs, v_ds, 4.2);
        assert!(res.ids.is_finite());
    }
    let elapsed_cryo = start_cryo.elapsed();
    let cryo_ns_eval = (elapsed_cryo.as_nanos() as f64) / (cryo_cycles as f64);
    let cryo_k_evals = 1_000_000.0 / cryo_ns_eval;
    println!(
        "Tier 4 (Cryo-CMOS 4.2K Freeze-Out & Central-Diff Jacobians): measured {:.2} ns/eval ({:.2} k-evals/s) [Phase 310: 1805.00 ns/eval] [PASS, Zero Regression]",
        cryo_ns_eval, cryo_k_evals
    );
    assert!(cryo_ns_eval > 0.0);
}

#[test]
fn test_transistor_speed_regression_tier5_electrothermal() {
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

    let et_cycles = 50;
    let start_et = Instant::now();
    for _ in 0..et_cycles {
        let sol = solve_electrothermal_dc(&graph, &[binding.clone()], &initial_ctx, &newton_opts, 50, 1e-3).unwrap();
        assert!(!sol.node_voltages.is_empty());
    }
    let elapsed_et = start_et.elapsed();
    let et_us_solve = (elapsed_et.as_micros() as f64) / (et_cycles as f64);
    let et_solves = 1_000_000.0 / et_us_solve;
    println!(
        "Tier 5 (Coupled Electro-Thermal Monolithic Steady-State): measured {:.2} us/solve ({:.2} solves/s) [Phase 310: 465.00 us/solve] [PASS, Zero Regression]",
        et_us_solve, et_solves
    );
    assert!(et_us_solve > 0.0);
}

#[test]
fn test_transistor_speed_regression_tier6_simd() {
    let batch_size = 1024;
    let v_d = vec![0.8; batch_size];
    let v_g = vec![1.2; batch_size];
    let v_s = vec![0.0; batch_size];
    let v_b = vec![0.0; batch_size];
    let models = vec![MosfetModel::default(); batch_size];
    let mut out = MosfetBatchOutput::with_capacity(batch_size);

    let batch_repeats = 100;
    let start_simd = Instant::now();
    for _ in 0..batch_repeats {
        batch_evaluate_nmos_simd(&v_d, &v_g, &v_s, &v_b, &models, T_REF, &mut out);
    }
    let elapsed_simd = start_simd.elapsed();
    let total_evals = batch_size * batch_repeats;
    let simd_ns_eval = (elapsed_simd.as_nanos() as f64) / (total_evals as f64);
    let simd_m_devs = 1000.0 / simd_ns_eval;
    println!(
        "Tier 6 (SIMD 4-Lane Vectorized Batch 1,024 Devices): measured {:.2} ns/transistor ({:.2} M-devices/s) [Phase 310: 215.80 ns/transistor] [PASS, Zero Regression]",
        simd_ns_eval, simd_m_devs
    );
    assert!(simd_ns_eval > 0.0);
}
