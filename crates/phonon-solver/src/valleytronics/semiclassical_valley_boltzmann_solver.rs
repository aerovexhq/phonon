//! Semiclassical Peierls-Boltzmann valley transport solver,
//! non-linear anomalous Hall currents, and pure valley currents.
//!
//! # Physical Formalism
//! - Wavepacket Semiclassical Velocity:
//!   $$\dot{\vec{r}} = \frac{1}{\hbar}\nabla_{\vec{k}} \mathcal{E}_n(\vec{k}) - \dot{\vec{k}} \times \vec{\Omega}_n(\vec{k})$$
//! - Second-Harmonic Non-Linear Hall Current:
//!   $$j_y^{(2\omega)} = \frac{e^3 \tau}{2 \hbar^2} D_{xz} E_x^2$$
//! - Pure Valley Current:
//!   $$j_v = j_y^K - j_y^{K'} = 2 j_y^K$$
//!   while total charge current $j_c = j_y^K + j_y^{K'} \to 0$ in the absence of Berry dipole.

use phonon_models::valleytronics::{BerryCurvatureDipole, TmdMaterialParams, ValleyLattice};

/// Universal physical constants
const ELECTRON_CHARGE_C: f64 = 1.602_176_634e-19; // Coulomb
const HBAR_J_S: f64 = 1.054_571_817e-34; // J*s

/// Result of semiclassical valley Boltzmann transport solver.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyTransportResult {
    /// Applied AC electric field amplitude in V/m.
    pub electric_field_v_m: f64,
    /// Chemical potential $\mu$ in eV.
    pub chemical_potential_ev: f64,
    /// Longitudinal current density $j_x^{(\omega)}$ in $\mathrm{A/m}$.
    pub longitudinal_current_a_m: f64,
    /// Transverse non-linear Hall current density $j_y^{(2\omega)}$ in $\mathrm{A/m}$.
    pub nonlinear_hall_current_a_m: f64,
    /// Pure valley current density $j_v = j_K - j_{K'}$ in $\mathrm{A/m}$.
    pub pure_valley_current_a_m: f64,
    /// Berry curvature dipole $D_{xz}$ in meters.
    pub berry_dipole_m: f64,
    /// Non-linear Hall rectification ratio in decibels.
    pub rectification_ratio_db: f64,
    /// Optical circular dichroism at band edge $\eta_{\mathrm{CD}} \in [-1, 1]$.
    pub circular_dichroism_edge: f64,
}

/// Semiclassical Peierls-Boltzmann transport solver for 2D valleytronics.
#[derive(Debug, Clone, PartialEq)]
pub struct SemiclassicalValleyBoltzmannSolver {
    pub dipole: BerryCurvatureDipole,
}

impl SemiclassicalValleyBoltzmannSolver {
    pub fn new(params: TmdMaterialParams) -> Self {
        let lattice = ValleyLattice::new(params);
        let dipole = BerryCurvatureDipole::new(lattice);
        Self { dipole }
    }

    /// Solves the linear and non-linear transport at given electric field and chemical potential.
    pub fn solve(
        &mut self,
        electric_field_v_m: f64,
        chemical_potential_ev: f64,
        grid_size: usize,
    ) -> ValleyTransportResult {
        let d_xz = self.dipole.compute_dipole(chemical_potential_ev, grid_size);
        let chi = self.dipole.nonlinear_hall_susceptibility();

        let sigma_xx = 1.8e-3; // S longitudinal sheet conductance
        let j_x = sigma_xx * electric_field_v_m;
        let j_y_2w = chi.abs() * electric_field_v_m.powi(2);

        // Valley Hall current from anomalous velocity
        let k_edge_state = self.dipole.lattice.evaluate_state(1e7, 1e7, 1);
        let cd = k_edge_state.circular_dichroism;
        let omega_z = k_edge_state.berry_curvature_m2.abs();

        let anomalous_v = (ELECTRON_CHARGE_C / HBAR_J_S) * electric_field_v_m * omega_z;
        let j_v = 2.0 * ELECTRON_CHARGE_C * 1e15 * anomalous_v; // n ~ 10^11 cm^-2

        let r_db = self.dipole.rectification_ratio_db(electric_field_v_m);

        ValleyTransportResult {
            electric_field_v_m,
            chemical_potential_ev,
            longitudinal_current_a_m: j_x,
            nonlinear_hall_current_a_m: j_y_2w,
            pure_valley_current_a_m: j_v,
            berry_dipole_m: d_xz,
            rectification_ratio_db: r_db,
            circular_dichroism_edge: cd,
        }
    }
}
