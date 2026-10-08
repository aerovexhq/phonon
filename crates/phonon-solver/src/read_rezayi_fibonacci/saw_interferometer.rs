#![deny(unsafe_code)]

//! Phase 447: Surface Acoustic Wave (SAW) Fibonacci Anyon Fabry-Pérot Interferometer.
//!
//! Models acoustic beam-splitting via piezoelectrically driven QPCs, Aharonov-Bohm
//! flux oscillations, and the signature 1/phi non-Abelian visibility suppression
//! when an odd number of Fibonacci anyons are enclosed in the interferometric loop.

/// Topological charge state trapped within the SAW interferometric cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnclosedTopologicalCharge {
    /// Vacuum state (0 quasiparticles).
    Vacuum1,
    /// Single non-Abelian Fibonacci anyon (tau).
    SingleTau,
    /// Pair of Fibonacci anyons (tau x tau = 1 + tau).
    TwoTau,
}

impl EnclosedTopologicalCharge {
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Vacuum1 => "Vacuum (1)",
            Self::SingleTau => "Single Fibonacci Anyon (tau)",
            Self::TwoTau => "Two Fibonacci Anyons (tau x tau)",
        }
    }
}

/// Simulation parameters for the SAW Fabry-Pérot interferometer.
#[derive(Debug, Clone)]
pub struct SawInterferometerParams {
    pub acoustic_frequency_ghz: f64,
    pub saw_velocity_ms: f64,
    pub qpc1_transmission: f64,
    pub qpc2_transmission: f64,
    pub loop_area_um2: f64,
    pub magnetic_field_t: f64,
    pub enclosed_charge: EnclosedTopologicalCharge,
    pub temperature_mk: f64,
}

impl Default for SawInterferometerParams {
    fn default() -> Self {
        Self {
            acoustic_frequency_ghz: 2.85,
            saw_velocity_ms: 2865.0, // Rayleigh SAW velocity in GaAs / LiNbO3
            qpc1_transmission: 0.50,
            qpc2_transmission: 0.50,
            loop_area_um2: 4.20,
            magnetic_field_t: 5.80,
            enclosed_charge: EnclosedTopologicalCharge::SingleTau,
            temperature_mk: 16.0,
        }
    }
}

/// Evaluated metrics for the SAW anyon interferometer.
#[derive(Debug, Clone)]
pub struct SawInterferometerMetrics {
    pub vacuum_visibility_percent: f64,
    pub active_visibility_percent: f64,
    pub visibility_suppression_ratio: f64,
    pub target_golden_ratio_inverse: f64,
    pub longitudinal_resistance_peak_kohm: f64,
    pub saw_transduction_efficiency_percent: f64,
    pub aharonov_bohm_period_mt: f64,
    pub dephasing_length_um: f64,
}

/// Point on the longitudinal resistance R_xx vs magnetic flux curve.
#[derive(Debug, Clone)]
pub struct InterferometerFluxPoint {
    pub flux_over_phi0: f64,
    pub resistance_rxx_kohm: f64,
    pub vacuum_envelope_kohm: f64,
}

/// Point on the acoustic SAW transmission frequency sweep.
#[derive(Debug, Clone)]
pub struct SawAcousticTransmissionPoint {
    pub frequency_ghz: f64,
    pub transmission_s21_db: f64,
    pub acoustic_phase_rad: f64,
}

/// Numerical solver for the SAW Fibonacci interferometer.
#[derive(Debug, Clone)]
pub struct SawInterferometerSolver {
    pub params: SawInterferometerParams,
}

impl SawInterferometerSolver {
    pub fn new(params: SawInterferometerParams) -> Self {
        Self { params }
    }

    /// Evaluates interferometer metrics and non-Abelian visibility reduction.
    pub fn evaluate_metrics(&self) -> SawInterferometerMetrics {
        let p = &self.params;
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
        let inv_phi = 1.0 / phi; // ~ 0.618034

        // Base vacuum interference visibility V_0 depends on QPC transmission match
        let t1 = p.qpc1_transmission;
        let t2 = p.qpc2_transmission;
        let r1 = 1.0 - t1;
        let r2 = 1.0 - t2;
        let ideal_vis = 2.0 * (t1 * r1 * t2 * r2).sqrt() / (t1 * r2 + t2 * r1 + 1e-9);
        let thermal_factor = (-p.temperature_mk / 140.0).exp();
        let vacuum_vis = (ideal_vis * thermal_factor * 94.0).min(96.0);

        // Under an enclosed tau anyon, non-Abelian quantum trace suppresses visibility by 1/d_tau = 1/phi
        let (active_vis, suppression) = match p.enclosed_charge {
            EnclosedTopologicalCharge::Vacuum1 => (vacuum_vis, 1.0),
            EnclosedTopologicalCharge::SingleTau => {
                let s = inv_phi;
                (vacuum_vis * s, s)
            }
            EnclosedTopologicalCharge::TwoTau => {
                // Fusion tau x tau = 1 + tau gives statistical mixture: (1*1 + phi*(1/phi)) / (1 + phi^2) = 2 / (1 + phi^2)
                let s = 2.0 / (1.0 + phi * phi);
                (vacuum_vis * s, s)
            }
        };

        // Quasiparticle flux quantum: phi_0* = h / (e / 5) = 5 * h/e ~ 5 * 4.1357e-15 T*m^2
        let flux_quantum_wb = 5.0 * 2.067833848e-15;
        let area_m2 = p.loop_area_um2 * 1.0e-12;
        let delta_b_t = flux_quantum_wb / area_m2;
        let delta_b_mt = delta_b_t * 1.0e3;

        let r_peak = 12.90 + 3.2 * (active_vis / 100.0);
        let saw_efficiency = 91.5 * (p.saw_velocity_ms / 2865.0);
        let l_phi = 28.5 * (15.0 / p.temperature_mk.max(1.0)).sqrt();

        SawInterferometerMetrics {
            vacuum_visibility_percent: vacuum_vis,
            active_visibility_percent: active_vis,
            visibility_suppression_ratio: suppression,
            target_golden_ratio_inverse: inv_phi,
            longitudinal_resistance_peak_kohm: r_peak,
            saw_transduction_efficiency_percent: saw_efficiency.min(99.0),
            aharonov_bohm_period_mt: delta_b_mt,
            dephasing_length_um: l_phi,
        }
    }

    /// Computes longitudinal resistance R_xx vs magnetic flux sweeps.
    pub fn compute_flux_oscillations(&self, points_count: usize) -> Vec<InterferometerFluxPoint> {
        let metrics = self.evaluate_metrics();
        let mut result = Vec::with_capacity(points_count);

        let max_flux = 3.5; // multiples of phi_0*
        let step = if points_count > 1 { max_flux / (points_count - 1) as f64 } else { 0.2 };

        let r_base = 12.90;
        let delta_r_vac = 3.20 * (metrics.vacuum_visibility_percent / 100.0);
        let delta_r_act = 3.20 * (metrics.active_visibility_percent / 100.0);

        for i in 0..points_count {
            let flux = i as f64 * step;
            let phase = 2.0 * std::f64::consts::PI * flux;

            let r_xx = r_base + delta_r_act * phase.cos();
            let r_vac = r_base + delta_r_vac * phase.cos();

            result.push(InterferometerFluxPoint {
                flux_over_phi0: flux,
                resistance_rxx_kohm: r_xx,
                vacuum_envelope_kohm: r_vac,
            });
        }

        result
    }

    /// Computes acoustic transmission S_21 across the IDT transducers.
    pub fn compute_acoustic_spectrum(&self, points_count: usize) -> Vec<SawAcousticTransmissionPoint> {
        let f0 = self.params.acoustic_frequency_ghz;
        let mut result = Vec::with_capacity(points_count);

        let span_ghz = 0.60;
        let f_min = f0 - span_ghz / 2.0;
        let step = if points_count > 1 { span_ghz / (points_count - 1) as f64 } else { 0.05 };

        for i in 0..points_count {
            let f = f_min + i as f64 * step;
            let detuning = (f - f0) / 0.05; // 50 MHz bandwidth
            let sinc_arg = std::f64::consts::PI * detuning;
            let sinc = if sinc_arg.abs() < 1e-6 { 1.0 } else { sinc_arg.sin() / sinc_arg };

            let transmission_mag = sinc.abs() * 0.92;
            let s21_db = 20.0 * (transmission_mag.max(1e-4)).log10();
            let phase = -detuning * 1.8;

            result.push(SawAcousticTransmissionPoint {
                frequency_ghz: f,
                transmission_s21_db: s21_db,
                acoustic_phase_rad: phase,
            });
        }

        result
    }
}
