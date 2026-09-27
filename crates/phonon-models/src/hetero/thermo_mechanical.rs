//! Coupled thermo-mechanical strain, Black's electromigration, and 2D thermal hotspot solvers.
//!
//! Provides:
//! 1. `ThermoMechanicalStressModel`: Coefficient of Thermal Expansion (CTE) mismatch strain,
//!    bi-axial stress, piezoresistive mobility modulation, and mechanical delamination safety.
//! 2. `ElectromigrationModel`: Black's equation MTTF and maximum safe current density.
//! 3. `ThermalHotspotSolver`: 2D finite-difference heat diffusion across heterogeneous thermal conductivities,
//!    extracting peak hotspot temperatures and thermal gradients.

use crate::hetero::cpu_floorplan::ProcessorFloorplan;
use crate::hetero::material_allocation::HeteroMaterialType;

const K_B_EV: f64 = 8.617_333_262e-5; // Boltzmann constant in eV/K

/// Thermo-mechanical stress and piezoresistive evaluation result for a macro block.
#[derive(Debug, Clone, Copy)]
pub struct BlockStressReport {
    /// Thermal strain epsilon_th
    pub thermal_strain: f64,
    /// Bi-axial thermal stress sigma_th [MPa]
    pub thermal_stress_mpa: f64,
    /// Fractional mobility modulation Delta_mu / mu
    pub mobility_shift_fraction: f64,
    /// Mechanical integrity safety factor against delamination
    pub delamination_safety_factor: f64,
    /// True if stress is below critical fracture threshold (< 800 MPa)
    pub is_mechanically_sound: bool,
}

/// Evaluator for thermo-mechanical strain and piezoresistive stress effects.
pub struct ThermoMechanicalStressModel;

impl ThermoMechanicalStressModel {
    /// Evaluates thermal stress in a block integrated onto a Silicon substrate.
    pub fn evaluate_block_stress(
        material: HeteroMaterialType,
        temp_k: f64,
        t_ref_k: f64,
    ) -> BlockStressReport {
        let props = material.properties();
        let sub_props = HeteroMaterialType::SiliconGaa.properties();

        let delta_t = temp_k - t_ref_k;
        // CTE mismatch: Delta_alpha = (alpha_block - alpha_sub) * 1e-6
        let delta_alpha = (props.cte_ppm_per_k - sub_props.cte_ppm_per_k) * 1e-6;
        let thermal_strain = delta_alpha * delta_t;

        // Bi-axial stress: sigma = (E / (1 - nu)) * strain [Pa] -> / 1e6 for MPa
        let e_pa = props.youngs_modulus_gpa * 1e9;
        let stress_pa = (e_pa / (1.0 - props.poisson_ratio).max(0.1)) * thermal_strain;
        let thermal_stress_mpa = stress_pa * 1.0e-6;

        // Piezoresistive mobility shift: Delta_mu / mu = Pi * sigma
        // Typical piezoresistive coefficient Pi_11 ~ -3.5e-4 MPa^-1 for holes, +6.0e-4 MPa^-1 for electrons
        let pi_eff = match material {
            HeteroMaterialType::StrainedGePmos => -4.5e-4,
            HeteroMaterialType::InGaAsNmos => 6.5e-4,
            _ => 2.0e-4,
        };
        let mobility_shift_fraction = pi_eff * thermal_stress_mpa;

        // Critical delamination / fracture stress limit ~ 800 MPa
        let critical_stress_mpa = 800.0;
        let delamination_safety_factor = critical_stress_mpa / thermal_stress_mpa.abs().max(1.0);
        let is_mechanically_sound = thermal_stress_mpa.abs() < critical_stress_mpa;

        BlockStressReport {
            thermal_strain,
            thermal_stress_mpa,
            mobility_shift_fraction,
            delamination_safety_factor,
            is_mechanically_sound,
        }
    }
}

/// Black's Electromigration Model for evaluating interconnect wire lifetime.
pub struct BlackElectromigrationModel;

impl BlackElectromigrationModel {
    /// Computes Mean Time to Failure (MTTF) in hours using Black's Equation:
    /// \[MTTF = \frac{A}{J^n} \exp\left(\frac{E_a}{k_B T}\right)\]
    pub fn compute_mttf_hours(
        material: HeteroMaterialType,
        current_density_a_cm2: f64,
        temp_k: f64,
    ) -> f64 {
        let props = material.properties();
        let a_const = 1.0e14; // Pre-exponential empirical constant
        let n_exp = 2.0; // Current exponent
        let j = current_density_a_cm2.max(1.0);

        let t = temp_k.max(100.0);
        let exp_term = (props.black_activation_energy_ev / (K_B_EV * t)).exp();

        (a_const / j.powf(n_exp)) * exp_term
    }

    /// Computes MTTF in years.
    pub fn compute_mttf_years(
        material: HeteroMaterialType,
        current_density_a_cm2: f64,
        temp_k: f64,
    ) -> f64 {
        Self::compute_mttf_hours(material, current_density_a_cm2, temp_k) / 8760.0
    }
}

/// 2D thermal hotspot solution report across processor die.
#[derive(Debug, Clone)]
pub struct ThermalHotspotReport {
    /// Peak junction hotspot temperature [K]
    pub t_peak_k: f64,
    /// Peak junction hotspot temperature [°C]
    pub t_peak_c: f64,
    /// Mean processor die temperature [K]
    pub t_mean_k: f64,
    /// Maximum thermal lateral gradient [K / mm]
    pub max_thermal_gradient_k_per_mm: f64,
    /// Name of the hottest macro block
    pub hottest_block_name: String,
}

impl ThermalHotspotReport {
    /// Mean processor die temperature [°C].
    pub fn t_mean_c(&self) -> f64 {
        self.t_mean_k - 273.15
    }
}

/// 2D finite-difference heat diffusion solver for heterogeneous floorplans.
#[derive(Debug, Clone)]
pub struct ThermalHotspotSolver {
    grid_nx: usize,
    grid_ny: usize,
    ambient_temp_k: f64,
    r_sink_k_cm2_per_w: f64, // Thermal resistance to heat sink
}

impl Default for ThermalHotspotSolver {
    fn default() -> Self {
        Self {
            grid_nx: 50,
            grid_ny: 25,
            ambient_temp_k: 300.0,
            r_sink_k_cm2_per_w: 0.15, // High-performance vapor chamber heat sink
        }
    }
}

impl ThermalHotspotSolver {
    /// Solves 2D thermal diffusion across the floorplan for specified operating conditions.
    pub fn solve(
        &self,
        floorplan: &ProcessorFloorplan,
        v_dd: f64,
        freq_ghz: f64,
    ) -> ThermalHotspotReport {
        let nx = self.grid_nx;
        let ny = self.grid_ny;

        let dx_m = (floorplan.die_width_um * 1.0e-6) / (nx as f64);
        let dy_m = (floorplan.die_height_um * 1.0e-6) / (ny as f64);
        let cell_area_cm2 = (dx_m * dy_m) * 1.0e4;
        let die_thickness_m = 50.0e-6; // 50 um thinned die

        // Populate local thermal conductivity and heat generation grids
        let mut kappa_grid = vec![vec![140.0; ny]; nx];
        let mut heat_gen_grid = vec![vec![0.0; ny]; nx];

        for i in 0..nx {
            let x_um = ((i as f64) + 0.5) * (floorplan.die_width_um / (nx as f64));
            for j in 0..ny {
                let y_um = ((j as f64) + 0.5) * (floorplan.die_height_um / (ny as f64));

                // Find block enclosing this cell
                for block in &floorplan.blocks {
                    if x_um >= block.x_um
                        && x_um <= block.x_um + block.width_um
                        && y_um >= block.y_um
                        && y_um <= block.y_um + block.height_um
                    {
                        kappa_grid[i][j] = block.material.properties().thermal_conductivity_w_mk;
                        let block_power_w = block.total_power_mw(v_dd, freq_ghz) * 1.0e-3;
                        let block_volume_m3 = (block.area_um2() * 1.0e-12) * die_thickness_m;
                        heat_gen_grid[i][j] = block_power_w / block_volume_m3.max(1e-18);
                        break;
                    }
                }
            }
        }

        // Successive Over-Relaxation (SOR) solver for 2D steady-state conduction
        let mut temp = vec![vec![self.ambient_temp_k; ny]; nx];
        let g_sink_cell = cell_area_cm2 / self.r_sink_k_cm2_per_w; // Conductance to ambient [W/K]

        let omega = 1.35; // SOR relaxation parameter
        for _iter in 0..250 {
            for i in 0..nx {
                for j in 0..ny {
                    let k_c = kappa_grid[i][j];

                    // Harmonic mean conductivities across cell interfaces
                    let k_w = if i > 0 {
                        2.0 * k_c * kappa_grid[i - 1][j] / (k_c + kappa_grid[i - 1][j])
                    } else {
                        k_c
                    };
                    let k_e = if i < nx - 1 {
                        2.0 * k_c * kappa_grid[i + 1][j] / (k_c + kappa_grid[i + 1][j])
                    } else {
                        k_c
                    };
                    let k_s = if j > 0 {
                        2.0 * k_c * kappa_grid[i][j - 1] / (k_c + kappa_grid[i][j - 1])
                    } else {
                        k_c
                    };
                    let k_n = if j < ny - 1 {
                        2.0 * k_c * kappa_grid[i][j + 1] / (k_c + kappa_grid[i][j + 1])
                    } else {
                        k_c
                    };

                    let g_w = k_w * (dy_m * die_thickness_m) / dx_m;
                    let g_e = k_e * (dy_m * die_thickness_m) / dx_m;
                    let g_s = k_s * (dx_m * die_thickness_m) / dy_m;
                    let g_n = k_n * (dx_m * die_thickness_m) / dy_m;

                    let t_w = if i > 0 { temp[i - 1][j] } else { temp[i][j] };
                    let t_e = if i < nx - 1 {
                        temp[i + 1][j]
                    } else {
                        temp[i][j]
                    };
                    let t_s = if j > 0 { temp[i][j - 1] } else { temp[i][j] };
                    let t_n = if j < ny - 1 {
                        temp[i][j + 1]
                    } else {
                        temp[i][j]
                    };

                    let q_cell_w = heat_gen_grid[i][j] * (dx_m * dy_m * die_thickness_m);

                    let sum_conductance = g_w + g_e + g_s + g_n + g_sink_cell;
                    let rhs = g_w * t_w
                        + g_e * t_e
                        + g_s * t_s
                        + g_n * t_n
                        + g_sink_cell * self.ambient_temp_k
                        + q_cell_w;

                    let t_new = rhs / sum_conductance;
                    temp[i][j] = (1.0 - omega) * temp[i][j] + omega * t_new;
                }
            }
        }

        // Extract statistics
        let mut t_peak_k = self.ambient_temp_k;
        let mut t_sum_k = 0.0;
        let mut max_grad_k_per_mm = 0.0;

        for i in 0..nx {
            for j in 0..ny {
                let t_val = temp[i][j];
                if t_val > t_peak_k {
                    t_peak_k = t_val;
                }
                t_sum_k += t_val;

                if i < nx - 1 {
                    let grad_x = (temp[i + 1][j] - t_val).abs() / (dx_m * 1.0e3);
                    if grad_x > max_grad_k_per_mm {
                        max_grad_k_per_mm = grad_x;
                    }
                }
                if j < ny - 1 {
                    let grad_y = (temp[i][j + 1] - t_val).abs() / (dy_m * 1.0e3);
                    if grad_y > max_grad_k_per_mm {
                        max_grad_k_per_mm = grad_y;
                    }
                }
            }
        }

        let t_mean_k = t_sum_k / ((nx * ny) as f64);
        let hottest = floorplan.find_peak_hotspot_block(v_dd, freq_ghz);

        ThermalHotspotReport {
            t_peak_k,
            t_peak_c: t_peak_k - 273.15,
            t_mean_k,
            max_thermal_gradient_k_per_mm: max_grad_k_per_mm,
            hottest_block_name: hottest.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hetero::cpu_floorplan::RiscVFloorplanBuilder;
    use crate::hetero::material_allocation::BlockAllocationMap;

    #[test]
    fn test_thermo_mechanical_stress_and_safety() {
        let rep_ge = ThermoMechanicalStressModel::evaluate_block_stress(
            HeteroMaterialType::StrainedGePmos,
            360.0, // 60 C above ambient
            300.0,
        );

        assert!(rep_ge.is_mechanically_sound);
        assert!(rep_ge.delamination_safety_factor > 1.5);
        assert!(rep_ge.thermal_stress_mpa > 0.0); // Tensile thermal stress
    }

    #[test]
    fn test_black_electromigration_cnt_vs_copper() {
        let temp = 350.0;
        let j = 5.0e6; // 5 MA/cm^2 (high current clock tree)

        let mttf_cu =
            BlackElectromigrationModel::compute_mttf_years(HeteroMaterialType::SiliconGaa, j, temp);
        let mttf_cnt = BlackElectromigrationModel::compute_mttf_years(
            HeteroMaterialType::CntBundleInterconnect,
            j,
            temp,
        );

        // CNT bundle must exhibit vastly superior electromigration lifetime (> 1000x)
        assert!(mttf_cnt > mttf_cu * 1000.0);
    }

    #[test]
    fn test_thermal_hotspot_solver_convergence() {
        let alloc = BlockAllocationMap::uniform_silicon();
        let floorplan = RiscVFloorplanBuilder::build(&alloc);

        let solver = ThermalHotspotSolver::default();
        let report = solver.solve(&floorplan, 0.8, 3.0);

        assert!(report.t_peak_k > 300.0);
        assert!(report.t_peak_c > 26.85);
        assert!(report.t_peak_k >= report.t_mean_k);
        assert!(report.max_thermal_gradient_k_per_mm > 0.0);
    }
}
