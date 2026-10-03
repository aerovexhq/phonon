#![deny(unsafe_code)]

//! Discrete Non-Linear Josephson Transmission Line physical model.
//!
//! Models a spatially distributed superconducting LC ladder network loaded with
//! non-linear Josephson junction inductors, ground capacitance, junction intrinsic capacitance,
//! and periodic Resonant Phase Matching (RPM) resonant shunt stubs.

use std::f64::consts::PI;

/// Physical parameters for a single discrete Josephson unit cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JosephsonCellParams {
    /// Linear small-signal Josephson inductance L_J0 in Henries (default ~80 pH / 80e-12 H).
    pub l_j0: f64,
    /// Critical current I_c in Amperes (default ~5.0 uA / 5e-6 A).
    pub i_c: f64,
    /// Ground shunt capacitance C_g in Farads (default ~50 fF / 50e-15 F).
    pub c_g: f64,
    /// Junction intrinsic geometric capacitance C_J in Farads (default ~10 fF / 10e-15 F).
    pub c_j: f64,
    /// Physical unit cell length a in meters (default ~10 um / 10e-6 m).
    pub a: f64,
}

impl Default for JosephsonCellParams {
    fn default() -> Self {
        Self {
            l_j0: 80.0e-12, // 80 pH
            i_c: 5.0e-6,   // 5 uA
            c_g: 50.0e-15,  // 50 fF
            c_j: 10.0e-15,  // 10 fF
            a: 10.0e-6,     // 10 um
        }
    }
}

impl JosephsonCellParams {
    /// Constructs custom Josephson cell parameters.
    pub fn new(l_j0: f64, i_c: f64, c_g: f64, c_j: f64, a: f64) -> Self {
        Self {
            l_j0: l_j0.max(1.0e-15),
            i_c: i_c.max(1.0e-9),
            c_g: c_g.max(1.0e-18),
            c_j: c_j.max(1.0e-18),
            a: a.max(1.0e-9),
        }
    }
}

/// Parameters for periodic Resonant Phase Matching (RPM) stubs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RpmStubParams {
    /// Stub insertion period in number of unit cells M_cells (e.g. 16 or 32 cells).
    pub m_cells: usize,
    /// Resonant stub inductance L_rpm in Henries.
    pub l_rpm: f64,
    /// Resonant stub capacitance C_rpm in Farads.
    pub c_rpm: f64,
}

impl Default for RpmStubParams {
    fn default() -> Self {
        // Tuned for stopband near 12 GHz (~2 * f_p with f_p = 6 GHz)
        let f_rpm = 12.0e9;
        let l_rpm = 80.0e-12; // 80 pH
        let omega = 2.0 * PI * f_rpm;
        let c_rpm = 1.0 / (omega * omega * l_rpm); // ~2.2 fF
        Self {
            m_cells: 16,
            l_rpm,
            c_rpm,
        }
    }
}

impl RpmStubParams {
    /// Creates custom RPM stub parameters.
    pub fn new(m_cells: usize, l_rpm: f64, c_rpm: f64) -> Self {
        Self {
            m_cells: m_cells.max(1),
            l_rpm: l_rpm.max(1.0e-15),
            c_rpm: c_rpm.max(1.0e-18),
        }
    }

    /// Constructs RPM stub parameters targeting a specific stopband frequency f_rpm in Hz.
    pub fn from_stopband_freq(m_cells: usize, f_rpm_hz: f64, l_rpm: f64) -> Self {
        let omega = 2.0 * PI * f_rpm_hz.max(1.0e6);
        let l_eff = l_rpm.max(1.0e-15);
        let c_rpm = 1.0 / (omega * omega * l_eff);
        Self {
            m_cells: m_cells.max(1),
            l_rpm: l_eff,
            c_rpm,
        }
    }

    /// Resonant frequency of the RPM stub f_rpm in Hz.
    pub fn resonant_frequency_hz(&self) -> f64 {
        1.0 / (2.0 * PI * (self.l_rpm * self.c_rpm).sqrt().max(1.0e-18))
    }
}

/// Spatially distributed discrete Josephson Transmission Line (JTL).
#[derive(Debug, Clone, PartialEq)]
pub struct JosephsonTransmissionLine {
    /// Total number of cells N (default ~1000 cells, range 200 - 4000).
    pub num_cells: usize,
    /// Unit cell electrical parameters.
    pub cell_params: JosephsonCellParams,
    /// Optional Resonant Phase Matching (RPM) configuration.
    pub rpm_params: Option<RpmStubParams>,
}

impl Default for JosephsonTransmissionLine {
    fn default() -> Self {
        Self {
            num_cells: 1000,
            cell_params: JosephsonCellParams::default(),
            rpm_params: Some(RpmStubParams::default()),
        }
    }
}

impl JosephsonTransmissionLine {
    /// Creates a new Josephson transmission line configuration.
    pub fn new(
        num_cells: usize,
        cell_params: JosephsonCellParams,
        rpm_params: Option<RpmStubParams>,
    ) -> Self {
        Self {
            num_cells: num_cells.clamp(200, 4000),
            cell_params,
            rpm_params,
        }
    }

    /// Total physical length of the transmission line L in meters: L = N * a.
    pub fn total_length_m(&self) -> f64 {
        self.num_cells as f64 * self.cell_params.a
    }

    /// Non-linear Josephson inductance under bias current I:
    /// L_J(I) = L_J0 / sqrt(1 - (I / I_c)^2)
    /// Safely clamped if |I| >= 0.95 * I_c to avoid non-physical singularities.
    pub fn non_linear_inductance(&self, current: f64) -> f64 {
        let ratio = (current.abs() / self.cell_params.i_c).min(0.95);
        let denom = (1.0 - ratio * ratio).sqrt();
        self.cell_params.l_j0 / denom
    }

    /// Characteristic small-signal impedance Z_0 = sqrt(L_J0 / C_g) in Ohms.
    pub fn characteristic_impedance(&self) -> f64 {
        (self.cell_params.l_j0 / self.cell_params.c_g).sqrt()
    }

    /// Discrete ladder cutoff frequency f_cutoff = 1 / (pi * sqrt(L_J0 * C_g)) in Hz.
    pub fn cutoff_frequency(&self) -> f64 {
        1.0 / (PI * (self.cell_params.l_j0 * self.cell_params.c_g).sqrt())
    }

    /// Discrete ladder cutoff angular frequency omega_c = 2 / sqrt(L_J0 * C_g) in rad/s.
    pub fn cutoff_angular_frequency(&self) -> f64 {
        2.0 / (self.cell_params.l_j0 * self.cell_params.c_g).sqrt()
    }

    /// Calculates discrete propagation constant k(omega) in rad/m from:
    /// cos(k * a) = 1 - omega^2 * L_J0 * C_g / (2 * (1 - omega^2 * L_J0 * C_J))
    /// with RPM stopband dispersion modification near f_rpm.
    pub fn dispersion_k(&self, freq_hz: f64, with_rpm: bool) -> f64 {
        let omega = 2.0 * PI * freq_hz.max(1.0);
        let l_j0 = self.cell_params.l_j0;
        let c_g = self.cell_params.c_g;
        let c_j = self.cell_params.c_j;
        let a = self.cell_params.a;

        let num = omega * omega * l_j0 * c_g;
        let den = 2.0 * (1.0 - omega * omega * l_j0 * c_j).max(1.0e-24);
        let arg = (1.0 - num / den).clamp(-1.0, 1.0);
        let k_bare = arg.acos() / a;

        if !with_rpm {
            return k_bare;
        }

        if let Some(rpm) = &self.rpm_params {
            let f_rpm = rpm.resonant_frequency_hz();
            let ratio = freq_hz / f_rpm.max(1.0);
            let det = 1.0 - ratio * ratio;
            // Resonant stopband dispersion modification near f_rpm
            let coupling = (rpm.c_rpm / (2.0 * c_g * rpm.m_cells as f64).max(1.0e-24)).min(0.002);
            let stopband = k_bare
                * coupling
                * (ratio * ratio * det / (det * det + 0.05));
            k_bare + stopband
        } else {
            k_bare
        }
    }

    /// Non-linear Kerr coefficient gamma_NL = L_J0 * omega / (4 * I_c^2 * Z_0 * a) in 1 / (W * m).
    pub fn kerr_coefficient(&self, freq_hz: f64) -> f64 {
        let omega = 2.0 * PI * freq_hz.max(1.0);
        let z0 = self.characteristic_impedance();
        let i_c_sq = self.cell_params.i_c * self.cell_params.i_c;
        let denom = 4.0 * i_c_sq * z0 * self.cell_params.a;
        (self.cell_params.l_j0 * omega) / denom.max(1.0e-30)
    }
}
