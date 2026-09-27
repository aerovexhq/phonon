use phonon_cli::commands::execute_monte_carlo;
use phonon_cli::telemetry::OutputFormat;
use std::io::Write;

#[test]
fn test_parallel_monte_carlo_tolerance() {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let netlist_path = std::env::temp_dir().join(format!("phonon_mc_{test_id}.cir"));
    let output_path = std::env::temp_dir().join(format!("phonon_mc_out_{test_id}.csv"));

    let netlist_src = r#"
* Monte Carlo Voltage Divider
V1 in 0 10.0
R1 in out 1000
R2 out 0 1000
.OP
"#;
    {
        let mut file = std::fs::File::create(&netlist_path).expect("create netlist");
        file.write_all(netlist_src.as_bytes())
            .expect("write netlist");
    }

    let stats = execute_monte_carlo(
        &netlist_path,
        200,
        0.05, // 5% tolerance
        12345,
        OutputFormat::Csv,
        Some(&output_path),
    )
    .expect("Monte carlo execution failed");

    // Output variables: V(in), V(out), I(V1)
    // V(out) is index 1
    let v_out_mean = stats.mean[1];
    let v_out_std = stats.std_dev[1];
    let v_out_min = stats.min[1];
    let v_out_max = stats.max[1];

    // Mean should be near 5.0 V
    assert!(
        (v_out_mean - 5.0).abs() < 0.05,
        "Expected mean near 5.0, got {v_out_mean}"
    );

    // Standard deviation should be positive and small (~0.12 V for 5% resistors)
    assert!(v_out_std > 0.01 && v_out_std < 0.3, "Std was {v_out_std}");

    // Min and max should be bounded within tolerance limits
    assert!(v_out_min > 4.5 && v_out_max < 5.5);

    // Verify CSV file content
    let csv = std::fs::read_to_string(&output_path).expect("read mc csv");
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 201); // 1 header + 200 samples
    assert_eq!(lines[0], "sample_id,V(in),V(out),I(V1)");
}

#[test]
fn test_monte_carlo_reproducibility() {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let netlist_path = std::env::temp_dir().join(format!("phonon_mc_repro_{test_id}.cir"));

    let netlist_src = r#"
V1 in 0 5.0
R1 in out 100
R2 out 0 100
.OP
"#;
    std::fs::write(&netlist_path, netlist_src).expect("write netlist");

    let stats1 =
        execute_monte_carlo(&netlist_path, 50, 0.1, 999, OutputFormat::Jsonl, None).expect("run 1");
    let stats2 =
        execute_monte_carlo(&netlist_path, 50, 0.1, 999, OutputFormat::Jsonl, None).expect("run 2");

    assert_eq!(stats1.mean, stats2.mean);
    assert_eq!(stats1.std_dev, stats2.std_dev);
}
