use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::rc::Rc;

use glib::{source_remove, MainContext, SourceId};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{
    default_host, Device, SampleFormat, SampleRate, Stream, StreamConfig, SupportedStreamConfig,
};
use hound::{WavReader, WavSpec, WavWriter};

use serde::{Deserialize, Serialize};

use anyhow::{bail, Result};

use gtk::prelude::*;
use gtk::Adjustment;

#[derive(Clone)]
struct PlaybackWidget {
    time_label: gtk::Label,
    progress_bar: gtk::Scrollbar,

    current_pos: usize,
    total: usize,
}

/// Converts ms to hours:minutes:seconds format
fn to_hh_mm_ss_str(secs: usize) -> String {
    let seconds = secs;
    let minutes = seconds / 60;
    let hours = minutes / 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

impl PlaybackWidget {
    pub fn new(time_label: gtk::Label, progress_bar: gtk::Scrollbar) -> PlaybackWidget {
        PlaybackWidget {
            time_label,
            progress_bar,

            current_pos: 0,
            total: 0,
        }
    }

    pub fn set_current(&mut self, pos_secs: usize) {
        self.current_pos = pos_secs;
    }

    pub fn set_total(&mut self, total_secs: usize) {
        self.total = total_secs;
    }

    pub fn update(&mut self) {
        let secs_passed = self.current_pos as f64;
        let secs_total = self.total as f64;

        self.progress_bar.set_adjustment(&Adjustment::new(
            secs_passed,
            0.0,
            secs_total,
            1.0,
            1.0,
            1.0,
        ));

        let playback_time = format!(
            "{}/{}",
            to_hh_mm_ss_str(self.current_pos),
            to_hh_mm_ss_str(self.total)
        );

        self.time_label.set_text(&playback_time);
    }

    pub fn reset(&mut self) {
        self.current_pos = 0;
        self.total = 0;
    }
}

pub struct MediaWidgets {
    pub open_menu_item: gtk::MenuItem,
    pub play_button: gtk::Button,
    pub stop_button: gtk::Button,
    pub record_button: gtk::Button,

    pub next_button: Rc<gtk::Button>,
    pub prev_button: Rc<gtk::Button>,

    pub progress_bar: gtk::Scrollbar,
    pub time_progress_label: gtk::Label,
}

pub struct Media {
    audio_location: Option<PathBuf>,
    stream_updater: Option<SourceId>,

    open_menu_item: gtk::MenuItem,

    play_button: gtk::Button,
    stop_button: gtk::Button,
    record_button: gtk::Button,

    next_button: Rc<gtk::Button>,
    prev_button: Rc<gtk::Button>,
    nav_button_state: (bool, bool),

    playback_widget: PlaybackWidget,
}

impl Media {
    pub fn new(widgets: MediaWidgets) -> Media {
        let playback_widget =
            PlaybackWidget::new(widgets.time_progress_label, widgets.progress_bar);

        Media {
            audio_location: None,
            stream_updater: None,

            open_menu_item: widgets.open_menu_item,

            play_button: widgets.play_button,
            stop_button: widgets.stop_button,
            record_button: widgets.record_button,

            next_button: widgets.next_button,
            prev_button: widgets.prev_button,
            nav_button_state: (false, false),

            playback_widget,
        }
    }

    pub fn load(&mut self, audio_file_location: PathBuf) {
        self.audio_location = Some(audio_file_location.clone());

        let host = cpal::default_host();
        let default_output_device = host
            .default_output_device()
            .expect("Unable to get default output device.");

        match output_stream_from(default_output_device, 0, audio_file_location) {
            Ok((_, length)) => {
                self.playback_widget.set_current(0);
                self.playback_widget.set_total(length);

                self.play_button.set_sensitive(true);
                self.stop_button.set_sensitive(false);
            }
            Err(_) => {
                self.playback_widget.reset();

                self.play_button.set_sensitive(false);
                self.stop_button.set_sensitive(false);
            }
        }

        self.record_button.set_sensitive(true);

        self.playback_widget.update();
    }

    pub fn play_at(&mut self, output_device: Device, pos_secs: usize) {
        let (tx, rx) = MainContext::channel(glib::PRIORITY_DEFAULT);

        let audio_location = self.audio_location.as_ref().unwrap().clone();

        thread::spawn(move || {
            let found_stream = output_stream_from(output_device, pos_secs, audio_location);
            if let Err(ref msg) = found_stream {
                eprintln!("Playback error: {}", msg);
                return;
            }

            let (_stream, duration_secs) = found_stream.unwrap();
            let mut current_pos_secs = pos_secs;

            while current_pos_secs <= duration_secs {
                let send_result = tx.send(current_pos_secs);
                if send_result.is_err() {
                    return;
                }

                current_pos_secs += 1;

                thread::sleep(Duration::from_secs(1));
            }
        });

        let open_menu_item = self.open_menu_item.clone();

        let play_button = self.play_button.clone();
        let record_button = self.record_button.clone();
        let stop_button = self.stop_button.clone();

        let next_button = self.next_button.clone();
        let prev_button = self.prev_button.clone();
        self.nav_button_state = (
            self.next_button.get_sensitive(),
            self.prev_button.get_sensitive(),
        );

        let (next_button_sensitivity, prev_button_sensitivity) = self.nav_button_state;

        let mut playback_widgets_clone = self.playback_widget.clone();
        let playback_id = rx.attach(None, move |new_pos_secs| {
            open_menu_item.set_sensitive(false);

            play_button.set_sensitive(false);
            record_button.set_sensitive(false);
            stop_button.set_sensitive(true);

            next_button.set_sensitive(false);
            prev_button.set_sensitive(false);

            playback_widgets_clone.set_current(new_pos_secs);
            playback_widgets_clone.update();

            if playback_widgets_clone.current_pos == playback_widgets_clone.total {
                open_menu_item.set_sensitive(true);

                play_button.set_sensitive(true);
                record_button.set_sensitive(true);
                stop_button.set_sensitive(false);

                next_button.set_sensitive(next_button_sensitivity);
                prev_button.set_sensitive(prev_button_sensitivity);
            }

            glib::Continue(true)
        });

        self.stream_updater = Some(playback_id);
    }

    pub fn play(&mut self, output_device: Device) {
        let progress_bar_pos_secs = self.playback_widget.progress_bar.value() as usize;

        let start_pos_secs = if progress_bar_pos_secs + 1 != self.playback_widget.total {
            progress_bar_pos_secs
        } else {
            0
        };

        self.play_at(output_device, start_pos_secs);
    }

    pub fn record(&mut self, input_device: &AudioInput) {
        let (tx, rx) = MainContext::channel(glib::PRIORITY_DEFAULT);

        let audio_location = self.audio_location.as_ref().unwrap().clone();
        let cpal_input_device = input_device.to_device();
        let cpal_input_config = input_device.config();

        thread::spawn(move || {
            let input_stream =
                input_stream_from(cpal_input_device, cpal_input_config, audio_location);
            if input_stream.is_err() {
                return;
            }

            let _stream = input_stream.unwrap();

            let mut current_pos_secs = 0;
            loop {
                let send_result = tx.send(current_pos_secs);
                if send_result.is_err() {
                    return;
                }

                current_pos_secs += 1;

                thread::sleep(Duration::from_secs(1));
            }
        });

        let open_menu_item = self.open_menu_item.clone();

        let play_button = self.play_button.clone();
        let record_button = self.record_button.clone();
        let stop_button = self.stop_button.clone();

        let next_button = self.next_button.clone();
        let prev_button = self.prev_button.clone();

        self.nav_button_state = (
            self.next_button.get_sensitive(),
            self.prev_button.get_sensitive(),
        );

        let mut playback_widgets_clone = self.playback_widget.clone();
        let recording_id = rx.attach(None, move |new_pos_secs| {
            open_menu_item.set_sensitive(false);

            play_button.set_sensitive(false);
            record_button.set_sensitive(false);
            stop_button.set_sensitive(true);

            next_button.set_sensitive(false);
            prev_button.set_sensitive(false);

            playback_widgets_clone.set_current(new_pos_secs);
            playback_widgets_clone.set_total(new_pos_secs);
            playback_widgets_clone.update();

            glib::Continue(true)
        });

        self.stream_updater = Some(recording_id);
    }

    /// Stops the current playback or recording, reverting the playback widgets
    /// back to normal.
    pub fn stop(&mut self) {
        if let Some(source) = self.stream_updater.take() {
            source_remove(source);
        }

        self.open_menu_item.set_sensitive(true);

        self.play_button.set_sensitive(true);
        self.record_button.set_sensitive(true);
        self.stop_button.set_sensitive(false);

        self.next_button.set_sensitive(self.nav_button_state.0);
        self.prev_button.set_sensitive(self.nav_button_state.1);
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct AudioOutput {
    output_device_name: String,
}

impl AudioOutput {
    pub fn new() -> AudioOutput {
        let host = default_host();
        let output_device = host
            .default_output_device()
            .expect("Could not retrieve a default output device.");

        AudioOutput {
            output_device_name: output_device
                .name()
                .unwrap_or_else(|_| "Default".to_string()),
        }
    }

    pub fn set_device_name(&mut self, name: String) {
        self.output_device_name = name;
    }

    pub fn device_name(&self) -> &str {
        &self.output_device_name
    }

    pub fn to_device(&self) -> Device {
        let host = default_host();
        let output_device = host
            .output_devices()
            .expect("No audio devices found for output.")
            .find(|device| {
                if let Ok(named_device) = device.name() {
                    named_device == self.output_device_name
                } else {
                    false
                }
            })
            .expect("Could not find output device.");

        output_device
    }
}

pub fn output_device_names() -> Vec<String> {
    let mut output_device_names = Vec::new();

    let host = default_host();
    let output_devices = host.output_devices().ok();
    if output_devices.is_none() {
        return output_device_names;
    }

    output_device_names = output_devices
        .unwrap()
        .filter_map(|device| device.name().ok())
        .collect::<Vec<String>>();

    output_device_names
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AudioInput {
    input_device_name: String,
    sample_rate: u32,
    channels: u16,
}

impl AudioInput {
    pub fn new() -> AudioInput {
        let host = default_host();
        let input_device = host
            .default_input_device()
            .expect("Could not retrieve a default input device.");

        let input_config = input_device
            .default_input_config()
            .expect("Could not retrieve the properties from the default input device.");

        AudioInput {
            input_device_name: input_device
                .name()
                .unwrap_or_else(|_| "Default".to_string()),
            sample_rate: input_config.sample_rate().0,
            channels: input_config.channels(),
        }
    }

    pub fn set_device_name(&mut self, name: String) {
        self.input_device_name = name;
    }

    pub fn device_name(&self) -> &str {
        &self.input_device_name
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn sample_rates(&self) -> Vec<u32> {
        let mut found_sample_rates = Vec::new();

        // 1: Get supported configurations
        let input_device = self.to_device();
        let supported_configs = input_device
            .supported_input_configs()
            .expect("Could not find input configs for calculating supported sample rates.");

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

    pub fn set_channels(&mut self, channels: u16) {
        self.channels = channels;
    }

    pub fn channel(&self) -> u16 {
        self.channels
    }

    pub fn channels(&self) -> Vec<u16> {
        let mut found_channels: Vec<u16> = Vec::new();

        // 1: Get supported configurations
        let input_device = self.to_device();
        let supported_configs = input_device
            .supported_input_configs()
            .expect("Could not find input configs for calculating supported channels.");

        // 2: Filter by channel count
        supported_configs.for_each(|config| {
            found_channels.push(config.channels());
        });

        found_channels.sort_unstable();
        found_channels.dedup();

        found_channels
    }

    pub fn to_device(&self) -> Device {
        let host = default_host();
        let input_device = host
            .input_devices()
            .expect("No audio devices found for output.")
            .find(|device| {
                if let Ok(named_device) = device.name() {
                    named_device == self.input_device_name
                } else {
                    false
                }
            })
            .expect("Could not find output device.");

        input_device
    }

    pub fn config(&self) -> SupportedStreamConfig {
        let input_device = self.to_device();

        input_device
            .supported_input_configs()
            .unwrap()
            .find(|config| config.channels() == self.channels)
            .expect("Could not find a device config with given sample rate and channels.")
            .with_sample_rate(SampleRate(self.sample_rate))
    }
}

pub fn input_device_names() -> Vec<String> {
    let mut input_device_names = Vec::new();

    let host = default_host();
    let input_devices = host.input_devices().ok();
    if input_devices.is_none() {
        return input_device_names;
    }

    input_device_names = input_devices
        .unwrap()
        .filter_map(|device| device.name().ok())
        .collect::<Vec<String>>();

    input_device_names
}

/// Returns a stream and duration in seconds tuple that will immediately start
/// playing audio from the specified output device and its starting position in
/// seconds from the location of the input file. An error is returned if something
/// went wrong in setting it up.
///
/// # Examples
///
/// ```
/// let host = cpal::default_host();
/// let default_output_device = host
///         .default_output_device()
///         .expect("Unable to get default output device.");
///
/// let default_output_config = default_output_device
///         .default_output_config()
///         .expect("Unable to get output's default config.");
///
/// let audio_path = Path::new("test.wav").to_path_buf();
///
/// let output_stream_result = output_stream_from(default_output_device, default_output_config, audio_path);
/// assert!(output_stream_result.is_ok());
/// ```
fn output_stream_from(
    output_device: Device,
    starting_pos_secs: usize,
    input_file: PathBuf,
) -> Result<(Stream, usize)> {
    let mut file_decoder = WavReader::open(input_file)?;
    let num_samples = file_decoder.duration();

    let file_spec = file_decoder.spec();
    let sample_rate = file_spec.sample_rate;
    let channels = file_spec.channels;
    let samples_to_skip = (starting_pos_secs as u32) * (sample_rate as u32);

    if samples_to_skip > num_samples {
        bail!("output_stream_from error: Starting position exceeds file time.");
    }

    file_decoder.seek(samples_to_skip)?;

    let output_config = output_device.default_output_config()?;
    let mut stream_config: StreamConfig = output_config.into();
    stream_config.sample_rate = SampleRate(sample_rate);
    stream_config.channels = channels;

    let output_stream = match (file_spec.bits_per_sample, file_spec.sample_format) {
        (32, hound::SampleFormat::Float) => {
            let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for (dst, src) in data.iter_mut().zip(file_decoder.samples::<f32>()) {
                    *dst = src.unwrap_or(0.0);
                }
            };

            output_device.build_output_stream(&stream_config, output_data_fn, |error| {
                eprintln!("an error occurred on stream: {:?}", error)
            })?
        }
        (16, hound::SampleFormat::Int) => {
            let output_data_fn = move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                for (dst, src) in data.iter_mut().zip(file_decoder.samples::<i16>()) {
                    *dst = src.unwrap_or(0);
                }
            };

            output_device.build_output_stream(&stream_config, output_data_fn, |error| {
                eprintln!("an error occurred on stream: {:?}", error)
            })?
        }
        _ => {
            bail!("Unsupported SampleFormat found for playback.");
        }
    };

    let duration_secs = (num_samples as f64 / sample_rate as f64).round() as usize;

    Ok((output_stream, duration_secs))
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

fn write_input_data<T, U>(input: &[T], writer: &mut WavWriter<BufWriter<File>>)
where
    T: cpal::Sample,
    U: cpal::Sample + hound::Sample,
{
    for &sample in input.iter() {
        let sample: U = cpal::Sample::from(&sample);
        writer.write_sample(sample).ok();
    }
}

/// Returns a stream that will immediately start recording audio from the specified
/// input device and its configuration (Sample Rate, Channels) to the location of the
/// input file. An error is returned if something went wrong in setting it up.
///
/// # Examples
///
/// ```
/// let host = cpal::default_host();
/// let default_input_device = host
///         .default_input_device()
///         .expect("Unable to get default input device.");
///
/// let default_input_config = default_input_device
///         .default_input_config()
///         .expect("Unable to get input's default config.");
///
/// let audio_path = Path::new("test.wav").to_path_buf();
///
/// let input_stream_result = input_stream_from(default_input_device, default_input_config, audio_path);
/// assert!(input_stream_result.is_ok());
/// ```
fn input_stream_from(
    input_device: Device,
    input_config: SupportedStreamConfig,
    input_file: PathBuf,
) -> Result<Stream> {
    let spec = wav_spec_from_config(&input_config);
    let mut writer = WavWriter::create(input_file, spec)?;

    let err_fn = move |err| {
        eprintln!("IO Recording error: {}", err);
    };

    // Use the config to hook up the input (Some microphone) to the output (A file)
    let io_stream = match input_config.sample_format() {
        SampleFormat::F32 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &mut writer),
            err_fn,
        )?,
        SampleFormat::I16 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input_data::<i16, i16>(data, &mut writer),
            err_fn,
        )?,
        SampleFormat::U16 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input_data::<u16, i16>(data, &mut writer),
            err_fn,
        )?,
    };

    Ok(io_stream)
}
