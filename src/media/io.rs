use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, RwLock};

use fltk::app;
use fltk::frame::Frame;
use fltk::prelude::{DisplayExt, ValuatorExt, WidgetExt};
use fltk::text::TextDisplay;
use fltk::valuator::HorNiceSlider;
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    default_host, Device, FromSample, SampleFormat, SampleRate, Stream, StreamConfig,
    SupportedStreamConfig,
};
use hound::{WavReader, WavSpec, WavWriter};

use serde::{Deserialize, Serialize};

use anyhow::{bail, Result};

use crate::ui::app::{MainUIWidgets, MediaTrackingWidgets};

#[derive(Clone)]
struct PlaybackWidget {
    time_label: Frame,
    progress_bar: HorNiceSlider,
    status_bar: TextDisplay,
}

/// Converts seconds to hours:minutes:seconds format
fn to_hh_mm_ss_str(secs: usize) -> String {
    let seconds = secs % 60;
    let minutes = (seconds / 60) % 60;
    let hours = minutes / 60;

    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

impl PlaybackWidget {
    pub fn new(
        time_label: Frame,
        progress_bar: HorNiceSlider,
        status_bar: TextDisplay,
    ) -> PlaybackWidget {
        PlaybackWidget {
            time_label,
            progress_bar,
            status_bar,
        }
    }

    pub fn set_current(&mut self, pos_secs: usize) {
        self.progress_bar.set_value(pos_secs as f64);
    }

    pub fn current(&self) -> usize {
        self.progress_bar.value() as usize
    }

    pub fn set_total(&mut self, total_secs: usize) {
        self.progress_bar.set_bounds(0.0, total_secs as f64);
    }

    pub fn total(&self) -> usize {
        self.progress_bar.maximum() as usize
    }

    pub fn update_playback(&mut self) {
        let current_pos = self.progress_bar.value() as usize;
        let total = self.progress_bar.maximum() as usize;

        let playback_time = format!(
            "{}/{}",
            to_hh_mm_ss_str(current_pos),
            to_hh_mm_ss_str(total)
        );

        self.time_label.set_label(&playback_time);
        app::awake();
    }

    pub fn update_recording(&mut self) {
        let total = self.progress_bar.maximum() as usize;
        self.set_current(total);

        let playback_time = format!("{}/{}", to_hh_mm_ss_str(total), to_hh_mm_ss_str(total));

        self.time_label.set_label(&playback_time);
        app::awake();
    }

    pub fn reset(&mut self) {
        self.progress_bar.set_bounds(0.0, 0.0);
        self.clear_notification();
    }

    pub fn notify_recording_complete(&mut self, filepath: &str) {
        self.status_bar
            .buffer()
            .unwrap()
            .set_text(&format!("Recording complete: {filepath}"));
        app::awake();
    }

    pub fn clear_notification(&mut self) {
        self.status_bar.buffer().unwrap().set_text("");
        app::awake();
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum MediaStates {
    Playing,
    Paused,
    Recording,

    StoppedPlaying,
    StoppedRecording,
}

pub struct Media {
    stream_updater: Sender<SenderMessages>,
    media_state: Arc<RwLock<MediaStates>>,

    audio_location: Option<PathBuf>,
}

enum SenderMessages {
    Load(usize),
    Clear,

    Play(AudioOutput, PathBuf),
    Record(AudioInput, PathBuf),
    PauseAt(usize),
    StopIfPaused,
}

fn spawn_media_ui_modifier(
    media_state: Arc<RwLock<MediaStates>>,
    msg_receiver: Receiver<SenderMessages>,
    mut playback_widget: PlaybackWidget,
    mut ui_widgets: MainUIWidgets,
) {
    thread::spawn(move || {
        let mut prev_button_active = false;
        let mut next_button_active = false;

        while let Ok(sender_msg) = msg_receiver.recv() {
            match sender_msg {
                SenderMessages::Play(output_device, audio_file_path) => {
                    // There's no way we would be performing playback when there are no entries
                    // seen in the Paragraph Viewer, so we want to capture if they were active
                    // when we are in a valid situation looking at text.
                    if ui_widgets.prev_button.active() || ui_widgets.next_button.active() {
                        prev_button_active = ui_widgets.prev_button.active();
                        ui_widgets.prev_button.deactivate();
                        next_button_active = ui_widgets.next_button.active();
                        ui_widgets.next_button.deactivate();
                    }

                    ui_widgets.play_button.set_label("Pause");
                    ui_widgets.record_button.deactivate();
                    ui_widgets.stop_button.activate();
                    ui_widgets.open_menu_item.deactivate();
                    app::awake();

                    let mut current_pos_secs = playback_widget.current();
                    let total_secs = playback_widget.total();
                    let (_audio, _) = output_stream_from(
                        output_device.to_device(),
                        current_pos_secs,
                        audio_file_path,
                    )
                    .expect("Could not start playing audio.");

                    while *media_state
                        .read()
                        .expect("Could not check for playing state")
                        == MediaStates::Playing
                        && current_pos_secs < total_secs
                    {
                        thread::sleep(Duration::from_secs(1));
                        current_pos_secs += 1;
                        playback_widget.set_current(current_pos_secs);
                        playback_widget.update_playback();

                        if current_pos_secs == total_secs {
                            *media_state.write().expect(
                                "Could not change state to stoppedplaying on reaching duration",
                            ) = MediaStates::StoppedPlaying;
                        }
                    }

                    let current_state = *media_state
                        .read()
                        .expect("Could not check whether paused or stopped on playback.");
                    if current_state == MediaStates::Paused {
                        ui_widgets.play_button.set_label("Play");
                    } else if current_state == MediaStates::StoppedPlaying {
                        ui_widgets.play_button.set_label("Play");
                        ui_widgets.record_button.activate();
                        ui_widgets.stop_button.deactivate();
                        ui_widgets.open_menu_item.activate();

                        if prev_button_active {
                            ui_widgets.prev_button.activate();
                        }

                        if next_button_active {
                            ui_widgets.next_button.activate();
                        }

                        playback_widget.set_current(0);
                        playback_widget.update_playback();
                    }
                }
                SenderMessages::Record(input_device, new_audio_file_path) => {
                    prev_button_active = ui_widgets.prev_button.active();
                    ui_widgets.prev_button.deactivate();
                    next_button_active = ui_widgets.next_button.active();
                    ui_widgets.next_button.deactivate();

                    ui_widgets.open_menu_item.deactivate();
                    ui_widgets.play_button.deactivate();
                    ui_widgets.stop_button.activate();
                    ui_widgets.record_button.deactivate();
                    app::awake();

                    let recording_status = input_stream_from(
                        input_device.to_device(),
                        input_device.config(),
                        new_audio_file_path.clone(),
                    );
                    if recording_status.is_err() {
                        continue;
                    }

                    let _recording_stream = recording_status.expect("Could not start recording.");

                    let mut current_pos_secs = 0;
                    while *media_state
                        .read()
                        .expect("Could not check if in recording state.")
                        == MediaStates::Recording
                    {
                        thread::sleep(Duration::from_secs(1));
                        current_pos_secs += 1;

                        playback_widget.set_current(current_pos_secs);
                        playback_widget.set_total(current_pos_secs);
                        playback_widget.update_recording();
                    }

                    // NOTE: Pausing is not currently supported, so the state should only be in StoppedRecording.
                    let current_state = *media_state
                        .read()
                        .expect("Could not check if in StoppedRecording state.");
                    assert!(current_state == MediaStates::StoppedRecording);

                    if prev_button_active {
                        ui_widgets.prev_button.activate();
                    }

                    if next_button_active {
                        ui_widgets.next_button.activate();
                    }

                    ui_widgets.open_menu_item.activate();
                    ui_widgets.play_button.activate();
                    ui_widgets.stop_button.deactivate();
                    ui_widgets.record_button.activate();

                    playback_widget
                        .notify_recording_complete(new_audio_file_path.to_str().unwrap());
                    playback_widget.set_current(0);
                    playback_widget.update_playback();
                }
                SenderMessages::PauseAt(current_pos_secs) => {
                    playback_widget.set_current(current_pos_secs);
                    playback_widget.update_playback();
                }
                SenderMessages::StopIfPaused => {
                    ui_widgets.play_button.set_label("Play");
                    ui_widgets.record_button.activate();
                    ui_widgets.stop_button.deactivate();
                    ui_widgets.open_menu_item.activate();

                    if prev_button_active {
                        ui_widgets.prev_button.activate();
                    }

                    if next_button_active {
                        ui_widgets.next_button.activate();
                    }

                    playback_widget.set_current(0);
                    playback_widget.update_playback();
                }
                SenderMessages::Load(length) => {
                    playback_widget.clear_notification();

                    playback_widget.set_current(0);
                    playback_widget.set_total(length);

                    ui_widgets.play_button.activate();
                    ui_widgets.stop_button.deactivate();

                    ui_widgets.record_button.activate();

                    playback_widget.update_playback();
                }
                SenderMessages::Clear => {
                    playback_widget.reset();

                    ui_widgets.play_button.deactivate();
                    ui_widgets.stop_button.deactivate();

                    ui_widgets.record_button.activate();

                    playback_widget.update_playback();
                }
            }
            app::awake();
        }
    });
}

impl Media {
    pub fn new(ui_widgets: MainUIWidgets, media_widgets: MediaTrackingWidgets) -> Media {
        let playback_widget = PlaybackWidget::new(
            media_widgets.time_progress_label,
            media_widgets.progress_bar,
            media_widgets.status_bar,
        );

        let media_state = Arc::new(RwLock::new(MediaStates::StoppedPlaying));

        let (stream_updater, rx) = mpsc::channel();
        spawn_media_ui_modifier(media_state.clone(), rx, playback_widget, ui_widgets);

        Media {
            stream_updater,
            media_state,

            audio_location: None,
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
                self.stream_updater
                    .send(SenderMessages::Load(length))
                    .expect("Load: Could not load current audio file.");
            }
            Err(_) => {
                self.stream_updater
                    .send(SenderMessages::Clear)
                    .expect("Load: Could not reset UI.");
            }
        }
    }

    pub fn play(&mut self, output_device: &AudioOutput) {
        let current_state = *self
            .media_state
            .read()
            .expect("Could not check state for pausing playback");
        if current_state == MediaStates::Playing {
            *self
                .media_state
                .write()
                .expect("Could not acquire lock to change state to paused") = MediaStates::Paused;
            return;
        }

        *self
            .media_state
            .write()
            .expect("Could not acquire lock to change state to playing") = MediaStates::Playing;
        self.stream_updater
            .send(SenderMessages::Play(
                output_device.clone(),
                self.audio_location.as_ref().unwrap().clone(),
            ))
            .expect("Could not communicate to thread to start playing");
    }

    pub fn pause_at(&mut self, current_pos_secs: usize) {
        if *self
            .media_state
            .read()
            .expect("Could not check if in recording state to prevent pausing")
            == MediaStates::Recording
        {
            return;
        }

        *self
            .media_state
            .write()
            .expect("Could not acquire lock to change state to paused") = MediaStates::Paused;
        self.stream_updater
            .send(SenderMessages::PauseAt(current_pos_secs))
            .expect("Could not communicate to thread to pause playback");
    }

    pub fn record(&mut self, input_device: &AudioInput) {
        *self
            .media_state
            .write()
            .expect("Could not acquire lock to change state to recording") = MediaStates::Recording;
        self.stream_updater
            .send(SenderMessages::Record(
                input_device.clone(),
                self.audio_location.as_ref().unwrap().clone(),
            ))
            .expect("Could not communicate to thread to start recording");
    }

    /// Stops the current playback or recording, reverting the playback widgets
    /// back to normal.
    pub fn stop(&mut self) {
        let current_state = *self
            .media_state
            .read()
            .expect("Could not check state for stopping playback or recording");
        if current_state == MediaStates::Playing {
            *self
                .media_state
                .write()
                .expect("Could not change state to StoppedPlaying") = MediaStates::StoppedPlaying;
        } else if current_state == MediaStates::Recording {
            *self
                .media_state
                .write()
                .expect("Could not change state to StoppedRecording") =
                MediaStates::StoppedRecording;
        } else if current_state == MediaStates::Paused {
            *self
                .media_state
                .write()
                .expect("Could not change state to StoppedPlaying") = MediaStates::StoppedPlaying;
            self.stream_updater
                .send(SenderMessages::StopIfPaused)
                .expect("Could not communicate to thread to stop if paused");
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
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
            });

        // WORKAROUND: Use the default output device if the one we asked for
        // wasn't found.
        if output_device.is_none() {
            return host
                .default_output_device()
                .expect("No default output device backup found. PANIC!");
        }

        output_device.expect("Unable to retrieve found output device.")
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
            });

        // WORKAROUND: Use the default input device if the one we asked for
        // wasn't found.
        if input_device.is_none() {
            return host
                .default_input_device()
                .expect("No default input device backup found. PANIC!");
        }

        input_device.expect("Unable to retrieve found input device.")
    }

    pub fn config(&self) -> SupportedStreamConfig {
        let input_device = self.to_device();

        let desired_sample_rate = SampleRate(self.sample_rate);

        input_device
            .supported_input_configs()
            .expect("No input configs found. No inputs in general?")
            .find(|config| {
                config.channels() == self.channels
                    && desired_sample_rate >= config.min_sample_rate()
                    && desired_sample_rate <= config.max_sample_rate()
            })
            .expect("Could not find a config with the desired channel and sample rate")
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
    let samples_to_skip = (starting_pos_secs as u32) * sample_rate;

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

            output_device.build_output_stream(
                &stream_config,
                output_data_fn,
                |error| eprintln!("an error occurred on stream: {error:?}"),
                None,
            )?
        }
        (16, hound::SampleFormat::Int) => {
            let output_data_fn = move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                for (dst, src) in data.iter_mut().zip(file_decoder.samples::<i16>()) {
                    *dst = src.unwrap_or(0);
                }
            };

            output_device.build_output_stream(
                &stream_config,
                output_data_fn,
                |error| eprintln!("an error occurred on stream: {error:?}"),
                None,
            )?
        }
        _ => {
            bail!("Unsupported SampleFormat found for playback.");
        }
    };

    let duration_secs = (num_samples as f64 / sample_rate as f64).round() as usize;

    output_stream.play()?;
    Ok((output_stream, duration_secs))
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    match format {
        cpal::SampleFormat::U16 => hound::SampleFormat::Int,
        cpal::SampleFormat::I16 => hound::SampleFormat::Int,
        cpal::SampleFormat::F32 => hound::SampleFormat::Float,
        _ => panic!("Sample format: Incompatible format found."),
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
    U: cpal::Sample + hound::Sample + FromSample<T>,
{
    for &sample in input.iter() {
        let sample: U = U::from_sample(sample);
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
        eprintln!("IO Recording error: {err}");
    };

    // Use the config to hook up the input (Some microphone) to the output (A file)
    let io_stream = match input_config.sample_format() {
        SampleFormat::F32 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &mut writer),
            err_fn,
            None,
        )?,
        SampleFormat::I16 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input_data::<i16, i16>(data, &mut writer),
            err_fn,
            None,
        )?,
        SampleFormat::U16 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input_data::<u16, i16>(data, &mut writer),
            err_fn,
            None,
        )?,
        _ => panic!("Input Stream: Incompatible format found."),
    };

    io_stream.play()?;
    Ok(io_stream)
}
