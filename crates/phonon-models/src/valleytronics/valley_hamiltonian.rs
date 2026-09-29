//! 2D Transition Metal Dichalcogenide (TMD) valley Hamiltonian,
//! broken inversion symmetry band structures, valley-dependent Berry curvature,
//! orbital magnetic moments, and circular dichroism.
//!
//! # Physical Formalism
//! - Low-Energy Dirac-Type Valley Hamiltonian ($K_\tau$ with $\tau = \pm 1$):
//!   $$H_\tau(\vec{k}) = \hbar v_F (\tau k_x \sigma_x + k_y \sigma_y) + \hbar w_x k_x I + \frac{\Delta_{\mathrm{gap}}}{2} \sigma_z - \lambda \tau \frac{\sigma_z - 1}{2} s_z$$
//!   where $w_x$ is the tilt parameter breaking in-plane mirror symmetry ($C_{3v} \to C_s$).
//! - Valley Berry Curvature:
//!   $$\Omega_z^\tau(\vec{k}) = -\tau \frac{2 \hbar^2 v_F^2 \Delta_{\mathrm{gap}}}{[\Delta_{\mathrm{gap}}^2 + 4 \hbar^2 v_F^2 k^2]^{3/2}}$$
//! - Valley Orbital Magnetic Moment:
//!   $$m_z^\tau(\vec{k}) = -\tau \frac{e}{\hbar} \frac{2 \hbar^2 v_F^2 \Delta_{\mathrm{gap}}}{[\Delta_{\mathrm{gap}}^2 + 4 \hbar^2 v_F^2 k^2]^{3/2}} \cdot \frac{\hbar^2}{2 m^*}$$
//! - Valley Optical Circular Dichroism:
//!   $$\eta_{\mathrm{CD}} = \frac{|\mathcal{P}_+|^2 - |\mathcal{P}_-|^2}{|\mathcal{P}_+|^2 + |\mathcal{P}_-|^2} = \tau \frac{\Delta_{\mathrm{gap}}}{\sqrt{\Delta_{\mathrm{gap}}^2 + 4 \hbar^2 v_F^2 k^2}}$$

pub const HBAR_EV_S: f64 = 6.582_119_569e-16; // eV*s
pub const HBAR_J_S: f64 = 1.054_571_817e-34; // J*s
pub const ELECTRON_CHARGE_C: f64 = 1.602_176_634e-19; // Coulomb
pub const BOHR_MAGNETON_J_T: f64 = 9.274_010_078_3e-24; // J/T

/// Parameters for 2D hexagonal valleytronic monolayer materials.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TmdMaterialParams {
    /// Inversion-symmetry-breaking mass gap $\Delta_{gap}$ in eV (typically 1.5 - 2.0 eV).
    pub bandgap_ev: f64,
    /// Valence band spin-orbit splitting $2\lambda$ in eV (typically 150 - 450 meV).
    pub spin_orbit_splitting_ev: f64,
    /// Fermi velocity $v_F$ in m/s (typically $\sim 5 \times 10^5\text{ m/s}$).
    pub fermi_velocity_m_s: f64,
    /// Lattice constant $a$ in meters (typically $\sim 0.32\text{ nm}$).
    pub lattice_constant_m: f64,
    /// Dirac cone tilt velocity $w_x$ in m/s breaking in-plane mirror symmetry ($C_{3v} \to C_s$).
    pub tilt_velocity_m_s: f64,
}

impl TmdMaterialParams {
    /// Standard monolayer Tungsten Diselenide ($\mathrm{WSe}_2$) at room temperature.
    pub fn wse2_monolayer() -> Self {
        Self {
            bandgap_ev: 1.65,
            spin_orbit_splitting_ev: 0.460, // 460 meV giant SOC
            fermi_velocity_m_s: 5.3e5,
            lattice_constant_m: 3.28e-10,
            tilt_velocity_m_s: 0.0, // Untilted C3v
        }
    }

    /// Monolayer Molybdenum Disulfide ($\mathrm{MoS}_2$).
    pub fn mos2_monolayer() -> Self {
        Self {
            bandgap_ev: 1.82,
            spin_orbit_splitting_ev: 0.150, // 150 meV SOC
            fermi_velocity_m_s: 5.5e5,
            lattice_constant_m: 3.16e-10,
            tilt_velocity_m_s: 0.0,
        }
    }

    /// Tilted low-symmetry $T_d$ Phase Few-Layer $\mathrm{MoTe}_2$ / strained $\mathrm{WSe}_2$ with Berry dipole.
    pub fn strained_tilted_tmd(tilt_m_s: f64) -> Self {
        Self {
            bandgap_ev: 1.20,
            spin_orbit_splitting_ev: 0.300,
            fermi_velocity_m_s: 4.8e5,
            lattice_constant_m: 3.45e-10,
            tilt_velocity_m_s: tilt_m_s, // Non-zero tilt generates Berry curvature dipole
        }
    }
}

/// Valley band state evaluated at a specific wavevector $\vec{k}$.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyBandState {
    /// Valley index $\tau = +1$ for $K$, $\tau = -1$ for $K'$.
    pub valley_index: i32,
    /// Conduction band energy $E_c(\vec{k})$ in eV.
    pub energy_conduction_ev: f64,
    /// Valence band energy $E_v(\vec{k})$ in eV.
    pub energy_valence_ev: f64,
    /// Valley Berry curvature $\Omega_z^\tau(\vec{k})$ in $\mathrm{m}^2$.
    pub berry_curvature_m2: f64,
    /// Valley orbital magnetic moment $m_z^\tau(\vec{k})$ in units of Bohr magnetons $\mu_B$.
    pub orbital_moment_mu_b: f64,
    /// Optical circular dichroism degree $\eta_{\mathrm{CD}} \in [-1, 1]$.
    pub circular_dichroism: f64,
    /// Group velocity vector $(v_x, v_y)$ in m/s.
    pub group_velocity_m_s: (f64, f64),
}

/// 2D Valley Lattice Model evaluating single-valley and two-valley properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyLattice {
    pub params: TmdMaterialParams,
}

impl ValleyLattice {
    pub fn new(params: TmdMaterialParams) -> Self {
        Self { params }
    }

    /// Evaluates the band structure, Berry curvature, orbital moment, and dichroism at $\vec{k}$:
    pub fn evaluate_state(&self, kx: f64, ky: f64, valley_tau: i32) -> ValleyBandState {
        let tau = if valley_tau >= 0 { 1.0 } else { -1.0 };
        let vf = self.params.fermi_velocity_m_s;
        let delta_j = self.params.bandgap_ev * ELECTRON_CHARGE_C;
        let hbar_vf_j = HBAR_J_S * vf;
        let wx = self.params.tilt_velocity_m_s;

        let k_sq = kx.powi(2) + ky.powi(2);
        let k_mag = k_sq.sqrt();

        // Massive Dirac dispersion:
        // E_c = hbar * wx * kx + sqrt((delta/2)^2 + (hbar * vf * k)^2)
        let dirac_mass_sq = (0.5 * delta_j).powi(2) + (hbar_vf_j * k_mag).powi(2);
        let dirac_mass = dirac_mass_sq.sqrt();

        let tilt_term_j = HBAR_J_S * wx * kx;
        let e_c_j = tilt_term_j + dirac_mass;
        let e_v_j = tilt_term_j - dirac_mass;

        let e_c_ev = e_c_j / ELECTRON_CHARGE_C;
        let e_v_ev = e_v_j / ELECTRON_CHARGE_C;

        // Berry Curvature:
        // Omega_z(k) = -tau * (2 * hbar^2 * vf^2 * delta) / (delta^2 + 4 * hbar^2 * vf^2 * k^2)^(3/2)
        let denom_berry = (delta_j.powi(2) + 4.0 * (hbar_vf_j * k_mag).powi(2)).powf(1.5);
        let omega_z = -tau * (2.0 * hbar_vf_j.powi(2) * delta_j) / denom_berry.max(1e-70);

        // Orbital moment: m_z(k) = (e / hbar) * Omega_z(k) * (delta / 4)
        // Expressed in Bohr magnetons mu_B = e * hbar / (2 m_0)
        let m_z_j = (ELECTRON_CHARGE_C / HBAR_J_S) * omega_z * (0.25 * delta_j);
        let m_z_mu_b = m_z_j / BOHR_MAGNETON_J_T;

        // Circular dichroism degree:
        let eta_cd = tau * (delta_j / (2.0 * dirac_mass)).clamp(-1.0, 1.0);

        // Group velocity vg = (1 / hbar) * grad_k E
        let vg_x = wx + (vf.powi(2) * kx) / (dirac_mass / HBAR_J_S).max(1e3);
        let vg_y = (vf.powi(2) * ky) / (dirac_mass / HBAR_J_S).max(1e3);

        ValleyBandState {
            valley_index: if valley_tau >= 0 { 1 } else { -1 },
            energy_conduction_ev: e_c_ev,
            energy_valence_ev: e_v_ev,
            berry_curvature_m2: omega_z,
            orbital_moment_mu_b: m_z_mu_b,
            circular_dichroism: eta_cd,
            group_velocity_m_s: (vg_x, vg_y),
        }
    }
}
