mod audio_player;
mod audio_recorder;

use crate::audio_processor::audio_recorder::CpalAudioRecorder;
use crate::audio_processor::audio_player::CpalAudioPlayer;
use cpal::PlayStreamError;
use std::path::Path;

#[derive(PartialEq)]
pub enum ProcessingStatus {
    Playing,
    Paused,
    Stopped,
    Recording,
}

#[derive(PartialEq)]
pub enum FileStatus {
    Exists,
    New,
}

pub trait AudioProcessor {
    // Starts playing audio from some file
    fn play(&mut self) -> Result<ProcessingStatus, String>;

    // Pauses current audio.
    fn pause(&mut self) -> Result<ProcessingStatus, String>;

    // Stops currently playing audio.
    fn stop(&mut self) -> Result<ProcessingStatus, String>;

    // Skip to n seconds in for the audio.
    fn skip_to(&mut self, position_in_seconds: u32);

    // Proceeds to next audio/video track.
    fn next(&mut self) -> Result<FileStatus, String>;

    // Proceeds to previous audio/video track.
    fn prev(&mut self) -> Result<FileStatus, String>;

    // Proceeds to particular audio/video track.
    fn go_to(&mut self, idx: usize) -> Result<FileStatus, String>;

    // Starts recording audio.
    fn record(&mut self) -> Result<ProcessingStatus, String>;

    // Stop recording audio.
    fn stop_recording(&mut self) -> Result<ProcessingStatus, String>;
}

pub struct ChunkAudioIO {
    current_chunk_index: u64,
    total_num_chunks: usize,

    recorder: CpalAudioRecorder,
    player: CpalAudioPlayer,
    player_length: u32,
}

impl ChunkAudioIO {
    pub fn new(num_chunks: usize) -> Self {
        Self {
            current_chunk_index: 0,
            total_num_chunks: num_chunks,

            recorder: CpalAudioRecorder::new(),
            player: CpalAudioPlayer::new(),
            player_length: 0,
        }
    }
}

impl AudioProcessor for ChunkAudioIO {
    fn play(&mut self) -> Result<ProcessingStatus, String> {
        match self.player.play(String::from(format!("part{}.wav", self.current_chunk_index))) {
            Ok(audio_duration) => {
                self.player_length = audio_duration;
                Ok(ProcessingStatus::Playing)
            }
            Err(playback_error) => Err(String::from(format!("ChunkAudioIO play error: {:?}", playback_error)))
        }
    }

    fn pause(&mut self) -> Result<ProcessingStatus, String> {
        todo!()
    }

    fn stop(&mut self) -> Result<ProcessingStatus, String> {
        self.player.stop();

        Ok(ProcessingStatus::Stopped)
    }

    fn skip_to(&mut self, position_in_seconds: u32) {
        todo!()
    }

    fn next(&mut self) -> Result<FileStatus, String> {
        if self.current_chunk_index + 1 >= self.total_num_chunks as u64 {
            return Err(String::from(format!("ChunkAudioIO next error: current chunk {} exceeds total of {}", self.current_chunk_index, self.total_num_chunks)));
        }

        self.current_chunk_index += 1;

        if Path::new(&format!("part{}.wav", self.current_chunk_index)).exists() {
            return Ok(FileStatus::Exists);
        }

        Ok(FileStatus::New)
    }

    fn prev(&mut self) -> Result<FileStatus, String> {
        if self.current_chunk_index == 0 {
            return Err(String::from(format!("ChunkAudioIO prev error: current chunk {} cannot go any lower.", self.current_chunk_index)));
        }

        self.current_chunk_index -= 1;

        if Path::new(&format!("part{}.wav", self.current_chunk_index)).exists() {
            return Ok(FileStatus::Exists);
        }

        Ok(FileStatus::New)
    }

    fn go_to(&mut self, idx: usize) -> Result<FileStatus, String> {
        if idx > self.total_num_chunks {
            return Err(String::from(format!("ChunkAudioIO go_to error: index {} greater than available chunks.", idx)));
        }

        self.current_chunk_index = idx as u64;

        if Path::new(&format!("part{}.wav", self.current_chunk_index)).exists() {
            return Ok(FileStatus::Exists);
        }

        Ok(FileStatus::New)
    }

    fn record(&mut self) -> Result<ProcessingStatus, String> {
        if let Err(msg) = self.recorder.record(String::from(format!("part{}.wav", self.current_chunk_index))) {
            return Err(String::from(format!("ChunkAudioIO record error: {}", msg)));
        }

        Ok(ProcessingStatus::Recording)
    }

    fn stop_recording(&mut self) -> Result<ProcessingStatus, String> {
        self.recorder.stop();

        Ok(ProcessingStatus::Stopped)
    }
}