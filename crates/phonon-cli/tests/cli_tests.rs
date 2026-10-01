#![deny(unsafe_code)]

use phonon_cli::commands::{execute_run, execute_validate};
use phonon_cli::telemetry::OutputFormat;

#[test]
fn test_validate_valid_circuit() {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let netlist_path = std::env::temp_dir().join(format!("phonon_val_ok_{test_id}.cir"));

    let netlist_src = r#"
* Simple Valid Divider
V1 in 0 5.0
R1 in out 1000
R2 out 0 1000
.OP
"#;
    std::fs::write(&netlist_path, netlist_src).expect("write netlist");

    let report = execute_validate(&netlist_path).expect("validation should succeed");
    assert!(report.is_valid);
    assert_eq!(report.title, "Simple Valid Divider");
    assert_eq!(report.active_nodes, 2); // in, out
    assert_eq!(report.total_branches, 1); // V1
    assert_eq!(report.total_components, 3);
}

#[test]
fn test_validate_empty_circuit_fails() {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let netlist_path = std::env::temp_dir().join(format!("phonon_val_empty_{test_id}.cir"));

    let netlist_src = r#"
* Empty Netlist
.OP
"#;
    std::fs::write(&netlist_path, netlist_src).expect("write netlist");

    let res = execute_validate(&netlist_path);
    assert!(res.is_err(), "Expected empty circuit error");
}

#[test]
fn test_validate_voltage_source_loop_fails() {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let netlist_path = std::env::temp_dir().join(format!("phonon_val_loop_{test_id}.cir"));

    // Two parallel voltage sources form a closed loop of ideal sources
    let netlist_src = r#"
V1 in 0 5.0
V2 in 0 5.0
R1 in 0 1000
.OP
"#;
    std::fs::write(&netlist_path, netlist_src).expect("write netlist");

    let res = execute_validate(&netlist_path);
    assert!(res.is_err(), "Expected voltage source loop error");
}

#[test]
fn test_execute_run_op() {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let netlist_path = std::env::temp_dir().join(format!("phonon_run_op_{test_id}.cir"));
    let output_path = std::env::temp_dir().join(format!("phonon_run_op_{test_id}.csv"));

    let netlist_src = r#"
* OP Test
V1 in 0 12.0
R1 in out 2000
R2 out 0 1000
.OP
"#;
    std::fs::write(&netlist_path, netlist_src).expect("write netlist");

    execute_run(&netlist_path, OutputFormat::Csv, Some(&output_path)).expect("run op failed");

    let csv = std::fs::read_to_string(&output_path).expect("read op csv");
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "V(in),V(out),I(V1)");

    // V(out) should be 12.0 * 1000 / 3000 = 4.0 V
    assert!(lines[1].contains("4.0000000000e0"));
}

#[test]
fn test_execute_run_dc_sweep() {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let netlist_path = std::env::temp_dir().join(format!("phonon_run_dc_{test_id}.cir"));
    let output_path = std::env::temp_dir().join(format!("phonon_run_dc_{test_id}.jsonl"));

    let netlist_src = r#"
* DC Sweep Test
V1 in 0 0.0
R1 in out 100
R2 out 0 100
.DC V1 0.0 2.0 0.5
"#;
    std::fs::write(&netlist_path, netlist_src).expect("write netlist");

    execute_run(&netlist_path, OutputFormat::Jsonl, Some(&output_path)).expect("run dc failed");

    let jsonl = std::fs::read_to_string(&output_path).expect("read dc jsonl");
    let lines: Vec<&str> = jsonl.lines().collect();

    // 0.0, 0.5, 1.0, 1.5, 2.0 -> 5 data points
    assert_eq!(lines.len(), 5);
    assert!(lines[0].contains("\"V1\": 0.0000000000e0"));
    assert!(lines[4].contains("\"V1\": 2.0000000000e0"));
    // At V1 = 2.0, V(out) = 1.0
    assert!(lines[4].contains("\"V(out)\": 1.0000000000e0"));
}

#[test]
fn test_run_transient_rc_csv_and_jsonl() {
    let test_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let netlist_path = std::env::temp_dir().join(format!("phonon_run_tran_{test_id}.cir"));
    let csv_path = std::env::temp_dir().join(format!("phonon_run_tran_{test_id}.csv"));
    let jsonl_path = std::env::temp_dir().join(format!("phonon_run_tran_{test_id}.jsonl"));

    let netlist_src = r#"
* Transient RC Circuit
V1 in 0 5.0
R1 in out 1000
C1 out 0 1u
.TRAN 100u 1m
"#;
    std::fs::write(&netlist_path, netlist_src).expect("write netlist");

    // Run CSV
    execute_run(&netlist_path, OutputFormat::Csv, Some(&csv_path)).expect("run tran csv failed");
    let csv_content = std::fs::read_to_string(&csv_path).expect("read csv");
    let csv_lines: Vec<&str> = csv_content.lines().collect();
    assert!(csv_lines.len() >= 10);
    assert_eq!(csv_lines[0], "time,V(in),V(out),I(V1)");

    // Run JSONL
    execute_run(&netlist_path, OutputFormat::Jsonl, Some(&jsonl_path))
        .expect("run tran jsonl failed");
    let jsonl_content = std::fs::read_to_string(&jsonl_path).expect("read jsonl");
    let jsonl_lines: Vec<&str> = jsonl_content.lines().collect();
    assert!(jsonl_lines.len() >= 10);
    assert!(jsonl_lines[0].contains("\"time\": 0.0000000000e0"));
    assert!(jsonl_lines[0].contains("\"V(in)\": 5.0000000000e0"));
}

#[test]
fn test_phonon_gui_command_parsing() {
    use clap::Parser;
    use phonon_cli::{Cli, Commands};

    let cli = Cli::try_parse_from(["phonon", "gui"]).expect("phonon gui command should parse");
    assert_eq!(cli.command, Some(Commands::Gui));
}

#[test]
fn test_legacy_phonon_ui_command_rejected() {
    use clap::Parser;
    use phonon_cli::Cli;

    let res = Cli::try_parse_from(["phonon", "ui"]);
    assert!(
        res.is_err(),
        "Legacy phonon ui command must be rejected as an unrecognized subcommand"
    );
}
