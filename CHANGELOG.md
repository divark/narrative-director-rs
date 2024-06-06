# Changelog 
## Notes
- The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
- This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
- This project uses [ISO Standard](https://www.iso.org/iso-8601-date-and-time-format.html) date formatting

## [1.0.1]
### Fixed
- Minutes and Hours now displaying properly.
- Linux release lacking wayland library.

## [1.0.0] - 2024-05-16
### Added
- Wayland support.
- Text Extraction Preferences.

### Changed
- Updated FLTK and various developer dependencies.
- Changed from set_size to fixed, removing deprecations.

### Fixed
- Recording halting from unsupported Sample Format.
- Fixed Changelog ordering of tags.

## [0.1.2] - 2023-05-20
### Added
- Close button to About dialog.

### Changed
- Updated CPAL dependency to 0.15
- Updated various dependencies to latest version via cargo update.

### Fixed
- Text not wrapping.
- Project directory not being saved properly when changed in Preferences.
- macOS builds only being compatible with latest macOS version.
- macOS not saving audio file after recording.
- Existing audio not being loaded after using go-to prompt.

## [0.1.1] - 2023-01-30
### Added
- Permission to use Microphone in MacOS build.

### Changed
- Replaced action-rs with dtolnay/rust-toolchain for CI.
- Updated actions/checkout to v3.

### Fixed
- Icon not being found in runtime.

## [0.1.0] - 2023-01-29
### Added
- FLTK-RS as the new GUI framework.
- Creation of Main Window UI in code.
- Changelog to keep track of changes from now on.
- Unit Testing and Documentation for Goto Prompt.
- Mock Diagram for General Preferences.
- Wireframe Diagram for General Preferences.
- Mock Diagram for Audio Preferences.
- Wireframe Diagram for Audio Preferences.
- App icons for Windows and MacOS.

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
- README to include new screenshots of application and library usage.

### Fixed
- Clippy suggestions for improved code quality.
- Not being able to stop recording or playing, even if UI suggested otherwise.
- Icon not being able to load on MacOS build.

### Removed
- Relm dependency for GUI framework.
- GTK3-related dependencies.
- Dialogs created in common.rs in favor of per-file declarations.
- main-window UI file in favor of making UI in code.
- Unit Testing in GitHub Workflow files, temporarily.

[Unreleased]: https://github.com/divark/narrative-director-rs/blob/main/CHANGELOG.md
[1.0.1]: https://github.com/divark/narrative-director-rs/releases/tag/v1.0.1
[1.0.0]: https://github.com/divark/narrative-director-rs/releases/tag/v1.0.0
[0.1.2]: https://github.com/divark/narrative-director-rs/releases/tag/v0.1.2
[0.1.1]: https://github.com/divark/narrative-director-rs/releases/tag/v0.1.1
[0.1.0]: https://github.com/divark/narrative-director-rs/releases/tag/v0.1.0
