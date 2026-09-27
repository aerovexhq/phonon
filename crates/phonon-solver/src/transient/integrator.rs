//! Numerical integration methods (TR-BDF2, Trapezoidal, Backward Euler) and companion models.

/// Numerical integration scheme for stiff ordinary differential and algebraic equations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IntegrationMethod {
    /// 1st-order Backward Euler. Strongly L-stable, ideal for initial step startup.
    BackwardEuler,
    /// 2nd-order Trapezoidal rule. A-stable, energy-conservative, but susceptible to ringing on stiff steps.
    Trapezoidal,
    /// 2nd-order composite TR-BDF2 (Bank et al., 1985).
    /// Strictly L-stable, zero trapezoidal ringing, with embedded Local Truncation Error (LTE) estimation.
    #[default]
    TrBdf2,
}

/// The canonical gamma parameter for TR-BDF2: $\gamma = 2 - \sqrt{2} \approx 0.5857864376$.
pub const TR_BDF2_GAMMA: f64 = 0.5857864376269049;

/// Companion linear model for a dynamic two-terminal capacitor $i = C \frac{dv}{dt}$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapacitorCompanion {
    /// Effective equivalent parallel conductance $G_{eq}$ (Siemens).
    pub g_eq: f64,
    /// Effective equivalent parallel current source $I_{eq}$ (Amperes), flowing from node 2 to node 1.
    pub i_eq: f64,
}

/// Companion linear model for a dynamic branch inductor $v = L \frac{di}{dt}$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InductorCompanion {
    /// Effective equivalent series resistance $R_{eq}$ (Ohms).
    pub r_eq: f64,
    /// Effective equivalent series voltage source $V_{eq}$ (Volts).
    pub v_eq: f64,
}

impl CapacitorCompanion {
    /// Computes the Backward Euler companion model for a step of length $h$.
    #[inline]
    pub fn backward_euler(c: f64, h: f64, v_n: f64) -> Self {
        let g_eq = c / h;
        let i_eq = g_eq * v_n;
        Self { g_eq, i_eq }
    }

    /// Computes the Trapezoidal companion model for a step of length $h$.
    #[inline]
    pub fn trapezoidal(c: f64, h: f64, v_n: f64, i_n: f64) -> Self {
        let g_eq = (2.0 * c) / h;
        let i_eq = g_eq * v_n + i_n;
        Self { g_eq, i_eq }
    }

    /// Computes TR-BDF2 Stage 1 (Trapezoidal with step $\gamma h$).
    #[inline]
    pub fn tr_bdf2_stage1(c: f64, h: f64, v_n: f64, i_n: f64) -> Self {
        Self::trapezoidal(c, TR_BDF2_GAMMA * h, v_n, i_n)
    }

    /// Computes TR-BDF2 Stage 2 (BDF2 with step $(1-\gamma) h$).
    #[inline]
    pub fn tr_bdf2_stage2(c: f64, h: f64, v_n: f64, v_gamma: f64) -> Self {
        let gamma = TR_BDF2_GAMMA;
        let g_eq = c * (2.0 - gamma) / ((1.0 - gamma) * h);
        let i_eq =
            c * (v_gamma / (gamma * (1.0 - gamma) * h) - ((1.0 - gamma) / (gamma * h)) * v_n);
        Self { g_eq, i_eq }
    }
}

impl InductorCompanion {
    /// Computes the Backward Euler companion model for a step of length $h$.
    #[inline]
    pub fn backward_euler(l: f64, h: f64, i_n: f64) -> Self {
        let r_eq = l / h;
        let v_eq = r_eq * i_n;
        Self { r_eq, v_eq }
    }

    /// Computes the Trapezoidal companion model for a step of length $h$.
    #[inline]
    pub fn trapezoidal(l: f64, h: f64, i_n: f64, v_n: f64) -> Self {
        let r_eq = (2.0 * l) / h;
        let v_eq = r_eq * i_n + v_n;
        Self { r_eq, v_eq }
    }

    /// Computes TR-BDF2 Stage 1 (Trapezoidal with step $\gamma h$).
    #[inline]
    pub fn tr_bdf2_stage1(l: f64, h: f64, i_n: f64, v_n: f64) -> Self {
        Self::trapezoidal(l, TR_BDF2_GAMMA * h, i_n, v_n)
    }

    /// Computes TR-BDF2 Stage 2 (BDF2 with step $(1-\gamma) h$).
    #[inline]
    pub fn tr_bdf2_stage2(l: f64, h: f64, i_n: f64, i_gamma: f64) -> Self {
        let gamma = TR_BDF2_GAMMA;
        let r_eq = l * (2.0 - gamma) / ((1.0 - gamma) * h);
        let v_eq =
            l * (i_gamma / (gamma * (1.0 - gamma) * h) - ((1.0 - gamma) / (gamma * h)) * i_n);
        Self { r_eq, v_eq }
    }
}
