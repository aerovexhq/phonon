#![deny(unsafe_code)]

//! Gradient-based worst-case circuit optimization and process-voltage-temperature (PVT) corner analysis engine.
//!
//! Evaluates the boundary hypercube of component tolerances ($\pm 3\sigma$, $\pm 10\%$, PVT extremes)
//! using backward adjoint gradient projections:
//! $$\mathbf{p}_{\text{worst}} = \mathbf{p}_{\text{nom}} + \text{diag}(\boldsymbol{\Delta} \mathbf{p}) \cdot \text{sgn}\left(\boldsymbol{\nabla}_{\mathbf{p}} J\right)$$

use super::adjoint_engine::{AdjointSensitivityEngine, CircuitParameter};
use crate::error::SolverError;
use crate::transient::solve_transient;
use std::collections::HashMap;

/// Circuit operational corner classification.
#[derive(Debug, Clone, PartialEq)]
pub enum CornerType {
    /// Nominal parameters under standard room temperature (300.0 K).
    Nominal,
    /// Parameter combination yielding maximum objective functional value.
    WorstCaseMax,
    /// Parameter combination yielding minimum objective functional value.
    WorstCaseMin,
    /// Named process-voltage-temperature extreme (e.g. "Fast-Fast", "Slow-Slow", "Cryo-77K").
    PvtExtreme { corner_name: String },
}

/// Evaluation record for a specific circuit parameter corner.
#[derive(Debug, Clone, PartialEq)]
pub struct CornerEvaluation {
    /// Corner type.
    pub corner: CornerType,
    /// Parameter settings active during evaluation: param_id -> value.
    pub param_values: HashMap<String, f64>,
    /// Calculated objective functional value.
    pub objective_value: f64,
    /// Percentage deviation relative to nominal: $((J - J_0) / |J_0|) \cdot 100\%$.
    pub deviation_percent: f64,
}

/// Comprehensive worst-case performance summary spanning all examined corners.
#[derive(Debug, Clone, PartialEq)]
pub struct WorstCaseSummary {
    /// Baseline nominal performance.
    pub nominal: CornerEvaluation,
    /// Absolute worst-case maximum corner.
    pub worst_max: CornerEvaluation,
    /// Absolute worst-case minimum corner.
    pub worst_min: CornerEvaluation,
    /// Evaluated PVT and boundary corners.
    pub corners: Vec<CornerEvaluation>,
    /// Maximum observed percentage degradation magnitude across all corners.
    pub max_degradation_percent: f64,
}

/// Gradient-based worst-case circuit optimizer and PVT corner evaluator.
#[derive(Debug, Clone)]
pub struct WorstCaseOptimizer {
    pub engine: AdjointSensitivityEngine,
}

impl WorstCaseOptimizer {
    /// Constructs a new WorstCaseOptimizer wrapping an AdjointSensitivityEngine.
    pub fn new(engine: AdjointSensitivityEngine) -> Self {
        Self { engine }
    }

    /// Evaluates circuit performance at an explicit parameter configuration.
    pub fn evaluate_corner(
        &self,
        corner: CornerType,
        param_values: &HashMap<String, f64>,
        nominal_obj: f64,
    ) -> Result<CornerEvaluation, SolverError> {
        let mut modified_graph = self.engine.graph.clone();
        for (id, &val) in param_values {
            let _ = modified_graph.update_component_value(id, val);
        }

        let sol = solve_transient(&modified_graph, &self.engine.context, &self.engine.options)?;
        let obj_val = self.engine.evaluate_objective_value(&sol);

        let denom = if nominal_obj.abs() > 1e-14 {
            nominal_obj.abs()
        } else {
            1.0
        };
        let deviation_percent = ((obj_val - nominal_obj) / denom) * 100.0;

        Ok(CornerEvaluation {
            corner,
            param_values: param_values.clone(),
            objective_value: obj_val,
            deviation_percent,
        })
    }

    /// Computes nominal, worst-case maximum, worst-case minimum, and PVT extreme corners.
    pub fn find_worst_case_corners(&self) -> Result<WorstCaseSummary, SolverError> {
        // 1. Evaluate baseline nominal state
        let nominal_sol =
            solve_transient(&self.engine.graph, &self.engine.context, &self.engine.options)?;
        let nominal_obj = self.engine.evaluate_objective_value(&nominal_sol);

        let mut nominal_params = HashMap::new();
        for p in &self.engine.parameters {
            nominal_params.insert(p.id().to_string(), p.nominal());
        }

        let nominal_eval = CornerEvaluation {
            corner: CornerType::Nominal,
            param_values: nominal_params.clone(),
            objective_value: nominal_obj,
            deviation_percent: 0.0,
        };

        // 2. Compute adjoint gradients
        let sensitivities = self.engine.solve()?;
        let gradient_map: HashMap<String, f64> = sensitivities
            .into_iter()
            .map(|s| (s.param_id, s.gradient))
            .collect();

        // 3. Worst-Case Maximum Corner: p_worst = p_nom + sgn(grad) * Delta_p
        let mut max_params = HashMap::new();
        for p in &self.engine.parameters {
            let nom = p.nominal();
            let delta = nom * p.tolerance();
            let grad = gradient_map.get(p.id()).copied().unwrap_or(0.0);

            let val = if grad > 1e-15 {
                nom + delta
            } else if grad < -1e-15 {
                nom - delta
            } else {
                nom
            };
            max_params.insert(p.id().to_string(), val);
        }
        let worst_max =
            self.evaluate_corner(CornerType::WorstCaseMax, &max_params, nominal_obj)?;

        // 4. Worst-Case Minimum Corner: p_worst = p_nom - sgn(grad) * Delta_p
        let mut min_params = HashMap::new();
        for p in &self.engine.parameters {
            let nom = p.nominal();
            let delta = nom * p.tolerance();
            let grad = gradient_map.get(p.id()).copied().unwrap_or(0.0);

            let val = if grad > 1e-15 {
                nom - delta
            } else if grad < -1e-15 {
                nom + delta
            } else {
                nom
            };
            min_params.insert(p.id().to_string(), val);
        }
        let worst_min =
            self.evaluate_corner(CornerType::WorstCaseMin, &min_params, nominal_obj)?;

        // 5. PVT Extreme Corners
        let mut corners = Vec::new();

        // PVT Corner A: Fast-Fast (+tol across components)
        let mut ff_params = HashMap::new();
        for p in &self.engine.parameters {
            ff_params.insert(p.id().to_string(), p.nominal() * (1.0 + p.tolerance()));
        }
        let ff_eval = self.evaluate_corner(
            CornerType::PvtExtreme {
                corner_name: "Fast-Fast (FF)".to_string(),
            },
            &ff_params,
            nominal_obj,
        )?;
        corners.push(ff_eval);

        // PVT Corner B: Slow-Slow (-tol across components)
        let mut ss_params = HashMap::new();
        for p in &self.engine.parameters {
            ss_params.insert(p.id().to_string(), p.nominal() * (1.0 - p.tolerance()));
        }
        let ss_eval = self.evaluate_corner(
            CornerType::PvtExtreme {
                corner_name: "Slow-Slow (SS)".to_string(),
            },
            &ss_params,
            nominal_obj,
        )?;
        corners.push(ss_eval);

        // PVT Corner C: High-Temperature (398.15 K / +125 C)
        let mut high_temp_context = self.engine.context.clone();
        high_temp_context.temperature_kelvin = 398.15;
        let mut high_temp_graph = self.engine.graph.clone();
        for (id, &val) in &nominal_params {
            let _ = high_temp_graph.update_component_value(id, val);
        }
        let high_temp_sol = solve_transient(
            &high_temp_graph,
            &high_temp_context,
            &self.engine.options,
        )?;
        let high_temp_obj = self.engine.evaluate_objective_value(&high_temp_sol);
        let high_temp_denom = if nominal_obj.abs() > 1e-14 {
            nominal_obj.abs()
        } else {
            1.0
        };
        let high_temp_eval = CornerEvaluation {
            corner: CornerType::PvtExtreme {
                corner_name: "High-Temp (+125C)".to_string(),
            },
            param_values: nominal_params.clone(),
            objective_value: high_temp_obj,
            deviation_percent: ((high_temp_obj - nominal_obj) / high_temp_denom) * 100.0,
        };
        corners.push(high_temp_eval);

        // PVT Corner D: Cryo-CMOS (77.0 K Liquid Nitrogen)
        let mut cryo_context = self.engine.context.clone();
        cryo_context.temperature_kelvin = 77.0;
        let mut cryo_graph = self.engine.graph.clone();
        for (id, &val) in &nominal_params {
            let _ = cryo_graph.update_component_value(id, val);
        }
        let cryo_sol = solve_transient(
            &cryo_graph,
            &cryo_context,
            &self.engine.options,
        )?;
        let cryo_obj = self.engine.evaluate_objective_value(&cryo_sol);
        let cryo_denom = if nominal_obj.abs() > 1e-14 {
            nominal_obj.abs()
        } else {
            1.0
        };
        let cryo_eval = CornerEvaluation {
            corner: CornerType::PvtExtreme {
                corner_name: "Cryo-CMOS (77K)".to_string(),
            },
            param_values: nominal_params,
            objective_value: cryo_obj,
            deviation_percent: ((cryo_obj - nominal_obj) / cryo_denom) * 100.0,
        };
        corners.push(cryo_eval);

        // Compute maximum degradation percentage
        let mut max_deg = worst_max
            .deviation_percent
            .abs()
            .max(worst_min.deviation_percent.abs());
        for c in &corners {
            if c.deviation_percent.abs() > max_deg {
                max_deg = c.deviation_percent.abs();
            }
        }

        Ok(WorstCaseSummary {
            nominal: nominal_eval,
            worst_max,
            worst_min,
            corners,
            max_degradation_percent: max_deg,
        })
    }
}
