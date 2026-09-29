//! Gate-tunable 2D Valley Hall Transistor solver and non-linear logic routing.
//!
//! # Physical Formalism
//! - Dual-gate displacement field $D_\perp$ tunes inversion asymmetry and Berry dipole $D_{xz}$.
//! - Transistor ON/OFF state switching:
//!   $$\mathcal{R}_{\mathrm{on/off}} = 20 \log_{10}\left( \frac{V_{\mathrm{out,on}}}{V_{\mathrm{out,off}}} \right) > 20\text{ dB}$$

use phonon_models::valleytronics::{TmdMaterialParams, ValleyHallTransistor};

/// Result of Valley Hall Transistor evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyTransistorResult {
    /// Gate displacement field $D_\perp$ in V/nm.
    pub displacement_field_v_nm: f64,
    /// ON/OFF switching ratio in decibels ($\ge 20\text{ dB}$ required).
    pub on_off_ratio_db: f64,
    /// Rectification ratio in the ON state in decibels.
    pub on_rectification_db: f64,
    /// Non-linear transverse voltage output $V_{\perp}^{(2\omega)}$ in Volts.
    pub output_transverse_voltage_v: f64,
}

/// Valley Hall Transistor solver.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyHallTransistorSolver {
    pub transistor: ValleyHallTransistor,
}

impl ValleyHallTransistorSolver {
    pub fn new(params: TmdMaterialParams) -> Self {
        let transistor = ValleyHallTransistor::new(params);
        Self { transistor }
    }

    /// Solves transistor switching characteristics under applied drive field.
    pub fn solve(&mut self, e_field_v_m: f64) -> ValleyTransistorResult {
        let on_off_db = self.transistor.on_off_switching_ratio_db(e_field_v_m);
        let on_db = self.transistor.dipole.rectification_ratio_db(e_field_v_m);

        let channel_w_m = self.transistor.channel_width_nm * 1e-9;
        let v_out = 1e-3 * (on_db / 20.0) * (e_field_v_m * channel_w_m);

        ValleyTransistorResult {
            displacement_field_v_nm: self.transistor.displacement_field_v_nm,
            on_off_ratio_db: on_off_db,
            on_rectification_db: on_db,
            output_transverse_voltage_v: v_out,
        }
    }
}
