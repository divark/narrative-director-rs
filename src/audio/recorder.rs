use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    default_host, ChannelCount, Device, SampleFormat, SampleRate, Stream, SupportedStreamConfig,
};
use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: cpal::Sample,
    U: cpal::Sample + hound::Sample,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = cpal::Sample::from(&sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    match format {
        cpal::SampleFormat::U16 => hound::SampleFormat::Int,
        cpal::SampleFormat::I16 => hound::SampleFormat::Int,
        cpal::SampleFormat::F32 => hound::SampleFormat::Float,
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

fn device_config_from_sample_and_channels(
    device: &Device,
    sample_rate: u32,
    num_channels: u16,
) -> SupportedStreamConfig {
    device
        .supported_input_configs()
        .unwrap()
        .find(|config| config.channels() == num_channels)
        .expect("Could not find a device config with given sample rate and channels.")
        .with_sample_rate(SampleRate(sample_rate))
}

pub struct CpalAudioRecorder {
    config: SupportedStreamConfig,
    input_device: Device,

    io_stream: Option<Stream>,
}

impl CpalAudioRecorder {
    pub fn new() -> Self {
        let host = default_host();
        let device = host
            .default_input_device()
            .expect("Could not get default input device.");

        let config = device
            .default_input_config()
            .expect("Could not get default input config from device");

        Self {
            config,
            input_device: device,

            io_stream: None,
        }
    }

    pub fn record(&mut self, output_file_name: String) -> Result<(), String> {
        let spec = wav_spec_from_config(&self.config);
        let writer = WavWriter::create(output_file_name, spec).unwrap();
        let writer = Arc::new(Mutex::new(Some(writer)));

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        // Use the config to hook up the input (Some microphone) to the output (A file)
        let io_stream = match self.config.sample_format() {
            SampleFormat::F32 => self
                .input_device
                .build_input_stream(
                    &self.config.clone().into(),
                    move |data, _: &_| write_input_data::<f32, f32>(data, &writer),
                    err_fn,
                )
                .unwrap(),
            SampleFormat::I16 => self
                .input_device
                .build_input_stream(
                    &self.config.clone().into(),
                    move |data, _: &_| write_input_data::<i16, i16>(data, &writer),
                    err_fn,
                )
                .unwrap(),
            SampleFormat::U16 => self
                .input_device
                .build_input_stream(
                    &self.config.clone().into(),
                    move |data, _: &_| write_input_data::<u16, i16>(data, &writer),
                    err_fn,
                )
                .unwrap(),
        };

        match io_stream.play() {
            Ok(_) => {
                self.io_stream = Some(io_stream);
                Ok(())
            }
            Err(err) => Err(format!("record io_stream play error: {:?}", err)),
        }
    }

    pub fn stop(&mut self) {
        self.io_stream = None;
    }

    pub fn get_input_device(&self) -> &Device {
        &self.input_device
    }

    pub fn get_input_devices(&self) -> Vec<Device> {
        let host = default_host();

        host.input_devices().unwrap().collect()
    }

    pub fn get_channels_for_device(&self, input_device: &Device) -> Vec<ChannelCount> {
        let mut found_channels: Vec<ChannelCount> = Vec::new();

        // 1: Get supported configurations
        let supported_configs = input_device.supported_input_configs().unwrap();

        // 2: Filter by channel count
        supported_configs.for_each(|config| {
            found_channels.push(config.channels());
        });

        found_channels.sort_unstable();
        found_channels.dedup();

        found_channels
    }

    pub fn get_sample_rates_for_device(&self, input_device: &Device) -> Vec<u32> {
        let mut found_sample_rates: Vec<u32> = Vec::new();

        // 1: Get supported configurations
        let supported_configs = input_device.supported_input_configs().unwrap();

        // 2: Calculate sample rates using the min and max as reference.
        const SUPPORTED_SAMPLE_RATES: [u32; 6] = [16000, 32000, 44100, 48000, 88200, 96000];

        supported_configs.for_each(|config| {
            for sample_rate in SUPPORTED_SAMPLE_RATES {
                if sample_rate >= config.min_sample_rate().0
                    && sample_rate <= config.max_sample_rate().0
                {
                    found_sample_rates.push(sample_rate);
                }
            }
        });

        found_sample_rates.sort_unstable();
        found_sample_rates.dedup();

        found_sample_rates
    }

    pub fn set_input_device(
        &mut self,
        new_input_device: Device,
        sample_rate: u32,
        num_channels: ChannelCount,
    ) {
        self.config =
            device_config_from_sample_and_channels(&new_input_device, sample_rate, num_channels);
        self.input_device = new_input_device;
    }
}

// This test suite is ignored, as the Continuous Integration setup
// does not include any form of audio devices.
//
// This should still be tested locally with
// cargo test -- --ignored
#[cfg(test)]
mod tests {
    use crate::audio::recorder::CpalAudioRecorder;
    use std::fs;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    #[ignore]
    fn can_record_once() {
        let mut recorder = CpalAudioRecorder::new();

        let file_to_record = String::from(concat!(env!("CARGO_MANIFEST_DIR"), "/test.wav"));
        let time_to_wait_secs = 3;

        assert!(recorder.record(file_to_record.clone()).is_ok());
        sleep(Duration::from_secs(time_to_wait_secs));
        recorder.stop();

        assert!(fs::metadata(file_to_record.as_str()).unwrap().is_file());
    }

    #[test]
    #[ignore]
    fn can_record_twice() {
        let mut recorder = CpalAudioRecorder::new();

        let file_to_record = String::from(concat!(env!("CARGO_MANIFEST_DIR"), "/test.wav"));
        let time_to_wait_secs = 3;

        //First time
        assert!(recorder.record(file_to_record.clone()).is_ok());
        sleep(Duration::from_secs(time_to_wait_secs));
        recorder.stop();

        let first_created_time = fs::metadata(file_to_record.clone())
            .unwrap()
            .modified()
            .unwrap();

        //Second time
        assert!(recorder.record(file_to_record.clone()).is_ok());
        sleep(Duration::from_secs(time_to_wait_secs));
        recorder.stop();

        let modified_time = fs::metadata(file_to_record.clone())
            .unwrap()
            .modified()
            .unwrap();

        assert!(fs::metadata(file_to_record).unwrap().is_file());
        assert_ne!(first_created_time, modified_time);
    }

    #[test]
    #[ignore]
    fn has_atleast_one_input() {
        let recorder = CpalAudioRecorder::new();

        let input_devices = recorder.get_input_devices();
        assert!(!input_devices.is_empty());
    }

    #[test]
    #[ignore]
    fn some_input_has_channels() {
        let recorder = CpalAudioRecorder::new();

        let input_devices = recorder.get_input_devices();
        let first_input_device = input_devices.get(0);
        assert!(first_input_device.is_some());

        let first_input_device = first_input_device.unwrap();
        let input_channels = recorder.get_channels_for_device(first_input_device);
        assert!(!input_channels.is_empty());
    }

    #[test]
    #[ignore]
    fn has_valid_sample_rates() {
        let recorder = CpalAudioRecorder::new();

        let input_devices = recorder.get_input_devices();
        let first_input_device = input_devices.get(0);
        assert!(first_input_device.is_some());

        let first_input_device = first_input_device.unwrap();
        let input_sample_rates = recorder.get_sample_rates_for_device(first_input_device);

        assert!(!input_sample_rates.is_empty());

        const SAMPLE_RATES: [u32; 6] = [16000, 32000, 44100, 48000, 88200, 96000];
        let has_valid_sample_rate = input_sample_rates
            .iter()
            .any(|sample_rate| SAMPLE_RATES.contains(sample_rate));

        assert!(has_valid_sample_rate);
    }
}
