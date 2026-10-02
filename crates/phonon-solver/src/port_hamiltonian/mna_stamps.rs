#![deny(unsafe_code)]

//! Port-Hamiltonian Symplectic Modified Nodal Analysis (MNA) Stamp Library.
//!
//! Formulates multi-physics discrete companion circuit stamps G_eq * v^{n+1} = i_eq^n,
//! enabling direct coupling of mechanical and acoustic Port-Hamiltonian nodes into
//! Phonon's sparse linear nodal circuits.

use super::dirac::{solve_linear_system, DiracInterconnection};

/// Companion circuit stamp mapping Port-Hamiltonian state updates into MNA format.
#[derive(Debug, Clone, PartialEq)]
pub struct PortHamiltonianMnaStamp {
    /// Local subsystem state dimension N.
    pub dim: usize,
    /// Timestep duration dt in seconds.
    pub dt: f64,
    /// Equivalent conductance matrix G_eq (N x N, row-major).
    pub g_eq: Vec<f64>,
    /// Equivalent history current vector i_eq (dimension N).
    pub i_eq: Vec<f64>,
}

impl PortHamiltonianMnaStamp {
    /// Creates a new MNA companion stamp initialized from a Dirac interconnection structure.
    pub fn new(dirac: &DiracInterconnection, dt: f64) -> Self {
        let n = dirac.dim;
        let mut stamp = Self {
            dim: n,
            dt,
            g_eq: vec![0.0; n * n],
            i_eq: vec![0.0; n],
        };
        stamp.compute_conductance_matrix(dirac);
        stamp
    }

    /// Evaluates the equivalent conductance matrix: G_eq = I - (dt / 2) * (J - R) * Q.
    pub fn compute_conductance_matrix(&mut self, dirac: &DiracInterconnection) {
        let n = self.dim;
        let half_dt = 0.5 * self.dt;

        // A = (J - R) * Q
        for i in 0..n {
            for j in 0..n {
                let mut a_ij = 0.0;
                for k in 0..n {
                    let jr = dirac.j_matrix[i * n + k] - dirac.r_matrix[i * n + k];
                    a_ij += jr * dirac.q_matrix[k * n + j];
                }
                let delta = if i == j { 1.0 } else { 0.0 };
                self.g_eq[i * n + j] = delta - half_dt * a_ij;
            }
        }
    }

    /// Updates the history equivalent source vector i_eq^n from the previous state x_n and input u:
    /// i_eq^n = (I + (dt / 2) * (J - R) * Q) * x_n + dt * B * u_mid.
    pub fn update_history(&mut self, dirac: &DiracInterconnection, x_n: &[f64], u_mid: &[f64]) {
        assert_eq!(x_n.len(), self.dim);
        let n = self.dim;
        let m = dirac.inputs;
        let half_dt = 0.5 * self.dt;

        for i in 0..n {
            let mut sum = 0.0;
            for j in 0..n {
                let mut a_ij = 0.0;
                for k in 0..n {
                    let jr = dirac.j_matrix[i * n + k] - dirac.r_matrix[i * n + k];
                    a_ij += jr * dirac.q_matrix[k * n + j];
                }
                let delta = if i == j { 1.0 } else { 0.0 };
                let m_rhs_ij = delta + half_dt * a_ij;
                sum += m_rhs_ij * x_n[j];
            }
            for k in 0..m {
                sum += self.dt * dirac.b_matrix[i * m + k] * u_mid[k];
            }
            self.i_eq[i] = sum;
        }
    }

    /// Solves the local companion stamp system G_eq * x^{n+1} = i_eq for the next state x^{n+1}.
    pub fn solve(&self) -> Vec<f64> {
        let n = self.dim;
        let mut m = self.g_eq.clone();
        let mut b = self.i_eq.clone();
        solve_linear_system(n, &mut m, &mut b)
    }

    /// Read-only slice access to G_eq.
    pub fn g_eq(&self) -> &[f64] {
        &self.g_eq
    }

    /// Read-only slice access to i_eq.
    pub fn i_eq(&self) -> &[f64] {
        &self.i_eq
    }

    /// Stamps the local G_eq and i_eq companion entries into a global MNA sparse/dense matrix.
    pub fn stamp_into_global(
        &self,
        global_g: &mut [f64],
        global_rhs: &mut [f64],
        node_map: &[usize],
        global_dim: usize,
    ) {
        assert_eq!(node_map.len(), self.dim);
        let n = self.dim;
        for i in 0..n {
            let gi = node_map[i];
            global_rhs[gi] += self.i_eq[i];
            for j in 0..n {
                let gj = node_map[j];
                global_g[gi * global_dim + gj] += self.g_eq[i * n + j];
            }
        }
    }
}
