//! GlobalFoundries GF180MCU Open-Source PDK device profiles.

use phonon_models::{MosfetModel, MosfetType};

/// GF180MCU device model catalog.
#[derive(Debug, Clone, PartialEq)]
pub struct Gf180McuPdk;

impl Gf180McuPdk {
    /// 3.3V standard NMOS (`gf180mcu_fd_pr__nfet_03v3`).
    /// Physical oxide thickness: tox = 6.8 nm.
    pub fn nfet_03v3(width_m: f64, length_m: f64) -> MosfetModel {
        MosfetModel {
            mos_type: MosfetType::Nmos,
            vth0: 0.60,
            w: width_m.max(0.28e-6),  // Min W = 0.28 um
            l: length_m.max(0.28e-6), // Min L = 0.28 um
            tox: 6.8e-9,
            mu0: 0.050,
            vsat: 1.0e5,
            lambda: 0.03,
            gamma: 0.55,
            phi_s: 0.70,
            eta_dibl: 0.06,
            subthreshold_n: 1.28,
            ..MosfetModel::default()
        }
    }

    /// 3.3V standard PMOS (`gf180mcu_fd_pr__pfet_03v3`).
    pub fn pfet_03v3(width_m: f64, length_m: f64) -> MosfetModel {
        MosfetModel {
            mos_type: MosfetType::Pmos,
            vth0: -0.65,
            w: width_m.max(0.28e-6),
            l: length_m.max(0.28e-6),
            tox: 6.8e-9,
            mu0: 0.014,
            vsat: 0.8e5,
            lambda: 0.04,
            gamma: 0.60,
            phi_s: 0.70,
            eta_dibl: 0.07,
            subthreshold_n: 1.32,
            ..MosfetModel::default()
        }
    }

    /// 6.0V high-voltage NMOS (`gf180mcu_fd_pr__nfet_06v0`).
    /// Thick gate oxide: tox = 12.8 nm.
    pub fn nfet_06v0(width_m: f64, length_m: f64) -> MosfetModel {
        MosfetModel {
            mos_type: MosfetType::Nmos,
            vth0: 0.75,
            w: width_m.max(0.60e-6),
            l: length_m.max(0.70e-6),
            tox: 12.8e-9,
            mu0: 0.040,
            vsat: 1.0e5,
            lambda: 0.015,
            gamma: 0.70,
            phi_s: 0.75,
            eta_dibl: 0.03,
            subthreshold_n: 1.22,
            ..MosfetModel::default()
        }
    }

    /// N+ diffusion sheet resistance in Ohms per square.
    pub fn nplus_sheet_resistance() -> f64 {
        85.0
    }

    /// P+ diffusion sheet resistance in Ohms per square.
    pub fn pplus_sheet_resistance() -> f64 {
        135.0
    }
}
