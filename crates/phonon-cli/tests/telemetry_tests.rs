use phonon_cli::telemetry::{CsvTelemetryWriter, JsonLinesTelemetryWriter, TelemetryWriter};

#[test]
fn test_csv_telemetry_writer() {
    let mut buffer = Vec::new();
    {
        let mut writer = CsvTelemetryWriter::new(&mut buffer);
        writer
            .write_header(&["time", "V(out)", "I(V1)"])
            .expect("header failed");
        writer.write_row(&[0.0, 5.0, -0.005]).expect("row 1 failed");
        writer
            .write_row(&[1e-3, 4.8, -0.0048])
            .expect("row 2 failed");
        writer.flush().expect("flush failed");
    }

    let output = String::from_utf8(buffer).expect("valid utf8");
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "time,V(out),I(V1)");
    assert!(lines[1].contains("5.0000000000e0"));
    assert!(lines[2].contains("1.0000000000e-3"));
}

#[test]
fn test_jsonl_telemetry_writer() {
    let mut buffer = Vec::new();
    {
        let mut writer = JsonLinesTelemetryWriter::new(&mut buffer);
        writer
            .write_header(&["time", "V(out)"])
            .expect("header failed");
        writer.write_row(&[0.0, 3.3]).expect("row 1 failed");
        writer.write_row(&[1.0, 3.29]).expect("row 2 failed");
        writer.flush().expect("flush failed");
    }

    let output = String::from_utf8(buffer).expect("valid utf8");
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("\"time\": 0.0000000000e0"));
    assert!(lines[0].contains("\"V(out)\": 3.3000000000e0"));
    assert!(lines[1].contains("\"time\": 1.0000000000e0"));
}
