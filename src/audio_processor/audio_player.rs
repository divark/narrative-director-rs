use std::fs::File;
use cpal::{Device, Stream, PlayStreamError, PauseStreamError, InputCallbackInfo, OutputCallbackInfo, StreamConfig, default_host, SupportedStreamConfig, SampleFormat};
use cpal::traits::{StreamTrait, DeviceTrait, HostTrait};
use hound::WavReader;
use std::io::BufReader;

pub struct AudioPlayer {
    output_device: Device,
    output_stream: Stream,
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {:?}", err);
}

impl AudioPlayer {
    fn new(input_file_name: String) -> AudioPlayer {
        let mut file_decoder = WavReader::open(input_file_name).unwrap();

        let host = default_host();
        let output_device = host.default_output_device().unwrap();
        let output_config: StreamConfig = output_device.default_output_config().unwrap().into();
        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for (dst, src) in data.iter_mut().zip(file_decoder.samples::<f32>()) {
                *dst = src.unwrap_or(0.0);
            }
        };

        let output_stream = output_device.build_output_stream(&output_config, output_data_fn, err_fn).unwrap();

        AudioPlayer {
            output_device,
            output_stream
        }
    }

    pub fn play(&self) -> Result<(), PlayStreamError> {
        let output_playing_status = self.output_stream.play();
        if let Err(output_stream_error) = output_playing_status {
            return Err(output_stream_error);
        }

        Ok(())
    }

    pub fn pause(&self) -> Result<(), PauseStreamError> {
        let output_pausing_status = self.output_stream.pause();
        if let Err(output_stream_error) = output_pausing_status {
            return Err(output_stream_error);
        }

        Ok(())
    }

    pub fn stop(&self) {

    }
}

#[cfg(test)]
mod tests {
    use crate::audio_processor::audio_player::AudioPlayer;
    use std::fs::File;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn plays_wav_file() {
        let audio_player = AudioPlayer::new(String::from(concat!(env!("CARGO_MANIFEST_DIR"), "/test.wav")));

        let status = audio_player.play();
        sleep(Duration::from_secs(3));

        assert!(status.is_ok());
    }

    #[test]
    fn pauses_playing_wav_file() {
        let audio_player = AudioPlayer::new(String::from(concat!(env!("CARGO_MANIFEST_DIR"), "/test.wav")));

        let playing_status = audio_player.play();
        sleep(Duration::from_secs(1));
        assert!(playing_status.is_ok());

        let paused_status = audio_player.pause();
        sleep(Duration::from_secs(2));
        assert!(paused_status.is_ok());
    }
}