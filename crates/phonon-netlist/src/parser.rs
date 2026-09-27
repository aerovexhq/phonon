//! Recursive-descent parser translating preprocessed SPICE lines into `ParsedNetlist` AST.

use crate::ast::*;
use crate::error::NetlistError;
use crate::lexer::{parse_spice_number, preprocess_netlist};
use std::collections::HashMap;

/// Parses a SPICE netlist string into a structured `ParsedNetlist` AST.
pub fn parse_netlist(input: &str) -> Result<ParsedNetlist, NetlistError> {
    let mut netlist = ParsedNetlist::default();

    // Check if the very first non-empty raw line is a title comment (* ...)
    let first_raw = input.lines().map(|l| l.trim()).find(|l| !l.is_empty());
    let mut title_found = false;
    if let Some(first) = first_raw {
        if first.starts_with('*') {
            netlist.title = first.trim_start_matches('*').trim().to_string();
            title_found = true;
        }
    }

    let lines = preprocess_netlist(input);
    if lines.is_empty() {
        return Ok(netlist);
    }

    let mut current_subcircuit: Option<SubcircuitDef> = None;
    let mut start_idx = 0;

    // If no comment title was found, check if the first preprocessed line is a title
    if !title_found {
        let (first_line_num, first_line) = &lines[0];
        if !first_line.starts_with('.') {
            // Test if it parses as a valid component statement
            let mut test_netlist = ParsedNetlist::default();
            let mut test_subckt = None;
            if parse_statement(
                &mut test_netlist,
                &mut test_subckt,
                first_line,
                *first_line_num,
            )
            .is_err()
            {
                // Not a valid statement -> it is the title!
                netlist.title = first_line.clone();
                start_idx = 1;
            }
        }
    }

    for (line_num, line) in lines.iter().skip(start_idx) {
        parse_statement(&mut netlist, &mut current_subcircuit, line, *line_num)?;
    }

    if let Some(subckt) = current_subcircuit {
        return Err(NetlistError::SyntaxError {
            line: lines.last().map(|(l, _)| *l).unwrap_or(0),
            message: format!("Unterminated .SUBCKT '{}': missing .ENDS", subckt.name),
        });
    }

    Ok(netlist)
}

fn parse_statement(
    netlist: &mut ParsedNetlist,
    current_subcircuit: &mut Option<SubcircuitDef>,
    line: &str,
    line_num: usize,
) -> Result<(), NetlistError> {
    let tokens = tokenize_line(line);
    if tokens.is_empty() {
        return Ok(());
    }

    let first = &tokens[0];

    // 1. Directives (.MODEL, .SUBCKT, .ENDS, .OP, .DC, .TRAN, .TEMP)
    if first.starts_with('.') {
        let directive_name = first.to_ascii_uppercase();
        match directive_name.as_str() {
            ".MODEL" => {
                let model_card = parse_model_card(&tokens, line_num)?;
                netlist.models.insert(model_card.name.clone(), model_card);
            }
            ".SUBCKT" => {
                if current_subcircuit.is_some() {
                    return Err(NetlistError::SyntaxError {
                        line: line_num,
                        message: "Nested .SUBCKT definitions are not supported".to_string(),
                    });
                }
                if tokens.len() < 2 {
                    return Err(NetlistError::SyntaxError {
                        line: line_num,
                        message: "Expected subcircuit name after .SUBCKT".to_string(),
                    });
                }
                let name = tokens[1].to_ascii_uppercase();
                let pins = tokens[2..].iter().map(|s| s.to_string()).collect();
                *current_subcircuit = Some(SubcircuitDef {
                    name,
                    pins,
                    components: Vec::new(),
                });
            }
            ".ENDS" => {
                if let Some(subckt) = current_subcircuit.take() {
                    netlist.subcircuits.insert(subckt.name.clone(), subckt);
                } else {
                    return Err(NetlistError::SyntaxError {
                        line: line_num,
                        message: ".ENDS encountered without matching .SUBCKT".to_string(),
                    });
                }
            }
            ".OP" => {
                netlist.directives.push(Directive::Op);
            }
            ".DC" => {
                if tokens.len() < 5 {
                    return Err(NetlistError::SyntaxError {
                        line: line_num,
                        message: "Expected: .DC <source> <start> <stop> <step>".to_string(),
                    });
                }
                let source_name = tokens[1].clone();
                let start = parse_spice_number(&tokens[2], line_num)?;
                let stop = parse_spice_number(&tokens[3], line_num)?;
                let step = parse_spice_number(&tokens[4], line_num)?;
                netlist.directives.push(Directive::Dc {
                    source_name,
                    start,
                    stop,
                    step,
                });
            }
            ".TRAN" => {
                if tokens.len() < 3 {
                    return Err(NetlistError::SyntaxError {
                        line: line_num,
                        message: "Expected: .TRAN <tstep> <tstop>".to_string(),
                    });
                }
                let tstep = parse_spice_number(&tokens[1], line_num)?;
                let tstop = parse_spice_number(&tokens[2], line_num)?;
                netlist.directives.push(Directive::Tran { tstep, tstop });
            }
            ".TEMP" => {
                if tokens.len() < 2 {
                    return Err(NetlistError::SyntaxError {
                        line: line_num,
                        message: "Expected: .TEMP <degrees_C>".to_string(),
                    });
                }
                let temp_c = parse_spice_number(&tokens[1], line_num)?;
                netlist.directives.push(Directive::Temp { temp_c });
            }
            _ => {
                // Ignore unknown directives (.GLOBAL, .END, etc.) gracefully
            }
        }
        return Ok(());
    }

    // 2. Component Statement
    let comp = parse_component(&tokens, line_num)?;
    if let Some(subckt) = current_subcircuit {
        subckt.components.push(comp);
    } else {
        netlist.components.push(comp);
    }

    Ok(())
}

fn parse_component(tokens: &[String], line_num: usize) -> Result<ComponentAst, NetlistError> {
    let name = tokens[0].clone();
    let prefix = name.chars().next().unwrap().to_ascii_uppercase();

    match prefix {
        'R' => {
            if tokens.len() < 4 {
                return Err(NetlistError::SyntaxError {
                    line: line_num,
                    message: format!("Expected: {name} <pos> <neg> <resistance>"),
                });
            }
            let pos = tokens[1].clone();
            let neg = tokens[2].clone();
            let resistance = parse_spice_number(&tokens[3], line_num)?;
            Ok(ComponentAst::Resistor {
                name,
                pos,
                neg,
                resistance,
            })
        }
        'C' => {
            if tokens.len() < 4 {
                return Err(NetlistError::SyntaxError {
                    line: line_num,
                    message: format!("Expected: {name} <pos> <neg> <capacitance>"),
                });
            }
            let pos = tokens[1].clone();
            let neg = tokens[2].clone();
            let capacitance = parse_spice_number(&tokens[3], line_num)?;
            Ok(ComponentAst::Capacitor {
                name,
                pos,
                neg,
                capacitance,
                initial_voltage: None,
            })
        }
        'L' => {
            if tokens.len() < 4 {
                return Err(NetlistError::SyntaxError {
                    line: line_num,
                    message: format!("Expected: {name} <pos> <neg> <inductance>"),
                });
            }
            let pos = tokens[1].clone();
            let neg = tokens[2].clone();
            let inductance = parse_spice_number(&tokens[3], line_num)?;
            Ok(ComponentAst::Inductor {
                name,
                pos,
                neg,
                inductance,
                initial_current: None,
            })
        }
        'V' => {
            if tokens.len() < 4 {
                return Err(NetlistError::SyntaxError {
                    line: line_num,
                    message: format!("Expected: {name} <pos> <neg> [DC] <value>"),
                });
            }
            let pos = tokens[1].clone();
            let neg = tokens[2].clone();
            let val_idx = if tokens[3].eq_ignore_ascii_case("DC") && tokens.len() >= 5 {
                4
            } else {
                3
            };
            let dc_value = parse_spice_number(&tokens[val_idx], line_num)?;
            Ok(ComponentAst::VoltageSource {
                name,
                pos,
                neg,
                dc_value,
            })
        }
        'I' => {
            if tokens.len() < 4 {
                return Err(NetlistError::SyntaxError {
                    line: line_num,
                    message: format!("Expected: {name} <pos> <neg> [DC] <value>"),
                });
            }
            let pos = tokens[1].clone();
            let neg = tokens[2].clone();
            let val_idx = if tokens[3].eq_ignore_ascii_case("DC") && tokens.len() >= 5 {
                4
            } else {
                3
            };
            let dc_value = parse_spice_number(&tokens[val_idx], line_num)?;
            Ok(ComponentAst::CurrentSource {
                name,
                pos,
                neg,
                dc_value,
            })
        }
        'E' => {
            // VCVS: E... out_pos out_neg ctrl_pos ctrl_neg gain
            if tokens.len() < 6 {
                return Err(NetlistError::SyntaxError {
                    line: line_num,
                    message: format!(
                        "Expected: {name} <out_pos> <out_neg> <ctrl_pos> <ctrl_neg> <gain>"
                    ),
                });
            }
            let out_pos = tokens[1].clone();
            let out_neg = tokens[2].clone();
            let ctrl_pos = tokens[3].clone();
            let ctrl_neg = tokens[4].clone();
            let gain = parse_spice_number(&tokens[5], line_num)?;
            Ok(ComponentAst::Vcvs {
                name,
                out_pos,
                out_neg,
                ctrl_pos,
                ctrl_neg,
                gain,
            })
        }
        'G' => {
            // VCCS: G... out_pos out_neg ctrl_pos ctrl_neg transconductance
            if tokens.len() < 6 {
                return Err(NetlistError::SyntaxError {
                    line: line_num,
                    message: format!(
                        "Expected: {name} <out_pos> <out_neg> <ctrl_pos> <ctrl_neg> <gm>"
                    ),
                });
            }
            let out_pos = tokens[1].clone();
            let out_neg = tokens[2].clone();
            let ctrl_pos = tokens[3].clone();
            let ctrl_neg = tokens[4].clone();
            let transconductance = parse_spice_number(&tokens[5], line_num)?;
            Ok(ComponentAst::Vccs {
                name,
                out_pos,
                out_neg,
                ctrl_pos,
                ctrl_neg,
                transconductance,
            })
        }
        'D' => {
            if name.to_ascii_uppercase().starts_with("D2A") {
                if tokens.len() < 3 {
                    return Err(NetlistError::SyntaxError {
                        line: line_num,
                        message: format!(
                            "Expected: {name} <in_dig> <out_pos> [out_neg] [VLO=...] [VHI=...]"
                        ),
                    });
                }
                let (in_dig, out_pos, out_neg, param_start) =
                    if tokens.len() >= 4 && !tokens[3].contains('=') {
                        (tokens[1].clone(), tokens[2].clone(), tokens[3].clone(), 4)
                    } else {
                        (tokens[1].clone(), tokens[2].clone(), "0".to_string(), 3)
                    };
                let mut v_low = 0.0;
                let mut v_high = 3.3;
                let mut rise_time = 1e-9;
                let mut fall_time = 1e-9;
                let mut r_out = 50.0;

                for param in tokens.iter().skip(param_start) {
                    if let Some((k, v)) = param.split_once('=') {
                        match k.to_ascii_uppercase().as_str() {
                            "VLO" | "VLOW" | "V_LOW" => v_low = parse_spice_number(v, line_num)?,
                            "VHI" | "VHIGH" | "V_HIGH" => v_high = parse_spice_number(v, line_num)?,
                            "TR" | "TRISE" => rise_time = parse_spice_number(v, line_num)?,
                            "TF" | "TFALL" => fall_time = parse_spice_number(v, line_num)?,
                            "ROUT" | "R_OUT" => r_out = parse_spice_number(v, line_num)?,
                            _ => {}
                        }
                    }
                }

                Ok(ComponentAst::D2aBridge {
                    name,
                    in_dig,
                    out_pos,
                    out_neg,
                    v_low,
                    v_high,
                    rise_time,
                    fall_time,
                    r_out,
                })
            } else {
                // D... anode cathode model_name
                if tokens.len() < 4 {
                    return Err(NetlistError::SyntaxError {
                        line: line_num,
                        message: format!("Expected: {name} <anode> <cathode> <model_name>"),
                    });
                }
                Ok(ComponentAst::Diode {
                    name,
                    pos: tokens[1].clone(),
                    neg: tokens[2].clone(),
                    model_name: tokens[3].clone(),
                })
            }
        }
        'M' => {
            // M... drain gate source bulk model_name [W=...] [L=...]
            if tokens.len() < 6 {
                return Err(NetlistError::SyntaxError {
                    line: line_num,
                    message: format!("Expected: {name} <drain> <gate> <source> <bulk> <model_name> [W=...] [L=...]"),
                });
            }
            let drain = tokens[1].clone();
            let gate = tokens[2].clone();
            let source = tokens[3].clone();
            let bulk = tokens[4].clone();
            let model_name = tokens[5].clone();

            let mut w = None;
            let mut l = None;

            for param in tokens.iter().skip(6) {
                if let Some((k, v)) = param.split_once('=') {
                    if k.eq_ignore_ascii_case("W") {
                        w = Some(parse_spice_number(v, line_num)?);
                    } else if k.eq_ignore_ascii_case("L") {
                        l = Some(parse_spice_number(v, line_num)?);
                    }
                }
            }

            Ok(ComponentAst::Mosfet {
                name,
                drain,
                gate,
                source,
                bulk,
                model_name,
                w,
                l,
            })
        }
        'Q' => {
            // Q... collector base emitter model_name
            if tokens.len() < 5 {
                return Err(NetlistError::SyntaxError {
                    line: line_num,
                    message: format!("Expected: {name} <collector> <base> <emitter> <model_name>"),
                });
            }
            Ok(ComponentAst::Bjt {
                name,
                collector: tokens[1].clone(),
                base: tokens[2].clone(),
                emitter: tokens[3].clone(),
                model_name: tokens[4].clone(),
            })
        }
        'X' => {
            // X... pin1 pin2 ... subckt_name
            if tokens.len() < 3 {
                return Err(NetlistError::SyntaxError {
                    line: line_num,
                    message: format!("Expected: {name} <pin1> ... <subcircuit_name>"),
                });
            }
            let subcircuit_name = tokens.last().unwrap().clone();
            let pins = tokens[1..tokens.len() - 1].to_vec();
            Ok(ComponentAst::SubcircuitInstance {
                name,
                pins,
                subcircuit_name,
            })
        }
        'T' => {
            // T... in_pos in_neg out_pos out_neg [Z0=...] [TD=...]
            if tokens.len() < 5 {
                return Err(NetlistError::SyntaxError {
                    line: line_num,
                    message: format!(
                        "Expected: {name} <in_pos> <in_neg> <out_pos> <out_neg> [Z0=...] [TD=...]"
                    ),
                });
            }
            let in_pos = tokens[1].clone();
            let in_neg = tokens[2].clone();
            let out_pos = tokens[3].clone();
            let out_neg = tokens[4].clone();

            let mut z0 = 50.0;
            let mut td = 1e-9;
            let mut positional_idx = 0;

            for param in tokens.iter().skip(5) {
                if let Some((k, v)) = param.split_once('=') {
                    if k.eq_ignore_ascii_case("Z0") || k.eq_ignore_ascii_case("ZO") {
                        z0 = parse_spice_number(v, line_num)?;
                    } else if k.eq_ignore_ascii_case("TD") {
                        td = parse_spice_number(v, line_num)?;
                    }
                } else if let Ok(val) = parse_spice_number(param, line_num) {
                    if positional_idx == 0 {
                        z0 = val;
                    } else if positional_idx == 1 {
                        td = val;
                    }
                    positional_idx += 1;
                }
            }

            Ok(ComponentAst::TransmissionLine {
                name,
                in_pos,
                in_neg,
                out_pos,
                out_neg,
                z0,
                td,
            })
        }
        'A' => {
            if tokens.len() < 3 {
                return Err(NetlistError::SyntaxError {
                    line: line_num,
                    message: format!(
                        "Expected: {name} <in_pos> [in_neg] <out_dig> [VTL=...] [VTH=...]"
                    ),
                });
            }
            let (in_pos, in_neg, out_dig, param_start) =
                if tokens.len() >= 4 && !tokens[3].contains('=') {
                    (tokens[1].clone(), tokens[2].clone(), tokens[3].clone(), 4)
                } else {
                    (tokens[1].clone(), "0".to_string(), tokens[2].clone(), 3)
                };
            let mut vth_low = 0.8;
            let mut vth_high = 2.0;
            let mut r_in = None;

            for param in tokens.iter().skip(param_start) {
                if let Some((k, v)) = param.split_once('=') {
                    match k.to_ascii_uppercase().as_str() {
                        "VTL" | "VTH_LOW" | "VTLOW" => vth_low = parse_spice_number(v, line_num)?,
                        "VTH" | "VTH_HIGH" | "VTHIGH" => {
                            vth_high = parse_spice_number(v, line_num)?
                        }
                        "RIN" | "R_IN" => r_in = Some(parse_spice_number(v, line_num)?),
                        _ => {}
                    }
                }
            }

            Ok(ComponentAst::A2dBridge {
                name,
                in_pos,
                in_neg,
                out_dig,
                vth_low,
                vth_high,
                r_in,
            })
        }
        _ => Err(NetlistError::SyntaxError {
            line: line_num,
            message: format!("Unknown component prefix '{}' in '{name}'", prefix),
        }),
    }
}

fn parse_model_card(tokens: &[String], line_num: usize) -> Result<ModelCard, NetlistError> {
    // .MODEL name type (param=val ...) or .MODEL name type param=val ...
    if tokens.len() < 3 {
        return Err(NetlistError::SyntaxError {
            line: line_num,
            message: "Expected: .MODEL <name> <type> [(param=val ...)]".to_string(),
        });
    }

    let name = tokens[1].clone();
    let model_type = tokens[2].to_ascii_uppercase();
    let mut params = HashMap::new();

    for token in tokens.iter().skip(3) {
        let clean = token.trim_matches(|c| c == '(' || c == ')');
        for part in clean.split_whitespace() {
            if let Some((k, v)) = part.split_once('=') {
                let key = k.trim().to_ascii_uppercase();
                let val = parse_spice_number(v.trim(), line_num)?;
                params.insert(key, val);
            }
        }
    }

    Ok(ModelCard {
        name,
        model_type,
        params,
    })
}

/// Tokenizes a single line into words, preserving `key=value` groupings.
fn tokenize_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for word in line.split_whitespace() {
        tokens.push(word.to_string());
    }
    tokens
}
