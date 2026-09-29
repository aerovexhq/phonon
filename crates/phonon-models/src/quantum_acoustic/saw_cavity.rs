#![deny(unsafe_code)]

//! Surface Acoustic Wave (SAW) Cavity Resonators & Interdigital Transducers (IDTs).
//!
//! Formulates piezoelectric electromechanical coupling, IDT radiation admittance,
//! distributed Bragg acoustic mirrors, acoustic Fabry-Pérot cavities, and zero-point
//! mechanical displacement fluctuations ($x_{zpf}$) and vacuum potentials ($V_{zpf}$).

use std::f64::consts::PI;

/// Reduced Planck constant $\hbar$ in J*s.
pub const HBAR: f64 = 1.054_571_817e-34;
/// Vacuum permittivity $\varepsilon_0$ in F/m.
pub const EPSILON_0: f64 = 8.854_187_812_8e-12;
/// Elementary charge $e$ in Coulombs.
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19;

/// Piezoelectric substrate material characteristics for Surface Acoustic Waves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SawSubstrateMaterial {
    /// Lithium Niobate (128° YX LiNbO3) - giant electromechanical coupling.
    LiNbO3_128YX,
    /// Aluminum Nitride (c-axis AlN) - high acoustic velocity and low acoustic loss.
    AlN,
    /// Gallium Arsenide (GaAs [100]) - moderate coupling, semiconductor integration.
    GaAs,
    /// ST-cut Quartz - zero temperature coefficient of delay, low coupling.
    QuartzST,
}

impl SawSubstrateMaterial {
    /// Surface acoustic wave Rayleigh velocity $v_{saw}$ in m/s.
    pub fn acoustic_velocity(&self) -> f64 {
        match self {
            Self::LiNbO3_128YX => 3980.0,
            Self::AlN => 5600.0,
            Self::GaAs => 2860.0,
            Self::QuartzST => 3158.0,
        }
    }

    /// Electromechanical coupling coefficient $K^2$ (dimensionless fraction).
    pub fn electromechanical_coupling(&self) -> f64 {
        match self {
            Self::LiNbO3_128YX => 0.056,
            Self::AlN => 0.0028,
            Self::GaAs => 0.0007,
            Self::QuartzST => 0.0016,
        }
    }

    /// Relative dielectric permittivity $\varepsilon_r$.
    pub fn relative_permittivity(&self) -> f64 {
        match self {
            Self::LiNbO3_128YX => 46.0,
            Self::AlN => 9.0,
            Self::GaAs => 12.9,
            Self::QuartzST => 4.5,
        }
    }

    /// Mass density $\rho$ in kg/m^3.
    pub fn density(&self) -> f64 {
        match self {
            Self::LiNbO3_128YX => 4700.0,
            Self::AlN => 3260.0,
            Self::GaAs => 5320.0,
            Self::QuartzST => 2650.0,
        }
    }

    /// Electrostatic capacitance per finger pair per unit aperture $C_s$ in F/m.
    pub fn capacitance_per_pair_meter(&self) -> f64 {
        match self {
            Self::LiNbO3_128YX => 4.5e-10,
            Self::AlN => 1.2e-10,
            Self::GaAs => 1.1e-10,
            Self::QuartzST => 0.5e-10,
        }
    }
}

/// Interdigital Transducer (IDT) for microwave-to-phonon conversion.
#[derive(Debug, Clone, Copy)]
pub struct InterdigitalTransducer {
    /// Substrate material.
    pub material: SawSubstrateMaterial,
    /// Center design frequency $f_0$ in Hz.
    pub center_frequency: f64,
    /// Number of finger pairs $N_p$.
    pub num_pairs: usize,
    /// Acoustic aperture width $W$ in meters.
    pub aperture: f64,
    /// Electrical RF generator / load source impedance $R_0$ in Ohms (typically 50.0).
    pub source_impedance: f64,
}

impl InterdigitalTransducer {
    /// Creates a new IDT configuration.
    pub fn new(
        material: SawSubstrateMaterial,
        center_frequency: f64,
        num_pairs: usize,
        aperture: f64,
        source_impedance: f64,
    ) -> Self {
        Self {
            material,
            center_frequency,
            num_pairs,
            aperture,
            source_impedance,
        }
    }

    /// Acoustic wavelength at center frequency $\lambda_0 = v_{saw} / f_0$ in meters.
    pub fn center_wavelength(&self) -> f64 {
        self.material.acoustic_velocity() / self.center_frequency
    }

    /// Periodic finger pitch $p = \lambda_0 / 2$ in meters.
    pub fn finger_pitch(&self) -> f64 {
        self.center_wavelength() / 2.0
    }

    /// Finger strip width $w = \lambda_0 / 4$ in meters (50% metallization ratio).
    pub fn finger_width(&self) -> f64 {
        self.center_wavelength() / 4.0
    }

    /// Total static capacitance $C_0 = N_p C_s W$ in Farads.
    pub fn static_capacitance(&self) -> f64 {
        self.num_pairs as f64 * self.material.capacitance_per_pair_meter() * self.aperture
    }

    /// Normalized frequency deviation parameter $X = N_p \pi \frac{f - f_0}{f_0}$.
    pub fn normalized_frequency_parameter(&self, freq: f64) -> f64 {
        self.num_pairs as f64 * PI * (freq - self.center_frequency) / self.center_frequency
    }

    /// Acoustic radiation conductance $G_a(f) = 8 K^2 f_0 N_p^2 C_s W \left(\frac{\sin X}{X}\right)^2$ in Siemens.
    pub fn radiation_conductance(&self, freq: f64) -> f64 {
        let k2 = self.material.electromechanical_coupling();
        let cs = self.material.capacitance_per_pair_meter();
        let np = self.num_pairs as f64;
        let g0 = 8.0 * k2 * self.center_frequency * np * np * cs * self.aperture;

        let x = self.normalized_frequency_parameter(freq);
        if x.abs() < 1e-6 {
            g0
        } else {
            let sinc = x.sin() / x;
            g0 * sinc * sinc
        }
    }

    /// Acoustic radiation susceptance $B_a(f) = 8 K^2 f_0 N_p^2 C_s W \frac{\sin(2X) - 2X}{2X^2}$ in Siemens.
    pub fn radiation_susceptance(&self, freq: f64) -> f64 {
        let k2 = self.material.electromechanical_coupling();
        let cs = self.material.capacitance_per_pair_meter();
        let np = self.num_pairs as f64;
        let b0 = 8.0 * k2 * self.center_frequency * np * np * cs * self.aperture;

        let x = self.normalized_frequency_parameter(freq);
        if x.abs() < 1e-5 {
            0.0
        } else {
            let num = (2.0 * x).sin() - 2.0 * x;
            let den = 2.0 * x * x;
            b0 * (num / den)
        }
    }

    /// Microwave-to-phonon power conversion efficiency $\eta(f) \in [0, 1]$.
    ///
    /// Evaluates $\eta(f) = \frac{4 R_0 G_a(f)}{(1 + R_0 G_a(f))^2 + (R_0 (B_a(f) + 2\pi f C_0))^2}$.
    pub fn conversion_efficiency(&self, freq: f64) -> f64 {
        let ga = self.radiation_conductance(freq);
        let ba = self.radiation_susceptance(freq);
        let b_c0 = 2.0 * PI * freq * self.static_capacitance();
        let b_tot = ba + b_c0;
        let r0 = self.source_impedance;

        let num = 4.0 * r0 * ga;
        let den_real = 1.0 + r0 * ga;
        let den_imag = r0 * b_tot;
        let den = den_real * den_real + den_imag * den_imag;

        if den <= 0.0 {
            0.0
        } else {
            (num / den).clamp(0.0, 1.0)
        }
    }
}

/// Distributed Bragg Reflector (DBR) acoustic mirror.
#[derive(Debug, Clone, Copy)]
pub struct BraggAcousticMirror {
    /// Number of reflective metallic/etched strips $N_m$.
    pub num_strips: usize,
    /// Single-strip acoustic reflection coefficient amplitude $|r_s|$ (typically $0.01 - 0.02$).
    pub strip_reflection: f64,
    /// Center Bragg wavelength $\lambda_0$ in meters.
    pub center_wavelength: f64,
}

impl BraggAcousticMirror {
    /// Creates a new Bragg acoustic mirror.
    pub fn new(num_strips: usize, strip_reflection: f64, center_wavelength: f64) -> Self {
        Self {
            num_strips,
            strip_reflection,
            center_wavelength,
        }
    }

    /// Power reflection coefficient $R_m = \tanh^2(N_m |r_s|)$.
    pub fn power_reflectivity(&self) -> f64 {
        let arg = self.num_strips as f64 * self.strip_reflection.abs();
        let t = arg.tanh();
        t * t
    }

    /// Effective penetration depth $L_{pen} = \frac{\lambda_0}{4 |r_s|}$ into the mirror in meters.
    pub fn penetration_depth(&self) -> f64 {
        if self.strip_reflection.abs() < 1e-9 {
            0.0
        } else {
            self.center_wavelength / (4.0 * self.strip_reflection.abs())
        }
    }
}

/// Quantum Surface Acoustic Wave (SAW) Fabry-Pérot Cavity Resonator.
#[derive(Debug, Clone, Copy)]
pub struct SawCavity {
    /// Substrate material.
    pub material: SawSubstrateMaterial,
    /// Physical distance between inner mirror edges $L_c$ in meters.
    pub physical_length: f64,
    /// Acoustic aperture width $W$ in meters.
    pub aperture: f64,
    /// Central IDT for excitation / readout.
    pub idt: InterdigitalTransducer,
    /// DBR acoustic mirrors.
    pub mirror: BraggAcousticMirror,
    /// Internal acoustic material quality factor $Q_{int}$ (typically $10^4 - 10^5$ at mK).
    pub internal_q: f64,
}

impl SawCavity {
    /// Creates a new quantum SAW cavity resonator.
    pub fn new(
        material: SawSubstrateMaterial,
        physical_length: f64,
        aperture: f64,
        idt: InterdigitalTransducer,
        mirror: BraggAcousticMirror,
        internal_q: f64,
    ) -> Self {
        Self {
            material,
            physical_length,
            aperture,
            idt,
            mirror,
            internal_q,
        }
    }

    /// Effective acoustic cavity length $L_{eff} = L_c + 2 L_{pen}$ in meters.
    pub fn effective_length(&self) -> f64 {
        self.physical_length + 2.0 * self.mirror.penetration_depth()
    }

    /// Free Spectral Range $\Delta f_{FSR} = \frac{v_{saw}}{2 L_{eff}}$ in Hz.
    pub fn free_spectral_range(&self) -> f64 {
        self.material.acoustic_velocity() / (2.0 * self.effective_length())
    }

    /// External quality factor limited by mirror transmission $Q_{ext} = \frac{\pi L_{eff}}{\lambda_0 (1 - R_m)}$.
    pub fn external_q(&self) -> f64 {
        let rm = self.mirror.power_reflectivity();
        let lambda0 = self.idt.center_wavelength();
        let transmission = (1.0 - rm).max(1e-9);
        (PI * self.effective_length()) / (lambda0 * transmission)
    }

    /// Loaded cavity quality factor $Q_L = \left(\frac{1}{Q_{int}} + \\frac{1}{Q_{ext}}\right)^{-1}$.
    pub fn loaded_q(&self) -> f64 {
        let q_int = self.internal_q;
        let q_ext = self.external_q();
        1.0 / (1.0 / q_int + 1.0 / q_ext)
    }

    /// Total acoustic decay rate $\kappa = \frac{2\pi f_m}{Q_L}$ in rad/s.
    pub fn phonon_decay_rate(&self, mode_freq: f64) -> f64 {
        (2.0 * PI * mode_freq) / self.loaded_q()
    }

    /// Phonon lifetime (storage coherence time) $T_{1,ph} = \frac{1}{\kappa}$ in seconds.
    pub fn phonon_lifetime(&self, mode_freq: f64) -> f64 {
        1.0 / self.phonon_decay_rate(mode_freq)
    }

    /// Acoustic cavity effective mode volume $V_{mode} = W \times L_{eff} \times \lambda_0$ in m^3.
    pub fn mode_volume(&self) -> f64 {
        self.aperture * self.effective_length() * self.idt.center_wavelength()
    }

    /// Zero-point displacement fluctuations $x_{zpf} = \sqrt{\frac{\hbar}{2 \rho V_{mode} \omega_m}}$ in meters.
    pub fn zero_point_displacement(&self, mode_freq: f64) -> f64 {
        let omega = 2.0 * PI * mode_freq;
        let mass = self.material.density() * self.mode_volume();
        (HBAR / (2.0 * mass * omega)).sqrt()
    }

    /// Zero-point electric potential / vacuum voltage fluctuations $V_{zpf}$ in Volts.
    ///
    /// Evaluates $V_{zpf} = \sqrt{\frac{\hbar \omega_m}{2 C_m}}$ where $C_m = C_s W N_{eff}$
    /// is the effective electrostatic capacitance of the localized standing acoustic wave mode.
    pub fn zero_point_voltage(&self, mode_freq: f64) -> f64 {
        let omega = 2.0 * PI * mode_freq;
        let cs = self.material.capacitance_per_pair_meter();
        let n_eff = 6.0; // effective standing wave finger pairs
        let c_m = (cs * self.aperture * n_eff).max(1e-15);
        ((HBAR * omega) / (2.0 * c_m)).sqrt()
    }
}
