//! Automated compact parameter extraction and calibration from microscopic TCAD device simulations.

use super::material::MaterialProperties;
use super::tcad_device::TcadDevice;
use crate::{DiodeModel, MosfetModel};

/// Linear least-squares regression y = m * x + c.
fn linear_regression(x: &[f64], y: &[f64]) -> Option<(f64, f64)> {
    let n = x.len() as f64;
    if n < 2.0 {
        return None;
    }

    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xx: f64 = x.iter().map(|&xi| xi * xi).sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(&xi, &yi)| xi * yi).sum();

    let denom = n * sum_xx - sum_x * sum_x;
    if denom.abs() < 1e-30 {
        return None;
    }

    let m = (n * sum_xy - sum_x * sum_y) / denom;
    let c = (sum_y - m * sum_x) / n;

    Some((m, c))
}

/// Characterizes a physical TCAD diode and extracts an equivalent compact DiodeModel.
pub fn extract_diode_model(tcad: &TcadDevice, temp_k: f64) -> Result<DiodeModel, String> {
    let vt = MaterialProperties::thermal_voltage(temp_k);

    // Sweep forward voltages where exponential behavior dominates
    let v_points = [0.15, 0.20, 0.25, 0.30, 0.35, 0.40];
    let mut x_vals = Vec::new();
    let mut y_vals = Vec::new();

    for &v in &v_points {
        let (i_d, _) = tcad.evaluate_diode(v, temp_k);
        if i_d > 1e-25 {
            x_vals.push(v);
            y_vals.push(i_d.ln());
        }
    }

    if x_vals.len() < 3 {
        return Err("Insufficient forward conduction data points for diode extraction".to_string());
    }

    let (m, c) = linear_regression(&x_vals, &y_vals)
        .ok_or_else(|| "Regression failed on diode I-V sweep".to_string())?;

    // slope m = 1 / (eta * Vt) -> eta = 1 / (m * Vt)
    let eta = (1.0 / (m * vt)).clamp(0.8, 3.0);
    // intercept c = ln(Is) -> Is = exp(c)
    let is = c.clamp(-80.0, -5.0).exp().clamp(1e-20, 1e-6);

    Ok(DiodeModel {
        is,
        n: eta,
        ..DiodeModel::default()
    })
}

/// Characterizes a physical TCAD MOSFET and extracts an equivalent compact MosfetModel.
pub fn extract_mosfet_model(
    tcad: &TcadDevice,
    channel_width: f64,
    temp_k: f64,
) -> Result<MosfetModel, String> {
    let v_ds_lin = 0.05; // 50 mV linear region

    // 1. Transfer characteristics sweep: V_GS in [0.1, 1.2] V
    let v_gs_points = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2];
    let mut i_ds_vals = Vec::with_capacity(v_gs_points.len());
    let mut gm_vals = Vec::with_capacity(v_gs_points.len());

    for &v_gs in &v_gs_points {
        let (i_ds, gm, _, _) = tcad.evaluate_mosfet(v_ds_lin, v_gs, 0.0, temp_k);
        i_ds_vals.push(i_ds);
        gm_vals.push(gm);
    }

    // Find point of peak transconductance gm_max
    let mut max_idx = 0;
    let mut max_gm = 0.0f64;
    for (i, &gm) in gm_vals.iter().enumerate() {
        if gm > max_gm {
            max_gm = gm;
            max_idx = i;
        }
    }

    if max_gm < 1e-15 {
        return Err("No channel transconductance detected in TCAD MOSFET".to_string());
    }

    // Linear extrapolation method for threshold voltage V_th
    let v_gs_peak = v_gs_points[max_idx];
    let i_ds_peak = i_ds_vals[max_idx];
    let v_th = (v_gs_peak - i_ds_peak / max_gm - 0.5 * v_ds_lin).clamp(0.05, 1.5);

    // Transconductance parameter K_p * (W / L) = gm_max / V_ds_lin
    let kp_w_over_l = max_gm / v_ds_lin;
    let length = tcad.mesh.coords.last().copied().unwrap_or(1e-6);
    let kp = (kp_w_over_l * length / channel_width.max(1e-9)).clamp(1e-6, 1e-1);

    // 2. Output characteristics sweep in saturation to extract lambda
    let (i_d1, _, g_ds1, _) = tcad.evaluate_mosfet(0.8, 1.0, 0.0, temp_k);
    let lambda = if i_d1 > 1e-12 {
        (g_ds1 / i_d1).clamp(0.001, 0.2)
    } else {
        0.02
    };

    let c_ox = (3.9 * 8.854_187_812_8e-12) / 4.0e-9;
    let mu0 = (kp / c_ox).clamp(0.005, 0.2);

    Ok(MosfetModel {
        vth0: v_th,
        mu0,
        w: channel_width,
        l: length,
        lambda,
        ..MosfetModel::default()
    })
}
