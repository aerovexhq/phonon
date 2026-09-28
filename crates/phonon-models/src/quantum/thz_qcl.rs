//! Terahertz Quantum Cascade Lasers (THz QCLs), Resonant LO-Phonon Depopulation,
//! Optical Intersubband Gain Spectra & Polaritonic Waveguides.
//!
//! Formulates:
//! - 1D Schrödinger solver evaluating bound intersubband eigenstates $\psi_i(z)$,
//!   energies $E_i$, and transition dipole matrix elements $z_{ij} = \int \psi_i^*(z) z \psi_j(z) dz$
//!   under the BenDaniel-Duke variable effective mass Hamiltonian:
//!   $$-\frac{\hbar^2}{2} \frac{d}{dz}\left( \frac{1}{m^*(z)} \frac{d\psi}{dz} \right) + V(z) \psi(z) = E \psi(z)$$
//! - Resonant LO-Phonon depopulation scheme:
//!   Lower laser level depopulation via resonant longitudinal optical phonon emission
//!   matching $\Delta E_{21} \approx \hbar \omega_{LO} \approx 36\text{ meV}$, enabling sub-picosecond
//!   carrier extraction ($\tau_{21} \approx 0.3\text{ ps}$) to sustain steady-state population inversion
//!   $\Delta n = n_3 - n_2 > 0$.
//! - Optical gain spectrum $g(\nu)$ across sub-millimeter / THz frequencies ($0.5-10\text{ THz}$, $\lambda \approx 30-600\,\mu\text{m}$):
//!   $$g(\nu) = \frac{4\pi e^2 |z_{32}|^2 \nu}{\epsilon_0 c n_r \hbar} \frac{\Delta n \cdot (\gamma_{32} / 2\pi)}{(\nu - \nu_0)^2 + (\gamma_{32} / 2\pi)^2}$$
//! - Metal-Metal (MM) and Semi-Insulating Surface-Plasmon (SI-SP) polaritonic waveguides:
//!   Optical confinement factors $\Gamma \approx 0.85-0.95$ for MM waveguides, sub-wavelength mode confinement,
//!   and Drude free-carrier mirror/waveguide losses $\alpha_w$.

use phonon_core::constants::{ELEMENTARY_CHARGE, EPSILON_0, H_BAR, SPEED_OF_LIGHT};

/// Rest mass of the electron $m_0$ in kilograms ($kg$).
pub const ELECTRON_MASS_KG: f64 = 9.109_383_701_5e-31;

/// Longitudinal Optical (LO) phonon energy in GaAs ($\hbar \omega_{LO} \approx 36.25\text{ meV}$).
pub const GAAS_LO_PHONON_ENERGY_EV: f64 = 0.036_25;

/// Longitudinal Optical (LO) phonon energy in Joules ($J$).
pub const GAAS_LO_PHONON_ENERGY_JOULES: f64 = GAAS_LO_PHONON_ENERGY_EV * ELEMENTARY_CHARGE;

/// Semiconductor material system for Multiple Quantum Well (MQW) heterostructures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QclMaterialSystem {
    /// Gallium Arsenide / Aluminum Gallium Arsenide ($\text{GaAs} / \text{Al}_x\text{Ga}_{1-x}\text{As}$).
    GaAsAlGaAs,
    /// Indium Gallium Arsenide / Indium Aluminum Arsenide ($\text{In}_{0.53}\text{Ga}_{0.47}\text{As} / \text{In}_{0.52}\text{Al}_{0.48}\text{As}$).
    InGaAsInAlAs,
}

impl QclMaterialSystem {
    /// Well electron effective mass ratio $m^* / m_0$.
    #[inline]
    pub fn well_effective_mass_ratio(&self) -> f64 {
        match self {
            Self::GaAsAlGaAs => 0.067,
            Self::InGaAsInAlAs => 0.043,
        }
    }

    /// Barrier electron effective mass ratio $m^* / m_0$ (at typical alloy compositions: $x=0.15$ for AlGaAs).
    #[inline]
    pub fn barrier_effective_mass_ratio(&self) -> f64 {
        match self {
            Self::GaAsAlGaAs => 0.079,
            Self::InGaAsInAlAs => 0.075,
        }
    }

    /// Conduction band offset $\Delta E_c$ in electron-volts ($eV$).
    #[inline]
    pub fn conduction_band_offset_ev(&self) -> f64 {
        match self {
            Self::GaAsAlGaAs => 0.125,   // 125 meV barrier for THz QCL (x ~ 0.15)
            Self::InGaAsInAlAs => 0.520, // 520 meV barrier
        }
    }

    /// Bulk LO-phonon energy $\hbar \omega_{LO}$ in electron-volts ($eV$).
    #[inline]
    pub fn lo_phonon_energy_ev(&self) -> f64 {
        match self {
            Self::GaAsAlGaAs => 0.036_25,   // 36.25 meV in GaAs
            Self::InGaAsInAlAs => 0.034_00, // ~34.0 meV in InGaAs
        }
    }

    /// Background refractive index $n_r$ at terahertz frequencies (~3 THz).
    #[inline]
    pub fn thz_refractive_index(&self) -> f64 {
        match self {
            Self::GaAsAlGaAs => 3.60,
            Self::InGaAsInAlAs => 3.52,
        }
    }

    /// High-frequency dielectric constant $\epsilon_\infty$.
    #[inline]
    pub fn high_frequency_permittivity(&self) -> f64 {
        match self {
            Self::GaAsAlGaAs => 10.89,
            Self::InGaAsInAlAs => 11.60,
        }
    }

    /// Static dielectric constant $\epsilon_s$.
    #[inline]
    pub fn static_permittivity(&self) -> f64 {
        match self {
            Self::GaAsAlGaAs => 12.90,
            Self::InGaAsInAlAs => 13.90,
        }
    }
}

/// Single semiconductor layer in an MQW active region period.
#[derive(Debug, Clone, PartialEq)]
pub struct HeterostructureLayer {
    /// Layer thickness in meters ($m$).
    pub thickness_m: f64,
    /// Potential barrier flag (true if barrier, false if well).
    pub is_barrier: bool,
    /// Conduction band edge potential energy $V_c$ in electron-volts ($eV$).
    pub potential_ev: f64,
    /// Electron effective mass ratio $m^* / m_0$.
    pub effective_mass_ratio: f64,
}

impl HeterostructureLayer {
    /// Creates a well layer with specified thickness.
    pub fn well(thickness_m: f64, material: QclMaterialSystem) -> Self {
        Self {
            thickness_m,
            is_barrier: false,
            potential_ev: 0.0,
            effective_mass_ratio: material.well_effective_mass_ratio(),
        }
    }

    /// Creates a barrier layer with specified thickness.
    pub fn barrier(thickness_m: f64, material: QclMaterialSystem) -> Self {
        Self {
            thickness_m,
            is_barrier: true,
            potential_ev: material.conduction_band_offset_ev(),
            effective_mass_ratio: material.barrier_effective_mass_ratio(),
        }
    }
}

/// Bound intersubband eigenstate evaluated by the 1D Schrödinger solver.
#[derive(Debug, Clone, PartialEq)]
pub struct IntersubbandEigenstate {
    /// Eigenstate subband index (0-indexed, $0 = \text{ground state}$).
    pub index: usize,
    /// Eigenenergy $E_i$ in electron-volts ($eV$).
    pub energy_ev: f64,
    /// Eigenenergy $E_i$ in Joules ($J$).
    pub energy_joules: f64,
    /// Normalized wavefunction spatial profile $\psi_i(z)$ ($\text{m}^{-1/2}$).
    pub wavefunction: Vec<f64>,
}

/// 1D Multiple Quantum Well (MQW) active region heterostructure profile.
#[derive(Debug, Clone, PartialEq)]
pub struct HeterostructureProfile {
    /// Semiconductor material system.
    pub material_system: QclMaterialSystem,
    /// Sequential layer stack comprising one or more periods.
    pub layers: Vec<HeterostructureLayer>,
    /// Total thickness across all layers in meters ($m$).
    pub total_thickness_m: f64,
    /// Spatial discretization step size $\Delta z$ in meters ($m$).
    pub grid_step_m: f64,
    /// Spatial grid coordinate points $z_k$ in meters ($m$).
    pub grid_points: Vec<f64>,
    /// Potential energy profile $V(z)$ in Joules ($J$).
    pub potential_profile_joules: Vec<f64>,
    /// Effective mass profile $m^*(z)$ in kilograms ($kg$).
    pub mass_profile_kg: Vec<f64>,
}

impl HeterostructureProfile {
    /// Constructs a discretized heterostructure profile from a sequence of layers.
    pub fn new(
        material_system: QclMaterialSystem,
        layers: Vec<HeterostructureLayer>,
        target_grid_step_m: f64,
    ) -> Self {
        let total_thickness: f64 = layers.iter().map(|l| l.thickness_m).sum();
        let grid_step = target_grid_step_m.max(1e-11);
        let num_points = ((total_thickness / grid_step).round() as usize).max(30);
        let actual_grid_step = total_thickness / (num_points - 1) as f64;

        let mut grid_points = Vec::with_capacity(num_points);
        let mut potential_profile_joules = Vec::with_capacity(num_points);
        let mut mass_profile_kg = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let z = i as f64 * actual_grid_step;
            grid_points.push(z);

            // Locate layer index for coordinate z:
            let mut cumulative_z = 0.0;
            let mut matched_layer = &layers[0];
            for layer in &layers {
                cumulative_z += layer.thickness_m;
                if z <= cumulative_z || (z - cumulative_z).abs() < 1e-14 {
                    matched_layer = layer;
                    break;
                }
            }

            potential_profile_joules.push(matched_layer.potential_ev * ELEMENTARY_CHARGE);
            mass_profile_kg.push(matched_layer.effective_mass_ratio * ELECTRON_MASS_KG);
        }

        Self {
            material_system,
            layers,
            total_thickness_m: total_thickness,
            grid_step_m: actual_grid_step,
            grid_points,
            potential_profile_joules,
            mass_profile_kg,
        }
    }

    /// Standard resonant LO-phonon depopulation THz QCL active stage design in $\text{GaAs}/\text{Al}_{0.15}\text{Ga}_{0.85}\text{As}$.
    ///
    /// Layer sequence (in nanometers):
    /// Barrier (thick outer 12 nm) / Injector Well (9.0 nm) / Barrier (4.4 nm) / Active Upper Well (15.5 nm) /
    /// Barrier (3.8 nm) / Active Lower Well (8.8 nm) / Collector Barrier (12 nm).
    ///
    /// Yields transition energy $\Delta E_{32} \approx 12-14\text{ meV}$ ($\nu_0 \approx 3.0-3.4\text{ THz}$)
    /// and resonant LO-phonon extraction spacing $\Delta E_{21} \approx 36\text{ meV}$.
    pub fn standard_gaas_thz_active_stage() -> Self {
        let mat = QclMaterialSystem::GaAsAlGaAs;
        let layers = vec![
            HeterostructureLayer::barrier(10.0e-9, mat), // Cladding barrier
            HeterostructureLayer::well(4.8e-9, mat),     // Upper laser well (level 3)
            HeterostructureLayer::barrier(3.0e-9, mat),  // Radiative tunnel barrier
            HeterostructureLayer::well(17.1e-9, mat),    // Phonon extraction well (levels 2 and 1)
            HeterostructureLayer::barrier(10.0e-9, mat), // Extraction barrier
        ];
        Self::new(mat, layers, 0.20e-9) // 0.20 nm grid resolution
    }

    /// Infinite square well benchmark heterostructure with width $L$ in meters.
    pub fn infinite_square_well(width_m: f64, material: QclMaterialSystem) -> Self {
        let layers = vec![HeterostructureLayer::well(width_m, material)];
        Self::new(material, layers, width_m / 150.0)
    }
}

/// Solves the 1D Schrödinger equation under the BenDaniel-Duke boundary conditions:
/// $$-\frac{\hbar^2}{2} \frac{d}{dz}\left( \frac{1}{m^*(z)} \frac{d\psi}{dz} \right) + V(z) \psi(z) = E \psi(z)$$
pub struct Schrodinger1DSolver;

impl Schrodinger1DSolver {
    /// Evaluates the lowest $M$ bound intersubband eigenstates $\psi_i(z)$ and eigenenergies $E_i$.
    pub fn solve_bound_states(
        profile: &HeterostructureProfile,
        max_states: usize,
    ) -> Vec<IntersubbandEigenstate> {
        let n_total = profile.grid_points.len();
        if n_total < 4 {
            return Vec::new();
        }

        // Interior grid nodes: k = 1 to n_total - 2 (enforcing Dirichlet psi(0) = psi(L) = 0):
        let m = n_total - 2;
        let dz = profile.grid_step_m;
        let dz_sq = dz * dz;

        let mut diag = vec![0.0; m];
        let mut subdiag = vec![0.0; m];

        for k in 0..m {
            let orig_k = k + 1;
            let v_k = profile.potential_profile_joules[orig_k];

            // Harmonic mean effective masses at half-steps:
            let m_plus =
                0.5 * (profile.mass_profile_kg[orig_k] + profile.mass_profile_kg[orig_k + 1]);
            let m_minus =
                0.5 * (profile.mass_profile_kg[orig_k] + profile.mass_profile_kg[orig_k - 1]);

            let t_plus = (H_BAR * H_BAR) / (2.0 * dz_sq * m_plus);
            let t_minus = (H_BAR * H_BAR) / (2.0 * dz_sq * m_minus);

            diag[k] = v_k + t_plus + t_minus;

            if k < m - 1 {
                subdiag[k] = -t_plus;
            }
        }

        // Compute eigenvalues and eigenvectors using symmetric tridiagonal QL solver:
        let mut z_mat = vec![0.0; m * m];
        for i in 0..m {
            z_mat[i * m + i] = 1.0;
        }

        symmetric_tridiagonal_ql(&mut diag, &mut subdiag, &mut z_mat, m);

        // Sort eigenstates by ascending energy:
        let mut indexed_energies: Vec<(usize, f64)> = diag.iter().copied().enumerate().collect();
        indexed_energies.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let num_to_take = max_states.min(m);
        let mut eigenstates = Vec::with_capacity(num_to_take);

        for (state_idx, &(orig_col, e_joules)) in
            indexed_energies.iter().take(num_to_take).enumerate()
        {
            let mut wf = vec![0.0; n_total];

            for k in 0..m {
                wf[k + 1] = z_mat[k * m + orig_col] / dz.sqrt();
            }

            eigenstates.push(IntersubbandEigenstate {
                index: state_idx,
                energy_ev: e_joules / ELEMENTARY_CHARGE,
                energy_joules: e_joules,
                wavefunction: wf,
            });
        }

        eigenstates
    }

    /// Evaluates the transition dipole matrix element between states $i$ and $j$:
    /// $$z_{ij} = \int_0^L \psi_i^*(z) z \psi_j(z) dz$$
    pub fn compute_dipole_matrix_element(
        profile: &HeterostructureProfile,
        state_i: &IntersubbandEigenstate,
        state_j: &IntersubbandEigenstate,
    ) -> f64 {
        let dz = profile.grid_step_m;
        let mut dipole = 0.0;
        let len = profile.grid_points.len();

        for k in 0..len {
            let z = profile.grid_points[k];
            let psi_i = state_i.wavefunction[k];
            let psi_j = state_j.wavefunction[k];
            dipole += psi_i * z * psi_j * dz;
        }

        dipole.abs()
    }

    /// Evaluates intersubband oscillator strength:
    /// $$f_{ij} = \frac{2 m^* \omega_{ij}}{\hbar} |z_{ij}|^2$$
    pub fn compute_oscillator_strength(
        material: QclMaterialSystem,
        delta_e_joules: f64,
        dipole_z_m: f64,
    ) -> f64 {
        let m_eff = material.well_effective_mass_ratio() * ELECTRON_MASS_KG;
        let omega = delta_e_joules / H_BAR;
        (2.0 * m_eff * omega / H_BAR) * dipole_z_m.powi(2)
    }
}

/// Resonant LO-Phonon depopulation scheme parameters and scattering dynamics.
#[derive(Debug, Clone, PartialEq)]
pub struct ResonantLoPhononDepopulation {
    /// Material system.
    pub material_system: QclMaterialSystem,
    /// Longitudinal optical phonon energy $\hbar \omega_{LO}$ in electron-volts ($eV$).
    pub lo_phonon_energy_ev: f64,
    /// Energy separation between lower laser level (2) and depopulation ground level (1) $\Delta E_{21}$ ($eV$).
    pub delta_e21_ev: f64,
    /// Intrinsic on-resonance LO-phonon scattering extraction time $\tau_{LO,0}$ in seconds ($s$) (~0.3 ps).
    pub resonant_extraction_tau_0_s: f64,
    /// Non-resonant intersubband relaxation lifetime between level 3 and level 2 $\tau_{32}$ in seconds ($s$) (~5 ps).
    pub upper_to_lower_tau_32_s: f64,
    /// Level 3 total upper lifetime $\tau_3$ in seconds ($s$) (~2-3 ps).
    pub upper_state_tau_3_s: f64,
    /// Resonance broadening $\Gamma_{LO}$ in electron-volts ($eV$) (~4 meV).
    pub resonance_fwhm_ev: f64,
}

impl ResonantLoPhononDepopulation {
    /// Creates a standard GaAs resonant LO-phonon depopulation configuration.
    pub fn standard_gaas(delta_e21_ev: f64) -> Self {
        Self {
            material_system: QclMaterialSystem::GaAsAlGaAs,
            lo_phonon_energy_ev: GAAS_LO_PHONON_ENERGY_EV,
            delta_e21_ev,
            resonant_extraction_tau_0_s: 0.30e-12, // 0.3 ps
            upper_to_lower_tau_32_s: 5.5e-12,      // 5.5 ps
            upper_state_tau_3_s: 2.8e-12,          // 2.8 ps
            resonance_fwhm_ev: 0.004,              // 4 meV
        }
    }

    /// Evaluates effective lower level (2) LO-phonon depopulation extraction lifetime $\tau_{21}$:
    /// $$\tau_{21}(\Delta E_{21}) = \tau_{LO,0} \left[ 1 + \left(\frac{\Delta E_{21} - \hbar \omega_{LO}}{\Gamma_{LO}}\right)^2 \right]$$
    pub fn extraction_lifetime_tau_21(&self) -> f64 {
        let detuning = (self.delta_e21_ev - self.lo_phonon_energy_ev) / self.resonance_fwhm_ev;
        self.resonant_extraction_tau_0_s * (1.0 + detuning * detuning)
    }

    /// Evaluates the extraction efficiency factor $(1 - \tau_2 / \tau_{32})$ sustaining steady-state inversion.
    #[inline]
    pub fn inversion_sustainability_factor(&self) -> f64 {
        let tau_2 = self.extraction_lifetime_tau_21();
        (1.0 - tau_2 / self.upper_to_lower_tau_32_s).max(0.0)
    }

    /// Evaluates steady-state 3D population inversion density $\Delta n = n_3 - n_2$ in $\text{m}^{-3}$
    /// for injected current density $J$ in $\text{A/m}^2$, injection efficiency $\eta_{inj}$, and stage length $L_p$ in meters ($m$):
    /// $$\Delta n = \frac{J \eta_{inj} \tau_3}{e L_p} \left( 1 - \frac{\tau_2}{\tau_{32}} \right)$$
    pub fn steady_state_population_inversion(
        &self,
        current_density_a_per_m2: f64,
        injection_efficiency: f64,
        stage_length_m: f64,
    ) -> f64 {
        let factor = self.inversion_sustainability_factor();
        let n3_gen = (current_density_a_per_m2 * injection_efficiency * self.upper_state_tau_3_s)
            / (ELEMENTARY_CHARGE * stage_length_m);
        n3_gen * factor
    }
}

/// Optical gain spectrum model across sub-millimeter / THz frequencies ($0.5 - 10\text{ THz}$, $\lambda \approx 30 - 600\,\mu\text{m}$).
#[derive(Debug, Clone, PartialEq)]
pub struct ThzOpticalGainModel {
    /// Intersubband transition peak frequency $\nu_0$ in Hertz ($Hz$).
    pub center_frequency_hz: f64,
    /// Transition dipole matrix element $|z_{32}|$ in meters ($m$).
    pub transition_dipole_z32_m: f64,
    /// Active region optical refractive index $n_r$.
    pub refractive_index: f64,
    /// Transition full-width at half-maximum (FWHM) linewidth $\Delta \nu = \gamma_{32} / 2\pi$ in Hertz ($Hz$) (~0.8 THz).
    pub linewidth_fwhm_hz: f64,
    /// Stage thickness / period length $L_p$ in meters ($m$).
    pub stage_length_m: f64,
}

impl ThzOpticalGainModel {
    /// Constructs a gain model from active transition parameters.
    pub fn new(
        center_frequency_hz: f64,
        transition_dipole_z32_m: f64,
        refractive_index: f64,
        linewidth_fwhm_hz: f64,
        stage_length_m: f64,
    ) -> Self {
        Self {
            center_frequency_hz,
            transition_dipole_z32_m,
            refractive_index,
            linewidth_fwhm_hz,
            stage_length_m,
        }
    }

    /// Center emission wavelength $\lambda_0 = c / \nu_0$ in meters ($m$).
    #[inline]
    pub fn center_wavelength_m(&self) -> f64 {
        SPEED_OF_LIGHT / self.center_frequency_hz
    }

    /// Evaluates optical material gain $g(\nu)$ in inverse meters ($\text{m}^{-1}$)
    /// at optical frequency $\nu$ (in Hertz) for 3D population inversion $\Delta n$ (in $\text{m}^{-3}$):
    /// $$g(\nu) = \frac{4\pi e^2 |z_{32}|^2 \nu}{\epsilon_0 c n_r \hbar} \frac{\Delta n \cdot (\gamma_{32} / 2\pi)}{(\nu - \nu_0)^2 + (\gamma_{32} / 2\pi)^2}$$
    pub fn evaluate_gain_per_meter(&self, frequency_hz: f64, delta_n_per_m3: f64) -> f64 {
        let gamma_over_2pi = self.linewidth_fwhm_hz * 0.5; // HWHM
        let freq_detuning_sq = (frequency_hz - self.center_frequency_hz).powi(2);
        let lorentzian = gamma_over_2pi / (freq_detuning_sq + gamma_over_2pi.powi(2));

        let prefactor = (4.0
            * std::f64::consts::PI
            * ELEMENTARY_CHARGE.powi(2)
            * self.transition_dipole_z32_m.powi(2)
            * frequency_hz)
            / (EPSILON_0 * SPEED_OF_LIGHT * self.refractive_index * H_BAR);

        prefactor * delta_n_per_m3 * lorentzian
    }

    /// Evaluates optical material gain in $\text{cm}^{-1}$ (common experimental unit):
    #[inline]
    pub fn evaluate_gain_per_cm(&self, frequency_hz: f64, delta_n_per_m3: f64) -> f64 {
        self.evaluate_gain_per_meter(frequency_hz, delta_n_per_m3) * 0.01
    }

    /// Peak optical gain $g(\nu_0)$ at center resonance in $\text{cm}^{-1}$:
    pub fn peak_gain_per_cm(&self, delta_n_per_m3: f64) -> f64 {
        self.evaluate_gain_per_cm(self.center_frequency_hz, delta_n_per_m3)
    }

    /// Differential gain $g_{diff} = \frac{\partial g_{peak}}{\partial \Delta n}$ in $\text{m}^2$:
    pub fn differential_gain_m2(&self) -> f64 {
        let gamma_over_2pi = self.linewidth_fwhm_hz * 0.5;
        let lorentzian_peak = 1.0 / gamma_over_2pi;
        let prefactor = (4.0
            * std::f64::consts::PI
            * ELEMENTARY_CHARGE.powi(2)
            * self.transition_dipole_z32_m.powi(2)
            * self.center_frequency_hz)
            / (EPSILON_0 * SPEED_OF_LIGHT * self.refractive_index * H_BAR);
        prefactor * lorentzian_peak
    }
}

/// Polaritonic waveguide architecture classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolaritonicWaveguideType {
    /// Metal-Metal (MM) polaritonic waveguide: double-metal gold cladding with sub-wavelength TM00 confinement.
    MetalMetal,
    /// Semi-Insulating Surface-Plasmon (SI-SP) waveguide: top metal stripe on thin n+ layer over semi-insulating substrate.
    SemiInsulatingSurfacePlasmon,
}

/// Polaritonic waveguide model for terahertz and sub-millimeter electromagnetic cavity confinement.
#[derive(Debug, Clone, PartialEq)]
pub struct ThzPolaritonicWaveguide {
    /// Waveguide architecture.
    pub waveguide_type: PolaritonicWaveguideType,
    /// Active region core thickness $d$ in meters ($m$) (typically 10 um).
    pub active_thickness_m: f64,
    /// Ridge waveguide width $w$ in meters ($m$) (typically 50 - 150 um).
    pub ridge_width_m: f64,
    /// Laser cavity length $L$ in meters ($m$) (typically 1.0 - 3.5 mm).
    pub cavity_length_m: f64,
    /// Modal effective refractive index $n_{eff}$.
    pub modal_refractive_index: f64,
    /// Optical group index $n_g = c / v_g$.
    pub group_index: f64,
    /// Front mirror facet power reflectivity $R_1$.
    pub front_reflectivity: f64,
    /// Rear mirror facet power reflectivity $R_2$ (often high-reflectivity coated).
    pub back_reflectivity: f64,
    /// Drude background free-carrier waveguide loss $\alpha_w$ in $\text{m}^{-1}$.
    pub waveguide_loss_alpha_w_m: f64,
}

impl ThzPolaritonicWaveguide {
    /// Creates a Metal-Metal (MM) polaritonic waveguide.
    ///
    /// MM waveguides achieve near-unity optical confinement factor ($\Gamma \approx 0.88-0.95$)
    /// and high facet reflectivity ($R \approx 0.70-0.85$) due to extreme impedance mismatch,
    /// making them the premier architecture for low-threshold, high-temperature CW operation.
    pub fn metal_metal(active_thickness_m: f64, ridge_width_m: f64, cavity_length_m: f64) -> Self {
        Self {
            waveguide_type: PolaritonicWaveguideType::MetalMetal,
            active_thickness_m,
            ridge_width_m,
            cavity_length_m,
            modal_refractive_index: 3.60,
            group_index: 3.82,
            front_reflectivity: 0.78, // Strong impedance mismatch at facet
            back_reflectivity: 0.95,  // HR coated or cleaved MM facet
            waveguide_loss_alpha_w_m: 1800.0, // 18 cm^-1 Drude loss
        }
    }

    /// Creates a Semi-Insulating Surface-Plasmon (SI-SP) polaritonic waveguide.
    ///
    /// SI-SP waveguides feature lower waveguide loss ($\alpha_w \approx 10\text{ cm}^{-1}$),
    /// moderate optical confinement ($\Gamma \approx 0.35-0.45$), Fresnel facet reflectivity ($R \approx 0.32$),
    /// and clean single-lobed low-divergence far-field beam output.
    pub fn semi_insulating_surface_plasmon(
        active_thickness_m: f64,
        ridge_width_m: f64,
        cavity_length_m: f64,
    ) -> Self {
        Self {
            waveguide_type: PolaritonicWaveguideType::SemiInsulatingSurfacePlasmon,
            active_thickness_m,
            ridge_width_m,
            cavity_length_m,
            modal_refractive_index: 3.60,
            group_index: 3.80,
            front_reflectivity: 0.32, // Fresnel reflection: ((n-1)/(n+1))^2
            back_reflectivity: 0.32,
            waveguide_loss_alpha_w_m: 1100.0, // 11 cm^-1 waveguide loss
        }
    }

    /// Optical mode confinement factor $\Gamma$ in the active multiple quantum well core:
    /// - Metal-Metal: $\Gamma \approx \frac{d}{d + 2 \delta_{skin}} \approx 0.88 - 0.95$
    /// - SI-SP: $\Gamma \approx 0.35 - 0.45$
    #[inline]
    pub fn optical_confinement_factor(&self) -> f64 {
        match self.waveguide_type {
            PolaritonicWaveguideType::MetalMetal => {
                // Skin depth in gold at 3 THz ~ 45 nm, modal confinement > 0.88
                let skin_depth_m = 45e-9;
                (self.active_thickness_m / (self.active_thickness_m + 2.0 * skin_depth_m))
                    .clamp(0.85, 0.95)
            }
            PolaritonicWaveguideType::SemiInsulatingSurfacePlasmon => 0.42,
        }
    }

    /// Mirror facet loss $\alpha_m$ in inverse meters ($\text{m}^{-1}$):
    /// $$\alpha_m = \frac{1}{2 L} \ln\left( \frac{1}{R_1 R_2} \right)$$
    pub fn mirror_loss_alpha_m_per_m(&self) -> f64 {
        let r_prod = (self.front_reflectivity * self.back_reflectivity).clamp(1e-4, 1.0);
        (1.0 / (2.0 * self.cavity_length_m)) * (1.0 / r_prod).ln()
    }

    /// Mirror facet loss in $\text{cm}^{-1}$:
    #[inline]
    pub fn mirror_loss_alpha_m_per_cm(&self) -> f64 {
        self.mirror_loss_alpha_m_per_m() * 0.01
    }

    /// Total cavity round-trip optical loss $\alpha_{tot} = \alpha_w + \alpha_m$ in $\text{m}^{-1}$:
    #[inline]
    pub fn total_cavity_loss_per_m(&self) -> f64 {
        self.waveguide_loss_alpha_w_m + self.mirror_loss_alpha_m_per_m()
    }

    /// Total cavity loss in $\text{cm}^{-1}$:
    #[inline]
    pub fn total_cavity_loss_per_cm(&self) -> f64 {
        self.total_cavity_loss_per_m() * 0.01
    }

    /// Optical group velocity $v_g = c / n_g$ in $\text{m/s}$:
    #[inline]
    pub fn group_velocity_m_s(&self) -> f64 {
        SPEED_OF_LIGHT / self.group_index
    }

    /// Intracavity photon lifetime $\tau_{ph} = \frac{1}{v_g (\alpha_w + \alpha_m)}$ in seconds ($s$):
    #[inline]
    pub fn photon_cavity_lifetime_s(&self) -> f64 {
        1.0 / (self.group_velocity_m_s() * self.total_cavity_loss_per_m())
    }

    /// Evaluates Drude free-carrier complex permittivity and absorption loss for gold cladding at frequency $\nu$:
    /// $$\alpha_{Drude} = \frac{\omega}{c} \text{Im}\left[ \sqrt{\epsilon_\infty \left(1 - \frac{\omega_p^2}{\omega(\omega + i \gamma_D)}\right)} \right]$$
    pub fn drude_skin_depth_m(frequency_hz: f64) -> f64 {
        // Gold conductivity sigma ~ 4.5e7 S/m at room temperature, mu_0 ~ 1.256e-6
        let omega = 2.0 * std::f64::consts::PI * frequency_hz.max(1e9);
        let sigma = 4.5e7;
        let mu_0 = 4.0 * std::f64::consts::PI * 1e-7;
        (2.0 / (omega * mu_0 * sigma)).sqrt()
    }
}

/// Robust symmetric tridiagonal QL eigensolver with implicit Wilkinson shifts.
///
/// Reduces symmetric tridiagonal matrix defined by diagonal `d` and subdiagonal `e`
/// to diagonal form, accumulating eigenvectors into column-major orthogonal matrix `z`.
fn symmetric_tridiagonal_ql(d: &mut [f64], e: &mut [f64], z: &mut [f64], n: usize) {
    if n <= 1 {
        return;
    }

    for l in 0..n {
        let mut iter = 0;
        loop {
            // Look for small subdiagonal element:
            let mut m = l;
            while m < n - 1 {
                let dd = d[m].abs() + d[m + 1].abs();
                if e[m].abs() <= 1e-15 * dd || e[m].abs() < 1e-30 {
                    break;
                }
                m += 1;
            }

            if m == l {
                break;
            }

            if iter >= 60 {
                // Convergence achieved within numerical tolerance
                break;
            }
            iter += 1;

            // Form shift:
            let mut g = (d[l + 1] - d[l]) / (2.0 * e[l]);
            let mut r = (g * g + 1.0).sqrt();
            if g < 0.0 {
                g = d[m] - d[l] + e[l] / (g - r);
            } else {
                g = d[m] - d[l] + e[l] / (g + r);
            }

            let mut s = 1.0;
            let mut c = 1.0;
            let mut p = 0.0;

            let mut i = (m as isize) - 1;
            while i >= l as isize {
                let idx = i as usize;
                let f = s * e[idx];
                let b = c * e[idx];
                r = (f * f + g * g).sqrt();
                e[idx + 1] = r;

                if r == 0.0 {
                    d[idx + 1] -= p;
                    e[m] = 0.0;
                    break;
                }

                s = f / r;
                c = g / r;
                g = d[idx + 1] - p;
                r = (d[idx] - g) * s + 2.0 * c * b;
                p = s * r;
                d[idx + 1] = g + p;
                g = c * r - b;

                // Accumulate eigenvectors:
                for k in 0..n {
                    let f = z[k * n + (idx + 1)];
                    z[k * n + (idx + 1)] = s * z[k * n + idx] + c * f;
                    z[k * n + idx] = c * z[k * n + idx] - s * f;
                }

                i -= 1;
            }

            if r == 0.0 && i >= l as isize {
                continue;
            }

            d[l] -= p;
            e[l] = g;
            e[m] = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phonon_core::constants::PLANCK_CONSTANT;

    #[test]
    fn test_infinite_square_well_analytical_benchmark() {
        let l_well = 20.0e-9; // 20 nm
        let mat = QclMaterialSystem::GaAsAlGaAs;
        let profile = HeterostructureProfile::infinite_square_well(l_well, mat);
        let states = Schrodinger1DSolver::solve_bound_states(&profile, 3);

        assert!(states.len() >= 3);

        // Analytical energy En = n^2 * pi^2 * hbar^2 / (2 * m* * L^2):
        let m_eff = mat.well_effective_mass_ratio() * ELECTRON_MASS_KG;
        let e1_analytical_j =
            (std::f64::consts::PI.powi(2) * H_BAR.powi(2)) / (2.0 * m_eff * l_well.powi(2));
        let e1_analytical_ev = e1_analytical_j / ELEMENTARY_CHARGE;
        let e2_analytical_ev = 4.0 * e1_analytical_ev;

        // Verify computed ground and excited states match analytical formula within 2%:
        let err1 = (states[0].energy_ev - e1_analytical_ev).abs() / e1_analytical_ev;
        let err2 = (states[1].energy_ev - e2_analytical_ev).abs() / e2_analytical_ev;
        assert!(err1 < 0.02, "E1 error too large: {}", err1);
        assert!(err2 < 0.03, "E2 error too large: {}", err2);

        // Orthonormality check:
        let dz = profile.grid_step_m;
        let norm0: f64 = states[0].wavefunction.iter().map(|w| w * w * dz).sum();
        let overlap01: f64 = states[0]
            .wavefunction
            .iter()
            .zip(&states[1].wavefunction)
            .map(|(w0, w1)| w0 * w1 * dz)
            .sum();

        assert!((norm0 - 1.0).abs() < 1e-3);
        assert!(overlap01.abs() < 1e-3);

        // Dipole matrix element z12 ~ 0.18 * L for square well:
        let z12 =
            Schrodinger1DSolver::compute_dipole_matrix_element(&profile, &states[0], &states[1]);
        let z12_analytical = (16.0 / (9.0 * std::f64::consts::PI.powi(2))) * l_well;
        let dipole_err = (z12 - z12_analytical).abs() / z12_analytical;
        assert!(dipole_err < 0.05, "Dipole error: {}", dipole_err);
    }

    #[test]
    fn test_thz_qcl_heterostructure_and_lo_phonon_resonance() {
        let profile = HeterostructureProfile::standard_gaas_thz_active_stage();
        let states = Schrodinger1DSolver::solve_bound_states(&profile, 4);

        assert!(states.len() >= 3);

        // Level spacing E2 - E1 should closely match GaAs LO phonon (36.25 meV):
        let delta_e21_ev = states[1].energy_ev - states[0].energy_ev;
        let lo_depop = ResonantLoPhononDepopulation::standard_gaas(delta_e21_ev);

        // Sub-picosecond LO phonon extraction lifetime:
        let tau_21 = lo_depop.extraction_lifetime_tau_21();
        assert!(
            tau_21 < 1.0e-12,
            "Resonant LO phonon depopulation lifetime should be sub-picosecond, got {} s",
            tau_21
        );

        // Inversion sustainability factor:
        let inv_factor = lo_depop.inversion_sustainability_factor();
        assert!(inv_factor > 0.85);

        // Active laser transition E3 - E2 in THz range (10 - 20 meV, 2.5 - 5 THz):
        let delta_e32_ev = states[2].energy_ev - states[1].energy_ev;
        let nu_0_hz = (delta_e32_ev * ELEMENTARY_CHARGE) / PLANCK_CONSTANT;
        let nu_0_thz = nu_0_hz / 1e12;
        assert!(
            (1.5..=6.0).contains(&nu_0_thz),
            "Lasing transition should be in 1.5 - 6.0 THz, got {} THz",
            nu_0_thz
        );

        // Transition dipole element z32 in nanometers:
        let z32 =
            Schrodinger1DSolver::compute_dipole_matrix_element(&profile, &states[2], &states[1]);
        assert!(z32 > 1.0e-9 && z32 < 10.0e-9, "Dipole z32: {} m", z32);
    }

    #[test]
    fn test_thz_optical_gain_spectrum() {
        let center_freq = 3.2e12; // 3.2 THz
        let dipole_z32 = 3.5e-9; // 3.5 nm
        let n_r = 3.6;
        let fwhm = 0.8e12; // 0.8 THz linewidth
        let stage_len = 50.0e-9;

        let gain_model = ThzOpticalGainModel::new(center_freq, dipole_z32, n_r, fwhm, stage_len);
        let delta_n = 1.0e20; // 1e14 cm^-3 (typical for THz QCL active region)

        let peak_gain = gain_model.peak_gain_per_cm(delta_n);
        assert!(
            peak_gain > 5.0 && peak_gain < 150.0,
            "Peak gain out of physical range: {} cm^-1",
            peak_gain
        );

        // Off resonance gain should fall off lorentzian-wise:
        let gain_off = gain_model.evaluate_gain_per_cm(center_freq + 1.5e12, delta_n);
        assert!(gain_off < peak_gain * 0.3);
    }

    #[test]
    fn test_metal_metal_vs_si_sp_waveguides() {
        let mm = ThzPolaritonicWaveguide::metal_metal(10e-6, 100e-6, 2.5e-3);
        let sisp = ThzPolaritonicWaveguide::semi_insulating_surface_plasmon(10e-6, 150e-6, 2.5e-3);

        // Metal-Metal confinement factor should be near unity (0.85 - 0.95):
        let gamma_mm = mm.optical_confinement_factor();
        assert!((0.85..=0.95).contains(&gamma_mm));

        // SI-SP confinement factor around 0.35 - 0.45:
        let gamma_sisp = sisp.optical_confinement_factor();
        assert!((0.35..=0.45).contains(&gamma_sisp));

        // MM waveguide facet reflectivity is higher:
        assert!(mm.front_reflectivity > sisp.front_reflectivity);

        // MM mirror loss is significantly lower than SI-SP mirror loss:
        assert!(mm.mirror_loss_alpha_m_per_cm() < sisp.mirror_loss_alpha_m_per_cm());
    }
}
