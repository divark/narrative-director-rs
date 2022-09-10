// This is needed to ensure that the application
// does not start with a console in the background
// when the application runs on Windows.
#![windows_subsystem = "windows"]

mod text;
mod media;

mod ui;
use ui::app::*;

mod sessions;

/// Spawns the application with a Graphical User Interface.
fn main() {
    let mut app = MainApplication::new();

    app.run();
}
