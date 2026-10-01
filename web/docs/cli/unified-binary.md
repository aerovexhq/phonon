# Unified Binary Architecture

In Phonon, a single binary file (`phonon`) services both headless terminal workflows and interactive graphical CAD requirements.

## How it works

- When invoked with no parameters (`phonon`), Phonon displays an interactive ASCII banner, current version, documentation links, and available command syntax.
- When invoked as `phonon gui`, the executable checks for a display server (Wayland or X11) and initializes the native `eframe`/`egui` desktop CAD application.
- When invoked with subcommands such as `validate`, `run`, `sweep`, or `mc`, the CLI parser directly executes the high-throughput headless solver pipeline without allocating GUI resources.

## Examples

```bash
# Terminal overview
phonon

# Launch CAD Studio
phonon gui

# Run headless simulation directly in CI/CD scripts
phonon run filter.cir --format json --output simulation_results.json
```
