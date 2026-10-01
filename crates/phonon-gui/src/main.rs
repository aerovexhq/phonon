#![deny(unsafe_code)]

//! Phonon GUI executable.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    phonon_gui::run_gui()
}
