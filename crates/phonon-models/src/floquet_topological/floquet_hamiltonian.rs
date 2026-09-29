//! 2D Dirac material Hamiltonian under periodic optical driving,
//! Floquet-Bloch quasi-energies, and topological invariants.

use super::floquet_drive::FloquetDriveParams;
use phonon_core::constants::{ELEMENTARY_CHARGE, PLANCK_CONSTANT};

/// Material parameters for a 2D Dirac or topological material.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetDiracMaterial {
    /// Fermi velocity $v_F$ in meters per second (nominal $1.0 \times 10^6\text{ m/s}$ for graphene).
    pub fermi_velocity_m_s: f64,
    /// Intrinsic equilibrium mass gap $\Delta_0$ in electron-volts ($\text{eV}$).
    pub intrinsic_gap_ev: f64,
    /// Effective degeneracy factor (e.g., spin and valley degeneracy $g = 2$ or $4$).
    pub degeneracy: usize,
}

impl Default for FloquetDiracMaterial {
    fn default() -> Self {
        Self {
            fermi_velocity_m_s: 1.0e6,
            intrinsic_gap_ev: 0.0,
            degeneracy: 1,
        }
    }
}

/// Evaluated Floquet band state at a specific momentum $\mathbf{p}$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetBandState {
    /// Upper conduction quasi-energy $\epsilon_+(\mathbf{p})$ in $\text{eV}$.
    pub quasienergy_plus_ev: f64,
    /// Lower valence quasi-energy $\epsilon_-(\mathbf{p})$ in $\text{eV}$.
    pub quasienergy_minus_ev: f64,
    /// Light-induced Floquet mass $M_F$ in $\text{eV}$.
    pub floquet_mass_ev: f64,
    /// Total dynamic bandgap $\Delta_{\mathrm{gap}}$ at Dirac point in $\text{eV}$.
    pub dynamic_gap_ev: f64,
    /// Dynamic topological Chern number $\mathcal{C} \in \{-1, 0, 1\}$.
    pub chern_number: i32,
    /// Dynamic quantized Hall conductance $\sigma_{xy}$ in Siemens ($S$).
    pub dynamic_hall_conductance_si: f64,
}

impl FloquetDiracMaterial {
    /// Creates a 2D Dirac material model with specified Fermi velocity and intrinsic gap.
    pub fn new(fermi_velocity_m_s: f64, intrinsic_gap_ev: f64, degeneracy: usize) -> Self {
        Self {
            fermi_velocity_m_s: fermi_velocity_m_s.max(1e3),
            intrinsic_gap_ev: intrinsic_gap_ev.max(0.0),
            degeneracy: degeneracy.max(1),
        }
    }

    /// Calculates the light-induced Floquet mass term $M_F$ in Joules:
    /// $$M_F = \sigma \frac{(e v_F A_0)^2}{\hbar \Omega} = \sigma \frac{e^2 v_F^2 E_0^2}{\hbar \Omega^3}$$
    #[inline]
    pub fn floquet_mass_joules(&self, drive: &FloquetDriveParams) -> f64 {
        let a0 = drive.vector_potential_amplitude();
        let e_v_a0 = ELEMENTARY_CHARGE * self.fermi_velocity_m_s * a0;
        let hbar_omega = drive.photon_energy_joules();
        drive.chirality * (e_v_a0.powi(2) / hbar_omega)
    }

    /// Calculates the light-induced Floquet mass term $M_F$ in electron-volts ($\text{eV}$).
    #[inline]
    pub fn floquet_mass_ev(&self, drive: &FloquetDriveParams) -> f64 {
        self.floquet_mass_joules(drive) / ELEMENTARY_CHARGE
    }

    /// Effective total mass $M_{\mathrm{eff}} = \frac{\Delta_0}{2} + M_F$ in $\text{eV}$.
    #[inline]
    pub fn effective_mass_ev(&self, drive: &FloquetDriveParams) -> f64 {
        0.5 * self.intrinsic_gap_ev + self.floquet_mass_ev(drive)
    }

    /// Dynamic Floquet bandgap opening at the Dirac point in $\text{eV}$:
    /// $$\Delta_{\mathrm{gap}} = 2 |M_{\mathrm{eff}}|$$
    #[inline]
    pub fn dynamic_gap_ev(&self, drive: &FloquetDriveParams) -> f64 {
        2.0 * self.effective_mass_ev(drive).abs()
    }

    /// Floquet topological Chern number:
    /// $$\mathcal{C} = \operatorname{sgn}(M_{\mathrm{eff}})$$
    /// (returns 0 if light is linearly polarized or unpolarized, $|\sigma| < 10^{-4}$).
    #[inline]
    pub fn chern_number(&self, drive: &FloquetDriveParams) -> i32 {
        if drive.chirality.abs() < 1e-4 && self.intrinsic_gap_ev == 0.0 {
            0
        } else {
            let m_eff = self.effective_mass_ev(drive);
            if m_eff > 1e-9 {
                1
            } else if m_eff < -1e-9 {
                -1
            } else {
                0
            }
        }
    }

    /// Dynamic quantized Hall conductance:
    /// $$\sigma_{xy} = g \cdot \mathcal{C} \frac{e^2}{h}$$
    #[inline]
    pub fn dynamic_hall_conductance_si(&self, drive: &FloquetDriveParams) -> f64 {
        let single_channel_g0 = ELEMENTARY_CHARGE.powi(2) / PLANCK_CONSTANT;
        let c = self.chern_number(drive) as f64;
        c * (self.degeneracy as f64) * single_channel_g0
    }

    /// Evaluates quasi-energies and topological invariants at momentum $\mathbf{p} = (p_x, p_y)$ (in $\text{J}\cdot\text{s/m}$).
    pub fn evaluate_band_state(
        &self,
        drive: &FloquetDriveParams,
        px_j_s_m: f64,
        py_j_s_m: f64,
    ) -> FloquetBandState {
        let m_eff_j = self.effective_mass_ev(drive) * ELEMENTARY_CHARGE;
        let vp_sq = (self.fermi_velocity_m_s * px_j_s_m).powi(2)
            + (self.fermi_velocity_m_s * py_j_s_m).powi(2);
        let energy_j = (vp_sq + m_eff_j.powi(2)).sqrt();
        let energy_ev = energy_j / ELEMENTARY_CHARGE;

        FloquetBandState {
            quasienergy_plus_ev: energy_ev,
            quasienergy_minus_ev: -energy_ev,
            floquet_mass_ev: self.floquet_mass_ev(drive),
            dynamic_gap_ev: self.dynamic_gap_ev(drive),
            chern_number: self.chern_number(drive),
            dynamic_hall_conductance_si: self.dynamic_hall_conductance_si(drive),
        }
    }

    /// Instantaneous $2 \times 2$ matrix elements of the time-dependent Dirac Hamiltonian:
    /// $$H(t) = v_F \sigma_x (p_x + e A_x(t)) + v_F \sigma_y (p_y + e A_y(t)) + \frac{\Delta_0}{2} \sigma_z$$
    /// Returns elements $[[H_{00}, H_{01}], [H_{10}, H_{11}]]$ as complex `(re, im)` pairs in Joules.
    pub fn instantaneous_hamiltonian(
        &self,
        drive: &FloquetDriveParams,
        px_j_s_m: f64,
        py_j_s_m: f64,
        t_s: f64,
    ) -> [[(f64, f64); 2]; 2] {
        let (ax, ay) = drive.vector_potential_at(t_s);
        let kx = px_j_s_m + ELEMENTARY_CHARGE * ax;
        let ky = py_j_s_m + ELEMENTARY_CHARGE * ay;

        let delta_half_j = 0.5 * self.intrinsic_gap_ev * ELEMENTARY_CHARGE;
        let h00 = (delta_half_j, 0.0);
        let h11 = (-delta_half_j, 0.0);

        let h01_re = self.fermi_velocity_m_s * kx;
        let h01_im = -self.fermi_velocity_m_s * ky;
        let h01 = (h01_re, h01_im);
        let h10 = (h01_re, -h01_im);

        [[h00, h01], [h10, h11]]
    }
}
