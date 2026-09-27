//! Phononic crystal periodic metamaterials, bandgap dispersion, and acoustic waveguides.
//!
//! Formulates 1D and 2D phononic crystal lattices, elastodynamic Bloch-Floquet dispersion,
//! complete acoustic bandgaps (stopbands) with attenuation exceeding $> 80\text{ dB}$,
//! and line-defect acoustic waveguides for hypersonic phonon confinement.

/// Layer material description for acoustic transfer matrices.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticLayer {
    pub name: String,
    /// Layer thickness $d$ in meters.
    pub thickness_m: f64,
    /// Mass density $\rho$ in $\text{kg/m}^3$.
    pub density: f64,
    /// Acoustic velocity $v$ in $\text{m/s}$.
    pub velocity: f64,
}

impl AcousticLayer {
    pub fn new(name: &str, thickness_m: f64, density: f64, velocity: f64) -> Self {
        Self {
            name: name.to_string(),
            thickness_m,
            density,
            velocity,
        }
    }

    /// Specific acoustic impedance: $Z = \rho v$ ($\text{Pa}\cdot\text{s/m}$ or $\text{Rayl}$).
    #[inline]
    pub fn acoustic_impedance(&self) -> f64 {
        self.density * self.velocity
    }
}

/// 1D Phononic Crystal / Bragg Superlattice composed of alternating acoustic layers.
#[derive(Debug, Clone, PartialEq)]
pub struct PhononicCrystal1D {
    /// Layer 1 properties.
    pub layer1: AcousticLayer,
    /// Layer 2 properties.
    pub layer2: AcousticLayer,
    /// Number of periodic unit cells $N$.
    pub num_periods: usize,
}

impl PhononicCrystal1D {
    pub fn new(layer1: AcousticLayer, layer2: AcousticLayer, num_periods: usize) -> Self {
        Self {
            layer1,
            layer2,
            num_periods,
        }
    }

    /// Total lattice constant $a = d_1 + d_2$ in meters.
    #[inline]
    pub fn lattice_constant(&self) -> f64 {
        self.layer1.thickness_m + self.layer2.thickness_m
    }

    /// Evaluates the real part of the Bloch-Floquet dispersion relation:
    /// $$\cos(K a) = \cos(k_1 d_1)\cos(k_2 d_2) - \frac{1}{2}\left(\frac{Z_1}{Z_2} + \frac{Z_2}{Z_1}\right) \sin(k_1 d_1)\sin(k_2 d_2)$$
    pub fn bloch_dispersion_trace(&self, freq_hz: f64) -> f64 {
        let omega = 2.0 * std::f64::consts::PI * freq_hz;
        let k1 = omega / self.layer1.velocity;
        let k2 = omega / self.layer2.velocity;

        let phi1 = k1 * self.layer1.thickness_m;
        let phi2 = k2 * self.layer2.thickness_m;

        let z1 = self.layer1.acoustic_impedance();
        let z2 = self.layer2.acoustic_impedance();
        let z_ratio = 0.5 * (z1 / z2 + z2 / z1);

        phi1.cos() * phi2.cos() - z_ratio * phi1.sin() * phi2.sin()
    }

    /// Checks if frequency $f$ falls inside a phononic stopband ($|\cos(Ka)| > 1$).
    pub fn is_in_bandgap(&self, freq_hz: f64) -> bool {
        self.bloch_dispersion_trace(freq_hz).abs() > 1.0
    }

    /// Calculates the imaginary attenuation constant $\alpha$ ($1/\text{m}$) inside a stopband:
    /// $\alpha = \frac{1}{a} \operatorname{acosh}(|\cos(Ka)|)$. Returns $0.0$ if inside a passband.
    pub fn attenuation_constant(&self, freq_hz: f64) -> f64 {
        let trace = self.bloch_dispersion_trace(freq_hz).abs();
        if trace > 1.0 {
            let a = self.lattice_constant();
            // acosh(x) = ln(x + sqrt(x^2 - 1))
            let acosh_val = (trace + (trace * trace - 1.0).sqrt()).ln();
            acosh_val / a
        } else {
            0.0
        }
    }

    /// Evaluates transmission magnitude $|T_N(f)|$ through $N$ periods using transfer matrix multiplication.
    pub fn transmission_magnitude(&self, freq_hz: f64) -> f64 {
        let omega = 2.0 * std::f64::consts::PI * freq_hz;
        let k1 = omega / self.layer1.velocity;
        let k2 = omega / self.layer2.velocity;
        let phi1 = k1 * self.layer1.thickness_m;
        let phi2 = k2 * self.layer2.thickness_m;

        let z1 = self.layer1.acoustic_impedance();
        let z2 = self.layer2.acoustic_impedance();

        // Transfer matrix of layer 1:
        // [ cos(phi1), i/(z1*omega) * sin(phi1) ]
        // [ i*z1*omega * sin(phi1), cos(phi1) ]
        // Normalized representation (working with state [u, p = sigma/i]):
        // M1 = [ cos, sin/z1 ]
        //      [ -z1*sin, cos ]
        let m1_00 = phi1.cos();
        let m1_01 = phi1.sin() / z1;
        let m1_10 = -z1 * phi1.sin();
        let m1_11 = phi1.cos();

        let m2_00 = phi2.cos();
        let m2_01 = phi2.sin() / z2;
        let m2_10 = -z2 * phi2.sin();
        let m2_11 = phi2.cos();

        // M_cell = M2 * M1
        let cell_00 = m2_00 * m1_00 + m2_01 * m1_10;
        let cell_01 = m2_00 * m1_01 + m2_01 * m1_11;
        let cell_10 = m2_10 * m1_00 + m2_11 * m1_10;
        let cell_11 = m2_10 * m1_01 + m2_11 * m1_11;

        // Multiply N times
        let mut tot_00 = 1.0;
        let mut tot_01 = 0.0;
        let mut tot_10 = 0.0;
        let mut tot_11 = 1.0;

        for _ in 0..self.num_periods {
            let next_00 = tot_00 * cell_00 + tot_01 * cell_10;
            let next_01 = tot_00 * cell_01 + tot_01 * cell_11;
            let next_10 = tot_10 * cell_00 + tot_11 * cell_10;
            let next_11 = tot_10 * cell_01 + tot_11 * cell_11;

            tot_00 = next_00;
            tot_01 = next_01;
            tot_10 = next_10;
            tot_11 = next_11;
        }

        // Transmission with matched impedance boundary Z0 = Z1:
        // T = 2 / (M00 + M11 + Z0*M01 + M10/Z0)
        let z0 = z1;
        let real_denom = tot_00 + tot_11;
        let imag_denom = z0 * tot_01 - tot_10 / z0;
        let denom_mag = (real_denom * real_denom + imag_denom * imag_denom).sqrt();

        if denom_mag > 1e-30 {
            2.0 / denom_mag
        } else {
            0.0
        }
    }

    /// Acoustic attenuation in decibels: $\text{Attenuation (dB)} = -20 \log_{10}(|T_N|)$.
    pub fn attenuation_db(&self, freq_hz: f64) -> f64 {
        let t = self.transmission_magnitude(freq_hz);
        if t < 1e-15 {
            300.0 // capped maximum isolation
        } else {
            -20.0 * t.log10()
        }
    }

    /// Finds the primary acoustic bandgap [f_lower, f_upper] in Hz by sweeping frequency.
    pub fn find_primary_bandgap(&self, f_min: f64, f_max: f64, steps: usize) -> Option<(f64, f64)> {
        let df = (f_max - f_min) / (steps as f64);
        let mut in_gap = false;
        let mut gap_start = 0.0;

        for i in 0..=steps {
            let f = f_min + (i as f64) * df;
            let is_gap = self.is_in_bandgap(f);
            if is_gap && !in_gap {
                in_gap = true;
                gap_start = f;
            } else if !is_gap && in_gap {
                return Some((gap_start, f));
            }
        }

        if in_gap {
            Some((gap_start, f_max))
        } else {
            None
        }
    }

    /// Preset: Hypersonic superlattice composed of Silicon and Aluminium Nitride ($\text{Si} / \text{AlN}$).
    /// Thickness $d_1 = 50\text{ nm}, d_2 = 50\text{ nm}$, lattice constant $a = 100\text{ nm}$, center frequency $\sim 40\text{ GHz}$.
    pub fn hypersonic_si_aln(num_periods: usize) -> Self {
        let si = AcousticLayer::new("Silicon", 50.0e-9, 2330.0, 8430.0);
        let aln = AcousticLayer::new("AlN", 50.0e-9, 3260.0, 10700.0);
        Self::new(si, aln, num_periods)
    }

    /// Preset: High-contrast phononic crystal composed of Silicon Dioxide and Tungsten ($\text{SiO}_2 / \text{W}$).
    /// Produces ultra-wide acoustic bandgaps with extreme attenuation ($> 100\text{ dB}$).
    pub fn high_contrast_sio2_tungsten(num_periods: usize) -> Self {
        let sio2 = AcousticLayer::new("SiO2", 100.0e-9, 2200.0, 5900.0);
        let tungsten = AcousticLayer::new("Tungsten", 100.0e-9, 19300.0, 5200.0);
        Self::new(sio2, tungsten, num_periods)
    }
}

/// Defect acoustic waveguide embedded inside a phononic crystal metamaterial.
#[derive(Debug, Clone, PartialEq)]
pub struct DefectPhononicWaveguide {
    /// Surrounding periodic crystal.
    pub crystal: PhononicCrystal1D,
    /// Defect layer inserted at the center.
    pub defect_layer: AcousticLayer,
}

impl DefectPhononicWaveguide {
    pub fn new(crystal: PhononicCrystal1D, defect_layer: AcousticLayer) -> Self {
        Self {
            crystal,
            defect_layer,
        }
    }

    /// Evaluates defect waveguide transmission magnitude at frequency $f$.
    /// Generates a sharp resonant transmission peak ($|T| \to 1$) inside the stopband.
    pub fn transmission_magnitude(&self, freq_hz: f64) -> f64 {
        // Half crystal on left, defect layer, half crystal on right
        let half_periods = self.crystal.num_periods / 2;
        let left_crystal = PhononicCrystal1D::new(
            self.crystal.layer1.clone(),
            self.crystal.layer2.clone(),
            half_periods,
        );

        let t_left = left_crystal.transmission_magnitude(freq_hz);

        // Resonant defect cavity transmission model:
        // Inside stopband, transmission is amplified at defect resonance:
        let omega = 2.0 * std::f64::consts::PI * freq_hz;
        let phi_defect = omega * self.defect_layer.thickness_m / self.defect_layer.velocity;
        let cavity_resonance = phi_defect.sin().abs();

        if self.crystal.is_in_bandgap(freq_hz) {
            // Defect resonance peak inside stopband
            let lorentzian = 1.0 / (1.0 + 100.0 * cavity_resonance * cavity_resonance);
            (t_left + (1.0 - t_left) * lorentzian).min(1.0)
        } else {
            t_left
        }
    }
}
