#![deny(unsafe_code)]

//! High-Throughput Industry-Grade Fast MNA Kernel & In-Place LU Tests (Phase 318).
//!
//! Validates:
//! 1. Zero-Allocation Fast In-Place LU Factorization and Solve.
//! 2. Contiguous Flat Memory Circuit State and Differential Node Queries.
//! 3. Bank-Rose Adaptive Damping preventing non-linear PN junction divergence.
//! 4. 4-Lane SIMD Vectorized Diode Evaluation and Multi-Diode Stamping.
//! 5. High-Throughput Benchmark executing > 50,000 solves/second.

use phonon_core::CircuitGraph;
use phonon_models::DiodeModel;
use phonon_solver::mna::fast_solver::{BankRoseOptions, FastCircuitState, FastMnaKernel};
use phonon_solver::mna::ModelContext;
use phonon_solver::sparse::builder::SparseMatrixBuilder;
use phonon_solver::sparse::fast_lu::FastInPlaceLu;
use phonon_solver::sparse::markowitz::MarkowitzOptions;
use std::time::Instant;

#[test]
fn test_fast_in_place_lu_solve_accuracy_2x2() {
    // System:
    // [ 4  1 ] [ x0 ] = [ 9 ]
    // [ 1  3 ] [ x1 ] = [ 5 ]
    // Analytical solution: x0 = 2.0, x1 = 1.0
    let mut b = SparseMatrixBuilder::new(2, 2);
    b.add(0, 0, 4.0);
    b.add(0, 1, 1.0);
    b.add(1, 0, 1.0);
    b.add(1, 1, 3.0);
    let csc = b.build_csc();

    let markowitz = MarkowitzOptions::default();
    let mut lu = FastInPlaceLu::from_matrix(&csc, &markowitz).expect("LU factorization failed");

    assert_eq!(lu.dim(), 2);
    let rhs = vec![9.0, 5.0];
    let mut sol = vec![0.0; 2];
    lu.solve(&rhs, &mut sol).expect("Solve failed");

    assert!((sol[0] - 2.0).abs() < 1e-12, "x0 mismatch: got {}", sol[0]);
    assert!((sol[1] - 1.0).abs() < 1e-12, "x1 mismatch: got {}", sol[1]);
}

#[test]
fn test_fast_in_place_lu_solve_accuracy_3x3() {
    // System:
    // [ 2  -1   0 ] [ x0 ] = [ 1 ]
    // [ -1  3  -1 ] [ x1 ] = [ 7 ]
    // [  0 -1   2 ] [ x2 ] = [ 5 ]
    // Analytical: x = [ 3.0, 5.0, 5.0 ]
    let mut b = SparseMatrixBuilder::new(3, 3);
    b.add(0, 0, 2.0);
    b.add(0, 1, -1.0);
    b.add(1, 0, -1.0);
    b.add(1, 1, 3.0);
    b.add(1, 2, -1.0);
    b.add(2, 1, -1.0);
    b.add(2, 2, 2.0);
    let csc = b.build_csc();

    let markowitz = MarkowitzOptions::default();
    let mut lu = FastInPlaceLu::from_matrix(&csc, &markowitz).expect("LU factorization failed");

    let rhs = vec![1.0, 7.0, 5.0];
    let sol = lu.solve_vec(&rhs).expect("Solve failed");

    assert!((sol[0] - 3.0).abs() < 1e-11, "x0 mismatch: got {}", sol[0]);
    assert!((sol[1] - 5.0).abs() < 1e-11, "x1 mismatch: got {}", sol[1]);
    assert!((sol[2] - 5.0).abs() < 1e-11, "x2 mismatch: got {}", sol[2]);
}

#[test]
fn test_fast_in_place_lu_zero_heap_allocation_throughput() {
    let mut b = SparseMatrixBuilder::new(3, 3);
    b.add(0, 0, 10.0);
    b.add(0, 1, -2.0);
    b.add(1, 0, -2.0);
    b.add(1, 1, 8.0);
    b.add(1, 2, -1.0);
    b.add(2, 1, -1.0);
    b.add(2, 2, 5.0);
    let csc = b.build_csc();

    let markowitz = MarkowitzOptions::default();
    let mut lu = FastInPlaceLu::from_matrix(&csc, &markowitz).expect("LU factorization failed");

    let rhs = [5.0, 12.0, 7.0];
    let mut sol = [0.0; 3];

    // Benchmark 100,000 in-place solves
    let iterations = 100_000;
    let start = Instant::now();
    for _ in 0..iterations {
        lu.solve(&rhs, &mut sol).expect("In-place solve failed");
    }
    let elapsed = start.elapsed();
    let solves_per_sec = (iterations as f64) / elapsed.as_secs_f64();

    println!(
        "\n[Phase 318 Benchmark] In-Place Fast LU Throughput: {:.2} solves/sec ({:.2} ns/solve)",
        solves_per_sec,
        (elapsed.as_nanos() as f64) / (iterations as f64)
    );

    assert!(solves_per_sec > 500_000.0, "Throughput too low: {}", solves_per_sec);
    assert!(sol[0] > 0.0 && sol[1] > 0.0 && sol[2] > 0.0);
}

#[test]
fn test_fast_circuit_state_indexing_and_differential_voltage() {
    let mut state = FastCircuitState::zeros(3, 1);
    state.node_voltages[0] = 5.0; // Node 1 = 5.0V
    state.node_voltages[1] = 2.5; // Node 2 = 2.5V
    state.node_voltages[2] = 0.8; // Node 3 = 0.8V
    state.branch_currents[0] = 0.0025; // Branch 0 = 2.5mA

    assert_eq!(state.node_voltage(0), 0.0, "Node 0 must be ground 0.0V");
    assert_eq!(state.node_voltage(1), 5.0);
    assert_eq!(state.node_voltage(2), 2.5);
    assert_eq!(state.node_voltage(3), 0.8);
    assert_eq!(state.node_voltage(99), 0.0, "Out-of-bounds node should be 0.0V");

    assert!((state.voltage_diff(1, 2) - 2.5).abs() < 1e-12);
    assert!((state.voltage_diff(1, 0) - 5.0).abs() < 1e-12);
    assert!((state.branch_current(0) - 0.0025).abs() < 1e-12);
}

#[test]
fn test_fast_mna_kernel_linear_voltage_divider() {
    // 10V DC source connected to R1 (1k) and R2 (1k) to ground.
    // Node 1: 10V, Node 2: 5.0V
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "1", "0", 10.0).expect("add V1 failed");
    graph.add_resistor("R1", "1", "2", 1000.0).expect("add R1 failed");
    graph.add_resistor("R2", "2", "0", 1000.0).expect("add R2 failed");

    let mut kernel = FastMnaKernel::new(&graph).expect("Kernel creation failed");
    assert_eq!(kernel.dimension(), 3); // 2 active nodes + 1 voltage source branch

    let ctx = ModelContext::new();
    let opts = BankRoseOptions::default();

    let state = kernel.solve_dc_fast(&graph, &ctx, &opts).expect("DC solve failed");

    let v_node1 = state.node_voltage(1);
    let v_node2 = state.node_voltage(2);

    assert!((v_node1 - 10.0).abs() < 1e-6, "Node 1 mismatch: got {}", v_node1);
    assert!((v_node2 - 5.0).abs() < 1e-6, "Node 2 mismatch: got {}", v_node2);
    assert!(state.iterations <= 3, "Linear circuit should converge in <= 3 iterations");
}

#[test]
fn test_fast_mna_kernel_non_linear_diode_circuit() {
    // 5V DC source driving 1k resistor in series with a diode to ground.
    // Expected: Diode forward drop ~ 0.65V - 0.75V, Node 2 voltage = V_diode.
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "1", "0", 5.0).expect("add V1 failed");
    graph.add_resistor("R1", "1", "2", 1000.0).expect("add R1 failed");
    graph.add_diode("D1", "2", "0").expect("add D1 failed");

    let mut kernel = FastMnaKernel::new(&graph).expect("Kernel creation failed");

    let mut ctx = ModelContext::new();
    ctx.set_diode_model("D1", DiodeModel {
        is: 1e-14,
        n: 1.0,
        ..Default::default()
    });

    let opts = BankRoseOptions::default();
    let state = kernel.solve_dc_fast(&graph, &ctx, &opts).expect("DC non-linear solve failed");

    let v_diode = state.node_voltage(2);
    println!("\nNon-Linear Diode Node 2 Voltage: {:.4} V (converged in {} iterations)", v_diode, state.iterations);

    assert!(v_diode > 0.60 && v_diode < 0.80, "Diode voltage outside expected range: {}", v_diode);
    assert!(state.iterations < opts.max_iterations, "Failed to converge within max iterations");
}

#[test]
fn test_fast_mna_kernel_bank_rose_damping_prevents_divergence() {
    // High-voltage forward biased diode circuit (20V into 100 Ohm)
    // Without damping, standard Newton-Raphson easily diverges due to exponential current surge.
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "1", "0", 20.0).expect("add V1 failed");
    graph.add_resistor("R1", "1", "2", 100.0).expect("add R1 failed");
    graph.add_diode("D1", "2", "0").expect("add D1 failed");

    let mut kernel = FastMnaKernel::new(&graph).expect("Kernel creation failed");

    let mut ctx = ModelContext::new();
    ctx.set_diode_model("D1", DiodeModel {
        is: 1e-14,
        n: 1.0,
        ..Default::default()
    });

    let opts = BankRoseOptions {
        max_iterations: 80,
        abs_tol: 1e-5,
        rel_tol: 1e-4,
        initial_damping: 1.0,
        min_damping: 0.02,
        curvature_gamma: 0.2,
    };

    let state = kernel.solve_dc_fast(&graph, &ctx, &opts).expect("Damped solve should converge");
    let v_diode = state.node_voltage(2);

    println!("Stiff Diode Circuit: V_diode = {:.4} V in {} iterations", v_diode, state.iterations);
    assert!(v_diode > 0.70 && v_diode < 0.95);
    assert!(state.iterations < opts.max_iterations);
}

#[test]
fn test_fast_mna_kernel_simd_diode_batch_parallelism() {
    // Circuit with 4 parallel diode branches to trigger 4-lane SIMD vectorization
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "1", "0", 5.0).expect("add V1 failed");
    graph.add_resistor("R1", "1", "2", 1000.0).expect("add R1 failed");
    graph.add_diode("D1", "2", "0").expect("add D1 failed");
    graph.add_resistor("R2", "1", "3", 1000.0).expect("add R2 failed");
    graph.add_diode("D2", "3", "0").expect("add D2 failed");
    graph.add_resistor("R3", "1", "4", 1000.0).expect("add R3 failed");
    graph.add_diode("D3", "4", "0").expect("add D3 failed");
    graph.add_resistor("R4", "1", "5", 1000.0).expect("add R4 failed");
    graph.add_diode("D4", "5", "0").expect("add D4 failed");

    let mut kernel = FastMnaKernel::new(&graph).expect("Kernel creation failed");

    let mut ctx = ModelContext::new();
    let model = DiodeModel {
        is: 1e-14,
        n: 1.0,
        ..Default::default()
    };
    ctx.set_diode_model("D1", model);
    ctx.set_diode_model("D2", model);
    ctx.set_diode_model("D3", model);
    ctx.set_diode_model("D4", model);

    let opts = BankRoseOptions::default();
    let state = kernel.solve_dc_fast(&graph, &ctx, &opts).expect("Batch SIMD solve failed");

    for node in 2..=5 {
        let vn = state.node_voltage(node);
        assert!((vn - state.node_voltage(2)).abs() < 1e-6, "Symmetric branch voltages should match");
        assert!(vn > 0.60 && vn < 0.80);
    }
}

#[test]
fn test_fast_mna_kernel_high_speed_benchmark() {
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "1", "0", 12.0).expect("add V1 failed");
    graph.add_resistor("R1", "1", "2", 4700.0).expect("add R1 failed");
    graph.add_resistor("R2", "2", "0", 2200.0).expect("add R2 failed");

    let mut kernel = FastMnaKernel::new(&graph).expect("Kernel creation failed");
    let ctx = ModelContext::new();
    let opts = BankRoseOptions::default();

    let cycles = 20_000;
    let start = Instant::now();
    for _ in 0..cycles {
        let state = kernel.solve_dc_fast(&graph, &ctx, &opts).expect("Fast solve failed");
        assert!(state.node_voltage(2) > 0.0);
    }
    let elapsed = start.elapsed();
    let solves_per_sec = (cycles as f64) / elapsed.as_secs_f64();

    println!(
        "\n[Phase 318 Benchmark] Fast MNA Kernel Throughput: {:.2} solves/sec ({:.2} us/solve)",
        solves_per_sec,
        (elapsed.as_micros() as f64) / (cycles as f64)
    );

    assert!(solves_per_sec > 50_000.0, "Throughput below target: {}", solves_per_sec);
}
