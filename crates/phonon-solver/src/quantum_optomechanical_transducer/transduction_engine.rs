#![deny(unsafe_code)]

//! Tripartite electro-optomechanical transduction physics engine.
//!
//! Models coherent quantum state conversion between microwave photons
//! (superconducting circuit / transmon domain) and telecom optical photons (1550 nm)
//! via a high-Q phononic crystal intermediate acoustic resonator.

/// Configuration parameters for the tripartite electro-optomechanical transducer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransductionParams {
    /// Intermediate mechanical resonator frequency $\Omega_m / (2\pi)$ in GHz (default: 4.0 GHz).
    pub omega_m_ghz: f64,
    /// Mechanical resonator damping linewidth $\gamma_m / (2\pi)$ in kHz (default: 1.8 kHz, $Q_m \approx 2.2 \times 10^6$).
    pub gamma_m_khz: f64,
    /// Microwave LC resonator resonance frequency $\omega_e / (2\pi)$ in GHz (default: 5.0 GHz).
    pub omega_e_ghz: f64,
    /// Total microwave cavity linewidth $\kappa_e / (2\pi)$ in MHz (default: 1.5 MHz).
    pub kappa_e_mhz: f64,
    /// External microwave coupling rate $\kappa_{e,\text{ext}} / (2\pi)$ in MHz (default: 1.35 MHz).
    pub kappa_e_ext_mhz: f64,
    /// Single-photon electromechanical coupling $g_{0,e} / (2\pi)$ in Hz (default: 320.0 Hz).
    pub g0_e_hz: f64,
    /// Intracavity microwave drive photon occupancy $n_e$ (default: 7.5e5).
    pub n_e_pump: f64,
    /// Telecom optical cavity resonance frequency $\omega_o / (2\pi)$ in THz (default: 193.4 THz, 1550 nm).
    pub omega_o_thz: f64,
    /// Total optical cavity linewidth $\kappa_o / (2\pi)$ in MHz (default: 2.0 MHz).
    pub kappa_o_mhz: f64,
    /// External optical coupling rate $\kappa_{o,\text{ext}} / (2\pi)$ in MHz (default: 1.80 MHz).
    pub kappa_o_ext_mhz: f64,
    /// Single-photon optomechanical coupling $g_{0,o} / (2\pi)$ in Hz (default: 280.0 Hz).
    pub g0_o_hz: f64,
    /// Intracavity optical drive photon occupancy $n_o$ (default: 1.25e6).
    pub n_o_pump: f64,
}

impl Default for TransductionParams {
    fn default() -> Self {
        Self {
            omega_m_ghz: 4.0,
            gamma_m_khz: 1.8,
            omega_e_ghz: 5.0,
            kappa_e_mhz: 1.5,
            kappa_e_ext_mhz: 1.35,
            g0_e_hz: 320.0,
            n_e_pump: 1.35e6,
            omega_o_thz: 193.4,
            kappa_o_mhz: 2.0,
            kappa_o_ext_mhz: 1.80,
            g0_o_hz: 280.0,
            n_o_pump: 2.20e6,
        }
    }
}

impl TransductionParams {
    /// Validates physical constraints and clamps parameter ranges.
    pub fn sanitized(&self) -> Self {
        Self {
            omega_m_ghz: self.omega_m_ghz.clamp(0.5, 20.0),
            gamma_m_khz: self.gamma_m_khz.clamp(0.01, 100.0),
            omega_e_ghz: self.omega_e_ghz.clamp(1.0, 30.0),
            kappa_e_mhz: self.kappa_e_mhz.clamp(0.1, 50.0),
            kappa_e_ext_mhz: self.kappa_e_ext_mhz.clamp(0.05, self.kappa_e_mhz),
            g0_e_hz: self.g0_e_hz.clamp(10.0, 50_000.0),
            n_e_pump: self.n_e_pump.clamp(1.0, 1.0e9),
            omega_o_thz: self.omega_o_thz.clamp(100.0, 300.0),
            kappa_o_mhz: self.kappa_o_mhz.clamp(0.1, 50.0),
            kappa_o_ext_mhz: self.kappa_o_ext_mhz.clamp(0.05, self.kappa_o_mhz),
            g0_o_hz: self.g0_o_hz.clamp(10.0, 50_000.0),
            n_o_pump: self.n_o_pump.clamp(1.0, 1.0e9),
        }
    }
}

/// Bidirectional S-parameter response at a given detuning frequency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScatteringMatrixPoint {
    /// Detuning $\delta = f - \Omega_m$ from acoustic resonance in MHz.
    pub detuning_mhz: f64,
    /// Forward microwave-to-optical conversion magnitude $|S_{oe}|^2$ (0.0 to 1.0).
    pub transmission_oe: f64,
    /// Reverse optical-to-microwave conversion magnitude $|S_{eo}|^2$ (0.0 to 1.0).
    pub transmission_eo: f64,
    /// Microwave reflection magnitude $|S_{ee}|^2$.
    pub reflection_ee: f64,
    /// Optical reflection magnitude $|S_{oo}|^2$.
    pub reflection_oo: f64,
    /// Bidirectional asymmetry $|S_{oe}|^2 - |S_{eo}|^2$ (strictly zero within numerical precision).
    pub asymmetry: f64,
}

/// Solves linearized Langevin scattering equations for the tripartite transducer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransductionEngine {
    pub params: TransductionParams,
}

impl TransductionEngine {
    /// Creates a new transduction engine instance.
    pub fn new(params: TransductionParams) -> Self {
        Self {
            params: params.sanitized(),
        }
    }

    /// Evaluates effective linearized electromechanical coupling $G_e = g_{0,e} \sqrt{n_e}$ in kHz.
    pub fn compute_linearized_g_e_khz(&self) -> f64 {
        let p = &self.params;
        (p.g0_e_hz * p.n_e_pump.sqrt()) / 1000.0
    }

    /// Evaluates effective linearized optomechanical coupling $G_o = g_{0,o} \sqrt{n_o}$ in kHz.
    pub fn compute_linearized_g_o_khz(&self) -> f64 {
        let p = &self.params;
        (p.g0_o_hz * p.n_o_pump.sqrt()) / 1000.0
    }

    /// Evaluates electromechanical cooperativity $C_e = \frac{4 G_e^2}{\kappa_e \gamma_m}$.
    pub fn compute_electromechanical_cooperativity(&self) -> f64 {
        let p = &self.params;
        let g_e_hz = p.g0_e_hz * p.n_e_pump.sqrt();
        let kappa_e_hz = p.kappa_e_mhz * 1.0e6;
        let gamma_m_hz = p.gamma_m_khz * 1000.0;
        (4.0 * g_e_hz * g_e_hz) / (kappa_e_hz * gamma_m_hz)
    }

    /// Evaluates optomechanical cooperativity $C_o = \frac{4 G_o^2}{\kappa_o \gamma_m}$.
    pub fn compute_optomechanical_cooperativity(&self) -> f64 {
        let p = &self.params;
        let g_o_hz = p.g0_o_hz * p.n_o_pump.sqrt();
        let kappa_o_hz = p.kappa_o_mhz * 1.0e6;
        let gamma_m_hz = p.gamma_m_khz * 1000.0;
        (4.0 * g_o_hz * g_o_hz) / (kappa_o_hz * gamma_m_hz)
    }

    /// Evaluates cooperativity impedance matching imbalance: $|C_e - C_o| / \max(C_e, C_o)$.
    pub fn compute_cooperativity_matching_ratio(&self) -> f64 {
        let c_e = self.compute_electromechanical_cooperativity();
        let c_o = self.compute_optomechanical_cooperativity();
        (c_e - c_o).abs() / c_e.max(c_o).max(1.0e-9)
    }

    /// Evaluates internal bidirectional quantum transduction efficiency $\eta_{\text{int}} = \frac{4 C_e C_o}{(1 + C_e + C_o)^2}$.
    pub fn compute_internal_efficiency(&self) -> f64 {
        let c_e = self.compute_electromechanical_cooperativity();
        let c_o = self.compute_optomechanical_cooperativity();
        let denom = 1.0 + c_e + c_o;
        (4.0 * c_e * c_o) / (denom * denom)
    }

    /// Evaluates microwave extraction efficiency $\eta_{e,\text{ext}} = \kappa_{e,\text{ext}} / \kappa_e$.
    pub fn compute_microwave_extraction_efficiency(&self) -> f64 {
        (self.params.kappa_e_ext_mhz / self.params.kappa_e_mhz.max(1.0e-9)).clamp(0.0, 1.0)
    }

    /// Evaluates optical extraction efficiency $\eta_{o,\text{ext}} = \kappa_{o,\text{ext}} / \kappa_o$.
    pub fn compute_optical_extraction_efficiency(&self) -> f64 {
        (self.params.kappa_o_ext_mhz / self.params.kappa_o_mhz.max(1.0e-9)).clamp(0.0, 1.0)
    }

    /// Evaluates total peak bidirectional end-to-end transduction efficiency $\eta_{\text{trans}} = \eta_{e,\text{ext}} \eta_{o,\text{ext}} \eta_{\text{int}}$.
    pub fn compute_peak_transduction_efficiency(&self) -> f64 {
        let eta_e = self.compute_microwave_extraction_efficiency();
        let eta_o = self.compute_optical_extraction_efficiency();
        let eta_int = self.compute_internal_efficiency();
        eta_e * eta_o * eta_int
    }

    /// Evaluates the 3-dB transduction bandwidth $\Delta f_{\text{3dB}} = \gamma_m (1 + C_e + C_o)$ in MHz.
    pub fn compute_transduction_bandwidth_mhz(&self) -> f64 {
        let p = &self.params;
        let c_e = self.compute_electromechanical_cooperativity();
        let c_o = self.compute_optomechanical_cooperativity();
        let gamma_m_mhz = p.gamma_m_khz * 1.0e-3;
        gamma_m_mhz * (1.0 + c_e + c_o)
    }

    /// Evaluates scattering matrix response at a specified detuning $\delta = f - \Omega_m$ in MHz.
    pub fn compute_scattering_point(&self, detuning_mhz: f64) -> ScatteringMatrixPoint {
        let p = &self.params;
        let c_e = self.compute_electromechanical_cooperativity();
        let c_o = self.compute_optomechanical_cooperativity();
        let eta_e = self.compute_microwave_extraction_efficiency();
        let eta_o = self.compute_optical_extraction_efficiency();

        let gamma_m_mhz = p.gamma_m_khz * 1.0e-3;
        let denom_re = 1.0 + c_e + c_o;
        let denom_im = 2.0 * detuning_mhz / gamma_m_mhz.max(1.0e-9);
        let denom_sq = denom_re * denom_re + denom_im * denom_im;

        let num_trans = 4.0 * eta_e * eta_o * c_e * c_o;
        let t_oe = num_trans / denom_sq;
        let t_eo = t_oe; // Exact bidirectional symmetry

        // Reflections:
        let r_ee = ((1.0 - c_e + c_o).powi(2) + denom_im * denom_im) / denom_sq;
        let r_oo = ((1.0 + c_e - c_o).powi(2) + denom_im * denom_im) / denom_sq;

        ScatteringMatrixPoint {
            detuning_mhz,
            transmission_oe: t_oe.clamp(0.0, 1.0),
            transmission_eo: t_eo.clamp(0.0, 1.0),
            reflection_ee: r_ee.clamp(0.0, 1.0),
            reflection_oo: r_oo.clamp(0.0, 1.0),
            asymmetry: (t_oe - t_eo).abs(),
        }
    }

    /// Evaluates transmission spectrum over a detuning frequency span (points count e.g. 100).
    pub fn compute_spectrum(&self, span_mhz: f64, points: usize) -> Vec<ScatteringMatrixPoint> {
        let count = points.max(2);
        let half_span = span_mhz.abs() * 0.5;
        (0..count)
            .map(|i| {
                let frac = i as f64 / (count - 1) as f64;
                let detuning = -half_span + frac * 2.0 * half_span;
                self.compute_scattering_point(detuning)
            })
            .collect()
    }
}
