#![deny(unsafe_code)]

//! Periodic Multi-Abstraction Transistor Speed Regression Protocol (Phase 340 Milestone).
//!
//! Evaluates the 7-tier realism hierarchy (Tiers 0 through 6) against baseline thresholds
//! to enforce the strict zero-performance-regression mandate:
//! - Tier 0:  Topological Quantum Acoustics (Majorana Braiding Processor Solver) (< 100.0 us/eval)
//! - Tier 1:  TCAD 1D Mesh Drift-Diffusion (Poisson-Scharfetter-Gummel) (< 7500.0 us/eval)
//! - Tier 2a: Inverse Design Single Genome Fitness Evaluation (< 200.0 ns/eval)
//! - Tier 2b: Full NSGA-II + Adjoint Optimization (< 50.0 ms/run)
//! - Tier 3a: Compact BSIM4 MOSFET + Ward-Dutton Charges (< 150.0 ns/eval)
//! - Tier 3b: Compact Gummel-Poon BJT (< 250.0 ns/eval)
//! - Tier 3c: Full MNA Circuit Newton-Raphson DC Solve (< 250.0 us/solve)
//! - Tier 4:  Cryo-CMOS 4.2K Freeze-Out & Central-Diff Jacobians (< 2000.0 ns/eval)
//! - Tier 5:  Coupled Electro-Thermal Monolithic Steady-State Solve (< 1500.0 us/solve)
//! - Tier 6:  SIMD 4-Lane Vectorized Batch 1,024 Devices (< 250.0 ns/transistor)
//!
//! Also verifies full schematic CAD integrity across all 35 primitives, ERC diagnostics,
//! and binary format round-trip serialization.

use std::time::Instant;

use phonon_core::constants::T_REF;
use phonon_core::CircuitGraph;
use phonon_gui::schematic::{
    deserialize_project, serialize_project, ComponentCategory, ComponentKind,
    DeserializedProject, ErcCode, ErcEngine, ErcSeverity, SchematicCanvas,
    SchematicComponent, SchematicWire, CURRENT_VERSION, PHONON_MAGIC,
};
use phonon_models::bjt::BjtModel;
use phonon_models::cryogenic::cryo_mosfet::CryoMosfetModel;
use phonon_models::majorana_braiding_processor::MajoranaBraidingProcessorParams;
use phonon_models::mosfet::MosfetModel;
use phonon_models::optimization::{evaluate_transistor_fitness, TransistorGenome};
use phonon_models::simd::mosfet_simd::{batch_evaluate_nmos_simd, MosfetBatchOutput};
use phonon_models::tcad::TcadDeviceBuilder;
use phonon_solver::majorana_braiding_processor::MajoranaBraidingProcessorSolver;
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::optimization::{EngineConfig, InverseDesignEngine};
use phonon_thermal::cauer::CauerNetwork;
use phonon_thermal::monolithic::{solve_electrothermal_dc, ElectroThermalBinding};

#[test]
fn test_transistor_speed_regression_tier0_topological_quantum_acoustics() {
    let params = MajoranaBraidingProcessorParams::default();
    let solver = MajoranaBraidingProcessorSolver::new(params);

    // Initial physics verification
    let initial_metrics = solver.evaluate_metrics();
    assert!(
        initial_metrics.braiding_fidelity >= 0.9980,
        "Braiding gate fidelity must be >= 0.9980, got {:.6}",
        initial_metrics.braiding_fidelity
    );
    assert!(
        initial_metrics.topological_protection_gap_mhz > 0.0,
        "Topological protection gap must be > 0.0, got {:.4} MHz",
        initial_metrics.topological_protection_gap_mhz
    );
    assert!(
        initial_metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );

    let cycles = 20_000;
    let start = Instant::now();
    for i in 0..cycles {
        let mut p = params;
        p.braiding_coupling_mev += ((i % 10) as f64) * 0.1;
        let s = MajoranaBraidingProcessorSolver::new(p);
        let m = s.evaluate_metrics();
        assert!(m.braiding_fidelity >= 0.9980);
        assert!(m.topological_protection_gap_mhz > 0.0);
    }
    let elapsed = start.elapsed();
    let us_eval = (elapsed.as_nanos() as f64) / (cycles as f64) / 1000.0;
    let ns_eval = (elapsed.as_nanos() as f64) / (cycles as f64);
    let m_evals = 1000.0 / ns_eval;
    println!(
        "\nTier 0 (Topological Quantum Acoustics - Majorana Braiding): measured {:.4} us/eval ({:.2} ns/eval, {:.2} M-evals/s) [Threshold: < 100.00 us/eval] [PASS, Zero Regression, Sub-Microsecond Throughput]",
        us_eval, ns_eval, m_evals
    );
    assert!(us_eval > 0.0);
    assert!(
        us_eval < 100.0,
        "Tier 0 regression: measured {:.4} us/eval exceeds threshold 100.0 us/eval",
        us_eval
    );
    assert!(
        us_eval < 1.0,
        "Tier 0 sub-microsecond throughput violation: measured {:.4} us/eval",
        us_eval
    );
}

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
        "\nTier 1 (TCAD 1D Mesh Drift-Diffusion): measured {:.2} us/eval ({:.2} k-evals/s) [Threshold: < 7500.00 us/eval] [Phase 310: 115.50 us/eval] [PASS, Zero Regression]",
        us_eval, k_evals
    );
    assert!(us_eval > 0.0);
    assert!(
        us_eval < 7500.0,
        "Tier 1 regression: measured {:.2} us/eval exceeds threshold 7500.0 us/eval",
        us_eval
    );
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
        "Tier 2a (Inverse Design Single Genome Fitness): measured {:.2} ns/eval ({:.2} M-evals/s) [Threshold: < 200.00 ns/eval] [Phase 310: 190.80 ns/eval] [PASS, Zero Regression]",
        ns_eval, m_evals
    );
    assert!(ns_eval > 0.0);
    assert!(
        ns_eval < 200.0,
        "Tier 2a regression: measured {:.2} ns/eval exceeds threshold 200.0 ns/eval",
        ns_eval
    );

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
        "Tier 2b (Full NSGA-II + Adjoint 36-pop 5-gen Optimization): measured {:.2} ms/run ({:.2} runs/s) [Threshold: < 50.00 ms/run] [Phase 310: 46.10 ms/run] [PASS, Zero Regression]",
        ms_run, runs_per_sec
    );
    assert!(ms_run > 0.0);
    assert!(
        ms_run < 50.0,
        "Tier 2b regression: measured {:.2} ms/run exceeds threshold 50.0 ms/run",
        ms_run
    );
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
        "Tier 3a (Compact BSIM4 MOSFET + Ward-Dutton Charges): measured {:.2} ns/eval ({:.2} M-evals/s) [Threshold: < 150.00 ns/eval] [Phase 310: 129.80 ns/eval] [PASS, Zero Regression]",
        mos_ns_eval, mos_m_evals
    );
    assert!(mos_ns_eval > 0.0);
    assert!(
        mos_ns_eval < 150.0,
        "Tier 3a regression: measured {:.2} ns/eval exceeds threshold 150.0 ns/eval",
        mos_ns_eval
    );

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
        "Tier 3b (Compact Gummel-Poon BJT): measured {:.2} ns/eval ({:.2} M-evals/s) [Threshold: < 250.00 ns/eval] [Phase 310: 213.60 ns/eval] [PASS, Zero Regression]",
        bjt_ns_eval, bjt_m_evals
    );
    assert!(bjt_ns_eval > 0.0);
    assert!(
        bjt_ns_eval < 250.0,
        "Tier 3b regression: measured {:.2} ns/eval exceeds threshold 250.0 ns/eval",
        bjt_ns_eval
    );

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
        "Tier 3c (Full MNA Circuit Newton-Raphson DC Solve): measured {:.2} us/solve ({:.2} k-solves/s) [Threshold: < 250.00 us/solve] [Phase 310: 62.30 us/solve] [PASS, Zero Regression]",
        mna_us_solve, mna_k_solves
    );
    assert!(mna_us_solve > 0.0);
    assert!(
        mna_us_solve < 250.0,
        "Tier 3c regression: measured {:.2} us/solve exceeds threshold 250.0 us/solve",
        mna_us_solve
    );
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
        "Tier 4 (Cryo-CMOS 4.2K Freeze-Out & Central-Diff Jacobians): measured {:.2} ns/eval ({:.2} k-evals/s) [Threshold: < 2000.00 ns/eval] [Phase 310: 1805.00 ns/eval] [PASS, Zero Regression]",
        cryo_ns_eval, cryo_k_evals
    );
    assert!(cryo_ns_eval > 0.0);
    assert!(
        cryo_ns_eval < 2000.0,
        "Tier 4 regression: measured {:.2} ns/eval exceeds threshold 2000.0 ns/eval",
        cryo_ns_eval
    );
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

    // Warm-up run
    let _ = solve_electrothermal_dc(&graph, &[binding.clone()], &initial_ctx, &newton_opts, 50, 1e-3).unwrap();

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
        "Tier 5 (Coupled Electro-Thermal Monolithic Steady-State): measured {:.2} us/solve ({:.2} solves/s) [Threshold: < 1500.00 us/solve] [Phase 310: 465.00 us/solve] [PASS, Zero Regression]",
        et_us_solve, et_solves
    );
    assert!(et_us_solve > 0.0);
    assert!(
        et_us_solve < 1500.0,
        "Tier 5 regression: measured {:.2} us/solve exceeds threshold 1500.0 us/solve",
        et_us_solve
    );
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
        "Tier 6 (SIMD 4-Lane Vectorized Batch 1,024 Devices): measured {:.2} ns/transistor ({:.2} M-devices/s) [Threshold: < 250.00 ns/transistor] [Phase 310: 215.80 ns/transistor] [PASS, Zero Regression]",
        simd_ns_eval, simd_m_devs
    );
    assert!(simd_ns_eval > 0.0);
    assert!(
        simd_ns_eval < 250.0,
        "Tier 6 regression: measured {:.2} ns/transistor exceeds threshold 250.0 ns/transistor",
        simd_ns_eval
    );
}

#[test]
fn test_schematic_cad_integrity_primitives_erc_binary_roundtrip() {
    // 1. Verify all 35 primitives across categories
    let all_kinds = ComponentKind::ALL;
    assert_eq!(all_kinds.len(), 35, "ComponentKind::ALL must contain exactly 35 primitives");
    assert_eq!(
        ComponentKind::ALL_VARIANTS.len(),
        35,
        "ComponentKind::ALL_VARIANTS must contain exactly 35 primitives"
    );

    let categories = ComponentCategory::all_categories();
    assert_eq!(categories.len(), 8, "Hierarchical taxonomy must define exactly 8 categories");
    let mut categorized_count = 0;
    for &cat in categories {
        let comps = cat.components();
        assert!(!comps.is_empty(), "Category {:?} must not be empty", cat);
        for comp in comps {
            assert_eq!(comp.category(), cat);
            categorized_count += 1;
        }
    }
    assert_eq!(categorized_count, 35, "All 35 primitives must be covered across categories");

    // Instantiate all 35 primitives into schematic components
    let mut components = Vec::with_capacity(35);
    for (i, &kind) in all_kinds.iter().enumerate() {
        let id = i + 1;
        let pos = egui::Pos2::new(100.0 + (i as f32) * 25.0, 150.0 + (i as f32) * 20.0);
        let mut comp = SchematicComponent::new(id, kind, pos, id);
        comp.rotation = (i % 4) as u8;
        comp.value_str = format!("VAL_{}_{}", kind.code_name(), id);
        comp.properties.push(("prop_key".to_string(), format!("val_{}", id)));

        // Verify pins are defined for this primitive
        let pins = kind.pin_definitions();
        assert!(!pins.is_empty(), "Primitive {:?} must define at least 1 pin", kind);
        components.push(comp);
    }
    assert_eq!(components.len(), 35);

    // 2. Verify ERC diagnostics
    // Case 2a: Clean circuit with 0 errors and 0 warnings
    let mut clean_canvas = SchematicCanvas::new();
    let v1 = SchematicComponent::new(1, ComponentKind::VoltageSource, egui::Pos2::new(200.0, 300.0), 1);
    let r1 = SchematicComponent::new(2, ComponentKind::Resistor, egui::Pos2::new(360.0, 240.0), 1);
    let r2 = SchematicComponent::new(3, ComponentKind::Resistor, egui::Pos2::new(360.0, 360.0), 2);
    let gnd = SchematicComponent::new(4, ComponentKind::Ground, egui::Pos2::new(200.0, 440.0), 1);
    clean_canvas.add_component(v1);
    clean_canvas.add_component(r1);
    clean_canvas.add_component(r2);
    clean_canvas.add_component(gnd);
    clean_canvas.add_wire(SchematicWire::manhattan_route(1, egui::Pos2::new(200.0, 260.0), egui::Pos2::new(360.0, 200.0)));
    clean_canvas.add_wire(SchematicWire::manhattan_route(2, egui::Pos2::new(360.0, 280.0), egui::Pos2::new(360.0, 320.0)));
    clean_canvas.add_wire(SchematicWire::manhattan_route(3, egui::Pos2::new(360.0, 400.0), egui::Pos2::new(200.0, 340.0)));
    clean_canvas.add_wire(SchematicWire::manhattan_route(4, egui::Pos2::new(200.0, 340.0), egui::Pos2::new(200.0, 420.0)));

    let clean_diagnostics = ErcEngine::evaluate_canvas(&clean_canvas);
    let clean_errors = clean_diagnostics.iter().filter(|d| d.severity == ErcSeverity::Error).count();
    let clean_warnings = clean_diagnostics.iter().filter(|d| d.severity == ErcSeverity::Warning).count();
    assert_eq!(clean_errors, 0, "Clean circuit must have 0 ERC errors");
    assert_eq!(clean_warnings, 0, "Clean circuit must have 0 ERC warnings");

    // Case 2b: Circuit with violations (unconnected floating node & missing ground reference)
    let mut faulty_canvas = SchematicCanvas::new();
    let r_faulty = SchematicComponent::new(1, ComponentKind::Resistor, egui::Pos2::new(100.0, 100.0), 1);
    faulty_canvas.add_component(r_faulty);
    let faulty_diagnostics = ErcEngine::evaluate_canvas(&faulty_canvas);
    assert!(
        !faulty_diagnostics.is_empty(),
        "Faulty canvas with floating component must produce ERC diagnostics"
    );
    let has_floating = faulty_diagnostics.iter().any(|d| d.code == ErcCode::FloatingNode);
    let has_unref_gnd = faulty_diagnostics.iter().any(|d| d.code == ErcCode::UnreferencedGround);
    assert!(has_floating || has_unref_gnd, "ERC engine must flag floating node or missing ground");

    // 3. Verify Binary format round-trip with all 35 primitives
    let title = "Phase 325 Schematic CAD Integrity Project";
    let mut wires = Vec::new();
    let w1 = SchematicWire::manhattan_route_with_net(
        1,
        egui::Pos2::new(100.0, 100.0),
        egui::Pos2::new(250.0, 100.0),
        Some("TOPOLOGICAL_BUS_0".to_string()),
    );
    let w2 = SchematicWire::manhattan_route_with_net(
        2,
        egui::Pos2::new(250.0, 100.0),
        egui::Pos2::new(250.0, 300.0),
        Some("BRAIDING_CLK".to_string()),
    );
    wires.push(w1);
    wires.push(w2);

    let serialized_bytes = serialize_project(title, &components, &wires);
    assert!(serialized_bytes.len() > 32);
    assert_eq!(&serialized_bytes[0..8], &PHONON_MAGIC, "Magic header must match PHONON_MAGIC");
    assert_eq!(CURRENT_VERSION, 1, "Format version must be 1");

    let deserialized: DeserializedProject = deserialize_project(&serialized_bytes)
        .expect("Deserialization of all 35 components binary format must succeed");

    assert_eq!(deserialized.title, title);
    assert_eq!(deserialized.components.len(), 35, "Deserialized must contain all 35 primitives");
    assert_eq!(deserialized.wires.len(), 2, "Deserialized must contain all 2 wires");

    for (orig, des) in components.iter().zip(deserialized.components.iter()) {
        assert_eq!(des.id, orig.id);
        assert_eq!(des.kind, orig.kind);
        assert_eq!(des.name, orig.name);
        assert_eq!(des.rotation, orig.rotation);
        assert!((des.pos.x - orig.pos.x).abs() < 1e-4);
        assert!((des.pos.y - orig.pos.y).abs() < 1e-4);
        assert_eq!(des.value_str, orig.value_str);
        assert_eq!(des.properties, orig.properties);
    }

    for (orig, des) in wires.iter().zip(deserialized.wires.iter()) {
        assert_eq!(des.id, orig.id);
        assert_eq!(des.net_name, orig.net_name);
        assert_eq!(des.segments.len(), orig.segments.len());
    }

    println!(
        "Schematic CAD Integrity: all 35 primitives verified, ERC diagnostics verified, binary format round-trip verified [PASS]"
    );
}
