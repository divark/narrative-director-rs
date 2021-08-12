mod audio_processor;
mod player;
mod recorder;
mod ui;

pub mod prelude {
    pub use super::audio_processor::*;
    pub use super::player::*;
    pub use super::recorder::*;
    pub use super::ui::*;

    #[derive(PartialEq)]
    pub enum ProcessingStatus {
        Playing,
        Paused,
        Stopped,
        Recording,
    }

    #[derive(PartialEq, Clone, Copy)]
    pub enum FileStatus {
        Exists,
        New,
    }

    pub struct InputDeviceSelection {
        pub name: String,
        pub sample_rate: u32,
        pub num_channels: u16,
    }

    pub struct OutputDeviceSelection {
        pub name: String,
    }

    pub trait MediaProcessor {
        // Starts playing media from some file
        fn play(&mut self) -> Result<ProcessingStatus, String>;

        // Pauses current media.
        fn pause(&mut self, pos_ms: u32) -> Result<ProcessingStatus, String>;

        // Stops currently playing media.
        fn stop(&mut self) -> Result<ProcessingStatus, String>;

        // Skip to n seconds in for the media.
        fn skip_to(&mut self, position_in_seconds: u32) -> Result<ProcessingStatus, String>;

        // Proceeds to next media track.
        fn next(&mut self) -> Result<FileStatus, String>;

        // Proceeds to previous media track.
        fn prev(&mut self) -> Result<FileStatus, String>;

        // Proceeds to particular media track.
        fn go_to(&mut self, idx: usize) -> Result<FileStatus, String>;

        // Starts recording media.
        fn record(&mut self) -> Result<ProcessingStatus, String>;

        // Stop recording media.
        fn stop_recording(&mut self) -> Result<ProcessingStatus, String>;

        // Gets duration in seconds of media.
        fn duration(&self) -> u32;
    }
}

pub use prelude::*;
