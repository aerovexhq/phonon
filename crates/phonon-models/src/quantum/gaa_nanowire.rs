//! Gate-All-Around (GAA) nanowire and nanosheet physical device models
//! incorporating 2D quantum confinement subband splitting and multi-subband ballistic transport.

use phonon_core::{ELEMENTARY_CHARGE, EPSILON_0, H_BAR, PLANCK_CONSTANT};

/// Geometric cross-section type for Gate-All-Around field-effect transistors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GaaCrossSection {
    /// Rectangular nanosheet with width $W$ and thickness $T_{nano}$.
    Nanosheet { width_m: f64, thickness_m: f64 },
    /// Cylindrical nanowire with diameter $D$.
    Cylindrical { diameter_m: f64 },
}

/// Physical parameters for a sub-3nm Gate-All-Around (GAA) transistor.
#[derive(Debug, Clone, PartialEq)]
pub struct GaaNanowireModel {
    pub geometry: GaaCrossSection,
    /// Channel length $L$ in meters ($m$).
    pub length_m: f64,
    /// Equivalent oxide thickness $t_{ox}$ in meters ($m$).
    pub tox: f64,
    /// Relative dielectric permittivity $\kappa$ of the gate insulator.
    pub kappa_ox: f64,
    /// Effective transport mass $m_x^*$ (longitudinal along channel).
    pub m_transport: f64,
    /// Confinement masses ($m_y^*, m_z^*$) in transverse directions.
    pub m_confinement_y: f64,
    pub m_confinement_z: f64,
    /// Maximum number of quantum subbands considered in the transport integral.
    pub max_subbands: usize,
    /// Zero-bias threshold voltage $V_{th0}$ in Volts ($V$).
    pub vth0: f64,
}

impl Default for GaaNanowireModel {
    fn default() -> Self {
        // Typical 2nm node nanosheet: W = 15 nm, T = 5 nm, L = 12 nm, tox = 1.0 nm (HfO2 kappa=20)
        Self {
            geometry: GaaCrossSection::Nanosheet {
                width_m: 15.0e-9,
                thickness_m: 5.0e-9,
            },
            length_m: 12.0e-9,
            tox: 1.0e-9,
            kappa_ox: 20.0,
            m_transport: 0.19, // Silicon Delta valley transport mass
            m_confinement_y: 0.19,
            m_confinement_z: 0.91,
            max_subbands: 4,
            vth0: 0.35,
        }
    }
}

impl GaaNanowireModel {
    /// Computes the gate oxide geometric capacitance $C_{ox}$ in Farads ($F$).
    pub fn gate_capacitance(&self) -> f64 {
        let eps_ox = self.kappa_ox * EPSILON_0;
        match self.geometry {
            GaaCrossSection::Nanosheet {
                width_m,
                thickness_m,
            } => {
                // Perimeter = 2 * (W + T)
                let perimeter = 2.0 * (width_m + thickness_m);
                (eps_ox * perimeter * self.length_m) / self.tox
            }
            GaaCrossSection::Cylindrical { diameter_m } => {
                let r_core = diameter_m / 2.0;
                let r_gate = r_core + self.tox;
                (2.0 * std::f64::consts::PI * eps_ox * self.length_m) / (r_gate / r_core).ln()
            }
        }
    }

    /// Computes the quantized subband energy levels $E_{n, m}$ (in $eV$) above the conduction band edge
    /// due to 2D transverse quantum confinement in the nanosheet:
    /// $$E_{n, m} = \frac{\hbar^2 \pi^2}{2 m_0} \left( \frac{n^2}{m_y^* W^2} + \frac{m^2}{m_z^* T^2} \right)$$
    pub fn subband_energies_ev(&self) -> Vec<f64> {
        let m0 = 9.109_383_701_5e-31;
        let mut energies = Vec::new();

        match self.geometry {
            GaaCrossSection::Nanosheet {
                width_m,
                thickness_m,
            } => {
                for n in 1..=self.max_subbands {
                    for m in 1..=self.max_subbands {
                        let ey = (H_BAR
                            * H_BAR
                            * std::f64::consts::PI
                            * std::f64::consts::PI
                            * (n * n) as f64)
                            / (2.0 * self.m_confinement_y * m0 * width_m * width_m);
                        let ez = (H_BAR
                            * H_BAR
                            * std::f64::consts::PI
                            * std::f64::consts::PI
                            * (m * m) as f64)
                            / (2.0 * self.m_confinement_z * m0 * thickness_m * thickness_m);

                        let e_tot_joules = ey + ez;
                        energies.push(e_tot_joules / ELEMENTARY_CHARGE);
                    }
                }
            }
            GaaCrossSection::Cylindrical { diameter_m } => {
                // Cylindrical Bessel zeros: j_01 = 2.4048, j_11 = 3.8317, etc.
                let bessel_zeros = [2.4048, 3.8317, 5.1356, 5.5201];
                let r = diameter_m / 2.0;
                let m_trans = (self.m_confinement_y * self.m_confinement_z).sqrt();
                for &zero in &bessel_zeros {
                    let e_joules = (H_BAR * H_BAR * zero * zero) / (2.0 * m_trans * m0 * r * r);
                    energies.push(e_joules / ELEMENTARY_CHARGE);
                }
            }
        }

        energies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        energies.truncate(self.max_subbands);
        energies
    }

    /// Evaluates multi-subband ballistic current $I_{DS}$ (in Amperes) at given bias $(V_{GS}, V_{DS})$
    /// and temperature $T$ (in Kelvin).
    pub fn evaluate_current(&self, v_gs: f64, v_ds: f64, temp_k: f64) -> f64 {
        let kb_ev = 8.617_333_262e-5;
        let kt = (kb_ev * temp_k).max(1e-5);

        // Electrostatic gate overdrive modulates the channel top-of-barrier:
        let eta_gate = 0.95; // High gate control efficiency in GAA
        let v_ov = v_gs - self.vth0;
        let barrier_shift_ev = eta_gate * v_ov;

        let subbands = self.subband_energies_ev();
        let mu_s = 0.0;
        let mu_d = -v_ds;

        let mut total_current = 0.0;
        let q = ELEMENTARY_CHARGE;
        let prefactor = 2.0 * q * kb_ev * temp_k * q / PLANCK_CONSTANT;

        for e_sub in subbands {
            let e_bot = e_sub - barrier_shift_ev;

            // Log(1 + exp((mu - E_bot)/(k_B*T))) 1D Landauer ballistic supply function
            let arg_s = ((mu_s - e_bot) / kt).clamp(-40.0, 40.0);
            let arg_d = ((mu_d - e_bot) / kt).clamp(-40.0, 40.0);

            let supply_s = (1.0 + arg_s.exp()).ln();
            let supply_d = (1.0 + arg_d.exp()).ln();

            let sub_current = prefactor * (supply_s - supply_d);
            total_current += sub_current;
        }

        total_current
    }
}
