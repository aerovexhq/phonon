#![deny(unsafe_code)]

//! Continuous-Variable Quantum Key Distribution (CV-QKD) & Gaussian Entanglement Engine for Phase 429.
//!
//! Models two-mode squeezed EPR entanglement, symplectic Gaussian covariance matrix algebra,
//! channel noise, Holevo information bound, and asymptotic secret key rate evaluation for
//! quantum-secured avionics and acoustic metamaterial buses.

use std::f64::consts::PI;

/// Parameters for Continuous-Variable Quantum Key Distribution (CV-QKD).
#[derive(Debug, Clone, PartialEq)]
pub struct CvQkdParams {
    /// Two-mode squeezing parameter r (>= 0.0).
    pub squeezing_r: f64,
    /// Channel interaction / transmission distance in meters.
    pub distance_m: f64,
    /// Acoustic waveguide channel attenuation in dB/m.
    pub waveguide_loss_db_m: f64,
    /// Channel excess noise in Shot-Noise Units (SNU).
    pub excess_noise_snu: f64,
    /// Receiver detection quantum efficiency (0.0 to 1.0).
    pub detection_efficiency: f64,
    /// Receiver electronic thermal noise in SNU.
    pub electronic_noise_snu: f64,
    /// Post-processing reverse reconciliation efficiency beta (0.80 to 1.0).
    pub reconciliation_efficiency_beta: f64,
    /// Pulse repetition rate in MHz.
    pub repetition_rate_mhz: f64,
}

impl Default for CvQkdParams {
    fn default() -> Self {
        Self {
            squeezing_r: 1.15, // ~10.0 dB squeezing
            distance_m: 10.0,
            waveguide_loss_db_m: 0.12,
            excess_noise_snu: 0.005,
            detection_efficiency: 0.90,
            electronic_noise_snu: 0.03,
            reconciliation_efficiency_beta: 0.95,
            repetition_rate_mhz: 100.0,
        }
    }
}

/// 4x4 Real Symmetric Symplectic Covariance Matrix in shot noise units.
#[derive(Debug, Clone, PartialEq)]
pub struct CvQkdCovarianceMatrix {
    /// Matrix elements m[row][col].
    pub m: [[f64; 4]; 4],
    /// Symplectic eigenvalue nu (entangled if nu < 1.0).
    pub symplectic_eigenvalue: f64,
    /// Duan inseparability witness (< 2.0 indicates non-classical entanglement).
    pub duan_witness: f64,
    /// Squeezing level in dB below shot noise limit.
    pub squeezing_db: f64,
}

/// Telemetry metrics for the CV-QKD link.
#[derive(Debug, Clone, PartialEq)]
pub struct CvQkdTelemetry {
    /// Channel linear transmittance T.
    pub channel_transmittance: f64,
    /// Channel loss in dB.
    pub channel_loss_db: f64,
    /// Channel added noise chi_line in SNU.
    pub line_added_noise_snu: f64,
    /// Total added noise referred to channel input chi_tot in SNU.
    pub total_added_noise_snu: f64,
    /// Alice-Bob mutual information I_AB in bits/pulse.
    pub mutual_information_bits_pulse: f64,
    /// Eve's Holevo information bound chi_BE in bits/pulse.
    pub holevo_bound_bits_pulse: f64,
    /// Asymptotic secret key rate in bits/pulse.
    pub secret_key_rate_bits_pulse: f64,
    /// Asymptotic secret key rate in Mbps.
    pub secret_key_rate_mbps: f64,
    /// Quantum bit error rate equivalent in percent.
    pub equivalent_qber_percent: f64,
    /// Symplectic entanglement metric nu.
    pub symplectic_nu: f64,
    /// Squeezing level in dB below vacuum limit.
    pub squeezing_db: f64,
}

/// Key rate vs transmission distance curve point.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyRateDistancePoint {
    /// Distance in meters.
    pub distance_m: f64,
    /// Secret key rate in Mbps.
    pub secret_key_rate_mbps: f64,
    /// Mutual information I_AB in bits/pulse.
    pub mutual_info: f64,
    /// Holevo bound chi_BE in bits/pulse.
    pub holevo_bound: f64,
}

/// 1D slice of the two-mode Wigner distribution across quadrature x.
#[derive(Debug, Clone, PartialEq)]
pub struct WignerSlicePoint {
    /// Quadrature coordinate x in shot-noise units.
    pub x: f64,
    /// Wigner quasi-probability density W(x).
    pub wigner_density: f64,
}

/// Engine for evaluating Continuous-Variable Quantum Key Distribution (CV-QKD).
#[derive(Debug, Clone)]
pub struct CvQkdEngine {
    pub params: CvQkdParams,
}

impl CvQkdEngine {
    /// Creates a new CV-QKD engine.
    pub fn new(params: CvQkdParams) -> Self {
        Self { params }
    }

    /// Evaluates the 4x4 covariance matrix of the entangled EPR polariton state.
    pub fn evaluate_covariance_matrix(&self) -> CvQkdCovarianceMatrix {
        let r = self.params.squeezing_r.max(0.01);
        let v = (2.0 * r).cosh(); // Variance V >= 1.0 in SNU
        let c = (v.powi(2) - 1.0).max(0.0).sqrt(); // Correlation cross-term

        let mut m = [[0.0; 4]; 4];
        // Alice mode (x_A, p_A)
        m[0][0] = v;
        m[1][1] = v;
        // Bob mode (x_B, p_B)
        m[2][2] = v;
        m[3][3] = v;
        // Cross-correlations: x_A x_B = +c, p_A p_B = -c
        m[0][2] = c;
        m[2][0] = c;
        m[1][3] = -c;
        m[3][1] = -c;

        // Symplectic eigenvalue of partial transpose: nu = exp(-2r)
        let nu = (-2.0 * r).exp();
        // Duan criterion: <(x_A - x_B)^2> + <(p_A + p_B)^2> = 2 * exp(-2r)
        let duan_witness = 2.0 * (-2.0 * r).exp();
        // Squeezing in dB: 10 * log10(exp(2r))
        let squeezing_db = (20.0 / 10.0_f64.ln()) * r;

        CvQkdCovarianceMatrix {
            m,
            symplectic_eigenvalue: nu,
            duan_witness,
            squeezing_db,
        }
    }

    /// Evaluates link metrics and secret key rate under reverse reconciliation.
    pub fn evaluate_telemetry(&self) -> CvQkdTelemetry {
        let v = (2.0 * self.params.squeezing_r).cosh().max(1.001);
        let squeezing_db = (20.0 / 10.0_f64.ln()) * self.params.squeezing_r;
        let symplectic_nu = (-2.0 * self.params.squeezing_r).exp();

        // Channel loss in dB: alpha * L
        let channel_loss_db = (self.params.waveguide_loss_db_m * self.params.distance_m).max(0.0);
        let channel_t = 10.0_f64.powf(-channel_loss_db / 10.0).clamp(1e-6, 1.0);

        // Noise referred to channel input
        let chi_line = (1.0 / channel_t) - 1.0 + self.params.excess_noise_snu;
        let chi_hom = (1.0 - self.params.detection_efficiency + self.params.electronic_noise_snu)
            / self.params.detection_efficiency.max(1e-4);
        let chi_tot = chi_line + (chi_hom / channel_t);

        // Alice-Bob mutual information for homodyne detection
        let snr = (v - 1.0) / (1.0 + chi_tot);
        let i_ab = 0.5 * (1.0 + snr).log2().max(0.0);

        // Eve's Holevo bound: chi_BE = S(rho_AB) - S(rho_A|x_B)
        // 1. S(rho_AB) = G((lambda_1 - 1)/2) + G((lambda_2 - 1)/2)
        // For standard two-mode squeezed vacuum, the state before channel is pure (lambda = 1),
        // but after channel of transmittance T with noise eps, the state is mixed:
        // Evaluated via covariance matrix symplectic invariants:
        let a = v;
        let b = channel_t * (v + chi_line);
        let c = (channel_t * (v.powi(2) - 1.0)).max(0.0).sqrt();

        let delta_ab = a.powi(2) + b.powi(2) - 2.0 * c.powi(2);
        let det_ab = (a * b - c.powi(2)).powi(2).max(1.0);

        let radical = (delta_ab.powi(2) - 4.0 * det_ab).max(0.0).sqrt();
        let lambda_1 = ((delta_ab + radical) / 2.0).max(1.0).sqrt();
        let lambda_2 = ((delta_ab - radical) / 2.0).max(1.0).sqrt();

        let s_ab = Self::g_function((lambda_1 - 1.0) / 2.0) + Self::g_function((lambda_2 - 1.0) / 2.0);

        // 2. Conditional state S(rho_A|x_B) = G((lambda_3 - 1)/2)
        let lambda_3 = (a * (a - (c.powi(2) / b.max(1e-6)))).max(1.0).sqrt();
        let s_a_cond_b = Self::g_function((lambda_3 - 1.0) / 2.0);

        let chi_be = (s_ab - s_a_cond_b).max(0.0);

        // Secret key rate per pulse
        let delta_info = self.params.reconciliation_efficiency_beta * i_ab - chi_be;
        let key_rate_bits_pulse = delta_info.max(0.0);
        let secret_key_rate_mbps = key_rate_bits_pulse * self.params.repetition_rate_mhz;

        // Equivalent QBER estimation
        let equivalent_qber_percent = (self.params.excess_noise_snu / 0.10 * 1.5)
            + (1.0 - channel_t) * 0.2;

        CvQkdTelemetry {
            channel_transmittance: channel_t,
            channel_loss_db,
            line_added_noise_snu: chi_line,
            total_added_noise_snu: chi_tot,
            mutual_information_bits_pulse: i_ab,
            holevo_bound_bits_pulse: chi_be,
            secret_key_rate_bits_pulse: key_rate_bits_pulse,
            secret_key_rate_mbps,
            equivalent_qber_percent: equivalent_qber_percent.clamp(0.05, 12.0),
            symplectic_nu,
            squeezing_db,
        }
    }

    /// Evaluates von Neumann entropy helper function G(x) = (x+1)log2(x+1) - x log2(x).
    fn g_function(x: f64) -> f64 {
        if x <= 1e-12 {
            0.0
        } else {
            (x + 1.0) * (x + 1.0).log2() - x * x.log2()
        }
    }

    /// Generates secret key rate vs distance curve from 0 to 50 meters.
    pub fn generate_key_rate_vs_distance(&self) -> Vec<KeyRateDistancePoint> {
        let max_dist = 50.0;
        let points = 50;
        let mut curve = Vec::with_capacity(points);

        let mut sim_engine = self.clone();

        for i in 0..points {
            let dist = (i as f64 / (points - 1) as f64) * max_dist;
            sim_engine.params.distance_m = dist;
            let tele = sim_engine.evaluate_telemetry();

            curve.push(KeyRateDistancePoint {
                distance_m: dist,
                secret_key_rate_mbps: tele.secret_key_rate_mbps,
                mutual_info: tele.mutual_information_bits_pulse,
                holevo_bound: tele.holevo_bound_bits_pulse,
            });
        }

        curve
    }

    /// Generates 1D slice of the Gaussian Wigner probability distribution.
    pub fn generate_wigner_slice(&self) -> Vec<WignerSlicePoint> {
        let cov = self.evaluate_covariance_matrix();
        let var = cov.m[0][0]; // Variance V
        let points = 80;
        let x_span = 4.0;
        let mut slice = Vec::with_capacity(points);

        for i in 0..points {
            let x = -x_span + (i as f64 / (points - 1) as f64) * (2.0 * x_span);
            // 1D marginal Gaussian: 1 / sqrt(2 * pi * V) * exp(-x^2 / (2 * V))
            let wigner_density = (1.0 / (2.0 * PI * var).sqrt()) * (-x.powi(2) / (2.0 * var)).exp();
            slice.push(WignerSlicePoint { x, wigner_density });
        }

        slice
    }
}
