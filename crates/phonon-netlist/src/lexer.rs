//! High-speed lexer and SPICE engineering number parser.

use crate::error::NetlistError;

/// Parses a SPICE numeric literal with optional engineering unit suffix into `f64`.
///
/// Follows SPICE 3f5 conventions:
/// - `T`   = 1e12 (Tera)
/// - `G`   = 1e9  (Giga)
/// - `MEG` = 1e6  (Mega)
/// - `K`   = 1e3  (Kilo)
/// - `MIL` = 25.4e-6 (Mils to meters)
/// - `M`   = 1e-3 (Milli) [distinct from MEG]
/// - `U`   = 1e-6 (Micro)
/// - `N`   = 1e-9 (Nano)
/// - `P`   = 1e-12 (Pico)
/// - `F`   = 1e-15 (Femto)
pub fn parse_spice_number(s: &str, line: usize) -> Result<f64, NetlistError> {
    let raw = s.trim();
    if raw.is_empty() {
        return Err(NetlistError::InvalidNumber {
            line,
            literal: s.to_string(),
            detail: "Empty string".to_string(),
        });
    }

    // Direct float parse attempt (handles pure numbers and scientific notation like 1e-3)
    if let Ok(val) = raw.parse::<f64>() {
        return Ok(val);
    }

    let lower = raw.to_ascii_lowercase();

    // Find where the numeric part ends and the engineering prefix begins
    // E.g., "100k", "1.5meg", "10uF", "-2.5m"
    let bytes = lower.as_bytes();
    let mut num_end = 0;
    let mut in_exponent = false;

    for (i, &b) in bytes.iter().enumerate() {
        if b.is_ascii_digit() || b == b'.' {
            num_end = i + 1;
        } else if (b == b'+' || b == b'-') && (i == 0 || in_exponent) {
            num_end = i + 1;
            in_exponent = false;
        } else if b == b'e' && i > 0 && bytes[i - 1].is_ascii_digit() {
            // Could be scientific exponent (e.g. 1e-3) or engineering prefix
            // Look ahead: if next char is digit or +/- followed by digit, it's scientific exponent
            if i + 1 < bytes.len()
                && (bytes[i + 1].is_ascii_digit()
                    || ((bytes[i + 1] == b'+' || bytes[i + 1] == b'-')
                        && i + 2 < bytes.len()
                        && bytes[i + 2].is_ascii_digit()))
            {
                num_end = i + 1;
                in_exponent = true;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if num_end == 0 {
        return Err(NetlistError::InvalidNumber {
            line,
            literal: s.to_string(),
            detail: "No numeric base found".to_string(),
        });
    }

    let base_str = &lower[..num_end];
    let suffix = &lower[num_end..];

    let base_val = base_str
        .parse::<f64>()
        .map_err(|e| NetlistError::InvalidNumber {
            line,
            literal: s.to_string(),
            detail: e.to_string(),
        })?;

    let mult = if suffix.starts_with("meg") {
        1e6
    } else if suffix.starts_with("mil") {
        25.4e-6
    } else if suffix.starts_with('m') {
        1e-3
    } else if suffix.starts_with('k') {
        1e3
    } else if suffix.starts_with('u') {
        1e-6
    } else if suffix.starts_with('n') {
        1e-9
    } else if suffix.starts_with('p') {
        1e-12
    } else if suffix.starts_with('f') {
        1e-15
    } else if suffix.starts_with('g') {
        1e9
    } else if suffix.starts_with('t') {
        1e12
    } else {
        1.0
    };

    Ok(base_val * mult)
}

/// Normalizes SPICE netlist text by:
/// 1. Stripping full-line and inline comments (`*`, `;`)
/// 2. Merging line continuations (`+`)
/// 3. Preserving line numbers for accurate error reporting.
pub fn preprocess_netlist(input: &str) -> Vec<(usize, String)> {
    let mut normalized = Vec::new();
    let mut current_line_num = 0;
    let mut current_line = String::new();

    for (line_idx, raw_line) in input.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = raw_line.trim();

        // Skip comment lines starting with '*' or ';'
        if trimmed.starts_with('*') || trimmed.starts_with(';') {
            continue;
        }

        // Strip inline comments starting with ';'
        let content = if let Some(idx) = trimmed.find(';') {
            trimmed[..idx].trim()
        } else {
            trimmed
        };

        if content.is_empty() {
            continue;
        }

        if let Some(stripped) = content.strip_prefix('+') {
            // Line continuation: append remainder of line
            let continuation = stripped.trim();
            if !continuation.is_empty() {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(continuation);
            }
        } else {
            // Emit previous accumulated line if any
            if !current_line.is_empty() {
                normalized.push((current_line_num, current_line.clone()));
            }
            current_line_num = line_num;
            current_line = content.to_string();
        }
    }

    if !current_line.is_empty() {
        normalized.push((current_line_num, current_line));
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spice_number_parsing() {
        assert_eq!(parse_spice_number("10k", 1).unwrap(), 10e3);
        assert_eq!(parse_spice_number("10Kohm", 1).unwrap(), 10e3);
        assert_eq!(parse_spice_number("1.5meg", 1).unwrap(), 1.5e6);
        assert_eq!(parse_spice_number("10m", 1).unwrap(), 10e-3); // milli
        assert!((parse_spice_number("10u", 1).unwrap() - 10e-6).abs() < 1e-15);
        assert_eq!(parse_spice_number("100p", 1).unwrap(), 100e-12);
        assert_eq!(parse_spice_number("22f", 1).unwrap(), 22e-15);
        assert_eq!(parse_spice_number("2.5g", 1).unwrap(), 2.5e9);
        assert_eq!(parse_spice_number("1t", 1).unwrap(), 1e12);
        assert_eq!(parse_spice_number("-5.0", 1).unwrap(), -5.0);
        assert_eq!(parse_spice_number("1.23e-4", 1).unwrap(), 1.23e-4);
    }

    #[test]
    fn test_line_continuations_and_comments() {
        let netlist = "
* Test Netlist
R1 in out 10k
+ ; continuation line comment
+ 0.5
C1 out 0 100p ; inline comment
";
        let lines = preprocess_netlist(netlist);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].1, "R1 in out 10k 0.5");
        assert_eq!(lines[1].1, "C1 out 0 100p");
    }
}
