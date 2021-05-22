use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{default_host, Device, SampleFormat, Stream, SupportedStreamConfig};
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

    pub fn get_input_devices(&self) -> Vec<Device> {
        let host = default_host();

        host.input_devices().unwrap().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::media_io::audio_recorder::CpalAudioRecorder;
    use std::fs;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
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

        assert!(fs::metadata(file_to_record.clone()).unwrap().is_file());
        assert_ne!(first_created_time, modified_time);
    }
}
