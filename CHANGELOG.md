# Changelog 
## Notes
- The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
- This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
- This project uses [ISO Standard](https://www.iso.org/iso-8601-date-and-time-format.html) date formatting

## [Unreleased]
### Added
- FLTK-RS as the new GUI framework.
- Creation of Main Window UI in code.
- Changelog to keep track of changes from now on.

### Changed
- About Dialog with description of program and icon.
- Audio widgets now use FLTK.
- Text viewing is now using FLTK widgets.
- Updated CPAL dependency to 0.14.
- Paragraph Viewer tests now use FLTK widgets.
- CI scripts to include FLTK dependencies.

### Fixed
- Clippy suggestions for improved code quality.

### Removed
- Relm dependency for GUI framework.
- GTK3-related dependencies.
- Dialogs created in common.rs in favor of per-file declarations.
- main-window UI file in favor of making UI in code.

[Unreleased]: https://github.com/divark/narrative-director-rs/compare/main..fltk-migration