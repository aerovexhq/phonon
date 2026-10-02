#![deny(unsafe_code)]

//! Articulatory Vocal Tract Area Function Vowel Morpher.
//!
//! Generates canonical cross-sectional area profiles for cardinal vowels (/a/, /i/, /u/, /e/, /o/)
//! and executes smooth exponential interpolation between articulatory gestures without
//! acoustic impedance discontinuities.

/// Vocal tract articulatory vowel morpher.
#[derive(Debug, Clone, PartialEq)]
pub struct VowelMorpher {
    /// Active target cardinal vowel index (0: /a/, 1: /i/, 2: /u/, 3: /e/, 4: /o/).
    pub target_vowel_idx: usize,
    /// Vocal tract morphing interpolation speed in s^-1.
    pub morph_speed: f64,
    /// Number of spatial tract sections (default 20).
    pub num_sections: usize,
    /// Current cross-sectional area profile along tract in m^2.
    pub current_areas: Vec<f64>,
    /// Target cross-sectional area profile along tract in m^2.
    pub target_areas: Vec<f64>,
}

impl VowelMorpher {
    /// Creates a new vowel morpher initialized to the specified target vowel.
    pub fn new(target_vowel_idx: usize, morph_speed: f64) -> Self {
        let num_sections = 20;
        let v_idx = target_vowel_idx.min(4);
        let speed = morph_speed.clamp(0.1, 50.0);
        let target = Self::cardinal_profile_m2(v_idx, num_sections);
        let current = target.clone();

        Self {
            target_vowel_idx: v_idx,
            morph_speed: speed,
            num_sections,
            current_areas: current,
            target_areas: target,
        }
    }

    /// Sets the cardinal target vowel index and updates the target area profile.
    pub fn set_target_vowel(&mut self, vowel_idx: usize) {
        let v_idx = vowel_idx.min(4);
        self.target_vowel_idx = v_idx;
        self.target_areas = Self::cardinal_profile_m2(v_idx, self.num_sections);
    }

    /// Updates the morphing interpolation speed in s^-1.
    pub fn set_morph_speed(&mut self, speed: f64) {
        self.morph_speed = speed.clamp(0.1, 50.0);
    }

    /// Steps the area interpolation smoothly toward target areas by time step dt in seconds.
    ///
    /// Executes monotonic first-order exponential relaxation:
    /// A(x, t + dt) = A(x, t) + (1 - exp(-r * dt)) * (A_target(x) - A(x, t))
    #[inline(always)]
    pub fn step(&mut self, dt: f64) {
        let alpha = (1.0 - (-self.morph_speed * dt).exp()).clamp(0.0, 1.0);
        for i in 0..self.num_sections {
            self.current_areas[i] += alpha * (self.target_areas[i] - self.current_areas[i]);
        }
    }

    /// Returns the current vocal tract area profile in m^2.
    #[inline(always)]
    pub fn current_areas(&self) -> &[f64] {
        &self.current_areas
    }

    /// Returns the current vocal tract area profile in m^2.
    #[inline(always)]
    pub fn current_areas_m2(&self) -> &[f64] {
        &self.current_areas
    }

    /// Returns the current vocal tract area profile converted to cm^2.
    pub fn current_areas_cm2(&self) -> Vec<f64> {
        self.current_areas.iter().map(|&a| a * 1.0e4).collect()
    }

    /// Returns the target vocal tract area profile converted to cm^2.
    pub fn target_areas_cm2(&self) -> Vec<f64> {
        self.target_areas.iter().map(|&a| a * 1.0e4).collect()
    }

    /// Resamples the current continuous area profile to an arbitrary section count
    /// (e.g. 10 sections or 24 sections for RiccatiWebsterHorn) via linear interpolation.
    pub fn interpolate_to_sections(&self, new_sections: usize) -> Vec<f64> {
        if new_sections == self.num_sections {
            return self.current_areas.clone();
        }
        let mut out = Vec::with_capacity(new_sections);
        let n_curr = self.num_sections;
        for j in 0..new_sections {
            let u = (j as f64 + 0.5) / (new_sections as f64);
            let src_idx_f = u * (n_curr as f64) - 0.5;
            let i0 = (src_idx_f.floor() as isize).clamp(0, (n_curr - 1) as isize) as usize;
            let i1 = (i0 + 1).min(n_curr - 1);
            let frac = (src_idx_f - (i0 as f64)).clamp(0.0, 1.0);
            let area = self.current_areas[i0] * (1.0 - frac) + self.current_areas[i1] * frac;
            out.push(area);
        }
        out
    }

    /// Evaluates the index of the tract constriction (minimum cross-sectional area).
    pub fn constriction_index(profile: &[f64]) -> usize {
        let mut min_idx = 0;
        let mut min_val = f64::MAX;
        for (i, &val) in profile.iter().enumerate() {
            if val < min_val {
                min_val = val;
                min_idx = i;
            }
        }
        min_idx
    }

    /// Evaluates the minimum cross-sectional area in the profile.
    pub fn constriction_area(profile: &[f64]) -> f64 {
        let mut min_val = f64::MAX;
        for &val in profile {
            if val < min_val {
                min_val = val;
            }
        }
        min_val
    }

    /// Evaluates normalized position (in [0.0, 1.0]) of the tract constriction.
    pub fn constriction_normalized_pos(profile: &[f64]) -> f64 {
        let idx = Self::constriction_index(profile);
        (idx as f64 + 0.5) / (profile.len() as f64)
    }

    /// Smooth landmark interpolation along the tract coordinate s in [0.0, 1.0].
    fn interpolate_landmarks(s: f64, landmarks: &[(f64, f64)]) -> f64 {
        let s_clamped = s.clamp(0.0, 1.0);
        if s_clamped <= landmarks[0].0 {
            return landmarks[0].1;
        }
        let last = landmarks.len() - 1;
        if s_clamped >= landmarks[last].0 {
            return landmarks[last].1;
        }

        for i in 0..last {
            let (s0, a0) = landmarks[i];
            let (s1, a1) = landmarks[i + 1];
            if s_clamped >= s0 && s_clamped <= s1 {
                let u = (s_clamped - s0) / (s1 - s0);
                // Smooth cosine blend with zero derivatives at landmarks
                let w = (1.0 - (std::f64::consts::PI * u).cos()) * 0.5;
                return a0 * (1.0 - w) + a1 * w;
            }
        }
        landmarks[last].1
    }

    /// Generates canonical cardinal area profiles in cm^2 for the 5 vowels across num_sections.
    ///
    /// Profiles:
    /// - 0: /a/ (open back: pharynx ~0.5 cm^2, mouth ~6.0 cm^2)
    /// - 1: /i/ (close front: pharynx ~8.0 cm^2, oral constriction ~0.5 cm^2)
    /// - 2: /u/ (close back rounded: velum ~0.8 cm^2, lips ~0.3 cm^2)
    /// - 3: /e/ (mid front: pharynx ~4.0 cm^2, palatal ~1.5 cm^2)
    /// - 4: /o/ (mid back: pharynx ~1.5 cm^2, oral ~3.0 cm^2, lips ~0.6 cm^2)
    pub fn cardinal_profile_cm2(vowel_idx: usize, num_sections: usize) -> Vec<f64> {
        let n = num_sections.max(2);
        let landmarks: &[(f64, f64)] = match vowel_idx {
            0 => &[
                // /a/ (open back): pharynx ~0.5 cm^2, mouth ~6.0 cm^2
                (0.00, 0.60),
                (0.15, 0.50), // pharynx constriction
                (0.35, 1.20),
                (0.55, 3.20),
                (0.75, 5.80),
                (1.00, 6.00), // wide mouth
            ],
            1 => &[
                // /i/ (close front): pharynx ~8.0 cm^2, oral constriction ~0.5 cm^2
                (0.00, 7.50),
                (0.15, 8.00), // wide pharynx
                (0.40, 6.00),
                (0.60, 2.00),
                (0.75, 0.50), // oral constriction
                (1.00, 2.00), // lips opening
            ],
            2 => &[
                // /u/ (close back rounded): velum ~0.8 cm^2, lips ~0.3 cm^2
                (0.00, 2.80),
                (0.20, 3.00), // pharynx
                (0.50, 0.80), // velum constriction
                (0.75, 2.50), // oral cavity
                (0.90, 1.20),
                (0.93, 0.30), // lips constriction
                (1.00, 0.30),
            ],
            3 => &[
                // /e/ (mid front): pharynx ~4.0 cm^2, palatal ~1.5 cm^2
                (0.00, 3.80),
                (0.20, 4.00), // pharynx
                (0.45, 3.00),
                (0.70, 1.50), // palatal constriction
                (0.90, 3.00),
                (1.00, 3.50), // mouth
            ],
            4 => &[
                // /o/ (mid back): pharynx ~1.5 cm^2, oral ~3.0 cm^2, lips ~0.6 cm^2
                (0.00, 1.80),
                (0.20, 1.50), // pharynx constriction
                (0.60, 3.00), // oral cavity
                (0.85, 1.80),
                (0.93, 0.60),
                (1.00, 0.60), // lips constriction
            ],
            _ => &[
                (0.00, 2.00),
                (1.00, 2.00),
            ],
        };

        let mut profile = Vec::with_capacity(n);
        for k in 0..n {
            let s = (k as f64 + 0.5) / (n as f64);
            profile.push(Self::interpolate_landmarks(s, landmarks));
        }

        profile
    }

    /// Generates canonical cardinal area profiles in m^2 for the 5 vowels across num_sections.
    pub fn cardinal_profile_m2(vowel_idx: usize, num_sections: usize) -> Vec<f64> {
        Self::cardinal_profile_cm2(vowel_idx, num_sections)
            .into_iter()
            .map(|cm2| cm2 * 1.0e-4)
            .collect()
    }
}
