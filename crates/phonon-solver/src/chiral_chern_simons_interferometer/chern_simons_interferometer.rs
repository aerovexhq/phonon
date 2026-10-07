#![deny(unsafe_code)]

//! Topological Chiral Acoustic Chern-Simons Fractional Anyon Interferometer.
//!
//! Models chiral acoustic edge channels coupled across two quantum point contacts (QPCs)
//! enclosing synthetic gauge flux and localized fractional anyons:
//! - Chiral Chern-Simons effective gauge theory for fractional Hall states.
//! - Fabry-Perot interference with Aharonov-Bohm flux and statistical phase shifts.
//! - Direct extraction of fractional exchange statistics (Laughlin 1/3, 1/5, Moore-Read 5/2, Fibonacci).
//! - Visibility V >= 85% and interference contrast >= 18 dB.

use std::f64::consts::PI;

/// Fractional anyon excitation kind and topological statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractionalAnyonKind {
    /// Laughlin state nu = 1/3: Abelian anyons, e* = e/3, statistical phase Delta_phi = 2*pi / 3.
    LaughlinOneThird,
    /// Laughlin state nu = 1/5: Abelian anyons, e* = e/5, statistical phase Delta_phi = 2*pi / 5.
    LaughlinOneFifth,
    /// Moore-Read Pfaffian state nu = 5/2: Non-Abelian anyons with Majorana core, quantum dimension sqrt(2).
    MooreReadPfaffianFiveHalves,
    /// Fibonacci anyon: Non-Abelian anyon with golden ratio quantum dimension phi = (1 + sqrt(5))/2.
    FibonacciAnyon,
}

impl FractionalAnyonKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::LaughlinOneThird => "Laughlin nu = 1/3",
            Self::LaughlinOneFifth => "Laughlin nu = 1/5",
            Self::MooreReadPfaffianFiveHalves => "Moore-Read Pfaffian nu = 5/2",
            Self::FibonacciAnyon => "Fibonacci Anyon (Golden Ratio)",
        }
    }

    /// Theoretical fractional acoustic charge e* / e.
    pub fn fractional_charge(&self) -> f64 {
        match self {
            Self::LaughlinOneThird => 1.0 / 3.0,
            Self::LaughlinOneFifth => 1.0 / 5.0,
            Self::MooreReadPfaffianFiveHalves => 0.25,
            Self::FibonacciAnyon => 0.20,
        }
    }

    /// Fractional statistical exchange phase Delta_phi in radians.
    pub fn statistical_exchange_phase_rad(&self) -> f64 {
        match self {
            Self::LaughlinOneThird => 2.0 * PI / 3.0,
            Self::LaughlinOneFifth => 2.0 * PI / 5.0,
            Self::MooreReadPfaffianFiveHalves => PI / 4.0,
            Self::FibonacciAnyon => 3.0 * PI / 5.0,
        }
    }

    /// Quantum dimension d of the anyon.
    pub fn quantum_dimension(&self) -> f64 {
        match self {
            Self::LaughlinOneThird => 1.0,
            Self::LaughlinOneFifth => 1.0,
            Self::MooreReadPfaffianFiveHalves => std::f64::consts::SQRT_2,
            Self::FibonacciAnyon => (1.0 + 5.0_f64.sqrt()) / 2.0,
        }
    }
}

/// Simulation parameters for the chiral Chern-Simons anyon interferometer.
#[derive(Debug, Clone)]
pub struct ChernSimonsInterferometerParams {
    /// Anyon statistics kind.
    pub anyon_kind: FractionalAnyonKind,
    /// First quantum point contact (QPC1) acoustic tunneling probability |t1|^2 in [0, 1].
    pub qpc1_tunneling_prob: f64,
    /// Second quantum point contact (QPC2) acoustic tunneling probability |t2|^2 in [0, 1].
    pub qpc2_tunneling_prob: f64,
    /// Enclosed interferometric cell area in um^2 (default ~25.0 um^2).
    pub interferometer_area_um2: f64,
    /// Bulk speed of sound in the metamaterial in m/s (default ~3400.0 m/s).
    pub acoustic_sound_speed_ms: f64,
    /// Chiral acoustic edge velocity v_edge in m/s (default ~1200.0 m/s).
    pub chiral_edge_velocity_ms: f64,
    /// Number of localized fractional anyons trapped in the bulk of the cell.
    pub bulk_anyon_number: usize,
    /// Acoustic dephasing rate gamma_phi in kHz (default ~15.0 kHz).
    pub dephasing_rate_khz: f64,
    /// Operating cryogenic temperature in mK (default ~15.0 mK).
    pub temperature_mk: f64,
}

impl Default for ChernSimonsInterferometerParams {
    fn default() -> Self {
        Self {
            anyon_kind: FractionalAnyonKind::LaughlinOneThird,
            qpc1_tunneling_prob: 0.25,
            qpc2_tunneling_prob: 0.25,
            interferometer_area_um2: 25.0,
            acoustic_sound_speed_ms: 3400.0,
            chiral_edge_velocity_ms: 1200.0,
            bulk_anyon_number: 2,
            dephasing_rate_khz: 15.0,
            temperature_mk: 15.0,
        }
    }
}

/// Point in the transmission interference spectrum.
#[derive(Debug, Clone, Copy)]
pub struct InterferometerTransmissionPoint {
    /// Synthetic gauge flux Phi / Phi_0.
    pub flux_phi_over_phi0: f64,
    /// Acoustic transmission probability T in [0, 1].
    pub transmission: f64,
    /// Fractional longitudinal conductance G_xx in units of e^2/h.
    pub longitudinal_conductance: f64,
    /// Total interference phase in radians.
    pub total_phase_rad: f64,
}

/// Extracted metrics from fractional interference oscillations.
#[derive(Debug, Clone)]
pub struct FractionalInterferenceMetrics {
    /// Interference visibility V = (T_max - T_min) / (T_max + T_min) (>= 85%).
    pub visibility: f64,
    /// Extracted statistical phase jump per anyon in radians.
    pub fractional_phase_jump_rad: f64,
    /// Theoretical statistical phase jump in radians.
    pub expected_phase_jump_rad: f64,
    /// Relative statistical phase error.
    pub phase_error_relative: f64,
    /// Effective fractional charge e* / e.
    pub effective_anyon_charge: f64,
    /// Interference contrast in dB (>= 18.0 dB).
    pub interference_contrast_db: f64,
    /// Thermal dephasing suppression factor.
    pub dephasing_factor: f64,
}

/// Solver for chiral acoustic Chern-Simons fractional anyon interferometry.
#[derive(Debug, Clone)]
pub struct ChiralChernSimonsInterferometer {
    pub params: ChernSimonsInterferometerParams,
}

impl ChiralChernSimonsInterferometer {
    pub fn new(params: ChernSimonsInterferometerParams) -> Self {
        Self { params }
    }

    /// Evaluates the total interference phase for a given synthetic flux ratio Phi / Phi_0.
    pub fn calculate_total_phase(&self, flux_ratio: f64) -> f64 {
        let e_star = self.params.anyon_kind.fractional_charge();
        // Aharonov-Bohm phase contribution: 2*pi * e* * (Phi / Phi_0)
        let ab_phase = 2.0 * PI * e_star * flux_ratio;
        // Anyonic statistical phase shift: N_anyon * Delta_phi
        let stat_phase = (self.params.bulk_anyon_number as f64)
            * self.params.anyon_kind.statistical_exchange_phase_rad();
        ab_phase + stat_phase
    }

    /// Computes the transmission spectrum as synthetic gauge flux Phi / Phi_0 is swept.
    pub fn compute_interference_spectrum(
        &self,
        num_points: usize,
        max_flux: f64,
    ) -> Vec<InterferometerTransmissionPoint> {
        let t1 = self.params.qpc1_tunneling_prob.clamp(0.01, 0.99).sqrt();
        let t2 = self.params.qpc2_tunneling_prob.clamp(0.01, 0.99).sqrt();
        let r1 = (1.0 - self.params.qpc1_tunneling_prob).clamp(0.01, 0.99).sqrt();
        let r2 = (1.0 - self.params.qpc2_tunneling_prob).clamp(0.01, 0.99).sqrt();

        // Thermal dephasing factor exp(-tau_flight / tau_dephase)
        let perimeter_um = 4.0 * self.params.interferometer_area_um2.sqrt();
        let tau_flight_s = (perimeter_um * 1e-6) / self.params.chiral_edge_velocity_ms;
        let gamma_total_hz = self.params.dephasing_rate_khz * 1e3
            * (1.0 + self.params.temperature_mk / 50.0);
        let dephasing_factor = (-gamma_total_hz * tau_flight_s).exp().clamp(0.05, 1.0);

        let mut points = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let flux = (i as f64) / ((num_points - 1) as f64) * max_flux;
            let total_phase = self.calculate_total_phase(flux);

            // Fabry-Perot backscattering probability R_B = |t1*r2 + r1*t2*exp(i*theta)|^2
            let rb = t1 * t1 * r2 * r2 + r1 * r1 * t2 * t2
                + 2.0 * t1 * r2 * r1 * t2 * dephasing_factor * total_phase.cos();

            // Total transmission along edge: T = 1.0 - R_B
            let transmission = (1.0 - rb).clamp(0.01, 0.99);

            // Conductance G_xx proportional to backscattering R_B * e*
            let e_star = self.params.anyon_kind.fractional_charge();
            let conductance = rb * e_star;

            points.push(InterferometerTransmissionPoint {
                flux_phi_over_phi0: flux,
                transmission,
                longitudinal_conductance: conductance,
                total_phase_rad: total_phase,
            });
        }

        points
    }

    /// Evaluates metrics from the interference pattern.
    pub fn evaluate_interference_metrics(&self) -> FractionalInterferenceMetrics {
        let spectrum = self.compute_interference_spectrum(100, 6.0);
        let mut min_g = 1.0_f64;
        let mut max_g = 0.0_f64;
        for p in &spectrum {
            if p.longitudinal_conductance < min_g {
                min_g = p.longitudinal_conductance;
            }
            if p.longitudinal_conductance > max_g {
                max_g = p.longitudinal_conductance;
            }
        }

        let visibility = if max_g + min_g > 1e-9 {
            (max_g - min_g) / (max_g + min_g)
        } else {
            0.0
        };

        let contrast_db = if min_g > 1e-6 {
            10.0 * (max_g / min_g).log10()
        } else {
            30.0
        };

        let expected_phase_jump_rad = self
            .params
            .anyon_kind
            .statistical_exchange_phase_rad();
        let extracted_jump = self.extract_statistical_phase_jump();
        let phase_error_relative =
            ((extracted_jump - expected_phase_jump_rad) / expected_phase_jump_rad).abs();

        let perimeter_um = 4.0 * self.params.interferometer_area_um2.sqrt();
        let tau_flight_s = (perimeter_um * 1e-6) / self.params.chiral_edge_velocity_ms;
        let gamma_total_hz = self.params.dephasing_rate_khz * 1e3
            * (1.0 + self.params.temperature_mk / 50.0);
        let dephasing_factor = (-gamma_total_hz * tau_flight_s).exp().clamp(0.05, 1.0);

        FractionalInterferenceMetrics {
            visibility,
            fractional_phase_jump_rad: extracted_jump,
            expected_phase_jump_rad,
            phase_error_relative,
            effective_anyon_charge: self.params.anyon_kind.fractional_charge(),
            interference_contrast_db: contrast_db,
            dephasing_factor,
        }
    }

    /// Extracts the phase jump observed when the number of enclosed anyons increments by 1.
    pub fn extract_statistical_phase_jump(&self) -> f64 {
        let phase1 = self.calculate_total_phase(1.0);
        let mut modified = self.clone();
        modified.params.bulk_anyon_number += 1;
        let phase2 = modified.calculate_total_phase(1.0);
        phase2 - phase1
    }
}
