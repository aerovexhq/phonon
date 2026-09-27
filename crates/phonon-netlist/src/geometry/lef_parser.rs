//! Library Exchange Format (LEF) parser for physical macro geometry and pin definitions.

/// 2D bounding rectangle in microns: [x1, y1] to [x2, y2].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LefRect {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl LefRect {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1, y1, x2, y2 }
    }

    pub fn width(&self) -> f64 {
        (self.x2 - self.x1).abs()
    }

    pub fn height(&self) -> f64 {
        (self.y2 - self.y1).abs()
    }

    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }
}

/// Logical electrical signal direction of a pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinDirection {
    Input,
    Output,
    Inout,
    Power,
    Ground,
}

/// A physical contact pin in a cell macro.
#[derive(Debug, Clone, PartialEq)]
pub struct LefPin {
    pub name: String,
    pub direction: PinDirection,
    pub layer: String,
    pub rects: Vec<LefRect>,
}

/// A standard cell or macro defined in a LEF library.
#[derive(Debug, Clone, PartialEq)]
pub struct LefMacro {
    pub name: String,
    pub class: String,
    pub origin_x: f64,
    pub origin_y: f64,
    pub width: f64,
    pub height: f64,
    pub pins: Vec<LefPin>,
}

impl LefMacro {
    pub fn get_pin(&self, name: &str) -> Option<&LefPin> {
        self.pins.iter().find(|p| p.name == name)
    }
}

/// Parsed LEF library containing macros and geometries.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LefLibrary {
    pub macros: Vec<LefMacro>,
}

impl LefLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses a LEF ASCII string into a structured LefLibrary.
    pub fn parse_from_str(content: &str) -> Result<Self, String> {
        let mut library = LefLibrary::new();
        let tokens: Vec<&str> = content.split_whitespace().collect();
        let mut idx = 0;

        while idx < tokens.len() {
            let tok = tokens[idx];
            if tok.eq_ignore_ascii_case("MACRO") && idx + 1 < tokens.len() {
                idx += 1;
                let macro_name = tokens[idx];
                let mut macro_obj = LefMacro {
                    name: macro_name.to_string(),
                    class: "CORE".to_string(),
                    origin_x: 0.0,
                    origin_y: 0.0,
                    width: 0.0,
                    height: 0.0,
                    pins: Vec::new(),
                };
                idx += 1;

                // Parse MACRO block
                while idx < tokens.len() {
                    let m_tok = tokens[idx];
                    if m_tok.eq_ignore_ascii_case("END")
                        && idx + 1 < tokens.len()
                        && tokens[idx + 1] == macro_name
                    {
                        idx += 2;
                        break;
                    }

                    if m_tok.eq_ignore_ascii_case("ORIGIN") && idx + 2 < tokens.len() {
                        macro_obj.origin_x = tokens[idx + 1].parse::<f64>().unwrap_or(0.0);
                        macro_obj.origin_y = tokens[idx + 2].parse::<f64>().unwrap_or(0.0);
                        idx += 3;
                        continue;
                    }

                    if m_tok.eq_ignore_ascii_case("SIZE")
                        && idx + 3 < tokens.len()
                        && tokens[idx + 2].eq_ignore_ascii_case("BY")
                    {
                        macro_obj.width = tokens[idx + 1].parse::<f64>().unwrap_or(0.0);
                        macro_obj.height = tokens[idx + 3].parse::<f64>().unwrap_or(0.0);
                        idx += 4;
                        continue;
                    }

                    if m_tok.eq_ignore_ascii_case("PIN") && idx + 1 < tokens.len() {
                        idx += 1;
                        let pin_name = tokens[idx];
                        let mut pin_dir = PinDirection::Inout;
                        let mut pin_layer = "met1".to_string();
                        let mut pin_rects = Vec::new();
                        idx += 1;

                        while idx < tokens.len() {
                            let p_tok = tokens[idx];
                            if p_tok.eq_ignore_ascii_case("END")
                                && idx + 1 < tokens.len()
                                && tokens[idx + 1] == pin_name
                            {
                                idx += 2;
                                break;
                            }

                            if p_tok.eq_ignore_ascii_case("DIRECTION") && idx + 1 < tokens.len() {
                                let dir_str = tokens[idx + 1].to_uppercase();
                                pin_dir = match dir_str.as_str() {
                                    "INPUT" => PinDirection::Input,
                                    "OUTPUT" => PinDirection::Output,
                                    "INOUT" => PinDirection::Inout,
                                    "POWER" => PinDirection::Power,
                                    "GROUND" => PinDirection::Ground,
                                    _ => PinDirection::Inout,
                                };
                                idx += 2;
                                continue;
                            }

                            if p_tok.eq_ignore_ascii_case("LAYER") && idx + 1 < tokens.len() {
                                pin_layer = tokens[idx + 1].to_string();
                                idx += 2;
                                continue;
                            }

                            if p_tok.eq_ignore_ascii_case("RECT") && idx + 4 < tokens.len() {
                                let x1 = tokens[idx + 1].parse::<f64>().unwrap_or(0.0);
                                let y1 = tokens[idx + 2].parse::<f64>().unwrap_or(0.0);
                                let x2 = tokens[idx + 3].parse::<f64>().unwrap_or(0.0);
                                let y2 = tokens[idx + 4].parse::<f64>().unwrap_or(0.0);
                                pin_rects.push(LefRect::new(x1, y1, x2, y2));
                                idx += 5;
                                continue;
                            }

                            idx += 1;
                        }

                        macro_obj.pins.push(LefPin {
                            name: pin_name.to_string(),
                            direction: pin_dir,
                            layer: pin_layer,
                            rects: pin_rects,
                        });
                        continue;
                    }

                    idx += 1;
                }

                library.macros.push(macro_obj);
            } else {
                idx += 1;
            }
        }

        Ok(library)
    }
}
