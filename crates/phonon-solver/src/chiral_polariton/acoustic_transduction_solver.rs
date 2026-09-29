//! Acoustic Spin Pumping & Non-Reciprocal Transduction Solver.
//!
//! Solves coupled elastodynamic-micromagnetic acoustic wave propagation,
//! dynamic acoustic spin pumping, transverse ISHE voltage generation,
//! and non-reciprocal acoustic isolation > 20 dB.

use phonon_models::chiral_polariton::{AcousticSpinPumpingInterface, MagnetoElasticMedium};

/// Results from acoustic transduction and spin pumping simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticTransductionResult {
    /// Excitation acoustic frequency in rad/s.
    pub excitation_freq_rad_per_s: f64,
    /// Magnetization precession cone angle in radians.
    pub precession_angle_rad: f64,
    /// Pumped spin current density across interface in J/m^2.
    pub spin_current_density_j_per_m2: f64,
    /// Transverse Inverse Spin Hall Effect DC voltage in Volts.
    pub ishe_dc_voltage_volts: f64,
    /// Forward acoustic transmission magnitude |S_21|.
    pub forward_transmission_s21: f64,
    /// Reverse acoustic transmission magnitude |S_12|.
    pub reverse_transmission_s12: f64,
    /// Non-reciprocal acoustic diode isolation in dB.
    pub non_reciprocal_isolation_db: f64,
}

/// Transduction solver for acoustic spin pumping and non-reciprocal acoustics.
#[derive(Debug, Default, Clone)]
pub struct AcousticTransductionSolver;

impl AcousticTransductionSolver {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates acoustic spin pumping and transmission S-parameters.
    pub fn solve_transduction(
        &self,
        medium: &MagnetoElasticMedium,
        interface: &AcousticSpinPumpingInterface,
        strain_amplitude: f64,
        frequency_rad_per_s: f64,
    ) -> AcousticTransductionResult {
        let theta_prec = interface.magnetization_precession_angle_rad(
            medium,
            strain_amplitude,
            frequency_rad_per_s,
        );

        let j_s = interface.pumped_spin_current_density_j_per_m2(theta_prec, frequency_rad_per_s);

        let v_ishe = interface.ishe_dc_voltage_volts(j_s);

        let vt = medium.transverse_sound_velocity_m_per_s();
        let k = frequency_rad_per_s / vt;
        let isolation_db = interface.non_reciprocal_isolation_db(medium, k);

        // Reverse mode is unattenuated; forward mode is attenuated by isolation factor
        let s12 = 0.98; // Low loss in uncoupled reverse direction
        let atten_linear = 10.0f64.powf(-isolation_db / 20.0);
        let s21 = s12 * atten_linear;

        AcousticTransductionResult {
            excitation_freq_rad_per_s: frequency_rad_per_s,
            precession_angle_rad: theta_prec,
            spin_current_density_j_per_m2: j_s,
            ishe_dc_voltage_volts: v_ishe,
            forward_transmission_s21: s21,
            reverse_transmission_s12: s12,
            non_reciprocal_isolation_db: isolation_db,
        }
    }
}
