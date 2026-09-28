//! 2D Nanoscale Magnetometry Probe Array & Inverse Biot-Savart Current Reconstruction
//!
//! Provides:
//! 1. 2D Nanoscale Magnetometry Probe Array: Grid of NV centers scanning over operating integrated circuit (IC) dies.
//! 2. Forward Biot-Savart magnetic field projection from 2D planar current densities $\mathbf{J}(x, y)$ to $B_z(x, y, d)$.
//! 3. Inverse Biot-Savart magnetic field-to-current density reconstruction using 2D spatial Fourier transforms:
//!    \[\tilde{J}_x(k_x, k_y) = \frac{2}{\mu_0} \frac{i k_y}{k} e^{k d} \tilde{B}_z(k_x, k_y)\]
//!    \[\tilde{J}_y(k_x, k_y) = -\frac{2}{\mu_0} \frac{i k_x}{k} e^{k d} \tilde{B}_z(k_x, k_y)\]
//!    with high-$k$ regularization / low-pass filtering to resolve sub-10 nm current paths,
//!    dielectric leakage currents, and localized circuit shorts.
//! 4. Pure safe Rust 2D spatial Fourier transform implementation.

use phonon_core::MU_0;
use phonon_models::quantum::Complex;
use phonon_models::sensors::NvCenter;

/// Pure safe Rust 1D Fast Fourier Transform (Cooley-Tukey radix-2 with DFT fallback).
pub fn fft_1d(input: &[Complex], inverse: bool) -> Vec<Complex> {
    let n = input.len();
    if n <= 1 {
        return input.to_vec();
    }

    // Check if N is a power of 2
    if (n & (n - 1)) == 0 {
        let mut out = input.to_vec();
        radix2_fft_in_place(&mut out, inverse);
        if inverse {
            let inv_n = 1.0 / n as f64;
            for x in &mut out {
                *x = x.scale(inv_n);
            }
        }
        out
    } else {
        // Direct discrete Fourier transform for non-power-of-2 dimensions
        let mut out = Vec::with_capacity(n);
        let sign = if inverse { 1.0 } else { -1.0 };
        let factor = 2.0 * std::f64::consts::PI * sign / n as f64;
        for k in 0..n {
            let mut sum = Complex::ZERO;
            for (j, item) in input.iter().enumerate() {
                let angle = factor * (k * j) as f64;
                let twiddle = Complex::new(angle.cos(), angle.sin());
                sum = sum + (*item * twiddle);
            }
            if inverse {
                sum = sum.scale(1.0 / n as f64);
            }
            out.push(sum);
        }
        out
    }
}

fn radix2_fft_in_place(data: &mut [Complex], inverse: bool) {
    let n = data.len();
    // Bit reversal permutation
    let mut j = 0;
    for i in 0..n {
        if i < j {
            data.swap(i, j);
        }
        let mut m = n >> 1;
        while m >= 1 && j >= m {
            j -= m;
            m >>= 1;
        }
        j += m;
    }

    // Cooley-Tukey butterflies
    let sign = if inverse { 1.0 } else { -1.0 };
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let angle = sign * 2.0 * std::f64::consts::PI / len as f64;
        let w_step = Complex::new(angle.cos(), angle.sin());

        for i in (0..n).step_by(len) {
            let mut w = Complex::ONE;
            for k in 0..half {
                let u = data[i + k];
                let v = data[i + k + half] * w;
                data[i + k] = u + v;
                data[i + k + half] = u - v;
                w = w * w_step;
            }
        }
        len <<= 1;
    }
}

/// 2D Discrete Fast Fourier Transform for an $(N_x \times N_y)$ matrix stored in row-major order.
pub fn fft_2d(matrix: &[Complex], nx: usize, ny: usize, inverse: bool) -> Vec<Complex> {
    assert_eq!(matrix.len(), nx * ny);
    let mut intermediate = vec![Complex::ZERO; nx * ny];

    // Transform along rows (ny elements each, nx rows)
    for r in 0..nx {
        let row_start = r * ny;
        let row_slice = &matrix[row_start..(row_start + ny)];
        let row_fft = fft_1d(row_slice, inverse);
        intermediate[row_start..(row_start + ny)].copy_from_slice(&row_fft);
    }

    // Transform along columns (nx elements each, ny columns)
    let mut result = vec![Complex::ZERO; nx * ny];
    let mut col_buf = vec![Complex::ZERO; nx];
    for c in 0..ny {
        for r in 0..nx {
            col_buf[r] = intermediate[r * ny + c];
        }
        let col_fft = fft_1d(&col_buf, inverse);
        for r in 0..nx {
            result[r * ny + c] = col_fft[r];
        }
    }

    result
}

/// Type of circuit failure or anomaly identified via nanoscale magnetometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefectType {
    /// Localized low-resistance short circuit between adjacent traces.
    ShortCircuit,
    /// Nanoscale dielectric barrier leakage current.
    LeakageCurrent,
    /// High-current transmission path.
    NominalConduction,
}

/// An identified localized defect on the integrated circuit die.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitDefect {
    /// Spatial X position in meters.
    pub x_m: f64,
    /// Spatial Y position in meters.
    pub y_m: f64,
    /// Peak localized current density in $\text{A/m}$.
    pub current_density_a_per_m: f64,
    /// Defect classification.
    pub defect_type: DefectType,
}

/// 2D Nanoscale Magnetometry Probe Array scanning an integrated circuit die.
#[derive(Debug, Clone, PartialEq)]
pub struct NvProbeGrid {
    /// Number of grid points along X.
    pub nx: usize,
    /// Number of grid points along Y.
    pub ny: usize,
    /// Grid pitch along X in meters ($\Delta x$).
    pub pitch_x_m: f64,
    /// Grid pitch along Y in meters ($\Delta y$).
    pub pitch_y_m: f64,
    /// Standoff distance $d$ of NV centers above the die plane ($z=d$) in meters.
    pub standoff_distance_d_m: f64,
    /// NV center sensor physical configuration.
    pub nv_sensor: NvCenter,
}

impl NvProbeGrid {
    /// Creates a new NV probe grid.
    ///
    /// For example: $64 \times 64$ grid with $10\text{ nm}$ pitch and $15\text{ nm}$ standoff distance.
    pub fn new(
        nx: usize,
        ny: usize,
        pitch_x_m: f64,
        pitch_y_m: f64,
        standoff_distance_d_m: f64,
        nv_sensor: NvCenter,
    ) -> Self {
        Self {
            nx,
            ny,
            pitch_x_m,
            pitch_y_m,
            standoff_distance_d_m,
            nv_sensor,
        }
    }

    /// Total physical width $L_x$ in meters.
    pub fn width_x(&self) -> f64 {
        self.nx as f64 * self.pitch_x_m
    }

    /// Total physical height $L_y$ in meters.
    pub fn height_y(&self) -> f64 {
        self.ny as f64 * self.pitch_y_m
    }

    /// Spatial coordinate $(x, y)$ in meters for grid indices $(r, c)$.
    pub fn grid_position(&self, r: usize, c: usize) -> (f64, f64) {
        (r as f64 * self.pitch_x_m, c as f64 * self.pitch_y_m)
    }

    /// Computes spatial frequency components $(k_x, k_y)$ in $\text{rad/m}$ for Fourier indices $(r, c)$.
    pub fn wavevector(&self, r: usize, c: usize) -> (f64, f64) {
        let lx = self.width_x();
        let ly = self.height_y();

        let kx = if r <= self.nx / 2 {
            2.0 * std::f64::consts::PI * r as f64 / lx
        } else {
            2.0 * std::f64::consts::PI * (r as f64 - self.nx as f64) / lx
        };

        let ky = if c <= self.ny / 2 {
            2.0 * std::f64::consts::PI * c as f64 / ly
        } else {
            2.0 * std::f64::consts::PI * (c as f64 - self.ny as f64) / ly
        };

        (kx, ky)
    }

    /// Forward Biot-Savart projection: computes the perpendicular magnetic field $B_z(x, y)$ in Tesla
    /// at height $z = d$ produced by 2D planar sheet current densities $(J_x, J_y)$ in $\text{A/m}$ on the die plane ($z=0$).
    ///
    /// Analytical Fourier relation:
    /// \[\tilde{B}_z(k_x, k_y) = \frac{\mu_0}{2} \frac{i (k_x \tilde{J}_y - k_y \tilde{J}_x)}{k} e^{-k d}\]
    pub fn forward_biot_savart(&self, jx: &[f64], jy: &[f64]) -> Vec<f64> {
        let n_tot = self.nx * self.ny;
        assert_eq!(jx.len(), n_tot);
        assert_eq!(jy.len(), n_tot);

        let jx_c: Vec<Complex> = jx.iter().map(|&v| Complex::new(v, 0.0)).collect();
        let jy_c: Vec<Complex> = jy.iter().map(|&v| Complex::new(v, 0.0)).collect();

        let jx_k = fft_2d(&jx_c, self.nx, self.ny, false);
        let jy_k = fft_2d(&jy_c, self.nx, self.ny, false);

        let d = self.standoff_distance_d_m;
        let mut bz_k = vec![Complex::ZERO; n_tot];

        for r in 0..self.nx {
            for c in 0..self.ny {
                let idx = r * self.ny + c;
                let (kx, ky) = self.wavevector(r, c);
                let k = (kx * kx + ky * ky).sqrt();

                if k < 1e-12 {
                    // DC component of Bz is zero outside a finite planar circuit
                    bz_k[idx] = Complex::ZERO;
                } else {
                    let jx_val = jx_k[idx];
                    let jy_val = jy_k[idx];

                    // i * (kx * Jy - ky * Jx)
                    let bracket = Complex::new(0.0, 1.0) * (jy_val.scale(kx) - jx_val.scale(ky));
                    let decay = (-k * d).exp();
                    let factor = (MU_0 * 0.5 / k) * decay;
                    bz_k[idx] = bracket.scale(factor);
                }
            }
        }

        let bz_space = fft_2d(&bz_k, self.nx, self.ny, true);
        bz_space.iter().map(|c| c.re).collect()
    }

    /// Inverse Biot-Savart reconstruction: inverts magnetic field $B_z(x, y)$ measured at standoff $d$
    /// back into 2D current density vector field $(J_x, J_y)$ in $\text{A/m}$ on the IC die plane.
    ///
    /// Analytical inverse formulas:
    /// \[\tilde{J}_x(k_x, k_y) = \frac{2}{\mu_0} \frac{i k_y}{k} e^{k d} \tilde{B}_z(k_x, k_y) \cdot W(k)\]
    /// \[\tilde{J}_y(k_x, k_y) = -\frac{2}{\mu_0} \frac{i k_x}{k} e^{k d} \tilde{B}_z(k_x, k_y) \cdot W(k)\]
    /// where $W(k) = \exp(-(k / k_{cut})^4)$ is a 4th-order low-pass filter preventing high-frequency noise amplification.
    pub fn inverse_biot_savart_reconstruction(
        &self,
        bz_tesla: &[f64],
        cutoff_k_rad_per_m: Option<f64>,
    ) -> (Vec<f64>, Vec<f64>) {
        let n_tot = self.nx * self.ny;
        assert_eq!(bz_tesla.len(), n_tot);

        let bz_c: Vec<Complex> = bz_tesla.iter().map(|&v| Complex::new(v, 0.0)).collect();
        let bz_k = fft_2d(&bz_c, self.nx, self.ny, false);

        let d = self.standoff_distance_d_m;
        // Default cutoff frequency corresponds to ~ 1.5 / d
        let k_cut = cutoff_k_rad_per_m.unwrap_or(1.5 / d);

        let mut jx_k = vec![Complex::ZERO; n_tot];
        let mut jy_k = vec![Complex::ZERO; n_tot];

        let inv_mu0_2 = 2.0 / MU_0;

        for r in 0..self.nx {
            for c in 0..self.ny {
                let idx = r * self.ny + c;
                let (kx, ky) = self.wavevector(r, c);
                let k = (kx * kx + ky * ky).sqrt();

                if k < 1e-12 {
                    jx_k[idx] = Complex::ZERO;
                    jy_k[idx] = Complex::ZERO;
                } else {
                    let bz_val = bz_k[idx];

                    // Low-pass filter to regularize evanescent exp(k*d) amplification
                    let window = (-(k / k_cut).powi(4)).exp();
                    let exp_kd = (k * d).min(20.0).exp(); // clamp to prevent numerical overflow
                    let pref = inv_mu0_2 * exp_kd * window / k;

                    // Jx_k = (2 / mu0) * (i * ky / k) * exp(kd) * Bz_k * W(k)
                    jx_k[idx] = (Complex::new(0.0, 1.0) * bz_val).scale(ky * pref);
                    // Jy_k = -(2 / mu0) * (i * kx / k) * exp(kd) * Bz_k * W(k)
                    jy_k[idx] = (Complex::new(0.0, -1.0) * bz_val).scale(kx * pref);
                }
            }
        }

        let jx_space = fft_2d(&jx_k, self.nx, self.ny, true);
        let jy_space = fft_2d(&jy_k, self.nx, self.ny, true);

        let jx_out: Vec<f64> = jx_space.iter().map(|c| c.re).collect();
        let jy_out: Vec<f64> = jy_space.iter().map(|c| c.re).collect();

        (jx_out, jy_out)
    }

    /// Evaluates total current density magnitude $|\mathbf{J}(x, y)| = \sqrt{J_x^2 + J_y^2}$ across the grid.
    pub fn current_magnitude(jx: &[f64], jy: &[f64]) -> Vec<f64> {
        jx.iter()
            .zip(jy.iter())
            .map(|(&x, &y)| (x * x + y * y).sqrt())
            .collect()
    }

    /// Detects circuit defects (shorts, leakage) from reconstructed current density maps.
    ///
    /// - `short_threshold_a_per_m`: Current density exceeding this threshold flags a localized short circuit.
    /// - `leakage_threshold_a_per_m`: Lower threshold for detecting inter-trace dielectric leakage currents.
    pub fn detect_defects(
        &self,
        jx: &[f64],
        jy: &[f64],
        short_threshold_a_per_m: f64,
        leakage_threshold_a_per_m: f64,
    ) -> Vec<CircuitDefect> {
        let mag = Self::current_magnitude(jx, jy);
        let mut defects = Vec::new();

        for r in 0..self.nx {
            for c in 0..self.ny {
                let idx = r * self.ny + c;
                let val = mag[idx];
                let (x_m, y_m) = self.grid_position(r, c);

                if val >= short_threshold_a_per_m {
                    defects.push(CircuitDefect {
                        x_m,
                        y_m,
                        current_density_a_per_m: val,
                        defect_type: DefectType::ShortCircuit,
                    });
                } else if val >= leakage_threshold_a_per_m {
                    defects.push(CircuitDefect {
                        x_m,
                        y_m,
                        current_density_a_per_m: val,
                        defect_type: DefectType::LeakageCurrent,
                    });
                }
            }
        }

        defects
    }
}
