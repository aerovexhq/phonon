//! Integration tests for Phase 44: Neuromorphic Reservoir Benchmarks & Chaotic Forecasting.

use phonon_models::memristor::reservoir::{MemristiveReservoir, MemristorTechnology};
use phonon_solver::neuromorphic::{
    generate_lorenz63, generate_mackey_glass, generate_narma10, NeuromorphicBenchmarkRunner,
    ReservoirSolver,
};

#[test]
fn test_mackey_glass_chaotic_forecasting_nrmse() {
    let mg_series = generate_mackey_glass(900, 17);
    assert_eq!(mg_series.len(), 900);

    let inputs: Vec<Vec<f64>> = mg_series.iter().map(|&x| vec![x]).collect();
    let targets: Vec<Vec<f64>> = mg_series[1..].iter().map(|&x| vec![x]).collect();
    let inputs_slice = &inputs[..targets.len()];

    let mut reservoir =
        MemristiveReservoir::new(60, 1, MemristorTechnology::FilamentaryRram, 0.92, 0.4, 42);

    let (readout, nrmse, predictions) = ReservoirSolver::train_and_evaluate(
        &mut reservoir,
        inputs_slice,
        &targets,
        100,
        0.75,
        1e-4,
    )
    .expect("Training failed");

    assert_eq!(readout.num_outputs, 1);
    assert!(!predictions.is_empty());
    assert!(
        nrmse < 0.05,
        "Mackey-Glass NRMSE must be < 0.05, got {nrmse}"
    );
}

#[test]
fn test_lorenz63_strange_attractor_reconstruction() {
    let lorenz_data = generate_lorenz63(600, 0.02);
    assert_eq!(lorenz_data.len(), 600);

    // Multi-dimensional input [x(t), y(t), z(t)] and multi-dimensional target [x(t+1), y(t+1), z(t+1)]
    let inputs: Vec<Vec<f64>> = lorenz_data
        .iter()
        .map(|s| vec![s[0] * 0.05, s[1] * 0.05, s[2] * 0.05])
        .collect();
    let targets: Vec<Vec<f64>> = lorenz_data[1..]
        .iter()
        .map(|s| vec![s[0] * 0.05, s[1] * 0.05, s[2] * 0.05])
        .collect();
    let inputs_slice = &inputs[..targets.len()];

    let mut reservoir = MemristiveReservoir::new(
        70,
        3,
        MemristorTechnology::PhaseChangeMemory,
        0.88,
        0.6,
        123,
    );

    let (readout, nrmse, _) = ReservoirSolver::train_and_evaluate(
        &mut reservoir,
        inputs_slice,
        &targets,
        100,
        0.75,
        1e-3,
    )
    .expect("Lorenz-63 training failed");

    assert_eq!(readout.num_outputs, 3);
    assert!(
        nrmse < 1.0,
        "Lorenz-63 3D attractor NRMSE should be < 1.0, got {nrmse}"
    );
}

#[test]
fn test_narma10_system_identification() {
    let (u, y) = generate_narma10(800, 43);
    assert_eq!(u.len(), 800);
    assert_eq!(y.len(), 800);

    let inputs: Vec<Vec<f64>> = u.iter().map(|&val| vec![val]).collect();
    let targets: Vec<Vec<f64>> = y.iter().map(|&val| vec![val]).collect();

    let mut reservoir =
        MemristiveReservoir::new(64, 1, MemristorTechnology::PhaseChangeMemory, 0.88, 0.5, 45);

    let (_, nrmse, _) =
        ReservoirSolver::train_and_evaluate(&mut reservoir, &inputs, &targets, 100, 0.75, 1e-3)
            .expect("NARMA-10 training failed");

    assert!(
        nrmse < 1.0,
        "NARMA-10 NRMSE should be < 1.0 for 64 nodes, got {nrmse}"
    );
}

#[test]
fn test_comparative_neuromorphic_benchmark_report() {
    let report = NeuromorphicBenchmarkRunner::run_benchmark(800, 42);

    assert!(
        report.benchmarks_passed,
        "All neuromorphic benchmark criteria must pass"
    );
    assert!(
        report.memristive_edp_js < report.dsp_edp_js,
        "Memristive EDP ({:e}) must be lower than DSP EDP ({:e})",
        report.memristive_edp_js,
        report.dsp_edp_js
    );
    assert!(
        report.dsp_edp_js < report.gpu_edp_js,
        "DSP EDP ({:e}) must be lower than GPU EDP ({:e})",
        report.dsp_edp_js,
        report.gpu_edp_js
    );
    assert!(
        report.memristive_energy_per_inf_j < 1.0e-9,
        "Memristive energy per inference ({:e} J) must be < 1 nJ",
        report.memristive_energy_per_inf_j
    );
    assert!(
        report.mackey_glass_nrmse < 0.08,
        "Mackey-Glass NRMSE ({}) must be < 0.08",
        report.mackey_glass_nrmse
    );
    assert!(
        report.memory_capacity > 5.0,
        "Memory capacity ({}) must be > 5.0",
        report.memory_capacity
    );
    assert!(
        report.training_throughput_solves_per_sec > 0.0,
        "Training throughput must be positive"
    );
}
