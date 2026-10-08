#![deny(unsafe_code)]

//! Floquet Chiral Magnon-Phonon Polariton Router for Phase 429.
//!
//! Models non-reciprocal Floquet polariton mode conversion between acoustic surface
//! waves and spin-wave magnons under time-periodic synthetic gauge driving, providing
//! high-isolation chiral routing for continuous-variable quantum state distribution.

use std::f64::consts::PI;

/// Parameters for Floquet chiral magnon-phonon polariton routing.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetRouterParams {
    /// Polariton center operational frequency in GHz.
    pub center_freq_ghz: f64,
    /// Floquet drive frequency in GHz.
    pub floquet_drive_freq_ghz: f64,
    /// Synthetic Floquet drive amplitude in MHz.
    pub drive_amplitude_mhz: f64,
    /// Magneto-acoustic coupling strength in MHz.
    pub magnetoacoustic_coupling_mhz: f64,
    /// Magnon resonance linewidth (FWHM) in MHz.
    pub magnon_linewidth_mhz: f64,
    /// Acoustic phonon waveguide linewidth in MHz.
    pub phonon_linewidth_mhz: f64,
    /// Waveguide interaction length in mm.
    pub waveguide_length_mm: f64,
    /// Number of routing ports.
    pub port_count: usize,
}

impl Default for FloquetRouterParams {
    fn default() -> Self {
        Self {
            center_freq_ghz: 2.4,
            floquet_drive_freq_ghz: 2.4,
            drive_amplitude_mhz: 45.0,
            magnetoacoustic_coupling_mhz: 32.0,
            magnon_linewidth_mhz: 1.2,
            phonon_linewidth_mhz: 0.4,
            waveguide_length_mm: 15.0,
            port_count: 4,
        }
    }
}

/// Telemetry metrics for the chiral polariton router.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonRouterTelemetry {
    /// Forward transmission (S21) in dB.
    pub forward_transmission_db: f64,
    /// Linear forward power transmittance (0.0 to 1.0).
    pub forward_transmittance: f64,
    /// Backward transmission (S12) in dB.
    pub backward_transmission_db: f64,
    /// Non-reciprocal chiral isolation in dB (S21_dB - S12_dB).
    pub chiral_isolation_db: f64,
    /// Non-reciprocal phase shift in degrees.
    pub nonreciprocal_phase_deg: f64,
    /// Polariton group velocity in km/s.
    pub polariton_group_velocity_km_s: f64,
    /// Directivity between Alice and Bob output ports in dB.
    pub port_directivity_db: f64,
}

/// Point on the S-parameter transmission and isolation spectrum.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonSpectrumPoint {
    /// Frequency in GHz.
    pub freq_ghz: f64,
    /// Forward transmission S21 in dB.
    pub s21_db: f64,
    /// Backward transmission S12 in dB.
    pub s12_db: f64,
    /// Chiral isolation in dB.
    pub isolation_db: f64,
}

/// Chiral Floquet Polariton Router Solver.
#[derive(Debug, Clone)]
pub struct ChiralPolaritonRouter {
    pub params: FloquetRouterParams,
}

impl ChiralPolaritonRouter {
    /// Creates a new router solver with the specified parameters.
    pub fn new(params: FloquetRouterParams) -> Self {
        Self { params }
    }

    /// Evaluates router telemetry metrics under resonant Floquet polariton drive.
    pub fn evaluate_telemetry(&self) -> PolaritonRouterTelemetry {
        // Effective cooperativity: C = 4 * g_ma^2 / (kappa_m * kappa_p)
        let cooperativity = (4.0 * self.params.magnetoacoustic_coupling_mhz.powi(2))
            / (self.params.magnon_linewidth_mhz * self.params.phonon_linewidth_mhz).max(1e-6);

        // Floquet non-reciprocal asymmetry factor governed by synthetic gauge phase and drive
        let drive_param = (self.params.drive_amplitude_mhz / 50.0).clamp(0.2, 2.5);
        let nonrecip_factor = 1.0 - (-cooperativity * 0.08 * drive_param).exp();

        // Forward transmittance S21
        let base_loss_db = 0.15 + (self.params.waveguide_length_mm / 15.0) * 0.08;
        let s21_db = -base_loss_db.max(0.05);
        let forward_transmittance = 10.0_f64.powf(s21_db / 10.0);

        // Backward transmission S12 is strongly suppressed due to destructive Floquet interference
        let raw_isolation_db = 28.0 + 8.5 * nonrecip_factor + 4.0 * (cooperativity / 200.0).clamp(0.1, 1.5);
        let isolation_db = raw_isolation_db.clamp(20.0, 52.0);
        let s12_db = s21_db - isolation_db;

        // Nonreciprocal phase shift across the interaction length
        let nonreciprocal_phase_deg = (90.0 * nonrecip_factor * (self.params.waveguide_length_mm / 15.0)).clamp(30.0, 180.0);

        // Acoustic-magnonic polariton group velocity
        let polariton_group_velocity_km_s = 3.48 / (1.0 + 0.15 * cooperativity.sqrt() / 15.0);

        let port_directivity_db = isolation_db - 2.5;

        PolaritonRouterTelemetry {
            forward_transmission_db: s21_db,
            forward_transmittance,
            backward_transmission_db: s12_db,
            chiral_isolation_db: isolation_db,
            nonreciprocal_phase_deg,
            polariton_group_velocity_km_s,
            port_directivity_db,
        }
    }

    /// Generates S-parameter frequency spectrum around the operational center.
    pub fn generate_spectrum(&self) -> Vec<PolaritonSpectrumPoint> {
        let f0 = self.params.center_freq_ghz;
        let span_ghz = 0.20; // +/- 100 MHz span
        let points = 80;
        let mut spectrum = Vec::with_capacity(points);

        let tele = self.evaluate_telemetry();
        let base_isolation = tele.chiral_isolation_db;
        let bandwidth_ghz = 0.040; // 40 MHz chiral isolation bandwidth

        for i in 0..points {
            let freq_ghz = f0 - span_ghz / 2.0 + (i as f64 / (points - 1) as f64) * span_ghz;
            let detuning = (freq_ghz - f0) / bandwidth_ghz;

            // Lorentzian-shaped isolation passband
            let isolation_db = base_isolation / (1.0 + detuning.powi(2));
            let s21_db = tele.forward_transmission_db - 0.5 * (detuning.powi(2) / (1.0 + detuning.powi(2)));
            let s12_db = s21_db - isolation_db;

            spectrum.push(PolaritonSpectrumPoint {
                freq_ghz,
                s21_db,
                s12_db,
                isolation_db,
            });
        }

        spectrum
    }

    /// Evaluates 4x4 scattering power matrix between ports: [Alice_In, Alice_Out, Bob_In, Bob_Out].
    pub fn evaluate_routing_matrix(&self) -> [[f64; 4]; 4] {
        let tele = self.evaluate_telemetry();
        let t_fwd = tele.forward_transmittance;
        let t_bwd = 10.0_f64.powf(tele.backward_transmission_db / 10.0);
        let leak = 0.005;

        // Port 0: Alice Input, Port 1: Alice Return/Mon, Port 2: Bob Input, Port 3: Bob Output
        [
            [0.01, leak, leak, t_fwd], // Alice In -> Bob Out
            [leak, 0.01, leak, leak],
            [t_bwd, leak, 0.01, leak], // Bob In -> Alice (strongly isolated)
            [leak, leak, leak, 0.01],
        ]
    }
}
