#![deny(unsafe_code)]

//! Continuous-time backward adjoint differential equation sensitivity engine for non-linear transient circuits.
//!
//! Evaluates performance functional sensitivities with respect to all circuit parameters in a single backward pass:
//! $$J(\mathbf{x}, \mathbf{p}) = \int_0^T g(\mathbf{x}(t), \mathbf{p}, t)\, dt + h(\mathbf{x}(T), \mathbf{p})$$
//!
//! Backward integration from $t = T$ down to $t = 0$:
//! $$\mathbf{C}^T \dot{\boldsymbol{\lambda}}(t) - \mathbf{J}_f(t)^T \boldsymbol{\lambda}(t) = -\left( \frac{\partial g}{\partial \mathbf{x}} \right)^T$$
//! with terminal condition:
//! $$\mathbf{C}^T \boldsymbol{\lambda}(T) = \left( \frac{\partial h}{\partial \mathbf{x}(T)} \right)^T$$
//!
//! Parametric gradient computation:
//! $$\frac{dJ}{dp_k} = \int_0^T \left( \frac{\partial g}{\partial p_k} - \boldsymbol{\lambda}^T(t) \left( \frac{\partial \mathbf{f}}{\partial p_k} + \frac{\partial \mathbf{C}}{\partial p_k} \dot{\mathbf{x}}(t) \right) \right) dt + \frac{\partial h}{\partial p_k}$$

use crate::error::SolverError;
use crate::mna::assembler::{assemble_mna_dc, SolverOptions};
use crate::mna::non_linear_solver::ModelContext;
use crate::sparse::builder::SparseMatrixBuilder;
use crate::sparse::csc::SparseMatrixCsc;
use crate::sparse::lu::SparseLuFactorization;
use crate::sparse::markowitz::MarkowitzOptions;
use crate::transient::{solve_transient, TransientOptions, TransientSolution};
use phonon_core::{CircuitGraph, ComponentRecord, NodeId};
use std::collections::HashMap;

/// Circuit performance objective functional to evaluate and optimize.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectiveKind {
    /// Terminal voltage at a specific circuit node at simulation stop time $T$: $h = V_{node}(T)$.
    TerminalVoltage { node: usize },
    /// Integral of squared error tracking a target voltage: $g = (V_{node}(t) - V_{target})^2$.
    IntegralSquaredError { node: usize, target_v: f64 },
    /// Total electrical energy dissipated across all resistive elements: $g = \sum (V_p - V_n)^2 / R$.
    EnergyDissipated,
    /// Maximum voltage peak overshoot at a designated node: $\max_{t \in [0, T]} V_{node}(t)$.
    PeakOvershoot { node: usize },
    /// Propagation delay to cross a specific threshold voltage: $t_{th}$ where $V_{node}(t_{th}) = V_{threshold}$.
    DelayToThreshold { node: usize, threshold: f64 },
}

/// Tunable circuit component parameter subjected to sensitivity analysis and corner optimization.
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitParameter {
    /// Two-terminal linear resistor.
    Resistance {
        id: String,
        nominal: f64,
        tolerance: f64,
    },
    /// Linear charge-storage capacitor.
    Capacitance {
        id: String,
        nominal: f64,
        tolerance: f64,
    },
    /// Linear magnetic flux inductor.
    Inductance {
        id: String,
        nominal: f64,
        tolerance: f64,
    },
    /// MOSFET channel width scaling conduction transconductance.
    MosfetWidth {
        id: String,
        nominal: f64,
        tolerance: f64,
    },
}

impl CircuitParameter {
    /// Returns the unique identifier of the parameter.
    pub fn id(&self) -> &str {
        match self {
            Self::Resistance { id, .. }
            | Self::Capacitance { id, .. }
            | Self::Inductance { id, .. }
            | Self::MosfetWidth { id, .. } => id.as_str(),
        }
    }

    /// Returns the nominal value of the parameter.
    pub fn nominal(&self) -> f64 {
        match self {
            Self::Resistance { nominal, .. }
            | Self::Capacitance { nominal, .. }
            | Self::Inductance { nominal, .. }
            | Self::MosfetWidth { nominal, .. } => *nominal,
        }
    }

    /// Returns the relative fractional tolerance (e.g. 0.05 for +/-5%).
    pub fn tolerance(&self) -> f64 {
        match self {
            Self::Resistance { tolerance, .. }
            | Self::Capacitance { tolerance, .. }
            | Self::Inductance { tolerance, .. }
            | Self::MosfetWidth { tolerance, .. } => *tolerance,
        }
    }

    /// Returns a new parameter instance with an updated value.
    pub fn with_value(&self, value: f64) -> Self {
        match self {
            Self::Resistance { id, tolerance, .. } => Self::Resistance {
                id: id.clone(),
                nominal: value,
                tolerance: *tolerance,
            },
            Self::Capacitance { id, tolerance, .. } => Self::Capacitance {
                id: id.clone(),
                nominal: value,
                tolerance: *tolerance,
            },
            Self::Inductance { id, tolerance, .. } => Self::Inductance {
                id: id.clone(),
                nominal: value,
                tolerance: *tolerance,
            },
            Self::MosfetWidth { id, tolerance, .. } => Self::MosfetWidth {
                id: id.clone(),
                nominal: value,
                tolerance: *tolerance,
            },
        }
    }

    /// Minimum allowable parameter value within the tolerance window.
    pub fn min_value(&self) -> f64 {
        self.nominal() * (1.0 - self.tolerance())
    }

    /// Maximum allowable parameter value within the tolerance window.
    pub fn max_value(&self) -> f64 {
        self.nominal() * (1.0 + self.tolerance())
    }
}

/// Sensitivity result for a single circuit parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct SensitivityResult {
    /// Parameter identifier.
    pub param_id: String,
    /// Nominal parameter value.
    pub nominal_value: f64,
    /// Un-normalized parametric gradient: $dJ / dp$.
    pub gradient: f64,
    /// Normalized percentage sensitivity: $(p_0 / J_0) \cdot (dJ / dp)$.
    pub normalized_sensitivity: f64,
}

/// Continuous-time backward adjoint sensitivity engine for non-linear transient circuit simulation.
#[derive(Debug, Clone)]
pub struct AdjointSensitivityEngine {
    pub graph: CircuitGraph,
    pub context: ModelContext,
    pub options: TransientOptions,
    pub objective: ObjectiveKind,
    pub parameters: Vec<CircuitParameter>,
}

impl AdjointSensitivityEngine {
    /// Constructs a new AdjointSensitivityEngine.
    pub fn new(
        graph: CircuitGraph,
        context: ModelContext,
        options: TransientOptions,
        objective: ObjectiveKind,
        parameters: Vec<CircuitParameter>,
    ) -> Self {
        Self {
            graph,
            context,
            options,
            objective,
            parameters,
        }
    }

    /// Adds a circuit parameter to the analysis registry.
    pub fn with_parameter(mut self, param: CircuitParameter) -> Self {
        self.parameters.push(param);
        self
    }

    /// Updates the active circuit objective functional.
    pub fn set_objective(&mut self, objective: ObjectiveKind) {
        self.objective = objective;
    }

    /// Evaluates the scalar objective functional value $J(\mathbf{x}, \mathbf{p})$ for a transient solution.
    pub fn evaluate_objective_value(&self, solution: &TransientSolution) -> f64 {
        if solution.steps.is_empty() {
            return 0.0;
        }

        match &self.objective {
            ObjectiveKind::TerminalVoltage { node } => {
                let last = &solution.steps[solution.steps.len() - 1];
                if *node == 0 {
                    0.0
                } else if *node < last.voltages.len() {
                    last.voltages[*node]
                } else {
                    0.0
                }
            }
            ObjectiveKind::IntegralSquaredError { node, target_v } => {
                let mut integral = 0.0;
                for i in 0..solution.steps.len() - 1 {
                    let s0 = &solution.steps[i];
                    let s1 = &solution.steps[i + 1];
                    let dt = s1.time - s0.time;
                    let v0 = if *node < s0.voltages.len() {
                        s0.voltages[*node]
                    } else {
                        0.0
                    };
                    let v1 = if *node < s1.voltages.len() {
                        s1.voltages[*node]
                    } else {
                        0.0
                    };
                    let err0 = (v0 - target_v).powi(2);
                    let err1 = (v1 - target_v).powi(2);
                    integral += 0.5 * dt * (err0 + err1);
                }
                integral
            }
            ObjectiveKind::EnergyDissipated => {
                let mut total_energy = 0.0;
                for i in 0..solution.steps.len() - 1 {
                    let s0 = &solution.steps[i];
                    let s1 = &solution.steps[i + 1];
                    let dt = s1.time - s0.time;

                    let p0 = self.compute_instantaneous_power(s0);
                    let p1 = self.compute_instantaneous_power(s1);
                    total_energy += 0.5 * dt * (p0 + p1);
                }
                total_energy
            }
            ObjectiveKind::PeakOvershoot { node } => {
                let mut max_v = f64::NEG_INFINITY;
                for step in &solution.steps {
                    let v = if *node < step.voltages.len() {
                        step.voltages[*node]
                    } else {
                        0.0
                    };
                    if v > max_v {
                        max_v = v;
                    }
                }
                if max_v.is_infinite() {
                    0.0
                } else {
                    max_v
                }
            }
            ObjectiveKind::DelayToThreshold { node, threshold } => {
                for i in 0..solution.steps.len() - 1 {
                    let s0 = &solution.steps[i];
                    let s1 = &solution.steps[i + 1];
                    let v0 = if *node < s0.voltages.len() {
                        s0.voltages[*node]
                    } else {
                        0.0
                    };
                    let v1 = if *node < s1.voltages.len() {
                        s1.voltages[*node]
                    } else {
                        0.0
                    };

                    if (v0 <= *threshold && v1 >= *threshold)
                        || (v0 >= *threshold && v1 <= *threshold)
                    {
                        let fraction = if (v1 - v0).abs() > 1e-15 {
                            (*threshold - v0) / (v1 - v0)
                        } else {
                            0.0
                        };
                        return s0.time + fraction * (s1.time - s0.time);
                    }
                }
                solution.steps.last().map(|s| s.time).unwrap_or(0.0)
            }
        }
    }

    /// Computes instantaneous resistive power dissipated across the circuit at a step.
    fn compute_instantaneous_power(&self, step: &crate::transient::TransientStep) -> f64 {
        let mut p = 0.0;
        for comp in self.graph.components() {
            if let ComponentRecord::Resistor {
                pos,
                neg,
                resistance,
                ..
            } = comp
            {
                let vp = if pos.is_ground() {
                    0.0
                } else if pos.index() < step.voltages.len() {
                    step.voltages[pos.index()]
                } else {
                    0.0
                };
                let vn = if neg.is_ground() {
                    0.0
                } else if neg.index() < step.voltages.len() {
                    step.voltages[neg.index()]
                } else {
                    0.0
                };
                let dv = vp - vn;
                p += (dv * dv) / resistance;
            }
        }
        p
    }

    /// Solves the transient circuit and executes backward adjoint integration to compute exact sensitivities.
    pub fn solve(&self) -> Result<Vec<SensitivityResult>, SolverError> {
        let forward_sol = solve_transient(&self.graph, &self.context, &self.options)?;
        let n_steps = forward_sol.steps.len();
        if n_steps < 2 {
            return Err(SolverError::NumericalAnomaly {
                detail: "Insufficient transient steps for backward adjoint sensitivity analysis"
                    .to_string(),
            });
        }

        let j0 = self.evaluate_objective_value(&forward_sol);
        let mna = assemble_mna_dc(&self.graph, &SolverOptions::default())?;
        let dim = mna.total_dim;
        let active_nodes = mna.active_nodes;

        // Build continuous state trajectory x(t) and calculate derivative dx/dt
        let mut state_history = Vec::with_capacity(n_steps);
        let mut times = Vec::with_capacity(n_steps);
        for step in &forward_sol.steps {
            times.push(step.time);
            let mut x = vec![0.0; dim];
            for k in 1..=active_nodes {
                if k < step.voltages.len() {
                    x[k - 1] = step.voltages[k];
                }
            }
            for (b_idx, &curr) in step.branch_currents.iter().enumerate() {
                if active_nodes + b_idx < dim {
                    x[active_nodes + b_idx] = curr;
                }
            }
            state_history.push(x);
        }

        // Discrete time derivatives dx/dt via central/boundary differences
        let mut dx_dt = vec![vec![0.0; dim]; n_steps];
        for i in 0..n_steps {
            if i == 0 {
                let dt = (times[1] - times[0]).max(1e-15);
                for k in 0..dim {
                    dx_dt[0][k] = (state_history[1][k] - state_history[0][k]) / dt;
                }
            } else if i == n_steps - 1 {
                let dt = (times[n_steps - 1] - times[n_steps - 2]).max(1e-15);
                for k in 0..dim {
                    dx_dt[n_steps - 1][k] =
                        (state_history[n_steps - 1][k] - state_history[n_steps - 2][k]) / dt;
                }
            } else {
                let dt = (times[i + 1] - times[i - 1]).max(1e-15);
                for k in 0..dim {
                    dx_dt[i][k] = (state_history[i + 1][k] - state_history[i - 1][k]) / dt;
                }
            }
        }

        // Determine active cutoff index for peak overshoot or delay to threshold
        let cutoff_step = match &self.objective {
            ObjectiveKind::PeakOvershoot { node } => {
                let mut best_idx = 0;
                let mut max_v = f64::NEG_INFINITY;
                for (i, step) in forward_sol.steps.iter().enumerate() {
                    let v = if *node < step.voltages.len() {
                        step.voltages[*node]
                    } else {
                        0.0
                    };
                    if v > max_v {
                        max_v = v;
                        best_idx = i;
                    }
                }
                best_idx.max(1)
            }
            ObjectiveKind::DelayToThreshold { node, threshold } => {
                let mut cross_idx = n_steps - 1;
                for i in 0..n_steps - 1 {
                    let v0 = if *node < forward_sol.steps[i].voltages.len() {
                        forward_sol.steps[i].voltages[*node]
                    } else {
                        0.0
                    };
                    let v1 = if *node < forward_sol.steps[i + 1].voltages.len() {
                        forward_sol.steps[i + 1].voltages[*node]
                    } else {
                        0.0
                    };
                    if (v0 <= *threshold && v1 >= *threshold)
                        || (v0 >= *threshold && v1 <= *threshold)
                    {
                        cross_idx = i + 1;
                        break;
                    }
                }
                cross_idx.max(1)
            }
            _ => n_steps - 1,
        };

        // Initialize adjoint multiplier trajectory lambda[n]
        let mut lambda_history = vec![vec![0.0; dim]; n_steps];

        // Terminal adjoint condition at t = cutoff_step
        let dt_term = (times[cutoff_step] - times[cutoff_step - 1]).max(1e-15);
        let (term_dh_dx, term_dg_dx) =
            self.eval_objective_derivatives_at_step(&forward_sol, cutoff_step, active_nodes, dim);

        // Terminal matrix: A_term = (C^T + dt * G^T)
        let a_term_csc = build_adjoint_system_matrix(&mna.c_matrix, &mna.g_matrix, dt_term);
        let lu_term =
            SparseLuFactorization::factor(&a_term_csc, &MarkowitzOptions::default())?;

        let mut b_term = vec![0.0; dim];
        for k in 0..dim {
            b_term[k] = term_dh_dx[k] + dt_term * term_dg_dx[k];
        }

        let mut lambda_term = vec![0.0; dim];
        lu_term.solve(&b_term, &mut lambda_term)?;
        lambda_history[cutoff_step] = lambda_term;

        // Backward adjoint time-marching from n = cutoff_step - 1 down to 0
        for n in (0..cutoff_step).rev() {
            let dt = (times[n + 1] - times[n]).max(1e-15);
            let (_, dg_dx) =
                self.eval_objective_derivatives_at_step(&forward_sol, n, active_nodes, dim);

            // Adjoint system matrix: A_n = C^T + dt * G^T
            let a_csc = build_adjoint_system_matrix(&mna.c_matrix, &mna.g_matrix, dt);
            let lu = SparseLuFactorization::factor(&a_csc, &MarkowitzOptions::default())?;

            // RHS: b_adj = C^T * lambda_{n+1} + dt * (dg/dx)^T
            let mut c_t_lambda = vec![0.0; dim];
            matvec_transpose(&mna.c_matrix, &lambda_history[n + 1], &mut c_t_lambda);

            let mut b_adj = vec![0.0; dim];
            for k in 0..dim {
                b_adj[k] = c_t_lambda[k] + dt * dg_dx[k];
            }

            let mut lambda_n = vec![0.0; dim];
            lu.solve(&b_adj, &mut lambda_n)?;
            lambda_history[n] = lambda_n;
        }

        // Evaluate parameter sensitivities via trapezoidal numerical quadrature
        let mut results = Vec::with_capacity(self.parameters.len());
        for param in &self.parameters {
            let mut gradient = 0.0;

            for n in 0..cutoff_step {
                let dt = times[n + 1] - times[n];
                let integrand_n = self.eval_parametric_integrand(
                    param,
                    &forward_sol.steps[n],
                    &state_history[n],
                    &dx_dt[n],
                    &lambda_history[n],
                    active_nodes,
                );
                let integrand_n1 = self.eval_parametric_integrand(
                    param,
                    &forward_sol.steps[n + 1],
                    &state_history[n + 1],
                    &dx_dt[n + 1],
                    &lambda_history[n + 1],
                    active_nodes,
                );
                gradient += 0.5 * dt * (integrand_n + integrand_n1);
            }

            // Direct terminal metric sensitivity dh/dp (if any)
            let dh_dp = match &self.objective {
                ObjectiveKind::TerminalVoltage { .. } => 0.0,
                ObjectiveKind::IntegralSquaredError { .. } => 0.0,
                ObjectiveKind::EnergyDissipated => 0.0,
                ObjectiveKind::PeakOvershoot { .. } => 0.0,
                ObjectiveKind::DelayToThreshold { .. } => 0.0,
            };
            gradient += dh_dp;

            let nom = param.nominal();
            let norm_sens = if j0.abs() > 1e-14 {
                (nom / j0) * gradient
            } else {
                (nom / 1e-14) * gradient
            };

            results.push(SensitivityResult {
                param_id: param.id().to_string(),
                nominal_value: nom,
                gradient,
                normalized_sensitivity: norm_sens,
            });
        }

        Ok(results)
    }

    /// Evaluates partial derivatives of objective functional dg/dx and dh/dx at a specific time step.
    fn eval_objective_derivatives_at_step(
        &self,
        solution: &TransientSolution,
        step_idx: usize,
        active_nodes: usize,
        dim: usize,
    ) -> (Vec<f64>, Vec<f64>) {
        let mut dh_dx = vec![0.0; dim];
        let mut dg_dx = vec![0.0; dim];

        let step = &solution.steps[step_idx];

        match &self.objective {
            ObjectiveKind::TerminalVoltage { node } => {
                if *node > 0 && *node <= active_nodes {
                    dh_dx[*node - 1] = 1.0;
                }
            }
            ObjectiveKind::IntegralSquaredError { node, target_v } => {
                if *node > 0 && *node <= active_nodes {
                    let v = if *node < step.voltages.len() {
                        step.voltages[*node]
                    } else {
                        0.0
                    };
                    dg_dx[*node - 1] = 2.0 * (v - target_v);
                }
            }
            ObjectiveKind::EnergyDissipated => {
                for comp in self.graph.components() {
                    if let ComponentRecord::Resistor {
                        pos,
                        neg,
                        resistance,
                        ..
                    } = comp
                    {
                        let vp = if pos.is_ground() {
                            0.0
                        } else if pos.index() < step.voltages.len() {
                            step.voltages[pos.index()]
                        } else {
                            0.0
                        };
                        let vn = if neg.is_ground() {
                            0.0
                        } else if neg.index() < step.voltages.len() {
                            step.voltages[neg.index()]
                        } else {
                            0.0
                        };
                        let dv = vp - vn;
                        let i_r = 2.0 * dv / resistance;
                        if !pos.is_ground() && pos.index() <= active_nodes {
                            dg_dx[pos.index() - 1] += i_r;
                        }
                        if !neg.is_ground() && neg.index() <= active_nodes {
                            dg_dx[neg.index() - 1] -= i_r;
                        }
                    }
                }
            }
            ObjectiveKind::PeakOvershoot { node } => {
                if *node > 0 && *node <= active_nodes {
                    dh_dx[*node - 1] = 1.0;
                }
            }
            ObjectiveKind::DelayToThreshold { node, .. } => {
                if *node > 0 && *node <= active_nodes {
                    let slope = if step_idx > 0 {
                        let dt = (step.time - solution.steps[step_idx - 1].time).max(1e-15);
                        let v_now = step.voltages[*node];
                        let v_prev = solution.steps[step_idx - 1].voltages[*node];
                        (v_now - v_prev) / dt
                    } else {
                        1.0
                    };
                    let inv_slope = if slope.abs() > 1e-15 {
                        -1.0 / slope
                    } else {
                        -1.0
                    };
                    dh_dx[*node - 1] = inv_slope;
                }
            }
        }

        (dh_dx, dg_dx)
    }

    /// Evaluates the integrand $\frac{\partial g}{\partial p} - \boldsymbol{\lambda}^T \left( \frac{\partial \mathbf{f}}{\partial p} + \frac{\partial \mathbf{C}}{\partial p} \dot{\mathbf{x}} \right)$.
    fn eval_parametric_integrand(
        &self,
        param: &CircuitParameter,
        _step: &crate::transient::TransientStep,
        state: &[f64],
        dx_dt: &[f64],
        lambda: &[f64],
        active_nodes: usize,
    ) -> f64 {
        let get_v = |node: NodeId| -> f64 {
            if node.is_ground() {
                0.0
            } else {
                let idx = node.index() - 1;
                if idx < state.len() {
                    state[idx]
                } else {
                    0.0
                }
            }
        };

        let get_dv_dt = |node: NodeId| -> f64 {
            if node.is_ground() {
                0.0
            } else {
                let idx = node.index() - 1;
                if idx < dx_dt.len() {
                    dx_dt[idx]
                } else {
                    0.0
                }
            }
        };

        let get_lam = |node: NodeId| -> f64 {
            if node.is_ground() {
                0.0
            } else {
                let idx = node.index() - 1;
                if idx < lambda.len() {
                    lambda[idx]
                } else {
                    0.0
                }
            }
        };

        let mut dg_dp = 0.0;
        let mut lambda_term = 0.0;

        match param {
            CircuitParameter::Resistance { id, nominal, .. } => {
                if let Some(comp) = self.graph.get_component(id) {
                    if let ComponentRecord::Resistor { pos, neg, .. } = comp {
                        let dv = get_v(*pos) - get_v(*neg);
                        let d_lam = get_lam(*pos) - get_lam(*neg);

                        // df/dR has -dv/R^2 injected out of pos, +dv/R^2 injected into neg
                        // lambda^T * (df/dR) = d_lam * (-dv / R^2)
                        let df_dr_proj = -d_lam * (dv / (nominal * nominal));
                        lambda_term = df_dr_proj;

                        if matches!(self.objective, ObjectiveKind::EnergyDissipated) {
                            dg_dp = -(dv * dv) / (nominal * nominal);
                        }
                    }
                }
            }
            CircuitParameter::Capacitance { id, .. } => {
                if let Some(comp) = self.graph.get_component(id) {
                    if let ComponentRecord::Capacitor { pos, neg, .. } = comp {
                        let dv_dt_diff = get_dv_dt(*pos) - get_dv_dt(*neg);
                        let d_lam = get_lam(*pos) - get_lam(*neg);

                        // dC_mat/dC * dx/dt = dv_dt_diff at pos, -dv_dt_diff at neg
                        // lambda^T * (dC_mat/dC * dx/dt) = d_lam * dv_dt_diff
                        lambda_term = d_lam * dv_dt_diff;
                    }
                }
            }
            CircuitParameter::Inductance { id, .. } => {
                if let Some(comp) = self.graph.get_component(id) {
                    if let ComponentRecord::Inductor { branch, .. } = comp {
                        let br_idx = active_nodes + branch.index();
                        if br_idx < lambda.len() && br_idx < dx_dt.len() {
                            let lam_br = lambda[br_idx];
                            let di_dt = dx_dt[br_idx];
                            // C_mat has -L at (br, br). dC_mat/dL = -1.
                            // lambda^T * (dC_mat/dL * dx/dt) = lam_br * (-1) * di_dt
                            lambda_term = -lam_br * di_dt;
                        }
                    }
                }
            }
            CircuitParameter::MosfetWidth { id, nominal, .. } => {
                if let Some(comp) = self.graph.get_component(id) {
                    if let ComponentRecord::Mosfet {
                        drain,
                        gate,
                        source,
                        bulk,
                        ..
                    } = comp
                    {
                        let vd = get_v(*drain);
                        let vg = get_v(*gate);
                        let vs = get_v(*source);
                        let vb = get_v(*bulk);
                        let model = self.context.get_mosfet_model(id);
                        let eval = model.evaluate(vd, vg, vs, vb, 300.0);

                        let d_lam = get_lam(*drain) - get_lam(*source);
                        let dids_dw = eval.i_ds / nominal.max(1e-12);
                        lambda_term = d_lam * dids_dw;
                    }
                }
            }
        }

        dg_dp - lambda_term
    }

    /// Evaluates finite-difference parameter sensitivities to provide ground-truth analytical cross-validation.
    pub fn solve_finite_difference(
        &self,
        rel_perturbation: f64,
    ) -> Result<Vec<SensitivityResult>, SolverError> {
        let nominal_sol = solve_transient(&self.graph, &self.context, &self.options)?;
        let j0 = self.evaluate_objective_value(&nominal_sol);

        let delta_frac = rel_perturbation.abs().clamp(1e-6, 0.05);
        let mut results = Vec::with_capacity(self.parameters.len());

        for param in &self.parameters {
            let nom = param.nominal();
            let delta = nom * delta_frac;

            let mut graph_perturbed = self.graph.clone();
            let _ = graph_perturbed.update_component_value(param.id(), nom + delta);

            let sol_perturbed =
                solve_transient(&graph_perturbed, &self.context, &self.options)?;
            let j_pert = self.evaluate_objective_value(&sol_perturbed);

            let gradient = (j_pert - j0) / delta;
            let norm_sens = if j0.abs() > 1e-14 {
                (nom / j0) * gradient
            } else {
                (nom / 1e-14) * gradient
            };

            results.push(SensitivityResult {
                param_id: param.id().to_string(),
                nominal_value: nom,
                gradient,
                normalized_sensitivity: norm_sens,
            });
        }

        Ok(results)
    }
}

/// Assembles adjoint time-step system matrix: $\mathbf{A} = \mathbf{C}^T + \Delta t \mathbf{J}_f^T$.
fn build_adjoint_system_matrix(
    c_matrix: &SparseMatrixCsc,
    g_matrix: &SparseMatrixCsc,
    dt: f64,
) -> SparseMatrixCsc {
    let n = c_matrix.nrows();
    let mut builder = SparseMatrixBuilder::with_capacity(n, n, c_matrix.nnz() + g_matrix.nnz() + n);

    // Add C^T entries: if C has (r, c, v), C^T has (c, r, v)
    for col in 0..c_matrix.ncols() {
        let start = c_matrix.col_ptrs()[col];
        let end = c_matrix.col_ptrs()[col + 1];
        for k in start..end {
            let row = c_matrix.row_indices()[k];
            let val = c_matrix.values()[k];
            builder.add(col, row, val);
        }
    }

    // Add dt * G^T entries: if G has (r, c, v), dt * G^T has (c, r, dt * v)
    for col in 0..g_matrix.ncols() {
        let start = g_matrix.col_ptrs()[col];
        let end = g_matrix.col_ptrs()[col + 1];
        for k in start..end {
            let row = g_matrix.row_indices()[k];
            let val = g_matrix.values()[k];
            builder.add(col, row, dt * val);
        }
    }

    builder.build_csc()
}

/// Computes sparse matrix transpose vector multiplication: $\mathbf{y} = \mathbf{M}^T \mathbf{x}$.
fn matvec_transpose(matrix: &SparseMatrixCsc, x: &[f64], y: &mut [f64]) {
    for yi in y.iter_mut() {
        *yi = 0.0;
    }
    for j in 0..matrix.ncols() {
        let start = matrix.col_ptrs()[j];
        let end = matrix.col_ptrs()[j + 1];
        let mut sum = 0.0;
        for k in start..end {
            let row = matrix.row_indices()[k];
            let val = matrix.values()[k];
            if row < x.len() {
                sum += val * x[row];
            }
        }
        if j < y.len() {
            y[j] = sum;
        }
    }
}
