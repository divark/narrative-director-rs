# Narrative Director (Rust Edition)

## Summary
Narrative Director is an Audio/Video Recording Pacer application, with a focus on error-free readings.
This tool aspires to keep editing to a minimum, keeping only recordings that are satisfactory by the reader.

The Rust Edition serves as a successor to the [Qt 5 edition of Narrative Director](https://github.com/divark/narrative-director), in
addition to having an excuse to learn about [Rust](https://www.rust-lang.org/) and [Relm](https://github.com/antoyo/relm).

## Features
### Current
- Read paragraph-by-paragraph (4 sentences) from UTF-8 text files.
- Jump to particular paragraph entry.
### Planned
- Play, Pause, Stop, and Record Audio for each paragraph entry.
- Play, Stop, and Record Video for each paragraph entry.

## Mock-ups
##### User Interface Mappings
![Interface Mappings](resources/images/Mappings.png)

## Known Working Environments
- Linux x64 (Arch Linux)
## Getting Started
1. Download [Rust](https://www.rust-lang.org/learn/get-started) if you have not already.
2. Download [GTK+3](https://www.gtk.org/docs/installations/).
3. Clone the repository.
4. In a terminal, navigate to the repository.
5. Run `cargo test` to ensure all features are working as intended.
6. Run `cargo run` to see the current state of the application.

## License
Narrative Director is distributed under the GPLv3 License, which can be viewed [here.](COPYING)
