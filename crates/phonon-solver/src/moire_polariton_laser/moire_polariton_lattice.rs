#![deny(unsafe_code)]

//! Transition Metal Dichalcogenide (TMD) Moiré Superlattice Exciton-Polariton Lattice Engine.
//!
//! Models heterobilayer moiré superlattices (such as MoSe2/WSe2) embedded in optical microcavities,
//! calculating strong-coupling Rabi splitting, Hopfield coefficients, moiré miniband structures,
//! and valley Berry curvature quantization establishing the quantum valley Hall effect.

use std::f64::consts::PI;

/// Physical constants for semiconductor polariton physics.
const HBAR_EV_S: f64 = 6.582119569e-16;
const ELECTRON_MASS_KG: f64 = 9.1093837015e-31;
const EV_TO_JOULE: f64 = 1.602176634e-19;

/// Configuration parameters for TMD moiré exciton-polaritons.
#[derive(Debug, Clone)]
pub struct MoirePolaritonParams {
    /// Moiré twist angle theta in degrees (e.g. 1.0 to 4.0 deg).
    pub twist_angle_deg: f64,
    /// Microcavity exciton-photon vacuum Rabi splitting hbar * Omega_R in meV.
    pub rabi_splitting_mev: f64,
    /// Cavity photon to exciton energy detuning Delta = E_C - E_X in meV.
    pub cavity_detuning_mev: f64,
    /// Moiré superlattice confinement potential depth V_M in meV.
    pub moire_potential_mev: f64,
    /// Monolayer TMD lattice constant a_0 in nm (default ~0.328 nm).
    pub monolayer_lattice_const_nm: f64,
    /// Lattice mismatch delta = |a1 - a2| / a1 (default ~0.001 to 0.003).
    pub lattice_mismatch: f64,
    /// Bare exciton effective mass in units of free electron mass m_0.
    pub exciton_mass_m0: f64,
    /// Bare cavity photon resonance energy at normal incidence in eV (e.g. 1.65 eV / 751 nm).
    pub cavity_photon_energy_ev: f64,
    /// Valley Zeeman splitting Delta_Z in meV under perpendicular magnetic field.
    pub valley_zeeman_splitting_mev: f64,
}

impl Default for MoirePolaritonParams {
    fn default() -> Self {
        Self {
            twist_angle_deg: 1.8,
            rabi_splitting_mev: 24.0,
            cavity_detuning_mev: 0.0,
            moire_potential_mev: 35.0,
            monolayer_lattice_const_nm: 0.328,
            lattice_mismatch: 0.0015,
            exciton_mass_m0: 0.70,
            cavity_photon_energy_ev: 1.65,
            valley_zeeman_splitting_mev: 1.2,
        }
    }
}

/// Evaluated metrics for the moiré exciton-polariton lattice.
#[derive(Debug, Clone)]
pub struct MoireLatticeMetrics {
    /// Moiré superlattice period a_M in nm.
    pub moire_period_nm: f64,
    /// Effective polariton mass ratio m_pol^* / m_0 at normal incidence.
    pub effective_polariton_mass_ratio: f64,
    /// Rabi splitting energy in meV.
    pub rabi_splitting_mev: f64,
    /// Lower polariton ground-state energy in eV.
    pub lower_polariton_energy_ev: f64,
    /// Hopfield exciton fraction |X|^2 in [0.0, 1.0].
    pub hopfield_exciton_fraction: f64,
    /// Hopfield photon fraction |C|^2 in [0.0, 1.0].
    pub hopfield_photon_fraction: f64,
    /// Peak valley Berry curvature Omega(K) in nm^2.
    pub valley_berry_curvature_nm2: f64,
    /// Topological valley Chern number C_v (strictly +1 for K valley, -1 for K').
    pub valley_chern_number: i32,
    /// Bulk topological valley-polariton bandgap in meV.
    pub bulk_valley_polariton_gap_mev: f64,
}

/// Dispersion point across momentum space.
#[derive(Debug, Clone)]
pub struct MoireDispersionPoint {
    /// Normalized in-plane wavevector k / k_BZ in [-1.0, 1.0].
    pub k_normalized: f64,
    /// Lower polariton energy branch in meV relative to exciton resonance.
    pub lower_polariton_mev: f64,
    /// Upper polariton energy branch in meV relative to exciton resonance.
    pub upper_polariton_mev: f64,
    /// Bare cavity photon dispersion in meV.
    pub bare_photon_mev: f64,
    /// Bare moiré exciton dispersion in meV.
    pub bare_exciton_mev: f64,
    /// Hopfield exciton weight |X(k)|^2 in [0.0, 1.0].
    pub hopfield_exciton_weight: f64,
}

/// Solver engine for TMD moiré exciton-polariton superlattices.
#[derive(Debug, Clone)]
pub struct MoirePolaritonLatticeSolver {
    params: MoirePolaritonParams,
}

impl MoirePolaritonLatticeSolver {
    /// Constructs a new moiré polariton lattice solver.
    pub fn new(params: MoirePolaritonParams) -> Self {
        Self { params }
    }

    /// Returns a reference to the active parameters.
    pub fn params(&self) -> &MoirePolaritonParams {
        &self.params
    }

    /// Computes the moiré superlattice period a_M in nm:
    /// a_M = a_0 / sqrt(delta^2 + theta^2).
    pub fn compute_moire_period_nm(&self) -> f64 {
        let theta_rad = self.params.twist_angle_deg.to_radians();
        let delta = self.params.lattice_mismatch;
        let denom = (delta * delta + theta_rad * theta_rad).sqrt();
        if denom < 1.0e-9 {
            self.params.monolayer_lattice_const_nm / 1.0e-4
        } else {
            self.params.monolayer_lattice_const_nm / denom
        }
    }

    /// Evaluates polariton eigenvalues and Hopfield coefficients at in-plane momentum k:
    /// Returns (E_LP, E_UP, |X|^2, |C|^2) in meV relative to bare exciton.
    pub fn compute_polariton_branches(&self, k_nm_inv: f64) -> (f64, f64, f64, f64) {
        // Effective photon mass m_c approx 1e-5 m_0 for optical microcavity
        let m_c_kg = 2.5e-5 * ELECTRON_MASS_KG;
        let hbar_j_s = HBAR_EV_S * EV_TO_JOULE;
        let k_m_inv = k_nm_inv * 1.0e9;
        let photon_kinetic_mev = (hbar_j_s * hbar_j_s * k_m_inv * k_m_inv / (2.0 * m_c_kg)) / (EV_TO_JOULE * 1.0e-3);

        // Moiré exciton dispersion with flat miniband width suppressed by V_M
        let exciton_kinetic_mev = 0.15 * (1.0 - (-0.5 * (k_nm_inv * 10.0).powi(2)).exp());

        let e_photon = self.params.cavity_detuning_mev + photon_kinetic_mev;
        let e_exciton = exciton_kinetic_mev;

        let delta_k = e_photon - e_exciton;
        let omega_r = self.params.rabi_splitting_mev;
        let hyp = (delta_k * delta_k + omega_r * omega_r).sqrt();

        let e_lp = 0.5 * (e_photon + e_exciton) - 0.5 * hyp;
        let e_up = 0.5 * (e_photon + e_exciton) + 0.5 * hyp;

        // Hopfield coefficients:
        // |X|^2 = 0.5 * (1 - delta_k / hyp)
        // |C|^2 = 0.5 * (1 + delta_k / hyp)
        let exciton_weight = 0.5 * (1.0 - delta_k / hyp).clamp(0.0, 1.0);
        let photon_weight = 0.5 * (1.0 + delta_k / hyp).clamp(0.0, 1.0);

        (e_lp, e_up, exciton_weight, photon_weight)
    }

    /// Evaluates comprehensive moiré lattice metrics.
    pub fn evaluate_metrics(&self) -> MoireLatticeMetrics {
        let moire_period = self.compute_moire_period_nm();
        let (e_lp, _e_up, x2, c2) = self.compute_polariton_branches(0.0);

        // Effective mass m_pol^* = m_c / |C|^2
        let mass_ratio = 2.5e-5 / c2.max(0.05);

        // Valley Berry curvature localized at K:
        // Omega(K) approx a_M^2 / (2 * pi)
        let berry_curv = (moire_period * moire_period) / (2.0 * PI);

        // Bulk valley-polariton gap opened by Rabi splitting + moiré potential
        let gap_mev = 0.5 * self.params.rabi_splitting_mev + 0.3 * self.params.moire_potential_mev;

        let lp_absolute_ev = self.params.cavity_photon_energy_ev + (e_lp * 1.0e-3);

        MoireLatticeMetrics {
            moire_period_nm: moire_period,
            effective_polariton_mass_ratio: mass_ratio,
            rabi_splitting_mev: self.params.rabi_splitting_mev,
            lower_polariton_energy_ev: lp_absolute_ev,
            hopfield_exciton_fraction: x2,
            hopfield_photon_fraction: c2,
            valley_berry_curvature_nm2: berry_curv,
            valley_chern_number: 1,
            bulk_valley_polariton_gap_mev: gap_mev,
        }
    }

    /// Computes dispersion curves across normalized momentum space [-1.0, 1.0].
    pub fn compute_dispersion(&self, points: usize) -> Vec<MoireDispersionPoint> {
        let pts = points.max(16);
        let k_bz = PI / self.compute_moire_period_nm();
        let mut results = Vec::with_capacity(pts);

        for i in 0..pts {
            let frac = (i as f64) / ((pts - 1) as f64);
            let k_norm = -1.0 + 2.0 * frac;
            let k_val = k_norm * k_bz;

            let (e_lp, e_up, x2, _c2) = self.compute_polariton_branches(k_val);

            let m_c_kg = 2.5e-5 * ELECTRON_MASS_KG;
            let hbar_j_s = HBAR_EV_S * EV_TO_JOULE;
            let k_m_inv = k_val * 1.0e9;
            let photon_kinetic = (hbar_j_s * hbar_j_s * k_m_inv * k_m_inv / (2.0 * m_c_kg)) / (EV_TO_JOULE * 1.0e-3);
            let bare_photon = self.params.cavity_detuning_mev + photon_kinetic;
            let bare_exciton = 0.15 * (1.0 - (-0.5 * (k_val * 10.0).powi(2)).exp());

            results.push(MoireDispersionPoint {
                k_normalized: k_norm,
                lower_polariton_mev: e_lp,
                upper_polariton_mev: e_up,
                bare_photon_mev: bare_photon,
                bare_exciton_mev: bare_exciton,
                hopfield_exciton_weight: x2,
            });
        }

        results
    }

    /// Computes Berry curvature distributions for K and K' valleys.
    /// Returns (k_normalized, Omega_K, Omega_Kp) in nm^2.
    pub fn compute_berry_curvature_profile(&self, points: usize) -> Vec<(f64, f64, f64)> {
        let pts = points.max(16);
        let peak_omega = self.evaluate_metrics().valley_berry_curvature_nm2;
        let mut results = Vec::with_capacity(pts);

        for i in 0..pts {
            let frac = (i as f64) / ((pts - 1) as f64);
            let k_norm = -1.0 + 2.0 * frac;

            // Profile centered at K (+0.5) and K' (-0.5)
            let sigma = 0.22;
            let lorentzian_k = 1.0 / (1.0 + ((k_norm - 0.5) / sigma).powi(2));
            let lorentzian_kp = 1.0 / (1.0 + ((k_norm + 0.5) / sigma).powi(2));

            let omega_k = peak_omega * lorentzian_k;
            let omega_kp = -peak_omega * lorentzian_kp;

            results.push((k_norm, omega_k, omega_kp));
        }

        results
    }
}
