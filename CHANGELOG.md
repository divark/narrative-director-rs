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
- Unit Testing and Documentation for Goto Prompt.
- Mock Diagram for General Preferences.
- Wireframe Diagram for General Preferences.
- Mock Diagram for Audio Preferences.
- Wireframe Diagram for Audio Preferences.

### Changed
- About Dialog with description of program and icon.
- Goto Dialog is now created in code using FLTK.
- Paragraph Counter Label is now a Button that opens Goto Prompt on click.
- Audio widgets now use FLTK.
- Text viewing is now using FLTK widgets.
- Updated CPAL dependency to 0.14.
- Paragraph Viewer tests now use FLTK widgets.
- CI scripts to include FLTK dependencies.
- Audio Preferences section of Preferences has been converted to use FLTK widgets.
- General Preferences section of Preferences has been converted to use FLTK widgets.

### Fixed
- Clippy suggestions for improved code quality.

### Removed
- Relm dependency for GUI framework.
- GTK3-related dependencies.
- Dialogs created in common.rs in favor of per-file declarations.
- main-window UI file in favor of making UI in code.

[Unreleased]: https://github.com/divark/narrative-director-rs/compare/main..fltk-migration