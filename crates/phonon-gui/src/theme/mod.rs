#![deny(unsafe_code)]

//! Centralized theme engine, visual palettes, and zero-dependency YAML parser/serializer for Phonon Studio.

use egui::Color32;
use std::path::PathBuf;

/// Complete color scheme specifying all visual styling across schematic canvas, CAD chrome, and components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhononTheme {
    pub name: String,
    pub canvas_bg: Color32,
    pub grid_dot: Color32,
    pub grid_line: Color32,
    pub wire_normal: Color32,
    pub wire_selected: Color32,
    pub component_body: Color32,
    pub component_stroke: Color32,
    pub component_selected: Color32,
    pub pin_normal: Color32,
    pub pin_connected: Color32,
    pub accent_primary: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub voltage_badge: Color32,
}

impl Default for PhononTheme {
    fn default() -> Self {
        ThemePreset::DarkNavy.to_theme()
    }
}

/// Built-in industry-grade color scheme presets for Phonon Visual Studio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemePreset {
    DarkNavy,
    LightPaper,
    HighContrastOled,
    MonokaiPro,
    CyberpunkNeon,
}

impl ThemePreset {
    /// Human-readable display label for preset selector UI.
    pub fn name(&self) -> &'static str {
        match self {
            Self::DarkNavy => "Dark Navy",
            Self::LightPaper => "Light Paper",
            Self::HighContrastOled => "High Contrast OLED",
            Self::MonokaiPro => "Monokai Pro",
            Self::CyberpunkNeon => "Cyberpunk Neon",
        }
    }

    /// List of all built-in presets.
    pub fn all() -> &'static [ThemePreset] {
        &[
            Self::DarkNavy,
            Self::LightPaper,
            Self::HighContrastOled,
            Self::MonokaiPro,
            Self::CyberpunkNeon,
        ]
    }

    /// Finds a preset matching a given string identifier.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_lowercase().as_str() {
            "dark navy" | "darknavy" | "navy" => Some(Self::DarkNavy),
            "light paper" | "lightpaper" | "light" | "paper" => Some(Self::LightPaper),
            "high contrast oled" | "oled" | "high contrast" => Some(Self::HighContrastOled),
            "monokai pro" | "monokai" => Some(Self::MonokaiPro),
            "cyberpunk neon" | "cyberpunk" | "neon" => Some(Self::CyberpunkNeon),
            _ => None,
        }
    }

    /// Instantiates the full `PhononTheme` specification for this preset.
    pub fn to_theme(&self) -> PhononTheme {
        match self {
            Self::DarkNavy => PhononTheme {
                name: "Dark Navy".to_string(),
                canvas_bg: Color32::from_rgb(16, 24, 38),
                grid_dot: Color32::from_rgb(74, 90, 110),
                grid_line: Color32::from_rgb(30, 42, 58),
                wire_normal: Color32::from_rgb(100, 220, 120),
                wire_selected: Color32::from_rgb(255, 180, 50),
                component_body: Color32::from_rgb(25, 35, 50),
                component_stroke: Color32::from_rgb(220, 230, 240),
                component_selected: Color32::from_rgb(255, 180, 50),
                pin_normal: Color32::from_rgb(80, 200, 255),
                pin_connected: Color32::from_rgb(100, 255, 160),
                accent_primary: Color32::from_rgb(60, 140, 240),
                text_primary: Color32::from_rgb(240, 245, 250),
                text_secondary: Color32::from_rgb(170, 185, 200),
                voltage_badge: Color32::from_rgb(100, 255, 160),
            },
            Self::LightPaper => PhononTheme {
                name: "Light Paper".to_string(),
                canvas_bg: Color32::from_rgb(248, 249, 250),
                grid_dot: Color32::from_rgb(208, 213, 221),
                grid_line: Color32::from_rgb(228, 231, 236),
                wire_normal: Color32::from_rgb(21, 128, 61),
                wire_selected: Color32::from_rgb(217, 119, 6),
                component_body: Color32::from_rgb(240, 242, 245),
                component_stroke: Color32::from_rgb(31, 41, 55),
                component_selected: Color32::from_rgb(217, 119, 6),
                pin_normal: Color32::from_rgb(2, 132, 199),
                pin_connected: Color32::from_rgb(22, 163, 74),
                accent_primary: Color32::from_rgb(37, 99, 235),
                text_primary: Color32::from_rgb(17, 24, 39),
                text_secondary: Color32::from_rgb(107, 114, 128),
                voltage_badge: Color32::from_rgb(22, 163, 74),
            },
            Self::HighContrastOled => PhononTheme {
                name: "High Contrast OLED".to_string(),
                canvas_bg: Color32::from_rgb(0, 0, 0),
                grid_dot: Color32::from_rgb(51, 51, 51),
                grid_line: Color32::from_rgb(26, 26, 26),
                wire_normal: Color32::from_rgb(0, 255, 102),
                wire_selected: Color32::from_rgb(255, 255, 0),
                component_body: Color32::from_rgb(10, 10, 10),
                component_stroke: Color32::from_rgb(255, 255, 255),
                component_selected: Color32::from_rgb(255, 255, 0),
                pin_normal: Color32::from_rgb(0, 255, 255),
                pin_connected: Color32::from_rgb(0, 255, 102),
                accent_primary: Color32::from_rgb(0, 136, 255),
                text_primary: Color32::from_rgb(255, 255, 255),
                text_secondary: Color32::from_rgb(204, 204, 204),
                voltage_badge: Color32::from_rgb(0, 255, 102),
            },
            Self::MonokaiPro => PhononTheme {
                name: "Monokai Pro".to_string(),
                canvas_bg: Color32::from_rgb(45, 42, 46),
                grid_dot: Color32::from_rgb(85, 81, 86),
                grid_line: Color32::from_rgb(62, 58, 63),
                wire_normal: Color32::from_rgb(169, 220, 118),
                wire_selected: Color32::from_rgb(252, 152, 103),
                component_body: Color32::from_rgb(38, 35, 39),
                component_stroke: Color32::from_rgb(253, 249, 243),
                component_selected: Color32::from_rgb(252, 152, 103),
                pin_normal: Color32::from_rgb(120, 220, 232),
                pin_connected: Color32::from_rgb(169, 220, 118),
                accent_primary: Color32::from_rgb(255, 97, 136),
                text_primary: Color32::from_rgb(252, 252, 250),
                text_secondary: Color32::from_rgb(147, 146, 147),
                voltage_badge: Color32::from_rgb(255, 216, 102),
            },
            Self::CyberpunkNeon => PhononTheme {
                name: "Cyberpunk Neon".to_string(),
                canvas_bg: Color32::from_rgb(13, 2, 33),
                grid_dot: Color32::from_rgb(46, 16, 101),
                grid_line: Color32::from_rgb(30, 8, 64),
                wire_normal: Color32::from_rgb(0, 240, 255),
                wire_selected: Color32::from_rgb(255, 230, 0),
                component_body: Color32::from_rgb(24, 7, 50),
                component_stroke: Color32::from_rgb(255, 0, 127),
                component_selected: Color32::from_rgb(255, 230, 0),
                pin_normal: Color32::from_rgb(123, 44, 191),
                pin_connected: Color32::from_rgb(0, 240, 255),
                accent_primary: Color32::from_rgb(255, 0, 127),
                text_primary: Color32::from_rgb(255, 255, 255),
                text_secondary: Color32::from_rgb(224, 170, 255),
                voltage_badge: Color32::from_rgb(255, 230, 0),
            },
        }
    }
}

/// Converts a `Color32` to standard hex string format `#RRGGBB`.
pub fn color32_to_hex(color: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

/// Parses a hex color string (`#RRGGBB`, `RRGGBB`, `#RGB`, `#RRGGBBAA`) into `Color32`.
pub fn hex_to_color32(hex: &str) -> Option<Color32> {
    let s = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Color32::from_rgb(r, g, b))
    } else if s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        let a = u8::from_str_radix(&s[6..8], 16).ok()?;
        Some(Color32::from_rgba_unmultiplied(r, g, b, a))
    } else if s.len() == 3 {
        let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
        let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
        let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
        Some(Color32::from_rgb(r, g, b))
    } else {
        None
    }
}

impl PhononTheme {
    /// Serializes this theme into human-readable YAML configuration string.
    pub fn to_yaml_string(&self) -> String {
        format!(
            "name: \"{}\"\n\
             canvas_bg: \"{}\"\n\
             grid_dot: \"{}\"\n\
             grid_line: \"{}\"\n\
             wire_normal: \"{}\"\n\
             wire_selected: \"{}\"\n\
             component_body: \"{}\"\n\
             component_stroke: \"{}\"\n\
             component_selected: \"{}\"\n\
             pin_normal: \"{}\"\n\
             pin_connected: \"{}\"\n\
             accent_primary: \"{}\"\n\
             text_primary: \"{}\"\n\
             text_secondary: \"{}\"\n\
             voltage_badge: \"{}\"\n",
            self.name,
            color32_to_hex(self.canvas_bg),
            color32_to_hex(self.grid_dot),
            color32_to_hex(self.grid_line),
            color32_to_hex(self.wire_normal),
            color32_to_hex(self.wire_selected),
            color32_to_hex(self.component_body),
            color32_to_hex(self.component_stroke),
            color32_to_hex(self.component_selected),
            color32_to_hex(self.pin_normal),
            color32_to_hex(self.pin_connected),
            color32_to_hex(self.accent_primary),
            color32_to_hex(self.text_primary),
            color32_to_hex(self.text_secondary),
            color32_to_hex(self.voltage_badge),
        )
    }

    /// Robust, zero-dependency YAML deserializer for theme configurations.
    pub fn from_yaml_str(yaml: &str) -> Result<Self, String> {
        let mut default = ThemePreset::DarkNavy.to_theme();
        let mut name = None;

        for line in yaml.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim();
                let mut val = val.trim();
                if (val.starts_with('"') && val.ends_with('"'))
                    || (val.starts_with('\'') && val.ends_with('\''))
                {
                    if val.len() >= 2 {
                        val = &val[1..val.len() - 1];
                    }
                }

                match key {
                    "name" => name = Some(val.to_string()),
                    "canvas_bg" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.canvas_bg = c;
                        }
                    }
                    "grid_dot" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.grid_dot = c;
                        }
                    }
                    "grid_line" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.grid_line = c;
                        }
                    }
                    "wire_normal" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.wire_normal = c;
                        }
                    }
                    "wire_selected" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.wire_selected = c;
                        }
                    }
                    "component_body" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.component_body = c;
                        }
                    }
                    "component_stroke" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.component_stroke = c;
                        }
                    }
                    "component_selected" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.component_selected = c;
                        }
                    }
                    "pin_normal" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.pin_normal = c;
                        }
                    }
                    "pin_connected" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.pin_connected = c;
                        }
                    }
                    "accent_primary" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.accent_primary = c;
                        }
                    }
                    "text_primary" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.text_primary = c;
                        }
                    }
                    "text_secondary" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.text_secondary = c;
                        }
                    }
                    "voltage_badge" => {
                        if let Some(c) = hex_to_color32(val) {
                            default.voltage_badge = c;
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(n) = name {
            default.name = n;
        }

        Ok(default)
    }
}

/// Directory path where custom user YAML themes are stored (`~/.phonon/themes/`).
pub fn themes_dir() -> PathBuf {
    #[cfg(target_arch = "wasm32")]
    {
        PathBuf::from("/themes")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".phonon").join("themes")
        } else {
            std::env::temp_dir().join("phonon").join("themes")
        }
    }
}

/// Scans the themes directory for user-defined `.yaml` and `.yml` theme files.
pub fn discover_custom_themes() -> Vec<PhononTheme> {
    #[cfg(target_arch = "wasm32")]
    {
        Vec::new()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut custom = Vec::new();
        let dir = themes_dir();
        if dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(theme) = PhononTheme::from_yaml_str(&content) {
                                    custom.push(theme);
                                }
                            }
                        }
                    }
                }
            }
        }
        custom
    }
}

/// Writes a custom theme YAML file into `~/.phonon/themes/`.
pub fn save_custom_theme(theme: &PhononTheme) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = theme;
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dir = themes_dir();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create themes directory {}: {}", dir.display(), e))?;

        let clean_filename = theme
            .name
            .to_lowercase()
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();
        let file_path = dir.join(format!("{}.yaml", clean_filename));

        let yaml = theme.to_yaml_string();
        std::fs::write(&file_path, yaml)
            .map_err(|e| format!("Failed to write theme file {}: {}", file_path.display(), e))
    }
}
