mod audio_recorder;

use crate::audio_processor::audio_recorder::CpalAudioRecorder;

pub enum ProcessingStatus {
    Playing,
    Paused,
    Stopped,
    Recording,
}

pub enum FileStatus {
    Exists,
    New,
}

pub trait AudioProcessor {
    // // Starts playing audio from some file
    // fn play(&mut self) -> Result<ProcessingStatus, String>;

    // // Pauses current audio.
    // fn pause(&mut self) -> Result<ProcessingStatus, String>;

    // // Stops currently playing audio.
    // fn stop(&mut self) -> Result<ProcessingStatus, String>;
    //
    // // Skip to n seconds in for the audio.
    // fn skip_to(&mut self, position_in_seconds: u32);

    // // Proceeds to next audio/video track.
    // fn next(&mut self) -> Result<FileStatus, String>;

    // // Proceeds to previous audio/video track.
    // fn next(&mut self) -> Result<FileStatus, String>;

    // Starts recording audio.
    fn record(&mut self) -> Result<ProcessingStatus, &str>;

    // Stop recording audio.
    fn stop_recording(&mut self) -> Result<ProcessingStatus, &str>;
}

pub struct ChunkAudioIO {
    current_chunk_index: u64,
    total_num_chunks: usize,

    recorder: CpalAudioRecorder,
    status: ProcessingStatus,
}

