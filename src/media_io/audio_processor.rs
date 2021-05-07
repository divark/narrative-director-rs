use std::path::Path;

use super::prelude::*;
use crate::media_io::MediaProcessor;

pub struct AudioIO {
    current_chunk_index: u64,
    total_num_chunks: usize,

    recorder: CpalAudioRecorder,
    player: CpalAudioPlayer,
    player_duration_ms: u32,
}

impl AudioIO {
    pub fn new(num_chunks: usize) -> Self {
        Self {
            current_chunk_index: 0,
            total_num_chunks: num_chunks,

            recorder: CpalAudioRecorder::new(),
            player: CpalAudioPlayer::new(),
            player_duration_ms: 0,
        }
    }
}

impl MediaProcessor for AudioIO {
    fn play(&mut self) -> Result<ProcessingStatus, String> {
        match self
            .player
            .play(format!("part{}.wav", self.current_chunk_index))
        {
            Ok(audio_duration) => {
                self.player_duration_ms = audio_duration;
                Ok(ProcessingStatus::Playing)
            }
            Err(playback_error) => Err(format!("AudioIO play error: {:?}", playback_error)),
        }
    }

    fn pause(&mut self, pos_ms: u32) -> Result<ProcessingStatus, String> {
        if let Err(pause_err) = self.player.pause(pos_ms) {
            return Err(format!("AudioIO pause error: {:?}", pause_err));
        }

        Ok(ProcessingStatus::Paused)
    }

    fn stop(&mut self) -> Result<ProcessingStatus, String> {
        self.player.stop();

        Ok(ProcessingStatus::Stopped)
    }

    fn skip_to(&mut self, pos_ms: u32) -> Result<ProcessingStatus, String> {
        let input_file = format!("part{}.wav", self.current_chunk_index);
        let input_path = Path::new(&input_file);
        if !input_path.exists() {
            return Err("AudioIO skip_to error: File does not exist.".to_string());
        }

        match self.player.play_at(input_file.clone(), pos_ms) {
            Ok(duration_ms) => {
                self.player_duration_ms = duration_ms;
                Ok(ProcessingStatus::Playing)
            }
            Err(playback_error) => Err(format!("AudioIO skip_to error: {:?}", playback_error)),
        }
    }

    fn next(&mut self) -> Result<FileStatus, String> {
        if self.current_chunk_index + 1 >= self.total_num_chunks as u64 {
            return Err(format!(
                "AudioIO next error: current chunk {} exceeds total of {}",
                self.current_chunk_index, self.total_num_chunks
            ));
        }

        self.current_chunk_index += 1;

        let input_file = format!("part{}.wav", self.current_chunk_index);
        let input_path = Path::new(&input_file);
        if input_path.exists() {
            self.player_duration_ms = self.player.duration_of(input_file.clone());
            return Ok(FileStatus::Exists);
        }

        Ok(FileStatus::New)
    }

    fn prev(&mut self) -> Result<FileStatus, String> {
        if self.current_chunk_index == 0 {
            return Err(format!(
                "AudioIO prev error: current chunk {} cannot go any lower.",
                self.current_chunk_index
            ));
        }

        self.current_chunk_index -= 1;

        let input_file = format!("part{}.wav", self.current_chunk_index);
        let input_path = Path::new(&input_file);
        if input_path.exists() {
            self.player_duration_ms = self.player.duration_of(input_file.clone());
            return Ok(FileStatus::Exists);
        }

        Ok(FileStatus::New)
    }

    fn go_to(&mut self, idx: usize) -> Result<FileStatus, String> {
        if idx > self.total_num_chunks {
            return Err(format!(
                "AudioIO go_to error: index {} greater than available chunks.",
                idx
            ));
        }

        self.current_chunk_index = idx as u64;

        let input_file = format!("part{}.wav", self.current_chunk_index);
        let input_path = Path::new(&input_file);
        if input_path.exists() {
            self.player_duration_ms = self.player.duration_of(input_file.clone());
            return Ok(FileStatus::Exists);
        }

        Ok(FileStatus::New)
    }

    fn record(&mut self) -> Result<ProcessingStatus, String> {
        if let Err(msg) = self
            .recorder
            .record(format!("part{}.wav", self.current_chunk_index))
        {
            return Err(format!("AudioIO record error: {}", msg));
        }

        Ok(ProcessingStatus::Recording)
    }

    fn stop_recording(&mut self) -> Result<ProcessingStatus, String> {
        self.recorder.stop();

        Ok(ProcessingStatus::Stopped)
    }

    /// Returns the duration in milliseconds.
    fn duration(&self) -> u32 {
        self.player_duration_ms
    }
}
