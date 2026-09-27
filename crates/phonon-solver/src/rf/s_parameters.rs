//! High-frequency 2-port RF network analysis: S-parameters, Return Loss, Insertion Loss, and VSWR.

use std::f64::consts::PI;

/// Complex number helper for RF matrix mathematics.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Complex64 {
    pub re: f64,
    pub im: f64,
}

impl Complex64 {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };

    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    #[inline]
    pub fn from_real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    #[inline]
    pub fn norm(&self) -> f64 {
        self.abs()
    }

    #[inline]
    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    #[inline]
    pub fn abs(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    #[inline]
    pub fn arg(&self) -> f64 {
        self.im.atan2(self.re)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, other: Self) -> Self {
        Self::new(self.re + other.re, self.im + other.im)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, other: Self) -> Self {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn div(self, other: Self) -> Self {
        let den = other.norm_sq().max(1e-30);
        Self::new(
            (self.re * other.re + self.im * other.im) / den,
            (self.im * other.re - self.re * other.im) / den,
        )
    }
}

impl std::ops::Add for Complex64 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}

impl std::ops::Sub for Complex64 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.sub(rhs)
    }
}

impl std::ops::Mul for Complex64 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul(rhs)
    }
}

impl std::ops::Div for Complex64 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.div(rhs)
    }
}

/// 2-Port Scattering Parameters ($S$-parameters) evaluated at frequency $f$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwoPortSParameters {
    /// Analysis frequency in Hertz ($f$).
    pub freq_hz: f64,
    /// Reference port characteristic impedance in Ohms ($Z_0$, standard $50\ \Omega$).
    pub z0_ref: f64,
    /// Input reflection coefficient $S_{11}$.
    pub s11: Complex64,
    /// Reverse transmission coefficient $S_{12}$.
    pub s12: Complex64,
    /// Forward transmission coefficient $S_{21}$.
    pub s21: Complex64,
    /// Output reflection coefficient $S_{22}$.
    pub s22: Complex64,
}

impl TwoPortSParameters {
    pub fn new(
        freq_hz: f64,
        z0_ref: f64,
        s11: Complex64,
        s12: Complex64,
        s21: Complex64,
        s22: Complex64,
    ) -> Self {
        Self {
            freq_hz,
            z0_ref,
            s11,
            s12,
            s21,
            s22,
        }
    }

    /// Input reflection magnitude $|S_{11}|$.
    #[inline]
    pub fn s11_mag(&self) -> f64 {
        self.s11.abs()
    }

    /// Forward transmission magnitude $|S_{21}|$.
    #[inline]
    pub fn s21_mag(&self) -> f64 {
        self.s21.abs()
    }

    /// Input Return Loss in decibels (dB):
    /// $$\text{RL}_{\text{dB}} = -20 \log_{10} |S_{11}|$$
    #[inline]
    pub fn return_loss_db(&self) -> f64 {
        let mag = self.s11.abs().max(1e-12);
        -20.0 * mag.log10()
    }

    /// Forward Insertion Loss in decibels (dB):
    /// $$\text{IL}_{\text{dB}} = -20 \log_{10} |S_{21}|$$
    #[inline]
    pub fn insertion_loss_db(&self) -> f64 {
        let mag = self.s21.abs().max(1e-12);
        -20.0 * mag.log10()
    }

    /// Voltage Standing Wave Ratio (VSWR) at Port 1:
    /// $$\text{VSWR} = \frac{1 + |S_{11}|}{1 - |S_{11}|}$$
    #[inline]
    pub fn vswr(&self) -> f64 {
        let gamma = self.s11.abs().min(0.999_999);
        (1.0 + gamma) / (1.0 - gamma)
    }

    /// Matrix determinant: $\Delta = S_{11} S_{22} - S_{12} S_{21}$.
    #[inline]
    pub fn delta(&self) -> Complex64 {
        self.s11.mul(self.s22).sub(self.s12.mul(self.s21))
    }

    /// Rollett's unconditional stability factor $K$:
    /// $$K = \frac{1 - |S_{11}|^2 - |S_{22}|^2 + |\Delta|^2}{2 |S_{12} S_{21}|}$$
    pub fn stability_factor_k(&self) -> f64 {
        let s11_sq = self.s11.norm_sq();
        let s22_sq = self.s22.norm_sq();
        let delta_sq = self.delta().norm_sq();
        let denom = 2.0 * (self.s12.mul(self.s21)).abs().max(1e-18);
        (1.0 - s11_sq - s22_sq + delta_sq) / denom
    }

    /// Converts 2-port Impedance matrix $[Z]$ to Scattering matrix $[S]$ with reference $Z_0$:
    /// $$\mathbf{S} = (\mathbf{Z} - Z_0 \mathbf{I})(\mathbf{Z} + Z_0 \mathbf{I})^{-1}$$
    pub fn from_z_matrix(z: [[Complex64; 2]; 2], z0: f64, freq_hz: f64) -> Self {
        let z0_c = Complex64::new(z0, 0.0);

        // Denominator: (Z11 + Z0)(Z22 + Z0) - Z12 * Z21
        let term1 = z[0][0].add(z0_c);
        let term2 = z[1][1].add(z0_c);
        let delta_z = term1.mul(term2).sub(z[0][1].mul(z[1][0]));

        // S11 = [(Z11 - Z0)(Z22 + Z0) - Z12 * Z21] / Delta
        let s11_num = z[0][0].sub(z0_c).mul(term2).sub(z[0][1].mul(z[1][0]));
        let s11 = s11_num.div(delta_z);

        // S12 = 2 * Z0 * Z12 / Delta
        let two_z0 = Complex64::new(2.0 * z0, 0.0);
        let s12 = two_z0.mul(z[0][1]).div(delta_z);

        // S21 = 2 * Z0 * Z21 / Delta
        let s21 = two_z0.mul(z[1][0]).div(delta_z);

        // S22 = [(Z11 + Z0)(Z22 - Z0) - Z12 * Z21] / Delta
        let s22_num = term1.mul(z[1][1].sub(z0_c)).sub(z[0][1].mul(z[1][0]));
        let s22 = s22_num.div(delta_z);

        Self::new(freq_hz, z0, s11, s12, s21, s22)
    }

    /// Converts 2-port Admittance matrix $[Y]$ to Scattering matrix $[S]$ with reference $Z_0$:
    /// Normalized admittance $\bar{y} = Z_0 \mathbf{Y}$
    pub fn from_y_matrix(y: [[Complex64; 2]; 2], z0: f64, freq_hz: f64) -> Self {
        let y0 = 1.0 / z0;
        let y0_c = Complex64::new(y0, 0.0);

        // Denominator: (Y11 + Y0)(Y22 + Y0) - Y12 * Y21
        let term1 = y[0][0].add(y0_c);
        let term2 = y[1][1].add(y0_c);
        let delta_y = term1.mul(term2).sub(y[0][1].mul(y[1][0]));

        // S11 = [(Y0 - Y11)(Y22 + Y0) + Y12 * Y21] / Delta
        let s11_num = y0_c.sub(y[0][0]).mul(term2).add(y[0][1].mul(y[1][0]));
        let s11 = s11_num.div(delta_y);

        // S12 = -2 * Y0 * Y12 / Delta
        let minus_two_y0 = Complex64::new(-2.0 * y0, 0.0);
        let s12 = minus_two_y0.mul(y[0][1]).div(delta_y);

        // S21 = -2 * Y0 * Y21 / Delta
        let s21 = minus_two_y0.mul(y[1][0]).div(delta_y);

        // S22 = [(Y11 + Y0)(Y0 - Y22) + Y12 * Y21] / Delta
        let s22_num = term1.mul(y0_c.sub(y[1][1])).add(y[0][1].mul(y[1][0]));
        let s22 = s22_num.div(delta_y);

        Self::new(freq_hz, z0, s11, s12, s21, s22)
    }

    /// Evaluates exact analytical S-parameters for an ideal lossless transmission line of impedance $Z_c$
    /// and electrical delay $\tau$, terminated into reference impedance $Z_0$:
    /// $$S_{11} = S_{22} = \frac{j (Z_c^2 - Z_0^2) \sin(\theta)}{2 Z_0 Z_c \cos(\theta) + j (Z_c^2 + Z_0^2) \sin(\theta)}$$
    /// $$S_{21} = S_{12} = \frac{2 Z_0 Z_c}{2 Z_0 Z_c \cos(\theta) + j (Z_c^2 + Z_0^2) \sin(\theta)}$$
    /// where $\theta = 2\pi f \tau$.
    pub fn for_transmission_line(zc: f64, tau: f64, z0: f64, freq_hz: f64) -> Self {
        let theta = 2.0 * PI * freq_hz * tau;
        let cos_th = theta.cos();
        let sin_th = theta.sin();

        let re_den = 2.0 * z0 * zc * cos_th;
        let im_den = (zc * zc + z0 * z0) * sin_th;
        let den = Complex64::new(re_den, im_den);

        let s11_num = Complex64::new(0.0, (zc * zc - z0 * z0) * sin_th);
        let s11 = s11_num.div(den);

        let s21_num = Complex64::new(2.0 * z0 * zc, 0.0);
        let s21 = s21_num.div(den);

        Self::new(freq_hz, z0, s11, s21, s21, s11)
    }

    /// Evaluates exact S-parameters for a symmetric Tee-attenuator network with series resistance $R_s$
    /// and shunt resistance $R_p$:
    pub fn for_tee_attenuator(r_series: f64, r_shunt: f64, z0: f64, freq_hz: f64) -> Self {
        // Z11 = Z22 = R_series + R_shunt
        // Z12 = Z21 = R_shunt
        let z11 = Complex64::new(r_series + r_shunt, 0.0);
        let z12 = Complex64::new(r_shunt, 0.0);
        let z = [[z11, z12], [z12, z11]];
        Self::from_z_matrix(z, z0, freq_hz)
    }

    /// Converts 2-port Scattering matrix $[S]$ back to Impedance matrix $[Z]$:
    /// $$\mathbf{Z} = Z_0 (\mathbf{I} + \mathbf{S})(\mathbf{I} - \mathbf{S})^{-1}$$
    pub fn to_z_matrix(&self) -> Result<[[Complex64; 2]; 2], &'static str> {
        let one = Complex64::ONE;
        // Denominator: (1 - S11)(1 - S22) - S12 * S21
        let delta_s = one
            .sub(self.s11)
            .mul(one.sub(self.s22))
            .sub(self.s12.mul(self.s21));
        if delta_s.norm_sq() < 1e-30 {
            return Err("Singular S-matrix: cannot invert (I - S)");
        }

        let z0_c = Complex64::from_real(self.z0_ref);
        let two_z0 = Complex64::from_real(2.0 * self.z0_ref);

        // Z11 = Z0 * [(1 + S11)(1 - S22) + S12 * S21] / Delta_s
        let z11_num = one
            .add(self.s11)
            .mul(one.sub(self.s22))
            .add(self.s12.mul(self.s21));
        let z11 = z0_c.mul(z11_num).div(delta_s);

        // Z12 = Z0 * [2 * S12] / Delta_s
        let z12 = two_z0.mul(self.s12).div(delta_s);

        // Z21 = Z0 * [2 * S21] / Delta_s
        let z21 = two_z0.mul(self.s21).div(delta_s);

        // Z22 = Z0 * [(1 - S11)(1 + S22) + S12 * S21] / Delta_s
        let z22_num = one
            .sub(self.s11)
            .mul(one.add(self.s22))
            .add(self.s12.mul(self.s21));
        let z22 = z0_c.mul(z22_num).div(delta_s);

        Ok([[z11, z12], [z21, z22]])
    }
}
