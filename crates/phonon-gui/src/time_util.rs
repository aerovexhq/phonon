#![deny(unsafe_code)]

//! Cross-platform time abstraction supporting both native desktop environments and WebAssembly (wasm32-unknown-unknown).

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Instant(f64);

#[cfg(target_arch = "wasm32")]
impl Instant {
    /// Returns the current timestamp in milliseconds via js_sys::Date::now().
    #[inline]
    pub fn now() -> Self {
        Self(js_sys::Date::now())
    }

    /// Computes the elapsed duration since this instant was recorded.
    #[inline]
    pub fn elapsed(&self) -> std::time::Duration {
        let now = js_sys::Date::now();
        let diff_ms = (now - self.0).max(0.0);
        std::time::Duration::from_secs_f64(diff_ms / 1000.0)
    }
}
