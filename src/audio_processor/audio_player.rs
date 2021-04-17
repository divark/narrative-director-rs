use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{default_host, Device, PauseStreamError, PlayStreamError, Stream, StreamConfig};
use hound::WavReader;

pub struct CpalAudioPlayer {
    input_file_name: String,

    output_device: Device,
    output_stream: Stream,
    output_duration_secs: u32,
    output_paused_pos_sec: u32,
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {:?}", err);
}

// Returns a stream that'll broadcast the input file provided, as well as the expected duration.
fn output_stream_from(output_device: &Device, input_file_name: String) -> (Stream, u32) {
    let mut file_decoder = WavReader::open(input_file_name).unwrap();
    let num_samples = file_decoder.duration();
    let sample_rate = file_decoder.spec().sample_rate;

    let output_config: StreamConfig = output_device.default_output_config().unwrap().into();
    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        for (dst, src) in data.iter_mut().zip(file_decoder.samples::<f32>()) {
            *dst = src.unwrap_or(0.0);
        }
    };

    let output_stream = output_device
        .build_output_stream(&output_config, output_data_fn, err_fn)
        .unwrap();
    let duration_secs = (num_samples as f64 / sample_rate as f64).round() as u32;

    (output_stream, duration_secs)
}

impl CpalAudioPlayer {
    pub fn new(input_file_name: String) -> CpalAudioPlayer {
        let host = default_host();
        let output_device = host.default_output_device().unwrap();
        let output_info = output_stream_from(&output_device, input_file_name.clone());

        CpalAudioPlayer {
            input_file_name,

            output_device,
            output_stream: output_info.0,
            output_duration_secs: output_info.1,
            output_paused_pos_sec: 0,
        }
    }

    // Returns the number of seconds to play audio.
    pub fn play(&self) -> Result<u32, PlayStreamError> {
        let output_playing_status = self.output_stream.play();
        if let Err(output_stream_error) = output_playing_status {
            return Err(output_stream_error);
        }

        Ok(self.output_duration_secs - self.output_paused_pos_sec)
    }

    pub fn pause(&mut self, paused_loc_secs: u32) -> Result<(), PauseStreamError> {
        let output_pausing_status = self.output_stream.pause();
        if let Err(output_stream_error) = output_pausing_status {
            return Err(output_stream_error);
        }

        self.output_paused_pos_sec = paused_loc_secs;

        Ok(())
    }

    pub fn stop(&mut self) {
        self.output_paused_pos_sec = 0;
        self.output_stream =
            output_stream_from(&self.output_device, self.input_file_name.clone()).0;
    }
}

#[cfg(test)]
mod tests {
    use crate::audio_processor::audio_player::CpalAudioPlayer;
    use std::thread::sleep;
    use std::time::Duration;

    const AUDIO_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test.wav");

    #[test]
    fn plays_wav_file() {
        let audio_player = CpalAudioPlayer::new(String::from(AUDIO_FILE));
        let expected_duration_secs = 3;

        let status = audio_player.play();
        assert!(status.is_ok());

        let actual_duration_secs = status.unwrap();
        assert_eq!(expected_duration_secs, actual_duration_secs);

        sleep(Duration::from_secs(actual_duration_secs as u64));
    }

    #[test]
    fn pauses_playing_wav_file() {
        let mut audio_player = CpalAudioPlayer::new(String::from(AUDIO_FILE));
        let expected_duration_secs = 3;

        let playing_status = audio_player.play();
        sleep(Duration::from_secs(1));
        assert!(playing_status.is_ok());

        let paused_status = audio_player.pause(1);
        sleep(Duration::from_secs(2));
        assert!(paused_status.is_ok());

        let playing_status = audio_player.play();
        assert!(playing_status.is_ok());

        let actual_duration_secs = playing_status.unwrap();
        assert_eq!(expected_duration_secs - 1, actual_duration_secs);

        sleep(Duration::from_secs(actual_duration_secs as u64));
    }

    #[test]
    fn stops_playing_audio() {
        let mut audio_player = CpalAudioPlayer::new(String::from(AUDIO_FILE));

        let playing_status = audio_player.play();
        assert!(playing_status.is_ok());

        let duration_to_wait = playing_status.unwrap() - 2;
        sleep(Duration::from_secs(duration_to_wait as u64));

        audio_player.stop();
    }

    #[test]
    fn can_play_again_after_stopped() {
        let mut audio_player = CpalAudioPlayer::new(String::from(AUDIO_FILE));

        let playing_status = audio_player.play();
        assert!(playing_status.is_ok());

        let duration_to_wait = playing_status.unwrap() - 2;
        sleep(Duration::from_secs(duration_to_wait as u64));

        audio_player.stop();

        // Now let's play again
        let playing_status = audio_player.play();
        assert!(playing_status.is_ok());

        let duration_to_wait = playing_status.unwrap();
        sleep(Duration::from_secs(duration_to_wait as u64));
    }
}
