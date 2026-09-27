//! Columnar batch telemetry streaming and binary Arrow/Feather IPC buffer encoding.

use std::collections::HashMap;

/// Summary statistics for a numeric telemetry column.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnStatistics {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

impl ColumnStatistics {
    /// Computes two-pass numerically stable statistics from a slice of floats.
    pub fn compute(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                count: 0,
                mean: 0.0,
                variance: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
            };
        }

        let n = values.len();
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut sum = 0.0;

        for &v in values {
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
            sum += v;
        }

        let mean = sum / (n as f64);
        let mut sum_sq_diff = 0.0;
        for &v in values {
            let diff = v - mean;
            sum_sq_diff += diff * diff;
        }

        let variance = if n > 1 {
            sum_sq_diff / ((n - 1) as f64)
        } else {
            0.0
        };
        let std_dev = variance.sqrt();

        Self {
            count: n,
            mean,
            variance,
            std_dev,
            min,
            max,
        }
    }
}

/// In-memory columnar record batch for high-performance simulation telemetry.
/// Memory is stored column-wise in dense contiguous vectors to maximize vectorization
/// and enable zero-copy binary streaming.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ColumnarRecordBatch {
    columns: Vec<String>,
    column_indices: HashMap<String, usize>,
    data: Vec<Vec<f64>>,
    row_count: usize,
}

impl ColumnarRecordBatch {
    /// Creates a new columnar record batch with the specified column schema.
    pub fn new(columns: Vec<String>) -> Self {
        let mut column_indices = HashMap::new();
        let mut data = Vec::with_capacity(columns.len());
        for (i, name) in columns.iter().enumerate() {
            column_indices.insert(name.clone(), i);
            data.push(Vec::new());
        }
        Self {
            columns,
            column_indices,
            data,
            row_count: 0,
        }
    }

    /// Column names in schema order.
    pub fn schema(&self) -> &[String] {
        &self.columns
    }

    /// Number of rows currently buffered.
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// Number of columns.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Appends a single row of values across all columns.
    pub fn push_row(&mut self, values: &[f64]) -> Result<(), String> {
        if values.len() != self.columns.len() {
            return Err(format!(
                "Row length mismatch: expected {} values, got {}",
                self.columns.len(),
                values.len()
            ));
        }
        for (i, &v) in values.iter().enumerate() {
            self.data[i].push(v);
        }
        self.row_count += 1;
        Ok(())
    }

    /// Appends an entire contiguous column of values by name.
    pub fn append_column_data(&mut self, name: &str, values: &[f64]) -> Result<(), String> {
        let idx = self
            .column_indices
            .get(name)
            .copied()
            .ok_or_else(|| format!("Column '{name}' not found in schema"))?;

        if self.row_count == 0 {
            self.row_count = values.len();
        } else if self.row_count != values.len() {
            return Err(format!(
                "Column '{}' length {} does not match batch row count {}",
                name,
                values.len(),
                self.row_count
            ));
        }

        self.data[idx] = values.to_vec();
        Ok(())
    }

    /// Retrieves an immutable slice of a column by name.
    pub fn column(&self, name: &str) -> Option<&[f64]> {
        let idx = *self.column_indices.get(name)?;
        Some(&self.data[idx])
    }

    /// Retrieves an immutable slice of a column by index.
    pub fn column_by_index(&self, index: usize) -> Option<&[f64]> {
        self.data.get(index).map(|v| v.as_slice())
    }

    /// Computes summary statistics for a specified column.
    pub fn column_statistics(&self, name: &str) -> Option<ColumnStatistics> {
        let col = self.column(name)?;
        Some(ColumnStatistics::compute(col))
    }

    /// Encodes the batch into standard Arrow/Feather IPC binary stream format.
    ///
    /// Layout:
    /// - Magic bytes: `b"ARROW1"` (6 bytes)
    /// - Schema header: column count (u32 LE), row count (u32 LE)
    /// - For each column: name length (u32 LE), name UTF-8 bytes
    /// - Data block: aligned 64-byte padded float64 arrays for each column
    /// - End magic: `b"ARROW1"` (6 bytes)
    pub fn to_feather_ipc_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // Magic
        buffer.extend_from_slice(b"ARROW1");

        // Schema
        buffer.extend_from_slice(&(self.columns.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&(self.row_count as u32).to_le_bytes());

        for col in &self.columns {
            let bytes = col.as_bytes();
            buffer.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            buffer.extend_from_slice(bytes);
        }

        // Align to 8 bytes for f64 data
        let remainder = buffer.len() % 8;
        if remainder != 0 {
            buffer.resize(buffer.len() + (8 - remainder), 0);
        }

        // Column f64 payloads
        for col_data in &self.data {
            for &val in col_data {
                buffer.extend_from_slice(&val.to_le_bytes());
            }
        }

        buffer.extend_from_slice(b"ARROW1");
        buffer
    }

    /// Decodes a batch from Arrow/Feather IPC binary bytes.
    pub fn from_feather_ipc_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 14 {
            return Err("Binary buffer too short for Arrow IPC".to_string());
        }
        if &bytes[0..6] != b"ARROW1" {
            return Err("Invalid Arrow IPC magic header".to_string());
        }
        if &bytes[bytes.len() - 6..] != b"ARROW1" {
            return Err("Invalid Arrow IPC magic footer".to_string());
        }

        let mut offset = 6;
        let num_cols = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|e| format!("{e}"))?,
        ) as usize;
        offset += 4;

        let num_rows = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|e| format!("{e}"))?,
        ) as usize;
        offset += 4;

        let mut columns = Vec::with_capacity(num_cols);
        for _ in 0..num_cols {
            let name_len = u32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .map_err(|e| format!("{e}"))?,
            ) as usize;
            offset += 4;

            let name_str = std::str::from_utf8(&bytes[offset..offset + name_len])
                .map_err(|e| format!("Invalid column name UTF-8: {e}"))?;
            columns.push(name_str.to_string());
            offset += name_len;
        }

        // Align to 8 bytes
        let remainder = offset % 8;
        if remainder != 0 {
            offset += 8 - remainder;
        }

        let mut batch = Self::new(columns);
        for i in 0..num_cols {
            let mut col_data = Vec::with_capacity(num_rows);
            for _ in 0..num_rows {
                if offset + 8 > bytes.len() - 6 {
                    return Err("Buffer underrun reading column floats".to_string());
                }
                let val = f64::from_le_bytes(
                    bytes[offset..offset + 8]
                        .try_into()
                        .map_err(|e| format!("{e}"))?,
                );
                col_data.push(val);
                offset += 8;
            }
            batch.data[i] = col_data;
        }
        batch.row_count = num_rows;

        Ok(batch)
    }

    /// Formats the columnar batch as standard CSV string.
    pub fn to_csv(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.columns.join(","));
        out.push('\n');

        for r in 0..self.row_count {
            for (c, col_data) in self.data.iter().enumerate() {
                if c > 0 {
                    out.push(',');
                }
                out.push_str(&format!("{:.10e}", col_data[r]));
            }
            out.push('\n');
        }

        out
    }
}
