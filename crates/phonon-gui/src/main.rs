//! Phonon GUI executable.

#![deny(unsafe_code)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    phonon_gui::run_gui()
}

