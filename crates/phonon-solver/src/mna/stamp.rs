//! Component stamp algorithms for Modified Nodal Analysis (MNA).

use crate::sparse::builder::SparseMatrixBuilder;
use phonon_core::NodeId;

/// Translates a circuit NodeId to an MNA zero-indexed row/column.
/// Returns None if the node is Ground (NodeId 0), since ground is eliminated from MNA.
#[inline(always)]
pub fn node_to_mna_idx(node: NodeId) -> Option<usize> {
    if node.is_ground() {
        None
    } else {
        Some(node.index() - 1)
    }
}

/// Stamps a two-terminal conductance $G = 1/R$ into the conductance matrix $\mathbf{G}$.
#[inline]
pub fn stamp_conductance(
    matrix: &mut SparseMatrixBuilder,
    pos: NodeId,
    neg: NodeId,
    conductance: f64,
) {
    let p_idx = node_to_mna_idx(pos);
    let n_idx = node_to_mna_idx(neg);

    match (p_idx, n_idx) {
        (Some(p), Some(n)) => {
            matrix.add(p, p, conductance);
            matrix.add(n, n, conductance);
            matrix.add(p, n, -conductance);
            matrix.add(n, p, -conductance);
        }
        (Some(p), None) => {
            matrix.add(p, p, conductance);
        }
        (None, Some(n)) => {
            matrix.add(n, n, conductance);
        }
        (None, None) => {}
    }
}

/// Stamps an independent current source into the RHS vector $\mathbf{b}$.
/// Current $I_{dc}$ flows from $pos$ to $neg$ through the external network.
#[inline]
pub fn stamp_current_source(rhs: &mut [f64], pos: NodeId, neg: NodeId, current: f64) {
    if let Some(p) = node_to_mna_idx(pos) {
        rhs[p] -= current;
    }
    if let Some(n) = node_to_mna_idx(neg) {
        rhs[n] += current;
    }
}

/// Stamps an independent voltage source with branch variable at MNA index `branch_mna_idx`.
/// Enforces $V_{pos} - V_{neg} = V_{dc}$.
#[inline]
pub fn stamp_voltage_source(
    matrix: &mut SparseMatrixBuilder,
    rhs: &mut [f64],
    pos: NodeId,
    neg: NodeId,
    branch_mna_idx: usize,
    dc_value: f64,
) {
    let p_idx = node_to_mna_idx(pos);
    let n_idx = node_to_mna_idx(neg);

    if let Some(p) = p_idx {
        matrix.add(p, branch_mna_idx, 1.0);
        matrix.add(branch_mna_idx, p, 1.0);
    }
    if let Some(n) = n_idx {
        matrix.add(n, branch_mna_idx, -1.0);
        matrix.add(branch_mna_idx, n, -1.0);
    }

    rhs[branch_mna_idx] += dc_value;
}

/// Stamps a Voltage-Controlled Voltage Source (VCVS):
/// $V_{out\_pos} - V_{out\_neg} = \text{gain} \cdot (V_{ctrl\_pos} - V_{ctrl\_neg})$.
#[inline]
pub fn stamp_vcvs(
    matrix: &mut SparseMatrixBuilder,
    out_pos: NodeId,
    out_neg: NodeId,
    ctrl_pos: NodeId,
    ctrl_neg: NodeId,
    branch_mna_idx: usize,
    gain: f64,
) {
    let op_idx = node_to_mna_idx(out_pos);
    let on_idx = node_to_mna_idx(out_neg);
    let cp_idx = node_to_mna_idx(ctrl_pos);
    let cn_idx = node_to_mna_idx(ctrl_neg);

    // KCL coupling for output current
    if let Some(op) = op_idx {
        matrix.add(op, branch_mna_idx, 1.0);
        matrix.add(branch_mna_idx, op, 1.0);
    }
    if let Some(on) = on_idx {
        matrix.add(on, branch_mna_idx, -1.0);
        matrix.add(branch_mna_idx, on, -1.0);
    }

    // Control voltage dependency
    if let Some(cp) = cp_idx {
        matrix.add(branch_mna_idx, cp, -gain);
    }
    if let Some(cn) = cn_idx {
        matrix.add(branch_mna_idx, cn, gain);
    }
}

/// Stamps a Voltage-Controlled Current Source (VCCS):
/// $I_{out} = g_m \cdot (V_{ctrl\_pos} - V_{ctrl\_neg})$.
#[inline]
pub fn stamp_vccs(
    matrix: &mut SparseMatrixBuilder,
    out_pos: NodeId,
    out_neg: NodeId,
    ctrl_pos: NodeId,
    ctrl_neg: NodeId,
    transconductance: f64,
) {
    let op_idx = node_to_mna_idx(out_pos);
    let on_idx = node_to_mna_idx(out_neg);
    let cp_idx = node_to_mna_idx(ctrl_pos);
    let cn_idx = node_to_mna_idx(ctrl_neg);

    if let (Some(op), Some(cp)) = (op_idx, cp_idx) {
        matrix.add(op, cp, transconductance);
    }
    if let (Some(op), Some(cn)) = (op_idx, cn_idx) {
        matrix.add(op, cn, -transconductance);
    }
    if let (Some(on), Some(cp)) = (on_idx, cp_idx) {
        matrix.add(on, cp, -transconductance);
    }
    if let (Some(on), Some(cn)) = (on_idx, cn_idx) {
        matrix.add(on, cn, transconductance);
    }
}

/// Stamps a linear capacitance $C$ into dynamic capacitance matrix $\mathbf{C}_{MNA}$.
#[inline]
pub fn stamp_capacitance(
    matrix: &mut SparseMatrixBuilder,
    pos: NodeId,
    neg: NodeId,
    capacitance: f64,
) {
    stamp_conductance(matrix, pos, neg, capacitance);
}

/// Stamps a linear inductance $L$ with auxiliary branch variable at `branch_mna_idx`.
#[inline]
pub fn stamp_inductor(
    g_matrix: &mut SparseMatrixBuilder,
    c_matrix: &mut SparseMatrixBuilder,
    pos: NodeId,
    neg: NodeId,
    branch_mna_idx: usize,
    inductance: f64,
) {
    let p_idx = node_to_mna_idx(pos);
    let n_idx = node_to_mna_idx(neg);

    if let Some(p) = p_idx {
        g_matrix.add(p, branch_mna_idx, 1.0);
        g_matrix.add(branch_mna_idx, p, 1.0);
    }
    if let Some(n) = n_idx {
        g_matrix.add(n, branch_mna_idx, -1.0);
        g_matrix.add(branch_mna_idx, n, -1.0);
    }

    // Dynamic inductive derivative: -L * dI/dt
    c_matrix.add(branch_mna_idx, branch_mna_idx, -inductance);
}

/// Stamps a linearized companion model for a Diode:
/// $I_D \approx g_d (V_p - V_n) - I_{eq}$, where $I_{eq} = g_d V_D^{(k)} - I_D(V_D^{(k)})$.
#[inline]
pub fn stamp_diode_companion(
    matrix: &mut SparseMatrixBuilder,
    rhs: &mut [f64],
    pos: NodeId,
    neg: NodeId,
    g_d: f64,
    i_eq: f64,
) {
    stamp_conductance(matrix, pos, neg, g_d);

    if let Some(p) = node_to_mna_idx(pos) {
        rhs[p] += i_eq;
    }
    if let Some(n) = node_to_mna_idx(neg) {
        rhs[n] -= i_eq;
    }
}

/// Linearized companion values for a 4-terminal MOSFET in Newton-Raphson iteration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MosfetCompanion {
    pub g_m: f64,
    pub g_ds: f64,
    pub g_mbs: f64,
    pub i_eq: f64,
}

/// Linearized companion values for a BJT in Newton-Raphson iteration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BjtCompanion {
    pub g_m: f64,
    pub g_pi: f64,
    pub g_o: f64,
    pub g_mu: f64,
    pub i_c_eq: f64,
    pub i_b_eq: f64,
}

/// Stamps a linearized companion model for a 4-terminal MOSFET:
/// $I_{ds} \approx g_{ds} V_{ds} + g_m V_{gs} + g_{mbs} V_{bs} - I_{eq}$.
#[inline]
pub fn stamp_mosfet_companion(
    matrix: &mut SparseMatrixBuilder,
    rhs: &mut [f64],
    drain: NodeId,
    gate: NodeId,
    source: NodeId,
    bulk: NodeId,
    comp: &MosfetCompanion,
) {
    let d = node_to_mna_idx(drain);
    let g = node_to_mna_idx(gate);
    let s = node_to_mna_idx(source);
    let b = node_to_mna_idx(bulk);

    // Drain row stamps (+Ids leaving source, entering drain -> positive at drain)
    if let Some(d_idx) = d {
        matrix.add(d_idx, d_idx, comp.g_ds);
        if let Some(s_idx) = s {
            matrix.add(d_idx, s_idx, -(comp.g_ds + comp.g_m + comp.g_mbs));
        }
        if let Some(g_idx) = g {
            matrix.add(d_idx, g_idx, comp.g_m);
        }
        if let Some(b_idx) = b {
            matrix.add(d_idx, b_idx, comp.g_mbs);
        }
        rhs[d_idx] += comp.i_eq;
    }

    // Source row stamps (negative of drain terms)
    if let Some(s_idx) = s {
        matrix.add(s_idx, s_idx, comp.g_ds + comp.g_m + comp.g_mbs);
        if let Some(d_idx) = d {
            matrix.add(s_idx, d_idx, -comp.g_ds);
        }
        if let Some(g_idx) = g {
            matrix.add(s_idx, g_idx, -comp.g_m);
        }
        if let Some(b_idx) = b {
            matrix.add(s_idx, b_idx, -comp.g_mbs);
        }
        rhs[s_idx] -= comp.i_eq;
    }
}

/// Stamps a linearized companion model for a Gummel-Poon BJT.
#[inline]
pub fn stamp_bjt_companion(
    matrix: &mut SparseMatrixBuilder,
    rhs: &mut [f64],
    collector: NodeId,
    base: NodeId,
    emitter: NodeId,
    comp: &BjtCompanion,
) {
    let c = node_to_mna_idx(collector);
    let b = node_to_mna_idx(base);
    let e = node_to_mna_idx(emitter);

    let g_oc = comp.g_o + comp.g_mu;

    // Collector row: I_C = g_oc * v_c + (g_m - g_oc) * v_b - g_m * v_e - I_C_eq
    if let Some(c_idx) = c {
        matrix.add(c_idx, c_idx, g_oc);
        if let Some(b_idx) = b {
            matrix.add(c_idx, b_idx, comp.g_m - g_oc);
        }
        if let Some(e_idx) = e {
            matrix.add(c_idx, e_idx, -comp.g_m);
        }
        rhs[c_idx] += comp.i_c_eq;
    }

    // Base row: I_B = -g_mu * v_c + (g_pi + g_mu) * v_b - g_pi * v_e - I_B_eq
    if let Some(b_idx) = b {
        matrix.add(b_idx, b_idx, comp.g_pi + comp.g_mu);
        if let Some(c_idx) = c {
            matrix.add(b_idx, c_idx, -comp.g_mu);
        }
        if let Some(e_idx) = e {
            matrix.add(b_idx, e_idx, -comp.g_pi);
        }
        rhs[b_idx] += comp.i_b_eq;
    }

    // Emitter row: I_E = -(I_C + I_B)
    if let Some(e_idx) = e {
        matrix.add(e_idx, e_idx, comp.g_m + comp.g_pi);
        if let Some(c_idx) = c {
            matrix.add(e_idx, c_idx, -comp.g_o);
        }
        if let Some(b_idx) = b {
            matrix.add(e_idx, b_idx, -(comp.g_m + comp.g_pi - comp.g_o));
        }
        rhs[e_idx] -= comp.i_c_eq + comp.i_b_eq;
    }
}
