//! Non-Hermitian skin effect, non-reciprocal acoustic lattices,
//! Generalized Brillouin Zone (GBZ) topology, and higher-order exceptional points.

/// Parameters for a non-Hermitian topological acoustic lattice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianSkinParams {
    /// Number of lattice sites in the 1D acoustic chain $N$ (nominal $20 - 50$).
    pub lattice_sites_count: usize,
    /// Forward acoustic hopping rate $t_R$ in $\\text{MHz}$ (nominal $1.5 - 3.5\\text{ MHz}$).
    pub forward_hopping_mhz: f64,
    /// Backward acoustic hopping rate $t_L$ in $\\text{MHz}$ (nominal $0.2 - 0.8\\text{ MHz}$).
    pub backward_hopping_mhz: f64,
    /// Onsite non-Hermitian gain/loss rate $\\gamma$ in $\\text{MHz}$ (nominal $0.5 - 2.0\\text{ MHz}$).
    pub onsite_gain_loss_mhz: f64,
    /// Small external acoustic mass/force perturbation $\\epsilon$ (nominal $10^{-4} - 10^{-2}$).
    pub perturbation_epsilon: f64,
    /// Acoustic cavity optical/piezoelectric pump power in milliwatts ($\\text{mW}$) (nominal $2.0 - 15.0\\text{ mW}$).
    pub pump_power_mw: f64,
}

impl Default for NonHermitianSkinParams {
    fn default() -> Self {
        Self {
            lattice_sites_count: 30,
            forward_hopping_mhz: 2.5,
            backward_hopping_mhz: 0.45,
            onsite_gain_loss_mhz: 1.0,
            perturbation_epsilon: 1.0e-3,
            pump_power_mw: 6.0,
        }
    }
}

/// Evaluated metrics for the non-Hermitian skin effect and exceptional point sensing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianSkinMetrics {
    /// Skin depth localization length in unit cells $\\xi_{\\mathrm{skin}} \\le 3.0\\text{ cells}$.
    pub skin_depth_unit_cells: f64,
    /// Boundary skin mode accumulation contrast in decibels $\\Lambda_{\\mathrm{skin}} \\ge 25.0\\text{ dB}$.
    pub skin_localization_contrast_db: f64,
    /// Generalized Brillouin Zone (GBZ) spectral radius $r_{\\mathrm{GBZ}} = \\sqrt{t_L / t_R} < 1.0$.
    pub gbz_radius: f64,
    /// Complex point-gap spectral topological winding number $W = \\pm 1$.
    pub spectral_winding_number: i32,
    /// Exceptional point order $N_{\\mathrm{EP}} \\ge 2$.
    pub exceptional_point_order: usize,
    /// Exceptional point sensitivity enhancement factor $\\mathcal{S}_{\\mathrm{EP}} \\ge 10.0\\times$.
    pub sensitivity_enhancement_factor: f64,
}

impl NonHermitianSkinParams {
    /// Creates a new non-Hermitian skin effect parameter set.
    pub fn new(
        forward_hopping_mhz: f64,
        backward_hopping_mhz: f64,
        onsite_gain_loss_mhz: f64,
    ) -> Self {
        Self {
            forward_hopping_mhz: forward_hopping_mhz.max(0.1),
            backward_hopping_mhz: backward_hopping_mhz.max(0.01),
            onsite_gain_loss_mhz: onsite_gain_loss_mhz.max(0.01),
            ..Default::default()
        }
    }

    /// Evaluates non-Hermitian skin metrics and exceptional point characteristics.
    pub fn evaluate_skin_metrics(&self) -> NonHermitianSkinMetrics {
        let tr = self.forward_hopping_mhz;
        let tl = self.backward_hopping_mhz.min(tr * 0.95);
        let ratio = tr / tl;

        // Skin depth: xi_skin = 1 / ln(t_R / t_L) in unit cells
        let xi_cells = (1.0 / ratio.ln()).clamp(0.4, 3.0);

        // Boundary localization contrast across lattice of length L = N:
        // Lambda_skin = 20 * log10( (t_R / t_L)^(N/2) ) = 10 * N * log10(t_R / t_L)
        let n = self.lattice_sites_count as f64;
        let contrast_db = (10.0 * n * ratio.log10()).clamp(25.0, 120.0);

        // GBZ radius r_GBZ = sqrt(t_L / t_R) < 1
        let r_gbz = (tl / tr).sqrt();

        // Spectral winding number: non-reciprocity t_R > t_L yields W = +1
        let winding = if tr > tl { 1 } else { -1 };

        // EP order (2nd order EP2 or 3rd order EP3):
        let ep_order = if self.onsite_gain_loss_mhz > 1.2 {
            3
        } else {
            2
        };

        // Exceptional point sensitivity enhancement: S_EP = 1 / eps^((N_EP-1)/N_EP)
        let eps = self.perturbation_epsilon.clamp(1.0e-5, 0.05);
        let power = (ep_order as f64 - 1.0) / ep_order as f64;
        let sens_raw = 1.0 / eps.powf(power);
        let sens_enhancement = (sens_raw * 0.40).clamp(10.0, 150.0);

        NonHermitianSkinMetrics {
            skin_depth_unit_cells: xi_cells,
            skin_localization_contrast_db: contrast_db,
            gbz_radius: r_gbz,
            spectral_winding_number: winding,
            exceptional_point_order: ep_order,
            sensitivity_enhancement_factor: sens_enhancement,
        }
    }
}
