#![deny(unsafe_code)]

//! Cavity quantum acoustodynamical (cQAD) spin-phonon interface solver and chiral squeezed vacuum model.

use phonon_models::cavity_acoustodynamical_spin::{
    CavityAcoustodynamicalSpinMetrics, CavityAcoustodynamicalSpinParams,
};

/// Solver evaluating acoustic quadrature squeezing, spin-phonon quantum state transfer fidelity,
/// spin coherence lifetime, thermal phonon occupancy, and Purcell enhancement factor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CavityAcoustodynamicalSpinSolver {
    pub params: CavityAcoustodynamicalSpinParams,
}

impl CavityAcoustodynamicalSpinSolver {
    /// Creates a new solver instance with the specified physical parameters.
    pub fn new(params: CavityAcoustodynamicalSpinParams) -> Self {
        Self { params }
    }

    /// Computes the acoustic quadrature squeezing below vacuum level in dB (target >= 12.0).
    pub fn compute_acoustic_quadrature_squeezing_db(&self) -> f64 {
        let p = &self.params;
        let p_ratio = p.pump_power_mw / 4.5;
        let g_ratio = p.non_linear_gain_db / 16.5;
        let decay_ratio = p.cavity_decay_rate_khz / 85.0;
        let temp_ratio = p.cryogenic_temp_mk / 20.0;

        let squeezing = 14.5 + 3.2 * p_ratio.ln() + 2.5 * (g_ratio - 1.0)
            - 2.0 * (decay_ratio - 1.0)
            - 1.5 * (temp_ratio - 1.0);
        squeezing.clamp(6.0, 30.0)
    }

    /// Computes the coherent spin-phonon quantum state transfer fidelity (target >= 0.9970).
    pub fn compute_spin_phonon_fidelity(&self) -> f64 {
        let p = &self.params;
        let g_ratio = p.spin_phonon_coupling_mhz / 5.8;
        let iso_ratio = p.chiral_isolation_db / 38.0;
        let deph_ratio = p.spin_dephasing_rate_hz / 12.0;
        let temp_ratio = p.cryogenic_temp_mk / 20.0;
        let decay_ratio = p.cavity_decay_rate_khz / 85.0;

        let fidelity = 0.9984 + 0.0006 * (g_ratio - 1.0) + 0.0003 * (iso_ratio - 1.0)
            - 0.0007 * (deph_ratio - 1.0)
            - 0.0004 * (temp_ratio - 1.0)
            - 0.0003 * (decay_ratio - 1.0);
        fidelity.clamp(0.980, 0.9999)
    }

    /// Computes the spin defect coherence lifetime T2 in milliseconds (target >= 50.0).
    pub fn compute_spin_coherence_lifetime_ms(&self) -> f64 {
        let p = &self.params;
        let deph_factor = 12.0 / p.spin_dephasing_rate_hz.max(1.0e-6);
        let temp_factor = (20.0 / p.cryogenic_temp_mk.max(1.0e-6)).sqrt();
        let iso_factor = (p.chiral_isolation_db / 38.0).powf(0.25);

        let lifetime = 68.0 * deph_factor * temp_factor * iso_factor;
        lifetime.clamp(10.0, 500.0)
    }

    /// Computes the thermal equilibrium phonon occupancy n_th in quanta (target <= 0.05).
    pub fn compute_thermal_phonon_occupancy(&self) -> f64 {
        let p = &self.params;
        // Effective cryogenic temperature accounting for acoustic pump dissipation
        let temp_eff_k = ((p.cryogenic_temp_mk + 0.2 * p.pump_power_mw) * 1.0e-3).max(1.0e-6);
        let freq_hz = p.acoustic_frequency_ghz * 1.0e9;

        let h_planck = 6.626_070_15e-34;
        let k_boltzmann = 1.380_649e-23;
        let x = (h_planck * freq_hz) / (k_boltzmann * temp_eff_k);

        let occupancy = if x > 50.0 {
            (-x).exp()
        } else if x < 1.0e-5 {
            1.0 / x
        } else {
            1.0 / (x.exp() - 1.0)
        };
        occupancy.clamp(0.0, 10.0)
    }

    /// Computes the spin-cavity acoustic Purcell enhancement factor F_P (target >= 25.0).
    pub fn compute_purcell_enhancement_factor(&self) -> f64 {
        let p = &self.params;
        let g_mhz = p.spin_phonon_coupling_mhz;
        let decay_mhz = (p.cavity_decay_rate_khz / 1000.0).max(1.0e-6);
        let deph_scaling = 1.0 + 0.05 * (p.spin_dephasing_rate_hz / 12.0);
        let gamma_free_eff_mhz = 40.0 * deph_scaling;

        let fp = (4.0 * g_mhz * g_mhz) / (decay_mhz * gamma_free_eff_mhz);
        fp.clamp(1.0, 250.0)
    }

    /// Evaluates complete multi-physics metrics and physical compliance status.
    pub fn evaluate_metrics(&self) -> CavityAcoustodynamicalSpinMetrics {
        let acoustic_quadrature_squeezing_db = self.compute_acoustic_quadrature_squeezing_db();
        let spin_phonon_fidelity = self.compute_spin_phonon_fidelity();
        let spin_coherence_lifetime_ms = self.compute_spin_coherence_lifetime_ms();
        let thermal_phonon_occupancy = self.compute_thermal_phonon_occupancy();
        let purcell_enhancement_factor = self.compute_purcell_enhancement_factor();

        let is_physically_compliant = acoustic_quadrature_squeezing_db >= 12.0
            && spin_phonon_fidelity >= 0.9970
            && spin_coherence_lifetime_ms >= 50.0
            && thermal_phonon_occupancy <= 0.05
            && purcell_enhancement_factor >= 25.0;

        CavityAcoustodynamicalSpinMetrics {
            acoustic_quadrature_squeezing_db,
            spin_phonon_fidelity,
            spin_coherence_lifetime_ms,
            thermal_phonon_occupancy,
            purcell_enhancement_factor,
            is_physically_compliant,
        }
    }
}
