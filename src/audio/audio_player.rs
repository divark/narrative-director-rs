use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{default_host, Device, PauseStreamError, PlayStreamError, Stream, StreamConfig};
use hound::WavReader;
use std::path::Path;

pub struct CpalAudioPlayer {
    output_device: Device,
    output_stream: Option<Stream>,

    output_duration_ms: u32,
    output_paused_pos_ms: u32,
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {:?}", err);
}

/// Returns a stream that'll broadcast the input file provided, as well as the expected duration in milliseconds.
fn output_stream_from(
    output_device: &Device,
    starting_pos_ms: u32,
    input_file_name: String,
) -> (Stream, u32) {
    let mut file_decoder = WavReader::open(input_file_name).unwrap();
    let num_samples = file_decoder.duration();
    let sample_rate = file_decoder.spec().sample_rate;
    let samples_to_skip = (starting_pos_ms / 1000) * sample_rate;

    if samples_to_skip > num_samples {
        panic!("output_stream_from error: Starting position exceeds file time.");
    }

    file_decoder.seek(samples_to_skip).unwrap();

    let output_config: StreamConfig = output_device.default_output_config().unwrap().into();
    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        for (dst, src) in data.iter_mut().zip(file_decoder.samples::<f32>()) {
            *dst = src.unwrap_or(0.0);
        }
    };

    let output_stream = output_device
        .build_output_stream(&output_config, output_data_fn, err_fn)
        .unwrap();
    let duration_ms = ((num_samples as f64 / sample_rate as f64).round() as u32) * 1000;

    (output_stream, duration_ms)
}

impl CpalAudioPlayer {
    pub fn new() -> Self {
        let host = default_host();
        let output_device = host.default_output_device().unwrap();

        Self {
            output_device,
            output_stream: None,
            output_duration_ms: 0,
            output_paused_pos_ms: 0,
        }
    }

    /// Returns the number of milliseconds to play audio.
    pub fn play(&mut self, input_file_name: String) -> Result<u32, PlayStreamError> {
        if self.output_stream.is_none() {
            let (output_stream, duration) =
                output_stream_from(&self.output_device, 0, input_file_name);

            self.output_stream = Some(output_stream);
            self.output_duration_ms = duration;
        }

        let output_playing_status = self.output_stream.as_ref().unwrap().play();
        if let Err(output_stream_error) = output_playing_status {
            return Err(output_stream_error);
        }

        Ok(self.output_duration_ms - self.output_paused_pos_ms)
    }

    /// Returns the number of milliseconds to play audio.
    pub fn play_at(
        &mut self,
        input_file_name: String,
        audio_pos_ms: u32,
    ) -> Result<u32, PlayStreamError> {
        self.stop();

        let (output_stream, duration) =
            output_stream_from(&self.output_device, audio_pos_ms, input_file_name);
        self.output_stream = Some(output_stream);
        self.output_duration_ms = duration - audio_pos_ms;

        let output_playing_status = self.output_stream.as_ref().unwrap().play();
        if let Err(output_stream_error) = output_playing_status {
            return Err(output_stream_error);
        }

        Ok(self.output_duration_ms)
    }

    pub fn pause(&mut self, paused_loc_ms: u32) -> Result<(), PauseStreamError> {
        if self.output_stream.is_none() {
            return Ok(());
        }

        let output_pausing_status = self.output_stream.as_ref().unwrap().pause();
        if let Err(output_stream_error) = output_pausing_status {
            return Err(output_stream_error);
        }

        self.output_paused_pos_ms = paused_loc_ms;

        Ok(())
    }

    pub fn stop(&mut self) {
        self.output_paused_pos_ms = 0;
        self.output_duration_ms = 0;

        self.output_stream = None;
    }

    // Returns the duration of the specified audio file in milliseconds.
    pub fn duration_of(&self, input_file_name: String) -> u32 {
        if !Path::new(&input_file_name).exists() {
            return 0;
        }

        let (_, duration) = output_stream_from(&self.output_device, 0, input_file_name);

        duration
    }

    pub fn get_output_devices(&self) -> Vec<Device> {
        let host = default_host();

        host.output_devices().unwrap().collect()
    }

    pub fn set_output_device(&mut self, new_output_device: Device) {
        self.output_device = new_output_device;
    }
}

#[cfg(test)]
mod tests {
    use crate::audio::audio_player::CpalAudioPlayer;
    use std::thread::sleep;
    use std::time::Duration;

    const AUDIO_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test.wav");

    #[test]
    fn plays_wav_file() {
        let mut audio_player = CpalAudioPlayer::new();
        let expected_duration_secs = 3;

        let status = audio_player.play(String::from(AUDIO_FILE));
        assert!(status.is_ok());

        let actual_duration_secs = status.unwrap() / 1000;
        assert_eq!(expected_duration_secs, actual_duration_secs);

        sleep(Duration::from_secs(actual_duration_secs as u64));
    }

    #[test]
    fn pauses_playing_wav_file() {
        let mut audio_player = CpalAudioPlayer::new();
        let expected_duration_secs = 3;

        let playing_status = audio_player.play(String::from(AUDIO_FILE));
        sleep(Duration::from_secs(1));
        assert!(playing_status.is_ok());

        let paused_status = audio_player.pause(1);
        sleep(Duration::from_secs(2));
        assert!(paused_status.is_ok());

        let playing_status = audio_player.play(String::from(AUDIO_FILE));
        assert!(playing_status.is_ok());

        let actual_duration_secs = playing_status.unwrap() / 1000;
        assert_eq!(expected_duration_secs - 1, actual_duration_secs);

        sleep(Duration::from_secs(actual_duration_secs as u64));
    }

    #[test]
    fn stops_playing_audio() {
        let mut audio_player = CpalAudioPlayer::new();

        let playing_status = audio_player.play(String::from(AUDIO_FILE));
        assert!(playing_status.is_ok());

        let duration_to_wait = playing_status.unwrap() / 1000 - 2;
        sleep(Duration::from_secs(duration_to_wait as u64));

        audio_player.stop();
    }

    #[test]
    fn can_play_again_after_stopped() {
        let mut audio_player = CpalAudioPlayer::new();

        let playing_status = audio_player.play(String::from(AUDIO_FILE));
        assert!(playing_status.is_ok());

        let duration_to_wait = playing_status.unwrap() / 1000 - 2;
        sleep(Duration::from_secs(duration_to_wait as u64));

        audio_player.stop();

        // Now let's play again
        let playing_status = audio_player.play(String::from(AUDIO_FILE));
        assert!(playing_status.is_ok());

        let duration_to_wait = playing_status.unwrap() / 1000;
        sleep(Duration::from_secs(duration_to_wait as u64));
    }

    #[test]
    fn skips_to_2s_in() {
        let mut audio_player = CpalAudioPlayer::new();
        let expected_duration_to_wait = 1;

        let playing_status = audio_player.play_at(String::from(AUDIO_FILE), 2000);
        assert!(playing_status.is_ok());

        let actual_duration_to_wait = playing_status.unwrap() / 1000;
        assert_eq!(expected_duration_to_wait, actual_duration_to_wait);

        sleep(Duration::from_secs(actual_duration_to_wait as u64));
    }
}
