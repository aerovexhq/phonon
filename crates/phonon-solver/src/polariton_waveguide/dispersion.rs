#![deny(unsafe_code)]

//! Multi-mode exciton-polariton waveguide dispersion solver.
//!
//! Models coupled planar microcavity exciton-photon polariton waveguides,
//! anti-crossing vacuum Rabi splitting, Hopfield coefficients, group velocity,
//! and polariton effective mass across transverse waveguide modes.

use std::f64::consts::PI;

/// Reduced Planck constant times speed of light: hbar * c in meV * um.
pub const HBAR_C_MEV_UM: f64 = 197.326_980_4;

/// Speed of light in vacuum in um / ps (299792458 m/s = 299.792458 um/ps).
pub const SPEED_OF_LIGHT_UM_PER_PS: f64 = 299.792_458;

/// Electron rest mass energy m_e * c^2 in meV.
pub const ELECTRON_REST_MASS_ENERGY_MEV: f64 = 5.109_989_5e8;

/// Physical configuration parameters for the polariton waveguide.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonWaveguideParams {
    /// Bare exciton resonance energy E_x in meV (default ~1500.0 meV / 1.5 eV).
    pub bare_exciton_energy_mev: f64,
    /// Cavity cutoff energy E_{c0} in meV (default ~1500.0 meV).
    pub cavity_cutoff_energy_mev: f64,
    /// Exciton-photon Rabi coupling g in meV (default ~15.0 meV, Rabi splitting 2g = 30 meV).
    pub rabi_coupling_g_mev: f64,
    /// Waveguide width W in um (default ~4.0 um).
    pub waveguide_width_um: f64,
    /// Refractive index n_{eff} (default ~3.5 for GaAs/dielectric).
    pub refractive_index: f64,
    /// Exciton decay rate gamma_x in meV (default ~0.1 meV).
    pub exciton_decay_rate_gamma_x: f64,
    /// Cavity photon decay rate kappa_c in meV (default ~0.2 meV).
    pub cavity_decay_rate_kappa_c: f64,
    /// Topological Chern number C in {-1, 0, +1} (default +1).
    pub chern_number: i32,
}

impl Default for PolaritonWaveguideParams {
    fn default() -> Self {
        Self {
            bare_exciton_energy_mev: 1500.0,
            cavity_cutoff_energy_mev: 1500.0,
            rabi_coupling_g_mev: 15.0,
            waveguide_width_um: 4.0,
            refractive_index: 3.5,
            exciton_decay_rate_gamma_x: 0.1,
            cavity_decay_rate_kappa_c: 0.2,
            chern_number: 1,
        }
    }
}

impl PolaritonWaveguideParams {
    /// Creates a new parameter builder initialized to standard default physical parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the minimum anti-crossing vacuum Rabi splitting Delta = 2g in meV.
    pub fn vacuum_rabi_splitting_mev(&self) -> f64 {
        2.0 * self.rabi_coupling_g_mev
    }

    /// Evaluates the bare cavity-exciton detuning delta_0 = E_{c0} - E_x in meV at k=0, m=0.
    pub fn cutoff_detuning_mev(&self) -> f64 {
        self.cavity_cutoff_energy_mev - self.bare_exciton_energy_mev
    }
}

/// Evaluated point on the exciton-polariton dispersion curve.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonDispersionPoint {
    /// Longitudinal wavevector k along waveguide propagation axis in um^-1.
    pub k_um: f64,
    /// Transverse mode index m (0, 1, 2, ...).
    pub mode_index_m: usize,
    /// Bare cavity photon energy E_c(k, m) in meV.
    pub energy_cavity_mev: f64,
    /// Bare exciton energy E_x in meV.
    pub energy_exciton_mev: f64,
    /// Lower Polariton (LPB) branch energy in meV.
    pub energy_lp_mev: f64,
    /// Upper Polariton (UPB) branch energy in meV.
    pub energy_up_mev: f64,
    /// Hopfield exciton fraction |X|^2 (with |X|^2 + |C|^2 = 1.0).
    pub exciton_fraction: f64,
    /// Hopfield photon fraction |C|^2 (with |X|^2 + |C|^2 = 1.0).
    pub photon_fraction: f64,
    /// Polariton group velocity v_g = dE_LP / dk in units of um / ps.
    pub group_velocity: f64,
    /// Polariton group velocity in units of c (dimensionless ratio v_g / c).
    pub group_velocity_c: f64,
    /// Polariton effective mass m_pol^* in units of free electron mass m_e.
    pub effective_mass: f64,
    /// Damping linewidth Gamma = |X|^2 * gamma_x + |C|^2 * kappa_c in meV.
    pub damping_linewidth: f64,
    /// Anti-crossing splitting Delta E = E_UP - E_LP in meV.
    pub splitting_mev: f64,
}

/// Multi-mode polariton waveguide dispersion solver.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiModePolaritonDispersionSolver {
    pub params: PolaritonWaveguideParams,
}

impl MultiModePolaritonDispersionSolver {
    /// Creates a new multi-mode dispersion solver instance with specified parameters.
    pub fn new(params: PolaritonWaveguideParams) -> Self {
        Self { params }
    }

    /// Evaluates bare cavity photon energy E_c(k, m) in meV.
    ///
    /// E_c(k, m) = sqrt( E_{c0}^2 + (hbar * c / n_{eff})^2 * (k^2 + (m * pi / W)^2) )
    pub fn bare_cavity_energy(&self, k: f64, m: usize) -> f64 {
        let v_phi = HBAR_C_MEV_UM / self.params.refractive_index;
        let k_y = (m as f64) * PI / self.params.waveguide_width_um;
        let k_sq = k * k + k_y * k_y;
        let term = v_phi * v_phi * k_sq;
        (self.params.cavity_cutoff_energy_mev.powi(2) + term).sqrt()
    }

    /// Solves the 2x2 exciton-photon secular Hamiltonian at wavevector k and mode m.
    ///
    /// H(k, m) = [[E_c(k, m), g], [g, E_x]]
    pub fn solve_point(&self, k: f64, m: usize) -> PolaritonDispersionPoint {
        let e_c = self.bare_cavity_energy(k, m);
        let e_x = self.params.bare_exciton_energy_mev;
        let g = self.params.rabi_coupling_g_mev;

        let detuning = e_c - e_x;
        let rabi_flopping = (detuning * detuning + 4.0 * g * g).sqrt();

        let energy_up = 0.5 * (e_c + e_x + rabi_flopping);
        let energy_lp = 0.5 * (e_c + e_x - rabi_flopping);
        let splitting = energy_up - energy_lp;

        // Hopfield fractions for lower polariton branch
        let photon_frac = 0.5 * (1.0 - detuning / rabi_flopping);
        let exciton_frac = 1.0 - photon_frac;

        // Damping linewidth Gamma = |X|^2 * gamma_x + |C|^2 * kappa_c
        let damping = exciton_frac * self.params.exciton_decay_rate_gamma_x
            + photon_frac * self.params.cavity_decay_rate_kappa_c;

        // Group velocity v_g = (1/hbar) * dE_LP / dk
        // dE_c / dk = (v_phi^2 * k) / E_c
        // dE_LP / dk = |C|^2 * (dE_c / dk)
        let v_phi = HBAR_C_MEV_UM / self.params.refractive_index;
        let de_c_dk = (v_phi * v_phi * k) / e_c;
        let de_lp_dk = photon_frac * de_c_dk;

        // In units of c: (dE / dk) / (hbar * c)
        let group_velocity_c = de_lp_dk / HBAR_C_MEV_UM;
        let group_velocity = group_velocity_c * SPEED_OF_LIGHT_UM_PER_PS;

        // Effective mass m_pol^* = hbar^2 / (d^2 E_LP / dk^2)
        // Evaluated via symmetric finite difference around k
        let dk = 1.0e-4;
        let lp_plus = self.eval_lp_energy(k + dk, m);
        let lp_minus = self.eval_lp_energy(k - dk, m);
        let curvature = (lp_plus - 2.0 * energy_lp + lp_minus) / (dk * dk);

        // Convert curvature (meV * um^2) to electron mass m_e:
        // (hbar * c)^2 / (m_e * c^2) = 38937.935 / 5.1099895e8 ~= 7.619965e-5 meV * um^2
        let mass_factor = (HBAR_C_MEV_UM * HBAR_C_MEV_UM) / ELECTRON_REST_MASS_ENERGY_MEV;
        let effective_mass = if curvature.abs() > 1.0e-9 {
            (mass_factor / curvature).abs()
        } else {
            1.0e-3
        };

        PolaritonDispersionPoint {
            k_um: k,
            mode_index_m: m,
            energy_cavity_mev: e_c,
            energy_exciton_mev: e_x,
            energy_lp_mev: energy_lp,
            energy_up_mev: energy_up,
            exciton_fraction: exciton_frac,
            photon_fraction: photon_frac,
            group_velocity,
            group_velocity_c,
            effective_mass,
            damping_linewidth: damping,
            splitting_mev: splitting,
        }
    }

    /// Internal evaluation of LP energy for numerical derivative calculations.
    fn eval_lp_energy(&self, k: f64, m: usize) -> f64 {
        let e_c = self.bare_cavity_energy(k, m);
        let e_x = self.params.bare_exciton_energy_mev;
        let g = self.params.rabi_coupling_g_mev;
        let detuning = e_c - e_x;
        let rabi_flopping = (detuning * detuning + 4.0 * g * g).sqrt();
        0.5 * (e_c + e_x - rabi_flopping)
    }

    /// Evaluates the minimum anti-crossing energy gap Delta = 2g across the mode.
    pub fn minimum_energy_gap(&self) -> f64 {
        2.0 * self.params.rabi_coupling_g_mev
    }

    /// Sweeps dispersion for a single transverse mode index across [k_min, k_max].
    pub fn sweep_mode(
        &self,
        m: usize,
        k_min: f64,
        k_max: f64,
        num_points: usize,
    ) -> Vec<PolaritonDispersionPoint> {
        let count = num_points.max(2);
        let step = (k_max - k_min) / ((count - 1) as f64);
        (0..count)
            .map(|i| {
                let k = k_min + (i as f64) * step;
                self.solve_point(k, m)
            })
            .collect()
    }

    /// Sweeps dispersion across multiple transverse modes m = 0, 1, ..., num_modes - 1.
    pub fn sweep_multi_mode(
        &self,
        num_modes: usize,
        k_min: f64,
        k_max: f64,
        num_points: usize,
    ) -> Vec<Vec<PolaritonDispersionPoint>> {
        (0..num_modes)
            .map(|m| self.sweep_mode(m, k_min, k_max, num_points))
            .collect()
    }
}
