#![deny(unsafe_code)]

//! Topological Acoustic Laser Engine & Gain-Loss Discrimination.
//!
//! Models single-mode topological corner acoustic lasing, modal threshold discrimination,
//! non-linear gain saturation, slope efficiency, and Schawlow-Townes linewidth narrowing.

/// Parameters for the topological acoustic laser.
#[derive(Debug, Clone, PartialEq)]
pub struct TopologicalLaserParams {
    /// Acoustic pump rate / gain rate in MHz.
    pub pump_rate_mhz: f64,
    /// Linear acoustic viscous loss rate in MHz.
    pub linear_loss_mhz: f64,
    /// Saturation power in mW.
    pub saturation_power_mw: f64,
    /// Laser threshold pump rate in MHz.
    pub threshold_pump_mhz: f64,
    /// Loaded acoustic cavity quality factor Q.
    pub cavity_q: f64,
    /// Henry linewidth enhancement factor alpha_H.
    pub linewidth_alpha_henry: f64,
    /// Spontaneous emission coupling factor beta.
    pub spontaneous_emission_beta: f64,
}

impl Default for TopologicalLaserParams {
    fn default() -> Self {
        Self {
            pump_rate_mhz: 3.5,
            linear_loss_mhz: 1.0,
            saturation_power_mw: 25.0,
            threshold_pump_mhz: 1.2,
            cavity_q: 15000.0,
            linewidth_alpha_henry: 2.0,
            spontaneous_emission_beta: 0.02,
        }
    }
}

/// A point along the laser power input-output (L-I) curve.
#[derive(Debug, Clone, PartialEq)]
pub struct LaserCurvePoint {
    /// Pump rate / input power (MHz).
    pub pump_mhz: f64,
    /// Output acoustic laser power (mW).
    pub output_power_mw: f64,
    /// Net modal gain (MHz).
    pub net_gain_mhz: f64,
    /// Emission linewidth (Hz).
    pub linewidth_hz: f64,
}

/// Evaluated metrics for the topological acoustic laser.
#[derive(Debug, Clone, PartialEq)]
pub struct TopologicalLaserMetrics {
    /// Net modal gain of the topological corner mode (MHz).
    pub corner_gain_mhz: f64,
    /// Net modal gain of the competing bulk modes (MHz).
    pub bulk_gain_mhz: f64,
    /// Modal gain discrimination in dB over competing modes.
    pub modal_discrimination_db: f64,
    /// Threshold pump rate (MHz).
    pub threshold_pump_mhz: f64,
    /// Slope efficiency eta_slope (0.0 to 1.0).
    pub slope_efficiency: f64,
    /// Steady-state output power at operational pump (mW).
    pub output_power_mw: f64,
    /// Narrowed acoustic laser linewidth (Hz).
    pub laser_linewidth_hz: f64,
    /// Whether the device is operating above laser threshold.
    pub is_lasing: bool,
}

/// Solver for the topological acoustic laser.
#[derive(Debug, Clone)]
pub struct TopologicalLaserSolver {
    pub params: TopologicalLaserParams,
}

impl TopologicalLaserSolver {
    /// Creates a new laser solver with the specified parameters.
    pub fn new(params: TopologicalLaserParams) -> Self {
        Self { params }
    }

    /// Evaluates operational laser performance and modal discrimination.
    pub fn evaluate_metrics(&self) -> TopologicalLaserMetrics {
        let p = &self.params;

        // Confinement factors: corner mode overlaps predominantly with pumped corner (0.88),
        // while bulk modes overlap weakly (0.12).
        let gamma_corner = 0.88;
        let gamma_bulk = 0.12;

        let corner_gain_mhz = gamma_corner * p.pump_rate_mhz - p.linear_loss_mhz;
        let bulk_gain_mhz = gamma_bulk * p.pump_rate_mhz - p.linear_loss_mhz;

        // Modal discrimination in dB
        let ratio = if corner_gain_mhz > 0.0 {
            let denom = if bulk_gain_mhz > 0.0 {
                bulk_gain_mhz
            } else {
                0.02
            };
            (corner_gain_mhz / denom).max(1.0)
        } else {
            1.0
        };
        let modal_discrimination_db = 10.0 * ratio.log10();

        // Slope efficiency
        let slope_efficiency = 0.584; // 58.4% differential quantum efficiency

        let is_lasing = p.pump_rate_mhz > p.threshold_pump_mhz;
        let output_power_mw = if is_lasing {
            slope_efficiency * (p.pump_rate_mhz - p.threshold_pump_mhz) * 10.0
        } else {
            p.spontaneous_emission_beta * p.pump_rate_mhz * 0.2
        };

        // Schawlow-Townes linewidth with Henry alpha enhancement
        let base_linewidth_hz = 12.0e6 / (2.0 * p.cavity_q); // ~400 Hz cold cavity linewidth
        let laser_linewidth_hz = if is_lasing {
            let sat_ratio = output_power_mw / p.saturation_power_mw.max(1.0);
            (base_linewidth_hz / (1.0 + 80.0 * sat_ratio)) * (1.0 + p.linewidth_alpha_henry.powi(2))
        } else {
            base_linewidth_hz
        };

        TopologicalLaserMetrics {
            corner_gain_mhz,
            bulk_gain_mhz,
            modal_discrimination_db,
            threshold_pump_mhz: p.threshold_pump_mhz,
            slope_efficiency,
            output_power_mw,
            laser_linewidth_hz,
            is_lasing,
        }
    }

    /// Computes the complete Light-Current / Power-Pump (L-I) curve.
    pub fn compute_laser_curve(&self, steps: usize) -> Vec<LaserCurvePoint> {
        let p = &self.params;
        let count = steps.max(20);
        let max_pump = (p.pump_rate_mhz * 2.0).max(6.0);
        let mut curve = Vec::with_capacity(count);

        let slope_eff = 0.584;
        let base_linewidth_hz = 12.0e6 / (2.0 * p.cavity_q);

        for i in 0..=count {
            let pump = (i as f64 / count as f64) * max_pump;
            let net_gain = 0.88 * pump - p.linear_loss_mhz;

            let is_lase = pump > p.threshold_pump_mhz;
            let output_power_mw = if is_lase {
                slope_eff * (pump - p.threshold_pump_mhz) * 10.0
            } else {
                p.spontaneous_emission_beta * pump * 0.2
            };

            let linewidth_hz = if is_lase {
                let sat_ratio = output_power_mw / p.saturation_power_mw.max(1.0);
                (base_linewidth_hz / (1.0 + 80.0 * sat_ratio)) * (1.0 + p.linewidth_alpha_henry.powi(2))
            } else {
                base_linewidth_hz
            };

            curve.push(LaserCurvePoint {
                pump_mhz: pump,
                output_power_mw,
                net_gain_mhz: net_gain,
                linewidth_hz,
            });
        }

        curve
    }
}
