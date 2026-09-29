//! Non-Hermitian chiral phonon higher-order topological insulators (HOTI),
//! quantized quadrupole polarizations, corner skin modes, and chiral acoustoelectricity.

/// Parameters for a 2D non-Hermitian chiral higher-order topological acoustic lattice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianHotiParams {
    /// Number of unit cells along the x-axis $L_x$ (nominal $12 - 32$).
    pub grid_dim_x: usize,
    /// Number of unit cells along the y-axis $L_y$ (nominal $12 - 32$).
    pub grid_dim_y: usize,
    /// Intra-cell acoustic hopping rate $\gamma$ in $\text{MHz}$ (nominal $0.2 - 0.7\text{ MHz}$).
    pub intra_cell_hopping_mhz: f64,
    /// Inter-cell acoustic hopping rate $\lambda$ in $\text{MHz}$ (nominal $1.2 - 2.5\text{ MHz}$).
    pub inter_cell_hopping_mhz: f64,
    /// Non-Hermitian non-reciprocal hopping asymmetry factor $\eta \in [0.2, 0.8]$.
    pub non_reciprocal_asymmetry_eta: f64,
    /// Traveling surface acoustic wave (SAW) power flux $I_{\mathrm{SAW}}$ in $\text{mW/mm}$ (nominal $0.5 - 12.0\text{ mW/mm}$).
    pub saw_power_flux_mw_mm: f64,
    /// Electromechanical piezoelectric coupling coefficient $K^2$ in percent (nominal $1.0\% - 5.5\%$).
    pub piezoelectric_coupling_k2_pct: f64,
    /// Effective electronic carrier mobility $\mu_{\mathrm{eff}}$ in $\text{cm}^2/(\text{V}\cdot\text{s})$ (nominal $1,000 - 15,000$).
    pub carrier_mobility_cm2_v_s: f64,
    /// SAW excitation frequency in $\text{GHz}$ (nominal $0.5 - 4.0\text{ GHz}$).
    pub saw_frequency_ghz: f64,
}

impl Default for NonHermitianHotiParams {
    fn default() -> Self {
        Self {
            grid_dim_x: 20,
            grid_dim_y: 20,
            intra_cell_hopping_mhz: 0.40,
            inter_cell_hopping_mhz: 1.80,
            non_reciprocal_asymmetry_eta: 0.45,
            saw_power_flux_mw_mm: 4.5,
            piezoelectric_coupling_k2_pct: 3.2,
            carrier_mobility_cm2_v_s: 4_500.0,
            saw_frequency_ghz: 1.5,
        }
    }
}

/// Evaluated metrics for non-Hermitian chiral higher-order topological insulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HotiCornerMetrics {
    /// 0D corner skin mode localization contrast in decibels $\Lambda_{\mathrm{corner}} \ge 30.0\text{ dB}$.
    pub corner_localization_contrast_db: f64,
    /// Corner skin depth in unit cells $\xi_{\mathrm{corner}} \le 2.0\text{ cells}$.
    pub corner_skin_depth_cells: f64,
    /// Quantized 2D bulk quadrupole topological polarization $|q_{xy}| = 0.5$.
    pub quantized_quadrupole_polarization: f64,
    /// Injected chiral acoustoelectric DC current density $j_{\mathrm{AE}}$ in $\text{A/m}^2$.
    pub acoustoelectric_current_density_a_m2: f64,
    /// Non-reciprocal acoustoelectric rectification ratio in decibels ($\ge 20.0\text{ dB}$).
    pub acoustoelectric_rectification_db: f64,
    /// Multi-terminal corner acoustic sensor signal-to-noise ratio in decibels ($\ge 20.0\text{ dB}$).
    pub corner_sensor_snr_db: f64,
    /// Number of localized zero-energy topological corner bound states.
    pub topological_corner_mode_count: usize,
}

impl NonHermitianHotiParams {
    /// Creates a new parameter set for non-Hermitian chiral HOTI systems.
    pub fn new(
        grid_dim_x: usize,
        grid_dim_y: usize,
        intra_hopping: f64,
        inter_hopping: f64,
    ) -> Self {
        Self {
            grid_dim_x: grid_dim_x.max(4),
            grid_dim_y: grid_dim_y.max(4),
            intra_cell_hopping_mhz: intra_hopping.max(0.01),
            inter_cell_hopping_mhz: inter_hopping.max(0.01),
            ..Default::default()
        }
    }
}
