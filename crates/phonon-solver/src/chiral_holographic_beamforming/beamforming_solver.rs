#![deny(unsafe_code)]

//! Quantum acoustic metasurface holography and chiral phonon beamforming solver.

use phonon_models::chiral_holographic_beamforming::{
    ChiralHolographicBeamformingMetrics, ChiralHolographicBeamformingParams,
};

/// Multi-physics solver evaluating holographic reconstruction fidelity, acoustic beam directivity,
/// beam steering angular resolution, side-lobe suppression ratio, and acoustic mode insertion loss.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralHolographicBeamformingSolver {
    pub params: ChiralHolographicBeamformingParams,
}

impl ChiralHolographicBeamformingSolver {
    /// Creates a new solver instance with the specified physical parameters.
    pub fn new(params: ChiralHolographicBeamformingParams) -> Self {
        Self { params }
    }

    /// Computes holographic target wavefront reconstruction fidelity (target >= 0.9960).
    pub fn compute_holographic_reconstruction_fidelity(&self) -> f64 {
        let p = &self.params;
        let elem_ratio = (p.metasurface_elements_count as f64) / 48.0;
        let phase_grad_ratio = p.synthetic_gauge_phase_gradient_rad_per_um / 3.2;
        let piezo_ratio = p.piezoelectric_coupling_efficiency / 0.91;
        let focus_ratio = p.sub_diffraction_focusing_ratio / 2.10;
        let chiral_ratio = p.chiral_isolation_db / 36.0;
        let temp_ratio = p.cryogenic_temperature_mk / 20.0;
        let freq_ratio = p.operating_frequency_ghz / 3.8;

        let fidelity = 0.9976
            + 0.0006 * (elem_ratio - 1.0)
            + 0.0004 * (phase_grad_ratio - 1.0)
            + 0.0005 * (piezo_ratio - 1.0)
            + 0.0003 * (focus_ratio - 1.0)
            + 0.0004 * (chiral_ratio - 1.0)
            - 0.0003 * (temp_ratio - 1.0)
            - 0.0002 * (freq_ratio - 1.0);
        fidelity.clamp(0.980, 0.9999)
    }

    /// Computes main acoustic beam directivity in dB (target >= 32.0).
    pub fn compute_acoustic_beam_directivity_db(&self) -> f64 {
        let p = &self.params;
        let elem_ratio = (p.metasurface_elements_count as f64) / 48.0;
        let spacing_ratio = p.element_spacing_um / 0.85;
        let freq_ratio = p.operating_frequency_ghz / 3.8;
        let focus_ratio = p.sub_diffraction_focusing_ratio / 2.10;
        let piezo_ratio = p.piezoelectric_coupling_efficiency / 0.91;
        let chiral_ratio = p.chiral_isolation_db / 36.0;
        let temp_ratio = p.cryogenic_temperature_mk / 20.0;

        let directivity = 34.8
            + 6.0 * (elem_ratio - 1.0)
            + 3.5 * (spacing_ratio - 1.0)
            + 3.0 * (freq_ratio - 1.0)
            + 2.2 * (focus_ratio - 1.0)
            + 1.5 * (piezo_ratio - 1.0)
            + 1.0 * (chiral_ratio - 1.0)
            - 0.5 * (temp_ratio - 1.0);
        directivity.clamp(15.0, 60.0)
    }

    /// Computes beam steering angular resolution in degrees (target <= 0.050).
    pub fn compute_beam_steering_angular_resolution_deg(&self) -> f64 {
        let p = &self.params;
        let base_res = 0.038;
        let elem_factor = (48.0 / (p.metasurface_elements_count as f64)).powf(0.8);
        let spacing_factor = (0.85 / p.element_spacing_um).powf(0.5);
        let freq_factor = (3.8 / p.operating_frequency_ghz).powf(0.4);
        let phase_grad_factor = (3.2 / p.synthetic_gauge_phase_gradient_rad_per_um).powf(0.2);
        let focus_factor = (2.10 / p.sub_diffraction_focusing_ratio).powf(0.25);
        let piezo_factor = (0.91 / p.piezoelectric_coupling_efficiency).powf(0.15);
        let chiral_factor = (36.0 / p.chiral_isolation_db).powf(0.1);
        let temp_factor = (p.cryogenic_temperature_mk / 20.0).powf(0.15);

        let res = base_res
            * elem_factor
            * spacing_factor
            * freq_factor
            * phase_grad_factor
            * focus_factor
            * piezo_factor
            * chiral_factor
            * temp_factor;
        res.clamp(0.005, 0.200)
    }

    /// Computes side-lobe suppression ratio in dB (target >= 28.0).
    pub fn compute_side_lobe_suppression_ratio_db(&self) -> f64 {
        let p = &self.params;
        let chiral_ratio = p.chiral_isolation_db / 36.0;
        let elem_ratio = (p.metasurface_elements_count as f64) / 48.0;
        let phase_grad_ratio = p.synthetic_gauge_phase_gradient_rad_per_um / 3.2;
        let focus_ratio = p.sub_diffraction_focusing_ratio / 2.10;
        let piezo_ratio = p.piezoelectric_coupling_efficiency / 0.91;
        let temp_ratio = p.cryogenic_temperature_mk / 20.0;
        let spacing_deviation = ((p.element_spacing_um - 0.85) / 0.85).abs();

        let slsr = 31.8
            + 3.5 * (chiral_ratio - 1.0)
            + 3.0 * (elem_ratio - 1.0)
            + 2.0 * (phase_grad_ratio - 1.0)
            + 1.8 * (focus_ratio - 1.0)
            + 1.2 * (piezo_ratio - 1.0)
            - 1.0 * (temp_ratio - 1.0)
            - 2.0 * spacing_deviation;
        slsr.clamp(10.0, 60.0)
    }

    /// Computes acoustic mode insertion loss in dB (target <= 1.20).
    pub fn compute_acoustic_mode_insertion_loss_db(&self) -> f64 {
        let p = &self.params;
        let base_loss = 0.82;
        let piezo_term = 1.4 * (1.0 - p.piezoelectric_coupling_efficiency / 0.91);
        let freq_term = 0.15 * (p.operating_frequency_ghz / 3.8 - 1.0);
        let temp_term = 0.12 * (p.cryogenic_temperature_mk / 20.0 - 1.0);
        let elem_term = 0.08 * ((p.metasurface_elements_count as f64) / 48.0 - 1.0);
        let chiral_term = -0.10 * (p.chiral_isolation_db / 36.0 - 1.0);
        let focus_term = -0.05 * (p.sub_diffraction_focusing_ratio / 2.10 - 1.0);

        let loss = base_loss
            + piezo_term
            + freq_term
            + temp_term
            + elem_term
            + chiral_term
            + focus_term;
        loss.clamp(0.20, 3.50)
    }

    /// Evaluates complete multi-physics metrics and physical compliance status.
    pub fn evaluate_metrics(&self) -> ChiralHolographicBeamformingMetrics {
        let holographic_reconstruction_fidelity =
            self.compute_holographic_reconstruction_fidelity();
        let acoustic_beam_directivity_db = self.compute_acoustic_beam_directivity_db();
        let beam_steering_angular_resolution_deg =
            self.compute_beam_steering_angular_resolution_deg();
        let side_lobe_suppression_ratio_db = self.compute_side_lobe_suppression_ratio_db();
        let acoustic_mode_insertion_loss_db = self.compute_acoustic_mode_insertion_loss_db();

        let is_physically_compliant = holographic_reconstruction_fidelity >= 0.9960
            && acoustic_beam_directivity_db >= 32.0
            && beam_steering_angular_resolution_deg <= 0.050
            && side_lobe_suppression_ratio_db >= 28.0
            && acoustic_mode_insertion_loss_db <= 1.20;

        ChiralHolographicBeamformingMetrics {
            holographic_reconstruction_fidelity,
            acoustic_beam_directivity_db,
            beam_steering_angular_resolution_deg,
            side_lobe_suppression_ratio_db,
            acoustic_mode_insertion_loss_db,
            is_physically_compliant,
        }
    }
}
