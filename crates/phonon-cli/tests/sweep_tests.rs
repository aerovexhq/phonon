use phonon_cli::commands::execute_sweep;
use phonon_cli::telemetry::OutputFormat;
use std::io::Write;

#[test]
fn test_parallel_parametric_sweep_resistor() {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let netlist_path = std::env::temp_dir().join(format!("phonon_sweep_{test_id}.cir"));
    let output_path = std::env::temp_dir().join(format!("phonon_sweep_out_{test_id}.csv"));

    let netlist_src = r#"
* Divider Sweep
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

    // Sweep R2 from 500 to 2000 ohms in 5 steps (500, 800, 1100, 1400, 1700, 2000)
    execute_sweep(
        &netlist_path,
        "R2",
        500.0,
        2000.0,
        5,
        OutputFormat::Csv,
        Some(&output_path),
    )
    .expect("sweep execution failed");

    let csv_content = std::fs::read_to_string(&output_path).expect("read sweep output");
    let lines: Vec<&str> = csv_content.lines().collect();

    // 1 header + 6 data rows = 7 lines
    assert_eq!(lines.len(), 7);
    assert_eq!(lines[0], "R2,V(in),V(out),I(V1)");

    // At R2 = 1000 (middle, step 2/3), V(out) should be exactly 5.0 V
    // First step: R2 = 500 -> V(out) = 10 * 500 / 1500 = 3.333 V
    let first_row = lines[1];
    assert!(first_row.starts_with("5.0000000000e2"));
    assert!(first_row.contains("3.3333333333e0"));

    // Last step: R2 = 2000 -> V(out) = 10 * 2000 / 3000 = 6.666 V
    let last_row = lines[6];
    assert!(last_row.starts_with("2.0000000000e3"));
    assert!(last_row.contains("6.6666666667e0"));
}

#[test]
fn test_sweep_non_existent_component_error() {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let netlist_path = std::env::temp_dir().join(format!("phonon_sweep_err_{test_id}.cir"));

    let netlist_src = r#"
V1 in 0 5.0
R1 in 0 100
.OP
"#;
    std::fs::write(&netlist_path, netlist_src).expect("write netlist");

    let res = execute_sweep(
        &netlist_path,
        "R_GHOST",
        10.0,
        100.0,
        5,
        OutputFormat::Csv,
        None,
    );
    assert!(res.is_err());
}
