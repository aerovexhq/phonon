#![deny(unsafe_code)]

//! Phonon UI DevTools: Interactive in-app inspector, synthetic action scripting, and visual screenshot capture.
//!
//! Note: This module is strictly gated under `#[cfg(any(test, feature = "devtools"))]` and is never included
//! in official release distributions.

pub mod capture;
pub mod panel;
pub mod script;

pub use capture::{capture_window_screenshot, find_phonon_window_id, ScreenshotInfo};
pub use panel::DevtoolsState;
pub use script::{ScriptRunner, UiAction, UiScript};
