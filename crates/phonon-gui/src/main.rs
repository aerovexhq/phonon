#![deny(unsafe_code)]

//! Phonon GUI executable.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        phonon_gui::run_gui()
    }
    #[cfg(target_arch = "wasm32")]
    {
        Ok(())
    }
}
