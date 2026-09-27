//! Microscopic physical fitness evaluator for inverse transistor design.
//!
//! Formulates:
//! - Subband quantum confinement shift $\Delta E_c(T_{ch})$ in ultra-thin channels.
//! - Direct source-to-drain Band-to-Band Tunneling (BTBT) WKB transmission and leakage floor.
//! - Electrostatic scaling length $\lambda$ and subthreshold swing $S$ (including NCFET steep slope).
//! - Quasi-ballistic velocity saturation drive current $I_{on}$.
//! - Quantum Sharvin mode limit and specific contact resistance $R_c$.
//! - Dynamic intrinsic delay $\tau = \frac{C_{gate} V_{dd}}{I_{on}}$ and Energy-Delay Product (EDP).
//! - Phonon-boundary-scattered self-heating thermal resistance and $\Delta T$.

use super::genome::{ArchitectureType, TransistorGenome};
use phonon_core::{
    BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, EPSILON_0, EPSILON_R_OX, H_BAR, PLANCK_CONSTANT,
    ROOM_TEMPERATURE_KELVIN,
};

/// Rest electron mass in kg.
const ELECTRON_MASS_KG: f64 = 9.109_383_701_5e-31;

/// Multi-objective evaluation result of a candidate transistor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FitnessEvaluation {
    /// On-state drive current in Amperes ($A$).
    pub i_on_a: f64,
    /// Off-state leakage current in Amperes ($A$).
    pub i_off_a: f64,
    /// $I_{on} / I_{off}$ switching on-off ratio.
    pub ion_ioff_ratio: f64,
    /// Subthreshold swing $S$ in $\text{mV/decade}$.
    pub subthreshold_swing_mv_per_dec: f64,
    /// Intrinsic gate switching delay $\tau = \frac{C_{gate} V_{dd}}{I_{on}}$ in picoseconds ($ps$).
    pub intrinsic_delay_ps: f64,
    /// Energy-Delay Product ($EDP$) in $\text{J}\cdot\text{s}$.
    pub energy_delay_product_js: f64,
    /// Self-heating peak junction temperature rise $\Delta T$ in Kelvin ($K$).
    pub self_heating_delta_t_k: f64,
    /// Conduction subband quantum confinement shift $\Delta E_c$ in $eV$.
    pub quantum_confinement_shift_ev: f64,
    /// Direct source-to-drain Band-to-Band Tunneling (BTBT) leakage in Amperes ($A$).
    pub btbt_leakage_a: f64,
    /// Total contact resistance $R_c$ (quantum modes + specific resistivity) in Ohms ($\Omega$).
    pub contact_resistance_ohms: f64,
    /// Effective channel conduction perimeter width $W_{eff}$ in nanometers ($nm$).
    pub effective_width_nm: f64,
    /// Total gate capacitance in Farads ($F$).
    pub gate_capacitance_f: f64,
    /// True if all physical viability constraints are satisfied.
    pub is_physically_viable: bool,
}

impl FitnessEvaluation {
    /// Evaluates the multi-objective cost vector $\mathbf{F} \in \mathbb{R}^5$
    /// where every objective is formulated to be MINIMIZED by the Pareto optimizer:
    ///
    /// 0. $-\log_{10}(I_{on} / I_{off})$ (higher switching ratio is better)
    /// 1. Subthreshold swing $S$ (lower is steeper)
    /// 2. Intrinsic delay $\tau$ in $ps$ (lower is faster)
    /// 3. Energy-Delay Product ($EDP$) scaled (lower is more efficient)
    /// 4. Self-heating temperature rise $\Delta T$ in $K$ (lower is cooler)
    #[inline]
    pub fn objectives(&self) -> [f64; 5] {
        let neg_log_ratio = if self.ion_ioff_ratio > 1.0 {
            -self.ion_ioff_ratio.log10()
        } else {
            0.0
        };

        // Scale EDP to order ~ 1.0 for balanced Pareto distance calculations
        let edp_scaled = self.energy_delay_product_js * 1.0e28;

        [
            neg_log_ratio,
            self.subthreshold_swing_mv_per_dec,
            self.intrinsic_delay_ps,
            edp_scaled,
            self.self_heating_delta_t_k,
        ]
    }
}

/// Evaluates the physical characteristics and figures of merit of a candidate transistor genome.
pub fn evaluate_transistor_fitness(genome: &TransistorGenome) -> FitnessEvaluation {
    let t_ch_m = genome.channel_thickness_nm * 1.0e-9;
    let w_ch_m = genome.channel_width_nm * 1.0e-9;
    let l_g_m = genome.gate_length_nm * 1.0e-9;
    let eot_m = genome.eot_nm * 1.0e-9;
    let vdd = genome.v_dd_volts;

    // 1. Effective conduction perimeter width
    // GAA Nanosheet / CFET has all-around gate control: perimeter = 2 * (W + T) per sheet
    let per_sheet_width_m = match genome.architecture {
        ArchitectureType::GaaNanosheet | ArchitectureType::Cfet | ArchitectureType::Ncfet => {
            2.0 * (w_ch_m + t_ch_m)
        }
        ArchitectureType::FinFet => 2.0 * t_ch_m + w_ch_m, // Tri-gate FinFET: 2 sidewalls + top
    };
    let w_eff_m = per_sheet_width_m * (genome.num_sheets as f64);
    let w_eff_nm = w_eff_m * 1.0e9;

    // 2. Quantum confinement subband shift
    let m_eff = genome.channel_material.effective_mass_ratio() * ELECTRON_MASS_KG;
    let delta_ec_joules = if genome.channel_material.is_2d_material() {
        // Monolayer TMDs have fixed 2D bandgap with no vertical thickness variation
        0.0
    } else {
        (H_BAR * H_BAR * std::f64::consts::PI * std::f64::consts::PI)
            / (2.0 * m_eff * t_ch_m * t_ch_m)
    };
    let delta_ec_ev = delta_ec_joules / ELEMENTARY_CHARGE;
    let eg_eff_ev = genome.channel_material.bandgap_ev() + delta_ec_ev;

    // 3. Oxide capacitance per unit area Cox = eps_ox / tox
    // Equivalent Oxide Thickness EOT uses SiO2 permittivity: Cox = eps_sio2 / EOT
    let eps_sio2 = EPSILON_R_OX * EPSILON_0;
    let c_ox = eps_sio2 / eot_m.max(1e-10);

    // 4. Electrostatic scaling length lambda
    let g_gate = match genome.architecture {
        ArchitectureType::GaaNanosheet | ArchitectureType::Cfet | ArchitectureType::Ncfet => 8.0,
        ArchitectureType::FinFet => 3.5,
    };
    let eps_ch = genome.channel_material.relative_permittivity() * EPSILON_0;
    let lambda_m = ((eps_ch / (g_gate * eps_sio2)) * t_ch_m * eot_m)
        .sqrt()
        .max(1e-10);

    // 5. Subthreshold Swing S
    // Baseline thermal swing: ln(10) * Vt
    let vt = (BOLTZMANN_CONSTANT * ROOM_TEMPERATURE_KELVIN) / ELEMENTARY_CHARGE;
    let s_ideal_v = std::f64::consts::LN_10 * vt; // ~0.0596 V/dec = 59.6 mV/dec
    let s_degradation = 1.0 + (eot_m / lambda_m) * (-l_g_m / (2.0 * lambda_m)).exp();
    let mut s_mv_per_dec = s_ideal_v * s_degradation * 1000.0;

    // NCFET steep slope amplification from ferroelectric negative capacitance
    if genome.architecture == ArchitectureType::Ncfet {
        // Typical NCFET achieves 10-25% steepening below the Boltzmann limit
        s_mv_per_dec *= 0.82;
    }

    // 6. Direct Source-to-Drain Band-to-Band Tunneling (BTBT) Leakage
    // Landauer-Büttiker transmission with WKB barrier penetration:
    // I_btbt = (2 * q^2 / h) * M_modes * V_dd * T_wkb
    let alpha_wkb = (4.0 * (2.0 * m_eff * ELEMENTARY_CHARGE).sqrt()) / (3.0 * H_BAR);
    let exponent = -alpha_wkb * eg_eff_ev.sqrt() * l_g_m;
    let t_wkb = exponent.exp().clamp(1e-30, 1.0);
    let g_quantum = 2.0 * ELEMENTARY_CHARGE.powi(2) / PLANCK_CONSTANT; // ~77.48 uS quantum conductance
    let k_fermi_tunnel = 1.0e9; // ~1 nm^-1 transverse wavevector
    let num_modes_tunnel = (k_fermi_tunnel * w_eff_m / std::f64::consts::PI)
        .floor()
        .max(1.0);
    let i_btbt = g_quantum * num_modes_tunnel * vdd * t_wkb;

    // 7. Threshold Voltage Vth
    // Workfunction difference + quantum confinement shift + DIBL
    let phi_m = genome.workfunction_ev;
    let chi_ch = 4.05; // Electron affinity of Si baseline (~4.05 eV)
    let v_bi = phi_m - chi_ch - (eg_eff_ev * 0.5);
    let dibl_shift = 0.080 * (lambda_m / l_g_m).powi(2) * vdd;
    let v_th = (v_bi + delta_ec_ev * 0.5 - dibl_shift).clamp(0.10, 0.60);

    // 8. Off-state leakage current Ioff
    let i_thermal_off = 1.0e-7 * w_eff_m * 10.0_f64.powf(-v_th / (s_mv_per_dec * 1e-3));
    let i_gate_leakage = 1.0e-11 * w_eff_m * (l_g_m / 10e-9); // 10 pA/um baseline gate tunneling
    let i_off_total = i_thermal_off + i_btbt + i_gate_leakage;

    // 9. Quantum Contact Resistance Limit
    // Sharvin modes M = Int[2 * W / lambda_Fermi]
    let k_fermi = 1.0e9; // ~1 nm^-1 Fermi wavevector in heavily doped S/D
    let num_modes = (k_fermi * w_eff_m / std::f64::consts::PI).floor().max(1.0);
    let r_quantum = PLANCK_CONSTANT / (2.0 * ELEMENTARY_CHARGE.powi(2) * num_modes);
    let contact_area_cm2 = (w_ch_m * 12e-9 * (genome.num_sheets as f64)) * 1.0e4;
    let r_specific =
        genome.contact_species.specific_resistivity_ohm_cm2() / contact_area_cm2.max(1e-14);
    let r_contact_total = r_quantum + r_specific;

    // 10. Quasi-ballistic On-state Drive Current Ion
    let lambda_mfp_m = 10.0e-9; // 10 nm mean free path at 300 K
    let ballisticity = lambda_mfp_m / (lambda_mfp_m + l_g_m);
    let v_inj = (2.0 * BOLTZMANN_CONSTANT * ROOM_TEMPERATURE_KELVIN
        / (std::f64::consts::PI * m_eff))
        .sqrt();
    let v_overdrive = (vdd - v_th).max(0.01);
    let i_on_intrinsic = w_eff_m * c_ox * v_overdrive * v_inj * ballisticity;

    // Contact resistance degradation: Ion_eff = Ion / (1 + 2 * Rc * Ion / Vdd)
    let contact_drop_factor = 1.0 + 2.0 * r_contact_total * (i_on_intrinsic / vdd);
    let i_on_eff = i_on_intrinsic / contact_drop_factor.max(1.0);

    let ion_ioff_ratio = if i_off_total > 1e-25 {
        i_on_eff / i_off_total
    } else {
        1e12
    };

    // 11. Gate Capacitance & Intrinsic Switching Delay
    let c_gate = c_ox * l_g_m * w_eff_m + (0.3e-10 * w_eff_m); // Fringe capacitance ~0.3 pF/cm
    let intrinsic_delay_s = (c_gate * vdd) / i_on_eff.max(1e-9);
    let intrinsic_delay_ps = intrinsic_delay_s * 1.0e12;

    // 12. Energy-Delay Product (EDP) = (C * Vdd^2) * tau
    let dynamic_energy_j = c_gate * vdd * vdd;
    let edp_js = dynamic_energy_j * intrinsic_delay_s;

    // 13. Self-Heating & Phonon Boundary Scattering
    // Thermal conductivity of nanometer silicon is reduced by boundary scattering:
    // kappa_nano = kappa_bulk / (1 + lambda_phonon / T_ch)
    let lambda_phonon_m = 40.0e-9; // 40 nm acoustic phonon mean free path in Si
    let kappa_bulk = 148.0; // W / (m * K)
    let kappa_nano = kappa_bulk / (1.0 + lambda_phonon_m / t_ch_m);
    let a_cross_m2 = (w_ch_m * t_ch_m * (genome.num_sheets as f64)).max(1e-18);
    // Heat conducts from channel midpoint to both source and drain in parallel (factor of 4)
    let r_ch_direct = l_g_m / (4.0 * kappa_nano * a_cross_m2);
    // Gate dielectric and metal gate stack act as a parallel heat sinking path
    let r_gate_stack = 4.0e5; // Equivalent thermal resistance to gate metal heatsink
    let r_ch_eff = (r_ch_direct * r_gate_stack) / (r_ch_direct + r_gate_stack);
    let r_th = r_ch_eff + 40_000.0; // Substrate spreading resistance in K/W
    let p_diss = i_on_eff * vdd;
    let delta_t_k = (p_diss * r_th).clamp(0.0, 150.0);

    // 14. Physical Viability Checks
    let is_physically_viable = i_on_eff > 1e-6
        && i_off_total < 1e-4
        && ion_ioff_ratio > 100.0
        && s_mv_per_dec < 120.0
        && delta_t_k < 120.0;

    FitnessEvaluation {
        i_on_a: i_on_eff,
        i_off_a: i_off_total,
        ion_ioff_ratio,
        subthreshold_swing_mv_per_dec: s_mv_per_dec,
        intrinsic_delay_ps,
        energy_delay_product_js: edp_js,
        self_heating_delta_t_k: delta_t_k,
        quantum_confinement_shift_ev: delta_ec_ev,
        btbt_leakage_a: i_btbt,
        contact_resistance_ohms: r_contact_total,
        effective_width_nm: w_eff_nm,
        gate_capacitance_f: c_gate,
        is_physically_viable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaa_nanosheet_fitness_evaluation() {
        let genome = TransistorGenome::n2_gaa_nanosheet_preset();
        let fit = evaluate_transistor_fitness(&genome);

        assert!(fit.is_physically_viable);
        assert!(fit.i_on_a > 1.0e-4); // > 100 uA
        assert!(fit.i_off_a < 1.0e-7); // < 100 nA
        assert!(fit.ion_ioff_ratio > 1.0e4); // > 10,000 on/off
        assert!(fit.subthreshold_swing_mv_per_dec < 75.0); // < 75 mV/dec
        assert!(fit.intrinsic_delay_ps < 1.0); // < 1 ps delay
        assert!(fit.self_heating_delta_t_k > 0.0 && fit.self_heating_delta_t_k < 60.0);
    }

    #[test]
    fn test_quantum_confinement_shift_increases_with_thinning() {
        let mut thick = TransistorGenome::n2_gaa_nanosheet_preset();
        thick.channel_thickness_nm = 5.0;

        let mut thin = TransistorGenome::n2_gaa_nanosheet_preset();
        thin.channel_thickness_nm = 2.0;

        let fit_thick = evaluate_transistor_fitness(&thick);
        let fit_thin = evaluate_transistor_fitness(&thin);

        // Delta Ec is inversely proportional to Tch^2 -> thinner must have higher confinement
        assert!(fit_thin.quantum_confinement_shift_ev > fit_thick.quantum_confinement_shift_ev);
        assert!(fit_thin.quantum_confinement_shift_ev > 0.05); // > 50 meV
    }

    #[test]
    fn test_ncfet_steep_subthreshold_swing() {
        let mut ncfet_genome = TransistorGenome::n2_gaa_nanosheet_preset();
        ncfet_genome.architecture = ArchitectureType::Ncfet;

        let baseline_gaa = TransistorGenome::n2_gaa_nanosheet_preset();

        let fit_ncfet = evaluate_transistor_fitness(&ncfet_genome);
        let fit_gaa = evaluate_transistor_fitness(&baseline_gaa);

        // NCFET must exhibit steeper subthreshold swing than baseline GAA
        assert!(fit_ncfet.subthreshold_swing_mv_per_dec < fit_gaa.subthreshold_swing_mv_per_dec);
        assert!(fit_ncfet.subthreshold_swing_mv_per_dec < 65.0);
    }

    #[test]
    fn test_btbt_leakage_increases_at_ultra_short_gate_length() {
        let mut long_gate = TransistorGenome::n2_gaa_nanosheet_preset();
        long_gate.gate_length_nm = 18.0;

        let mut short_gate = TransistorGenome::n2_gaa_nanosheet_preset();
        short_gate.gate_length_nm = 5.0;

        let fit_long = evaluate_transistor_fitness(&long_gate);
        let fit_short = evaluate_transistor_fitness(&short_gate);

        // BTBT leakage must surge at 5nm due to exponential barrier thinning
        assert!(fit_short.btbt_leakage_a > fit_long.btbt_leakage_a * 100.0);
    }
}
