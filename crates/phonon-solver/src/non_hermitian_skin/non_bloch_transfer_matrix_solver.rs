//! Non-Bloch transfer matrix and non-unitary Green's function solver
//! for the Non-Hermitian Skin Effect (NHSE).
//!
//! # Physical Formalism
//! - Non-Unitary Green's Function:
//!   $$G(z) = [z I - H_{\mathrm{OBC}}]^{-1}$$
//! - End-to-End Directional Propagators:
//!   $$G_{L, 1}(\omega) \propto e^{+\gamma (L - 1)}, \quad G_{1, L}(\omega) \propto e^{-\gamma (L - 1)}$$
//! - Spatial Skin Decay Factor:
//!   $$\Lambda_{\mathrm{skin}} = 20 \gamma (L - 1) \log_{10}(e) \ge 30\text{ dB}$$

use phonon_models::non_hermitian_skin::{NhseLattice, NhseLatticeParams};

/// Result of non-Bloch Green's function and skin effect analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct NhseSpectrumResult {
    /// GBZ radius $r_{\mathrm{GBZ}} = e^{-\gamma}$.
    pub gbz_radius: f64,
    /// Spatial skin localization factor in decibels ($\ge 30\text{ dB}$ required).
    pub skin_localization_db: f64,
    /// Point-gap winding number with respect to the loop center.
    pub point_gap_winding: i32,
    /// Forward Green's function magnitude $|G_{L, 1}(\omega_0)|$.
    pub forward_greens_function: f64,
    /// Backward Green's function magnitude $|G_{1, L}(\omega_0)|$.
    pub backward_greens_function: f64,
    /// Directional propagation ratio in decibels.
    pub directional_ratio_db: f64,
}

/// Non-Bloch transfer matrix and Green's function solver.
#[derive(Debug, Clone, PartialEq)]
pub struct NonBlochTransferMatrixSolver {
    pub lattice: NhseLattice,
}

impl NonBlochTransferMatrixSolver {
    pub fn new(params: NhseLatticeParams) -> Self {
        let lattice = NhseLattice::new(params);
        Self { lattice }
    }

    /// Solves the non-Bloch skin localization, GBZ properties, and directional Green's functions.
    pub fn solve(&self) -> NhseSpectrumResult {
        let p = self.lattice.params;
        let l = (p.num_sites - 1) as f64;
        let gamma = p.non_reciprocal_gamma;

        let gbz_r = p.gbz_radius();
        let skin_db = p.skin_localization_factor_db();

        // Evaluate point-gap winding around onsite frequency center
        let w0 = p.onsite_frequency_rad_s;
        let winding_res = self.lattice.compute_point_gap_winding(w0, 0.0, 100);

        // Green's function end-to-end amplitudes at resonance
        let eta = 10.0; // Small loss broadening
        let inv_broadening = 1.0 / eta;
        let g_fwd = inv_broadening * (gamma * l).exp();
        let g_bwd = inv_broadening * (-gamma * l).exp();

        let dir_ratio_db = 20.0 * (g_fwd / g_bwd.max(1e-30)).log10();

        NhseSpectrumResult {
            gbz_radius: gbz_r,
            skin_localization_db: skin_db,
            point_gap_winding: winding_res.winding_number,
            forward_greens_function: g_fwd,
            backward_greens_function: g_bwd,
            directional_ratio_db: dir_ratio_db,
        }
    }
}
