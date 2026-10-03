#![deny(unsafe_code)]

//! Measured curve data representation, experimental data ingestion, and synthetic curve synthesis.

use super::optimizer::{BjtTargetParams, Bsim4TargetParams, EkvTargetParams};
use thiserror::Error;

/// Error types occurring during curve data parsing, validation, and parameter extraction.
#[derive(Debug, Error, PartialEq)]
pub enum ExtractionError {
    #[error("Measured curve data is empty")]
    EmptyData,

    #[error("Insufficient measurement points: found {0}, minimum required is 3")]
    InsufficientPoints(usize),

    #[error("Invalid CSV formatting: {0}")]
    InvalidFormat(String),

    #[error("Failed to parse floating-point value: {0}")]
    ParseFloatError(String),

    #[error("Optimization failed to converge: {0}")]
    ConvergenceFailure(String),

    #[error("Invalid model parameter: {0}")]
    InvalidParameter(String),
}

/// A single electrical measurement point $(V_{ds}, V_{gs}, V_{bs}, I_{ds}, C_{gg})$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeasurementPoint {
    pub v_ds: f64,
    pub v_gs: f64,
    pub v_bs: f64,
    pub i_ds: f64,
    pub c_gg: Option<f64>,
}

impl MeasurementPoint {
    /// Constructs a new measurement point.
    pub fn new(v_ds: f64, v_gs: f64, v_bs: f64, i_ds: f64, c_gg: Option<f64>) -> Self {
        Self {
            v_ds,
            v_gs,
            v_bs,
            i_ds,
            c_gg,
        }
    }
}

/// An experimental or synthetic semiconductor characteristic curve.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasuredCurve {
    pub name: String,
    pub temperature_k: f64,
    pub channel_length_m: f64,
    pub channel_width_m: f64,
    pub points: Vec<MeasurementPoint>,
}

impl MeasuredCurve {
    /// Constructs a new measured curve with explicit channel geometry and temperature.
    pub fn new(
        name: impl Into<String>,
        temperature_k: f64,
        channel_length_m: f64,
        channel_width_m: f64,
        points: Vec<MeasurementPoint>,
    ) -> Self {
        Self {
            name: name.into(),
            temperature_k,
            channel_length_m,
            channel_width_m,
            points,
        }
    }

    /// Ingests measurement points from comma-, tab-, semicolon-, or whitespace-delimited CSV text.
    pub fn parse_csv(csv: &str) -> Result<Self, ExtractionError> {
        let lines: Vec<&str> = csv
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("//") && !l.starts_with(';'))
            .collect();

        if lines.is_empty() {
            return Err(ExtractionError::EmptyData);
        }

        let mut start_idx = 0;
        let mut col_vds = None;
        let mut col_vgs = None;
        let mut col_vbs = None;
        let mut col_ids = None;
        let mut col_cgg = None;

        // Inspect header if non-numeric characters appear in first line tokens
        let first_line_tokens: Vec<&str> = split_csv_line(lines[0]);
        let has_header = first_line_tokens.iter().any(|t| t.chars().any(|c| c.is_alphabetic()));

        if has_header {
            start_idx = 1;
            for (idx, token) in first_line_tokens.iter().enumerate() {
                let lower = token.to_ascii_lowercase();
                if lower.contains("vds") || lower == "vd" {
                    col_vds = Some(idx);
                } else if lower.contains("vgs") || lower == "vg" {
                    col_vgs = Some(idx);
                } else if lower.contains("vbs") || lower == "vb" {
                    col_vbs = Some(idx);
                } else if lower.contains("ids") || lower == "id" || lower == "i" {
                    col_ids = Some(idx);
                } else if lower.contains("cgg") || lower == "cg" {
                    col_cgg = Some(idx);
                }
            }
        }

        let mut points = Vec::with_capacity(lines.len() - start_idx);

        for line_str in lines.iter().skip(start_idx) {
            let tokens = split_csv_line(line_str);
            if tokens.is_empty() {
                continue;
            }

            if has_header && col_ids.is_some() {
                let parse_col = |col_opt: Option<usize>| -> Result<Option<f64>, ExtractionError> {
                    match col_opt {
                        Some(idx) => {
                            if idx < tokens.len() {
                                let val = tokens[idx]
                                    .parse::<f64>()
                                    .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                                Ok(Some(val))
                            } else {
                                Ok(None)
                            }
                        }
                        None => Ok(None),
                    }
                };

                let v_ds = parse_col(col_vds)?.unwrap_or(1.0);
                let v_gs = parse_col(col_vgs)?.unwrap_or(0.0);
                let v_bs = parse_col(col_vbs)?.unwrap_or(0.0);
                let i_ds = parse_col(col_ids)?.ok_or_else(|| {
                    ExtractionError::InvalidFormat("Missing I_ds value in row".to_string())
                })?;
                let c_gg = parse_col(col_cgg)?;

                points.push(MeasurementPoint::new(v_ds, v_gs, v_bs, i_ds, c_gg));
            } else {
                // Headerless format
                match tokens.len() {
                    2 => {
                        let v_gs = tokens[0]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        let i_ds = tokens[1]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        points.push(MeasurementPoint::new(1.0, v_gs, 0.0, i_ds, None));
                    }
                    3 => {
                        let v_ds = tokens[0]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        let v_gs = tokens[1]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        let i_ds = tokens[2]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        points.push(MeasurementPoint::new(v_ds, v_gs, 0.0, i_ds, None));
                    }
                    4 => {
                        let v_ds = tokens[0]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        let v_gs = tokens[1]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        let v_bs = tokens[2]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        let i_ds = tokens[3]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        points.push(MeasurementPoint::new(v_ds, v_gs, v_bs, i_ds, None));
                    }
                    5.. => {
                        let v_ds = tokens[0]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        let v_gs = tokens[1]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        let v_bs = tokens[2]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        let i_ds = tokens[3]
                            .parse::<f64>()
                            .map_err(|e| ExtractionError::ParseFloatError(e.to_string()))?;
                        let c_gg = tokens[4].parse::<f64>().ok();
                        points.push(MeasurementPoint::new(v_ds, v_gs, v_bs, i_ds, c_gg));
                    }
                    _ => {
                        return Err(ExtractionError::InvalidFormat(format!(
                            "Row with unexpected number of tokens: {}",
                            line_str
                        )));
                    }
                }
            }
        }

        if points.len() < 3 {
            return Err(ExtractionError::InsufficientPoints(points.len()));
        }

        Ok(Self {
            name: "Extracted_Measurement".to_string(),
            temperature_k: 300.0,
            channel_length_m: 100e-9,
            channel_width_m: 10e-6,
            points,
        })
    }

    /// Generates a synthetic NMOS transfer curve ($I_{ds}$ vs. $V_{gs}$) at a fixed $V_{ds}$ bias.
    pub fn synthetic_nmos_transfer_curve(
        v_ds: f64,
        v_gs_range: (f64, f64, usize),
        v_th: f64,
        u0: f64,
    ) -> Self {
        let (v_start, v_end, num_points) = v_gs_range;
        let count = num_points.max(3);
        let step = (v_end - v_start) / ((count - 1) as f64);

        let w = 10e-6;
        let l = 100e-9;
        let temp_k = 300.0;

        let params = Bsim4TargetParams {
            vth0: v_th,
            u0,
            ..Bsim4TargetParams::default()
        };

        let c_ox = (phonon_core::EPSILON_0 * phonon_core::EPSILON_R_OX) / 3.0e-9;
        let c_gg_val = w * l * c_ox;

        let mut points = Vec::with_capacity(count);
        for i in 0..count {
            let v_gs = v_start + (i as f64) * step;
            let i_ds = params.evaluate_ids(v_ds, v_gs, 0.0, w, l, temp_k);
            points.push(MeasurementPoint::new(v_ds, v_gs, 0.0, i_ds, Some(c_gg_val)));
        }

        Self {
            name: format!("Synthetic_NMOS_Transfer_Vds_{:.2}V", v_ds),
            temperature_k: temp_k,
            channel_length_m: l,
            channel_width_m: w,
            points,
        }
    }

    /// Generates a synthetic NMOS output curve ($I_{ds}$ vs. $V_{ds}$) at a fixed $V_{gs}$ bias.
    pub fn synthetic_nmos_output_curve(
        v_gs: f64,
        v_ds_range: (f64, f64, usize),
        v_th: f64,
        u0: f64,
    ) -> Self {
        let (v_start, v_end, num_points) = v_ds_range;
        let count = num_points.max(3);
        let step = (v_end - v_start) / ((count - 1) as f64);

        let w = 10e-6;
        let l = 100e-9;
        let temp_k = 300.0;

        let params = Bsim4TargetParams {
            vth0: v_th,
            u0,
            ..Bsim4TargetParams::default()
        };

        let c_ox = (phonon_core::EPSILON_0 * phonon_core::EPSILON_R_OX) / 3.0e-9;
        let c_gg_val = w * l * c_ox;

        let mut points = Vec::with_capacity(count);
        for i in 0..count {
            let v_ds = v_start + (i as f64) * step;
            let i_ds = params.evaluate_ids(v_ds, v_gs, 0.0, w, l, temp_k);
            points.push(MeasurementPoint::new(v_ds, v_gs, 0.0, i_ds, Some(c_gg_val)));
        }

        Self {
            name: format!("Synthetic_NMOS_Output_Vgs_{:.2}V", v_gs),
            temperature_k: temp_k,
            channel_length_m: l,
            channel_width_m: w,
            points,
        }
    }

    /// Generates a synthetic EKV transfer curve ($I_{ds}$ vs. $V_{gs}$) at a fixed $V_{ds}$ bias.
    pub fn synthetic_ekv_transfer_curve(
        v_ds: f64,
        v_gs_range: (f64, f64, usize),
        vto: f64,
        kp: f64,
        gamma: f64,
        theta: f64,
    ) -> Self {
        let (v_start, v_end, num_points) = v_gs_range;
        let count = num_points.max(3);
        let step = (v_end - v_start) / ((count - 1) as f64);

        let w = 10e-6;
        let l = 100e-9;
        let temp_k = 300.0;

        let params = EkvTargetParams {
            vto,
            kp,
            gamma,
            theta,
        };

        let mut points = Vec::with_capacity(count);
        for i in 0..count {
            let v_gs = v_start + (i as f64) * step;
            let i_ds = params.evaluate_ids(v_ds, v_gs, 0.0, w, l, temp_k);
            points.push(MeasurementPoint::new(v_ds, v_gs, 0.0, i_ds, None));
        }

        Self {
            name: format!("Synthetic_EKV_Transfer_Vds_{:.2}V", v_ds),
            temperature_k: temp_k,
            channel_length_m: l,
            channel_width_m: w,
            points,
        }
    }

    /// Generates a synthetic EKV output curve ($I_{ds}$ vs. $V_{ds}$) at a fixed $V_{gs}$ bias.
    pub fn synthetic_ekv_output_curve(
        v_gs: f64,
        v_ds_range: (f64, f64, usize),
        vto: f64,
        kp: f64,
        gamma: f64,
        theta: f64,
    ) -> Self {
        let (v_start, v_end, num_points) = v_ds_range;
        let count = num_points.max(3);
        let step = (v_end - v_start) / ((count - 1) as f64);

        let w = 10e-6;
        let l = 100e-9;
        let temp_k = 300.0;

        let params = EkvTargetParams {
            vto,
            kp,
            gamma,
            theta,
        };

        let mut points = Vec::with_capacity(count);
        for i in 0..count {
            let v_ds = v_start + (i as f64) * step;
            let i_ds = params.evaluate_ids(v_ds, v_gs, 0.0, w, l, temp_k);
            points.push(MeasurementPoint::new(v_ds, v_gs, 0.0, i_ds, None));
        }

        Self {
            name: format!("Synthetic_EKV_Output_Vgs_{:.2}V", v_gs),
            temperature_k: temp_k,
            channel_length_m: l,
            channel_width_m: w,
            points,
        }
    }

    /// Generates a synthetic BJT forward active curve ($I_c$ vs. $V_{be}$) at a fixed $V_{ce}$ bias.
    pub fn synthetic_bjt_forward_active_curve(
        v_ce: f64,
        v_be_range: (f64, f64, usize),
        is_val: f64,
        bf: f64,
        vaf: f64,
    ) -> Self {
        let (v_start, v_end, num_points) = v_be_range;
        let count = num_points.max(3);
        let step = (v_end - v_start) / ((count - 1) as f64);
        let temp_k = 300.0;

        let params = BjtTargetParams {
            is: is_val,
            bf,
            vaf,
        };

        let mut points = Vec::with_capacity(count);
        for i in 0..count {
            let v_be = v_start + (i as f64) * step;
            let i_c = params.evaluate_ic(v_ce, v_be, temp_k);
            points.push(MeasurementPoint::new(v_ce, v_be, 0.0, i_c, None));
        }

        Self {
            name: format!("Synthetic_BJT_Forward_Active_Vce_{:.2}V", v_ce),
            temperature_k: temp_k,
            channel_length_m: 1e-6,
            channel_width_m: 1e-6,
            points,
        }
    }

    /// Generates a synthetic BJT Gummel curve ($I_c$ vs. $V_{be}$) at a fixed $V_{ce}$ bias.
    pub fn synthetic_bjt_gummel_curve(
        v_ce: f64,
        v_be_range: (f64, f64, usize),
        is_val: f64,
        bf: f64,
        vaf: f64,
    ) -> Self {
        let mut curve = Self::synthetic_bjt_forward_active_curve(v_ce, v_be_range, is_val, bf, vaf);
        curve.name = format!("Synthetic_BJT_Gummel_Vce_{:.2}V", v_ce);
        curve
    }

    /// Generates a synthetic BJT output curve ($I_c$ vs. $V_{ce}$) at a fixed $V_{be}$ bias.
    pub fn synthetic_bjt_output_curve(
        v_be: f64,
        v_ce_range: (f64, f64, usize),
        is_val: f64,
        bf: f64,
        vaf: f64,
    ) -> Self {
        let (v_start, v_end, num_points) = v_ce_range;
        let count = num_points.max(3);
        let step = (v_end - v_start) / ((count - 1) as f64);
        let temp_k = 300.0;

        let params = BjtTargetParams {
            is: is_val,
            bf,
            vaf,
        };

        let mut points = Vec::with_capacity(count);
        for i in 0..count {
            let v_ce = v_start + (i as f64) * step;
            let i_c = params.evaluate_ic(v_ce, v_be, temp_k);
            points.push(MeasurementPoint::new(v_ce, v_be, 0.0, i_c, None));
        }

        Self {
            name: format!("Synthetic_BJT_Output_Vbe_{:.2}V", v_be),
            temperature_k: temp_k,
            channel_length_m: 1e-6,
            channel_width_m: 1e-6,
            points,
        }
    }
}

/// Splits a CSV row on commas, semicolons, tabs, or whitespace.
fn split_csv_line(line: &str) -> Vec<&str> {
    if line.contains(',') {
        line.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
    } else if line.contains(';') {
        line.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
    } else if line.contains('\t') {
        line.split('\t').map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
    } else {
        line.split_whitespace().collect()
    }
}
