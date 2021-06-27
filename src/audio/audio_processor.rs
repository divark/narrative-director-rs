use std::path::Path;

use super::prelude::*;
use crate::audio::MediaProcessor;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{default_host, ChannelCount, Device};

pub struct AudioIO {
    current_chunk_index: u64,
    total_num_chunks: usize,

    project_location: String,
    recorder: CpalAudioRecorder,
    player: CpalAudioPlayer,
    player_duration_ms: u32,
}

impl AudioIO {
    pub fn new(num_chunks: usize, project_location: String) -> Self {
        Self {
            current_chunk_index: 0,
            total_num_chunks: num_chunks,

            project_location,
            recorder: CpalAudioRecorder::new(),
            player: CpalAudioPlayer::new(),
            player_duration_ms: 0,
        }
    }

    pub fn get_input_devices(&self) -> Vec<Device> {
        self.recorder.get_input_devices()
    }

    pub fn get_input_device(&self) -> &Device {
        self.recorder.get_input_device()
    }

    pub fn get_input_channels_for(&self, input_device: &Device) -> Vec<ChannelCount> {
        self.recorder.get_channels_for_device(input_device)
    }

    pub fn get_input_sample_rates_for(&self, input_device: &Device) -> Vec<u32> {
        self.recorder.get_sample_rates_for_device(input_device)
    }

    pub fn set_input_device(&mut self, input_info: InputDeviceSelection) {
        let new_input_device = if input_info.name == "default" {
            default_host().default_input_device().unwrap()
        } else {
            default_host()
                .input_devices()
                .unwrap()
                .find(|input| input.name().unwrap() == input_info.name)
                .expect("Could not find input device")
        };

        self.recorder.set_input_device(
            new_input_device,
            input_info.sample_rate,
            input_info.num_channels,
        );
    }

    pub fn set_output_device(&mut self, output_info: OutputDeviceInfo) {
        let new_output_device = if output_info.name == "default" {
            default_host().default_output_device().unwrap()
        } else {
            default_host()
                .output_devices()
                .unwrap()
                .find(|output_device| output_device.name().unwrap() == output_info.name)
                .unwrap()
        };

        self.player.set_output_device(new_output_device);
    }

    pub fn get_output_devices(&self) -> Vec<Device> {
        self.player.get_output_devices()
    }

    pub fn get_output_device(&self) -> &Device {
        self.player.get_output_device()
    }
}

impl MediaProcessor for AudioIO {
    fn play(&mut self) -> Result<ProcessingStatus, String> {
        let project_path = Path::new(&self.project_location);
        let play_path = project_path
            .join(format!("part{}.wav", self.current_chunk_index))
            .to_str()
            .unwrap()
            .to_string();
        match self.player.play(play_path) {
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
        let project_path = Path::new(&self.project_location);
        let input_path = project_path.join(format!("part{}.wav", self.current_chunk_index));
        if !input_path.exists() {
            return Err("AudioIO skip_to error: File does not exist.".to_string());
        }

        match self
            .player
            .play_at(input_path.to_str().unwrap().to_string(), pos_ms)
        {
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

        let project_path = Path::new(&self.project_location);
        let input_path = project_path.join(format!("part{}.wav", self.current_chunk_index));
        if input_path.exists() {
            self.player_duration_ms = self
                .player
                .duration_of(input_path.to_str().unwrap().to_string());
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

        let project_path = Path::new(&self.project_location);
        let input_path = project_path.join(format!("part{}.wav", self.current_chunk_index));
        if input_path.exists() {
            self.player_duration_ms = self
                .player
                .duration_of(input_path.to_str().unwrap().to_string());
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

        let project_path = Path::new(&self.project_location);
        let input_path = project_path.join(format!("part{}.wav", self.current_chunk_index));
        if input_path.exists() {
            self.player_duration_ms = self
                .player
                .duration_of(input_path.to_str().unwrap().to_string());
            return Ok(FileStatus::Exists);
        }

        Ok(FileStatus::New)
    }

    fn record(&mut self) -> Result<ProcessingStatus, String> {
        let project_path = Path::new(&self.project_location);
        let record_path = project_path
            .join(format!("part{}.wav", self.current_chunk_index))
            .to_str()
            .unwrap()
            .to_string();
        if let Err(msg) = self.recorder.record(record_path) {
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
