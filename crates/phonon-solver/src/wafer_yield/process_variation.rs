#![deny(unsafe_code)]

//! Wafer-Scale Spatially Correlated Process Variations Engine.
//!
//! Models 300mm wafer layout, gross die placement, within-die (WID) and die-to-die (D2D)
//! parameter variations, radial CMP bowl/dome profiles, and spatial Gaussian random fields.

use std::f64::consts::PI;

/// Physical geometry configuration for standard 300mm semiconductor wafers.
#[derive(Debug, Clone)]
pub struct WaferGeometryParams {
    /// Total nominal wafer diameter in millimeters (typically 300.0 mm).
    pub diameter_mm: f64,
    /// Edge exclusion ring width in millimeters (typically 2.0 to 5.0 mm).
    pub edge_exclusion_mm: f64,
    /// Individual die drawn width in millimeters (x-axis).
    pub die_width_mm: f64,
    /// Individual die drawn height in millimeters (y-axis).
    pub die_height_mm: f64,
    /// Scribe line street kerf width in micrometers.
    pub scribe_street_um: f64,
}

impl Default for WaferGeometryParams {
    fn default() -> Self {
        Self {
            diameter_mm: 300.0,
            edge_exclusion_mm: 3.0,
            die_width_mm: 12.5,
            die_height_mm: 10.0,
            scribe_street_um: 80.0,
        }
    }
}

impl WaferGeometryParams {
    /// Total wafer radius in mm.
    pub fn wafer_radius_mm(&self) -> f64 {
        self.diameter_mm * 0.5
    }

    /// Usable active radius inside edge exclusion zone in mm.
    pub fn active_radius_mm(&self) -> f64 {
        (self.wafer_radius_mm() - self.edge_exclusion_mm).max(1.0)
    }

    /// Die step pitch along X in mm (including scribe lane).
    pub fn pitch_x_mm(&self) -> f64 {
        self.die_width_mm + self.scribe_street_um * 1.0e-3
    }

    /// Die step pitch along Y in mm (including scribe lane).
    pub fn pitch_y_mm(&self) -> f64 {
        self.die_height_mm + self.scribe_street_um * 1.0e-3
    }

    /// Individual die area in cm^2.
    pub fn die_area_cm2(&self) -> f64 {
        (self.die_width_mm * 0.1) * (self.die_height_mm * 0.1)
    }

    /// Individual die area in mm^2.
    pub fn die_area_mm2(&self) -> f64 {
        self.die_width_mm * self.die_height_mm
    }
}

/// Position and boundary containment status of a placed die on the wafer coordinate grid.
#[derive(Debug, Clone)]
pub struct DieGridPosition {
    pub col: i32,
    pub row: i32,
    pub center_x_mm: f64,
    pub center_y_mm: f64,
    pub distance_from_center_mm: f64,
    pub is_fully_within_wafer: bool,
}

/// Evaluates geometric Gross Dies Per Wafer (DPW) using the industrial analytical formula.
pub fn calculate_analytical_gross_dpw(wafer: &WaferGeometryParams) -> usize {
    let r = wafer.active_radius_mm();
    let a_die = wafer.pitch_x_mm() * wafer.pitch_y_mm();
    if a_die <= 0.0 {
        return 0;
    }

    // Standard industrial semi-empirical formula:
    // Gross DPW = (pi * R^2) / A_die - (2 * pi * R) / sqrt(2 * A_die)
    let area_term = (PI * r * r) / a_die;
    let edge_loss_term = (2.0 * PI * r) / (2.0 * a_die).sqrt();
    let dpw = (area_term - edge_loss_term).max(0.0);
    dpw as usize
}

/// Generates discrete 2D grid die placement across 300mm wafer, testing full 4-corner containment.
pub fn generate_die_grid(wafer: &WaferGeometryParams) -> Vec<DieGridPosition> {
    let r_active = wafer.active_radius_mm();
    let pitch_x = wafer.pitch_x_mm();
    let pitch_y = wafer.pitch_y_mm();

    let half_w = wafer.die_width_mm * 0.5;
    let half_h = wafer.die_height_mm * 0.5;

    let max_cols = (r_active / pitch_x).ceil() as i32 + 1;
    let max_rows = (r_active / pitch_y).ceil() as i32 + 1;

    let mut dies = Vec::new();

    for row in -max_rows..=max_rows {
        for col in -max_cols..=max_cols {
            // Die center coordinates (centered at wafer origin)
            let cx = col as f64 * pitch_x;
            let cy = row as f64 * pitch_y;
            let dist = (cx * cx + cy * cy).sqrt();

            // Check if all 4 corners of the die lie strictly within active radius
            let corners = [
                ((cx - half_w).powi(2) + (cy - half_h).powi(2)).sqrt(),
                ((cx + half_w).powi(2) + (cy - half_h).powi(2)).sqrt(),
                ((cx - half_w).powi(2) + (cy + half_h).powi(2)).sqrt(),
                ((cx + half_w).powi(2) + (cy + half_h).powi(2)).sqrt(),
            ];

            let is_fully_within = corners.iter().all(|&c| c <= r_active);

            // Keep dies that touch or are inside the wafer for visualization
            if dist <= wafer.wafer_radius_mm() + half_w.max(half_h) {
                dies.push(DieGridPosition {
                    col,
                    row,
                    center_x_mm: cx,
                    center_y_mm: cy,
                    distance_from_center_mm: dist,
                    is_fully_within_wafer: is_fully_within,
                });
            }
        }
    }

    dies
}

/// Statistical parameters describing spatially correlated process variations across wafer coordinates.
#[derive(Debug, Clone)]
pub struct ProcessVariationFieldParams {
    /// Nominal threshold voltage in Volts.
    pub vth_nominal_v: f64,
    /// 1-sigma relative threshold voltage variation (e.g. 0.04 = 4.0%, 3-sigma = 12%).
    pub vth_sigma_rel: f64,
    /// Nominal gate length in nm.
    pub lg_nominal_nm: f64,
    /// 1-sigma relative gate length variation (e.g. 0.027 = 2.7%, 3-sigma = 8.1%).
    pub lg_sigma_rel: f64,
    /// Nominal gate oxide thickness in nm.
    pub tox_nominal_nm: f64,
    /// 1-sigma relative oxide thickness variation (e.g. 0.018 = 1.8%, 3-sigma = 5.4%).
    pub tox_sigma_rel: f64,
    /// Systematic radial edge bowl shift percentage (e.g. +6.0% at wafer perimeter).
    pub radial_bowl_vth_pct: f64,
    /// Spatial correlation length in millimeters.
    pub correlation_length_mm: f64,
}

impl Default for ProcessVariationFieldParams {
    fn default() -> Self {
        Self {
            vth_nominal_v: 0.32,
            vth_sigma_rel: 0.040,
            lg_nominal_nm: 16.0,
            lg_sigma_rel: 0.027,
            tox_nominal_nm: 1.15,
            tox_sigma_rel: 0.018,
            radial_bowl_vth_pct: 5.5,
            correlation_length_mm: 35.0,
        }
    }
}

/// Realized physical process parameters for a specific die location on the wafer.
#[derive(Debug, Clone)]
pub struct DieProcessParameters {
    pub vth_v: f64,
    pub vth_delta_pct: f64,
    pub lg_nm: f64,
    pub lg_delta_pct: f64,
    pub tox_nm: f64,
    pub tox_delta_pct: f64,
}

/// Evaluates realized process variations at a die location using radial bowl + pseudo-random spatial field.
pub fn evaluate_die_process_parameters(
    pos: &DieGridPosition,
    wafer: &WaferGeometryParams,
    field: &ProcessVariationFieldParams,
) -> DieProcessParameters {
    let r_norm = (pos.distance_from_center_mm / wafer.wafer_radius_mm()).clamp(0.0, 1.0);

    // Systematic parabolic radial profile (CMP dishing / plasma etch edge effect)
    let radial_shift_pct = field.radial_bowl_vth_pct * r_norm.powi(2);

    // Deterministic pseudo-random spatially correlated field based on coordinates
    // Using sinusoidal multi-harmonic Fourier synthesis
    let k = 2.0 * PI / field.correlation_length_mm;
    let wave1 = (k * pos.center_x_mm).sin() * (k * 0.7 * pos.center_y_mm).cos();
    let wave2 = (k * 1.6 * pos.center_x_mm + 0.4).cos() * (k * 1.3 * pos.center_y_mm - 0.2).sin();
    let correlated_noise = 0.65 * wave1 + 0.35 * wave2;

    // V_th variation
    let vth_delta_pct = radial_shift_pct + correlated_noise * (field.vth_sigma_rel * 100.0);
    let vth_v = field.vth_nominal_v * (1.0 + vth_delta_pct / 100.0);

    // L_g variation (partially correlated with inverse sign due to litho dose/focus)
    let lg_delta_pct = -0.4 * radial_shift_pct - correlated_noise * (field.lg_sigma_rel * 100.0);
    let lg_nm = field.lg_nominal_nm * (1.0 + lg_delta_pct / 100.0);

    // t_ox variation
    let tox_delta_pct = 0.25 * radial_shift_pct + correlated_noise * (field.tox_sigma_rel * 100.0);
    let tox_nm = field.tox_nominal_nm * (1.0 + tox_delta_pct / 100.0);

    DieProcessParameters {
        vth_v,
        vth_delta_pct,
        lg_nm,
        lg_delta_pct,
        tox_nm,
        tox_delta_pct,
    }
}
