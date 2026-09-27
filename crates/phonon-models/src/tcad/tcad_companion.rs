//! MNA companion models and Norton equivalent representations for TCAD devices.

/// Norton equivalent companion circuit for a 2-terminal TCAD diode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TcadDiodeCompanion {
    /// Differential conductance dI_D / dV_D (Siemens).
    pub g_d: f64,
    /// Norton equivalent current source I_eq = g_d * V_D - I_D (Amperes).
    pub i_eq: f64,
}

impl TcadDiodeCompanion {
    pub fn new(i_d: f64, g_d: f64, v_d: f64) -> Self {
        Self {
            g_d,
            i_eq: g_d * v_d - i_d,
        }
    }
}

/// Norton equivalent companion circuit for a 4-terminal TCAD MOSFET.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TcadMosfetCompanion {
    /// Gate transconductance dI_DS / dV_GS (Siemens).
    pub g_m: f64,
    /// Channel output conductance dI_DS / dV_DS (Siemens).
    pub g_ds: f64,
    /// Body transconductance dI_DS / dV_BS (Siemens).
    pub g_mbs: f64,
    /// Norton equivalent current source (Amperes).
    pub i_eq: f64,
}

impl TcadMosfetCompanion {
    pub fn new(
        i_ds: f64,
        g_m: f64,
        g_ds: f64,
        g_mbs: f64,
        v_ds: f64,
        v_gs: f64,
        v_bs: f64,
    ) -> Self {
        let i_eq = g_m * v_gs + g_ds * v_ds + g_mbs * v_bs - i_ds;
        Self {
            g_m,
            g_ds,
            g_mbs,
            i_eq,
        }
    }
}
