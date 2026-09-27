//! Telemetry streaming writers for CSV and JSON Lines (JSONL).

use crate::error::CliError;
use std::io::Write;

/// Output formats supported by the telemetry subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Csv,
    Jsonl,
}

/// Common trait for real-time simulation telemetry stream writers.
pub trait TelemetryWriter {
    fn write_header(&mut self, headers: &[&str]) -> Result<(), CliError>;
    fn write_row(&mut self, values: &[f64]) -> Result<(), CliError>;
    fn flush(&mut self) -> Result<(), CliError>;
}

/// High-throughput streaming CSV telemetry writer.
pub struct CsvTelemetryWriter<W: Write> {
    writer: W,
}

impl<W: Write> CsvTelemetryWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: Write> TelemetryWriter for CsvTelemetryWriter<W> {
    fn write_header(&mut self, headers: &[&str]) -> Result<(), CliError> {
        let line = headers.join(",");
        writeln!(self.writer, "{line}")?;
        Ok(())
    }

    fn write_row(&mut self, values: &[f64]) -> Result<(), CliError> {
        let mut first = true;
        for &val in values {
            if !first {
                write!(self.writer, ",")?;
            }
            write!(self.writer, "{:.10e}", val)?;
            first = false;
        }
        writeln!(self.writer)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), CliError> {
        self.writer.flush()?;
        Ok(())
    }
}

/// High-throughput streaming JSON Lines (JSONL) telemetry writer.
pub struct JsonLinesTelemetryWriter<W: Write> {
    writer: W,
    headers: Vec<String>,
}

impl<W: Write> JsonLinesTelemetryWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            headers: Vec::new(),
        }
    }
}

impl<W: Write> TelemetryWriter for JsonLinesTelemetryWriter<W> {
    fn write_header(&mut self, headers: &[&str]) -> Result<(), CliError> {
        self.headers = headers.iter().map(|s| s.to_string()).collect();
        Ok(())
    }

    fn write_row(&mut self, values: &[f64]) -> Result<(), CliError> {
        write!(self.writer, "{{")?;
        for (i, &val) in values.iter().enumerate() {
            let key = self.headers.get(i).map(|s| s.as_str()).unwrap_or("unknown");
            if i > 0 {
                write!(self.writer, ", ")?;
            }
            write!(self.writer, "\"{key}\": {:.10e}", val)?;
        }
        writeln!(self.writer, "}}")?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), CliError> {
        self.writer.flush()?;
        Ok(())
    }
}
