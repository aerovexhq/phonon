//! SkyWater 130nm Open-Source PDK device profiles and physical constants.

use phonon_models::{MosfetModel, MosfetType};

/// SkyWater 130nm device model catalog.
#[derive(Debug, Clone, PartialEq)]
pub struct Sky130Pdk;

impl Sky130Pdk {
    /// Nominal 1.8V standard core NMOS (`sky130_fd_pr__nfet_01v8`).
    /// Physical oxide thickness: tox = 4.13 nm.
    pub fn nfet_01v8(width_m: f64, length_m: f64) -> MosfetModel {
        MosfetModel {
            mos_type: MosfetType::Nmos,
            vth0: 0.45,
            w: width_m.max(0.42e-6),  // Min W = 0.42 um
            l: length_m.max(0.15e-6), // Min L = 0.15 um
            tox: 4.13e-9,
            mu0: 0.055,
            vsat: 1.0e5,
            lambda: 0.04,
            gamma: 0.45,
            phi_s: 0.65,
            eta_dibl: 0.08,
            subthreshold_n: 1.3,
            ..MosfetModel::default()
        }
    }

    /// Nominal 1.8V standard core PMOS (`sky130_fd_pr__pfet_01v8`).
    pub fn pfet_01v8(width_m: f64, length_m: f64) -> MosfetModel {
        MosfetModel {
            mos_type: MosfetType::Pmos,
            vth0: -0.55,
            w: width_m.max(0.42e-6),
            l: length_m.max(0.15e-6),
            tox: 4.13e-9,
            mu0: 0.015,
            vsat: 0.8e5,
            lambda: 0.05,
            gamma: 0.50,
            phi_s: 0.65,
            eta_dibl: 0.09,
            subthreshold_n: 1.35,
            ..MosfetModel::default()
        }
    }

    /// High-voltage 5.0V I/O NMOS (`sky130_fd_pr__nfet_g5v0d10v5`).
    /// Thick gate oxide: tox = 12.5 nm.
    pub fn nfet_g5v0(width_m: f64, length_m: f64) -> MosfetModel {
        MosfetModel {
            mos_type: MosfetType::Nmos,
            vth0: 0.70,
            w: width_m.max(0.50e-6),
            l: length_m.max(0.50e-6),
            tox: 12.5e-9,
            mu0: 0.045,
            vsat: 1.0e5,
            lambda: 0.02,
            gamma: 0.60,
            phi_s: 0.70,
            eta_dibl: 0.04,
            subthreshold_n: 1.25,
            ..MosfetModel::default()
        }
    }

    /// Native low-threshold NMOS (`sky130_fd_pr__nfet_01v8_nvt`).
    pub fn nfet_nvt(width_m: f64, length_m: f64) -> MosfetModel {
        MosfetModel {
            mos_type: MosfetType::Nmos,
            vth0: -0.05,
            w: width_m.max(0.42e-6),
            l: length_m.max(0.15e-6),
            tox: 4.13e-9,
            mu0: 0.060,
            vsat: 1.0e5,
            lambda: 0.05,
            gamma: 0.35,
            phi_s: 0.60,
            eta_dibl: 0.10,
            subthreshold_n: 1.20,
            ..MosfetModel::default()
        }
    }

    /// Sheet resistance of N+ diffusion in Ohms per square.
    pub fn nplus_sheet_resistance() -> f64 {
        120.0
    }

    /// Sheet resistance of P+ diffusion in Ohms per square.
    pub fn pplus_sheet_resistance() -> f64 {
        197.0
    }

    /// Gate oxide physical thickness for 1.8V devices in meters (4.13 nm).
    pub fn tox_1v8() -> f64 {
        4.13e-9
    }
}
