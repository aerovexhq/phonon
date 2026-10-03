#![deny(unsafe_code)]

//! Verification test suite for Phonon Visual Studio Monte Carlo Yield Harvester and Latin Hypercube Dialog.

use phonon_core::CircuitGraph;
use phonon_gui::widgets::monte_carlo_dialog::MonteCarloYieldDialog;
use phonon_solver::mna::non_linear_solver::ModelContext;
use phonon_solver::monte_carlo::SamplingMode;
use phonon_solver::sensitivity::ObjectiveKind;
use phonon_solver::transient::TransientOptions;

#[test]
fn test_monte_carlo_dialog_initialization_defaults() {
    let dialog = MonteCarloYieldDialog::new();

    assert!(!dialog.is_open, "Dialog should be closed by default");
    assert_eq!(dialog.sample_count, 1000);
    assert_eq!(dialog.sampling_mode, SamplingMode::LatinHypercube);
    assert!(dialog.parameters.is_empty());
    assert_eq!(dialog.selected_objective_idx, 0);
    assert_eq!(dialog.target_node, 1);
    assert!(dialog.enable_lsl);
    assert_eq!(dialog.lsl_value, 2.0);
    assert!(dialog.enable_usl);
    assert_eq!(dialog.usl_value, 3.0);
    assert!(dialog.report.is_none());
    assert!(!dialog.run_requested);

    let spec = dialog.current_spec();
    assert_eq!(spec.lsl, Some(2.0));
    assert_eq!(spec.usl, Some(3.0));

    let obj = dialog.current_objective();
    assert_eq!(obj, ObjectiveKind::TerminalVoltage { node: 1 });
}

#[test]
fn test_parameter_auto_population_from_schematic_canvas() {
    let mut dialog = MonteCarloYieldDialog::new();

    let mut graph = CircuitGraph::new();
    let _ = graph.add_voltage_source("V1", "VIN", "0", 5.0);
    let _ = graph.add_resistor("R1", "VIN", "VMID", 1000.0);
    let _ = graph.add_resistor("R2", "VMID", "0", 2000.0);
    let _ = graph.add_capacitor("C1", "VMID", "0", 1.0e-6, Some(0.0));
    let _ = graph.add_inductor("L1", "VMID", "VOUT", 1.0e-3, Some(0.0));
    let _ = graph.add_mosfet("M1", "VOUT", "VMID", "0", "0");

    dialog.extract_parameters_from_graph(&graph);

    // Resistors R1, R2 + Capacitor C1 + Inductor L1 + Mosfet M1
    assert_eq!(dialog.parameters.len(), 5);

    let names: Vec<String> = dialog.parameters.iter().map(|p| p.param.name.clone()).collect();
    assert!(names.contains(&"R1".to_string()));
    assert!(names.contains(&"R2".to_string()));
    assert!(names.contains(&"C1".to_string()));
    assert!(names.contains(&"L1".to_string()));
    assert!(names.contains(&"M1_W".to_string()));

    // Verify all active parameters are enabled
    let active = dialog.active_parameters();
    assert_eq!(active.len(), 5);
}

#[test]
fn test_sweep_execution_and_yield_metrics_computation() {
    let mut dialog = MonteCarloYieldDialog::new();

    let mut graph = CircuitGraph::new();
    let _ = graph.add_voltage_source("V1", "VIN", "0", 5.0);
    let _ = graph.add_resistor("R1", "VIN", "VOUT", 1000.0);
    let _ = graph.add_capacitor("C1", "VOUT", "0", 1.0e-5, Some(0.0));

    let context = ModelContext::default();
    let mut options = TransientOptions::default();
    options.tstop = 0.005;
    options.tstep = 0.0001;
    options.uic = true;

    dialog.target_node = 2; // VOUT
    dialog.sample_count = 50; // Small batch for swift test
    dialog.lsl_value = 1.0;
    dialog.usl_value = 5.0;

    dialog.run_sweep(&graph, &context, &options);

    assert!(dialog.report.is_some(), "Report must be populated after sweep");
    let rep = dialog.report.as_ref().unwrap();

    assert_eq!(rep.metrics.sample_count, 50);
    assert!(rep.metrics.mean > 0.0);
    assert!(rep.metrics.variance >= 0.0);
    assert!(rep.metrics.yield_percentage > 0.0 && rep.metrics.yield_percentage <= 100.0);
    assert!(rep.passed_count + rep.failed_count == 50);
    assert!(dialog.status_msg.contains("Monte Carlo sweep completed"));
}

#[test]
fn test_histogram_and_corner_inspection() {
    let mut dialog = MonteCarloYieldDialog::new();

    // Populate parameters
    let mut graph = CircuitGraph::new();
    let _ = graph.add_resistor("R1", "1", "0", 100.0);
    let _ = graph.add_resistor("R2", "2", "0", 200.0);
    dialog.extract_parameters_from_graph(&graph);

    dialog.sample_count = 300;
    dialog.lsl_value = 45.0;
    dialog.usl_value = 55.0;

    // Use fast custom companion model evaluation
    dialog.run_sweep_custom(|params| {
        let r1 = params[0];
        let r2 = params[1];
        (r1 + r2) * 0.16666666666666666 + 0.5
    });

    assert!(dialog.report.is_some());
    let rep = dialog.report.as_ref().unwrap();

    // 1. Histogram verification
    assert!(!rep.histogram.is_empty());
    let total_counts: usize = rep.histogram.iter().map(|b| b.count).sum();
    assert_eq!(total_counts, 300);

    for bin in &rep.histogram {
        assert!(bin.bin_max >= bin.bin_min);
        assert!(bin.probability_density >= 0.0);
    }

    // 2. Corner verification
    let corners = &rep.corners;
    assert!(
        corners.minus_3sigma <= corners.nominal,
        "Theoretical -3sigma ({}) must be <= nominal ({})",
        corners.minus_3sigma,
        corners.nominal
    );
    assert!(
        corners.nominal <= corners.plus_3sigma,
        "Nominal ({}) must be <= +3sigma ({})",
        corners.nominal,
        corners.plus_3sigma
    );

    assert!(
        corners.empirical_minus_3sigma <= corners.empirical_nominal,
        "Empirical -3sigma must be <= empirical nominal"
    );
    assert!(
        corners.empirical_nominal <= corners.empirical_plus_3sigma,
        "Empirical nominal must be <= empirical +3sigma"
    );

    // 3. Empirical CDF verification
    assert_eq!(rep.empirical_cdf.len(), 300);
    for w in rep.empirical_cdf.windows(2) {
        assert!(w[0].value <= w[1].value);
        assert!(w[0].cumulative_probability <= w[1].cumulative_probability);
    }
}
