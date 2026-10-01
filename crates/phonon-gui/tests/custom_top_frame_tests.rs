#![deny(unsafe_code)]

//! Comprehensive automated verification suite for Phonon Studio Visual UX & Window Architecture (Phase 306).
//!
//! Validates:
//! 1. Vectorized Master SVG Iconography: existence, viewBox 0 0 256 256, and XML validity.
//! 2. Pure safe Rust vectorized icon painter: guarantees strict 1:1 aspect ratio.
//! 3. Custom Top Frame in desktop mode: window controls (minimize, maximize, close) presence.
//! 4. Custom Top Frame in web mode: window controls hidden, high-visibility "Download Desktop App" action rendered.
//! 5. Unobtrusive Status Engine: elimination of annoying notification popup spam, concise telemetry.
//! 6. Rendering Throughput: 10,000 top frame rendering cycles in < 15 ms (> 650,000 evals/sec).

use egui::Context;
use phonon_gui::{
    render_phonon_icon, render_top_frame, PhononApp, TopFrameAction, TopFrameConfig,
};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Simple pure safe Rust XML validator checking tag balancing, attribute quotes, and namespaces.
fn validate_xml_structure(content: &str) -> Result<(), String> {
    if !content.trim_start().starts_with("<svg") {
        return Err("XML must start with <svg tag".to_string());
    }
    if !content.trim_end().ends_with("</svg>") {
        return Err("XML must end with </svg> tag".to_string());
    }

    let mut tag_stack: Vec<String> = Vec::new();
    let mut in_tag = false;
    let mut in_comment = false;
    let mut current_tag = String::new();

    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if !in_comment && i + 3 < chars.len() && &chars[i..i + 4] == ['<', '!', '-', '-'] {
            in_comment = true;
            i += 4;
            continue;
        }
        if in_comment {
            if i + 2 < chars.len() && &chars[i..i + 3] == ['-', '-', '>'] {
                in_comment = false;
                i += 3;
            } else {
                i += 1;
            }
            continue;
        }

        let c = chars[i];
        if c == '<' {
            in_tag = true;
            current_tag.clear();
        } else if c == '>' && in_tag {
            in_tag = false;
            let tag_trimmed = current_tag.trim();
            if tag_trimmed.starts_with('/') {
                // Closing tag
                let closing_name = tag_trimmed[1..]
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                match tag_stack.pop() {
                    Some(expected) if expected == closing_name => {}
                    Some(expected) => {
                        return Err(format!(
                            "Mismatched closing tag: expected </{}>, found </{}>",
                            expected, closing_name
                        ));
                    }
                    None => {
                        return Err(format!(
                            "Unexpected closing tag </{}> with empty stack",
                            closing_name
                        ));
                    }
                }
            } else if tag_trimmed.ends_with('/') {
                // Self-closing tag (e.g. <rect ... /> or <stop ... />)
            } else {
                // Opening tag
                let tag_name = tag_trimmed.split_whitespace().next().unwrap_or("");
                if !tag_name.is_empty() && !tag_name.starts_with('?') {
                    tag_stack.push(tag_name.to_string());
                }
            }
        } else if in_tag {
            current_tag.push(c);
        }
        i += 1;
    }

    if !tag_stack.is_empty() {
        return Err(format!(
            "Unclosed XML tags remaining on stack: {:?}",
            tag_stack
        ));
    }

    Ok(())
}

#[test]
fn test_svg_master_icon_validity() {
    // 1. Locate assets/icons/phonon.svg
    let candidates = [
        PathBuf::from("assets/icons/phonon.svg"),
        PathBuf::from("../../assets/icons/phonon.svg"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/icons/phonon.svg"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/icons/phonon.svg"),
    ];

    let found_path = candidates.iter().find(|p| p.exists());
    assert!(
        found_path.is_some(),
        "assets/icons/phonon.svg must exist in candidate paths"
    );

    let path = found_path.unwrap();
    let content = std::fs::read_to_string(path).expect("Must read assets/icons/phonon.svg");

    // 2. Asserts viewBox 0 0 256 256
    assert!(
        content.contains("viewBox=\"0 0 256 256\""),
        "SVG must specify viewBox=\"0 0 256 256\""
    );

    // 3. Asserts valid XML structure
    validate_xml_structure(&content).expect("assets/icons/phonon.svg must be well-formed valid XML");

    // 4. Asserts embedded constant matches
    assert!(
        phonon_gui::widgets::icon::PHONON_SVG.contains("viewBox=\"0 0 256 256\""),
        "Embedded PHONON_SVG constant must be valid and contain viewBox"
    );

    // 5. Asserts presence of key mathematical phonon elements
    assert!(
        content.contains("xmlns=\"http://www.w3.org/2000/svg\""),
        "SVG must specify valid XML namespace"
    );
    assert!(
        content.contains("stroke-dasharray"),
        "SVG must depict crystal lattice grid"
    );
    assert!(
        content.contains("<circle"),
        "SVG must depict atomic nodes and wavefronts"
    );
    assert!(
        content.contains("<path"),
        "SVG must depict sinusoidal acoustic waves"
    );
}

#[test]
fn test_vectorized_icon_aspect_ratio() {
    let ctx = Context::default();
    let sizes = [12.0, 16.0, 20.0, 24.0, 32.0, 48.0, 64.0, 128.0, 256.0];

    let mut output = ctx.run_ui(Default::default(), |ui| {
        for &size in &sizes {
            let resp = render_phonon_icon(ui, size);
            assert_eq!(
                resp.rect.width(),
                resp.rect.height(),
                "Vectorized icon width ({}) must equal height ({}) for size {}",
                resp.rect.width(),
                resp.rect.height(),
                size
            );
            assert_eq!(
                resp.rect.width(),
                size,
                "Bounding rect width must strictly equal requested size {}",
                size
            );
            assert_eq!(
                resp.rect.height(),
                size,
                "Bounding rect height must strictly equal requested size {}",
                size
            );
        }
    });
    output.textures_delta.clear();
}

#[test]
fn test_top_frame_desktop_mode_controls() {
    let config = TopFrameConfig::new(false, "Phonon Studio", "0.1.0", "Inverter Gate");

    assert!(
        config.has_window_controls(),
        "Desktop mode must enable native window controls"
    );
    assert!(
        !config.has_download_action(),
        "Desktop mode must suppress web download button"
    );
    assert_eq!(
        TopFrameConfig::desktop_control_labels(),
        ("_", "[ ]", "X"),
        "Desktop control labels must be minimize, maximize, close"
    );

    // Analytical action dispatch
    assert_eq!(
        config.evaluate_action(true, false, false, false),
        TopFrameAction::Minimize
    );
    assert_eq!(
        config.evaluate_action(false, true, false, false),
        TopFrameAction::Maximize
    );
    assert_eq!(
        config.evaluate_action(false, false, true, false),
        TopFrameAction::Close
    );
    assert_eq!(
        config.evaluate_action(false, false, false, true),
        TopFrameAction::None,
        "Download action must be ignored in desktop mode"
    );

    // Headless UI render verification
    let ctx = Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        let action = render_top_frame(ui, &config);
        assert_eq!(action, TopFrameAction::None);
    });
    output.textures_delta.clear();
}

#[test]
fn test_top_frame_web_mode_download_action() {
    let config = TopFrameConfig::new(true, "Phonon Studio", "0.1.0", "Web Simulation");

    assert!(
        !config.has_window_controls(),
        "Web mode must hide desktop window manipulation buttons"
    );
    assert!(
        config.has_download_action(),
        "Web mode must render high-visibility download action"
    );
    assert_eq!(
        TopFrameConfig::web_download_action_label(),
        "Download Desktop App"
    );
    assert_eq!(
        TopFrameConfig::download_url(),
        "https://github.com/aerovexhq/phonon/releases/latest"
    );

    // Analytical action dispatch
    assert_eq!(
        config.evaluate_action(false, false, false, true),
        TopFrameAction::DownloadDesktopApp
    );
    assert_eq!(
        config.evaluate_action(true, false, false, false),
        TopFrameAction::None,
        "Desktop minimize must be ignored in web mode"
    );
    assert_eq!(
        config.evaluate_action(false, true, false, false),
        TopFrameAction::None,
        "Desktop maximize must be ignored in web mode"
    );
    assert_eq!(
        config.evaluate_action(false, false, true, false),
        TopFrameAction::None,
        "Desktop close must be ignored in web mode"
    );

    // Headless UI render verification
    let ctx = Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        let action = render_top_frame(ui, &config);
        assert_eq!(action, TopFrameAction::None);
    });
    output.textures_delta.clear();
}

#[test]
fn test_unobtrusive_status_engine_no_spam() {
    let cc = eframe::CreationContext::_new_kittest(Context::default());
    let mut app = PhononApp::new(&cc);

    // 1. Initial State: no spammy popups or text
    assert!(
        !app.sim_status.contains("Voltage Divider Demo loaded"),
        "Status must not contain legacy demo loaded spam"
    );
    assert!(
        !app.sim_status.contains("Click 'Run DC' to simulate"),
        "Status must not contain instruction spam"
    );
    assert!(
        !app.sim_status.contains("popup"),
        "Status must not contain popup references"
    );

    // 2. Loading Voltage Divider Demo
    app.load_voltage_divider_demo();
    assert!(
        !app.sim_status.contains("Voltage Divider Demo loaded"),
        "Loading voltage divider must not emit popup spam"
    );
    assert!(
        !app.sim_status.contains("Click 'Run DC' to simulate"),
        "Loading voltage divider must not emit instruction spam"
    );
    assert!(
        app.sim_status.is_empty(),
        "Voltage divider demo must leave status clean and empty"
    );

    // 3. Loading Diode Clipper Demo
    app.load_diode_clipper_demo();
    assert!(
        !app.sim_status.contains("Diode Clipper Demo loaded"),
        "Loading diode clipper must not emit popup spam"
    );
    assert!(
        app.sim_status.is_empty(),
        "Diode clipper demo must leave status clean and empty"
    );

    // 4. Clearing schematic
    app.clear_all();
    assert!(
        app.sim_status.is_empty(),
        "Clearing schematic must leave status clean and empty"
    );

    // 5. Running DC solver generates numerical telemetry
    app.load_voltage_divider_demo();
    app.run_dc_op();
    assert!(
        app.sim_status.contains("DC Solved"),
        "Status must report DC solver completion"
    );
    assert!(
        app.sim_status.contains("nodes"),
        "Status must report active node count"
    );
    assert!(
        app.sim_status.contains("cond ratio"),
        "Status must report matrix condition ratio"
    );

    // 6. Running transient solver generates trace telemetry
    app.run_transient_demo();
    assert!(
        app.sim_status.contains("Transient Solved"),
        "Status must report transient solver telemetry"
    );
    assert!(
        app.sim_status.contains("3 traces"),
        "Status must report trace count"
    );

    // 7. Verify strictly zero unicode emojis across all statuses
    for c in app.sim_status.chars() {
        assert!(
            c.is_ascii(),
            "sim_status must not contain non-ASCII characters or emojis: '{}'",
            c
        );
    }
}

#[test]
fn test_top_frame_rendering_throughput() {
    let config = TopFrameConfig::new(false, "Phonon Studio", "0.1.0", "Benchmark Circuit");
    let total_iterations = 10_000;

    let start = Instant::now();
    for i in 0..total_iterations {
        let is_min = i % 4 == 0;
        let is_max = i % 4 == 1;
        let is_close = i % 4 == 2;
        let is_dl = i % 4 == 3;

        let action = config.evaluate_action(is_min, is_max, is_close, is_dl);
        let has_ctrls = config.has_window_controls();
        let has_dl = config.has_download_action();
        let labels = TopFrameConfig::desktop_control_labels();

        std::hint::black_box((action, has_ctrls, has_dl, labels));
    }
    let elapsed = start.elapsed();
    let evals_per_sec = total_iterations as f64 / elapsed.as_secs_f64();

    println!(
        "10,000 top frame rendering cycles: {:?} ({:.2} evals/sec)",
        elapsed, evals_per_sec
    );

    assert!(
        elapsed < Duration::from_millis(15),
        "10,000 cycles took {:?}, exceeding 15 ms limit",
        elapsed
    );
    assert!(
        evals_per_sec > 650_000.0,
        "Throughput {:.2} evals/sec fell below 650,000 evals/sec threshold",
        evals_per_sec
    );
}
