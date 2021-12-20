use std::path::PathBuf;

use glib::{source_remove, MainContext, SourceId};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, SampleRate, Stream, StreamConfig};
use hound::WavReader;

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
fn to_hh_mm_ss_str(ms: usize) -> String {
    let seconds = ms / 1000;
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

    pub fn set_current(&mut self, pos: usize) {
        self.current_pos = pos;
    }

    pub fn set_total(&mut self, total: usize) {
        self.total = total;
    }

    pub fn update(&mut self) {
        let secs_passed = (self.current_pos / 1000) as f64;
        let secs_total = (self.total / 1000) as f64;

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
    pub play_button: gtk::Button,
    pub stop_button: gtk::Button,
    pub record_button: gtk::Button,

    pub progress_bar: gtk::Scrollbar,
    pub time_progress_label: gtk::Label,
}

pub struct Media {
    audio_location: Option<PathBuf>,
    playback_updater: Option<SourceId>,

    play_button: gtk::Button,
    stop_button: gtk::Button,
    record_button: gtk::Button,

    playback_widget: PlaybackWidget,
}

impl Media {
    pub fn new(widgets: MediaWidgets) -> Media {
        let playback_widget =
            PlaybackWidget::new(widgets.time_progress_label, widgets.progress_bar);

        Media {
            audio_location: None,
            playback_updater: None,

            play_button: widgets.play_button,
            stop_button: widgets.stop_button,
            record_button: widgets.record_button,

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

        self.playback_widget.update();
    }

    pub fn play(&mut self, output_device: Device) {
        let play_button_label = self
            .play_button
            .label()
            .expect("Could not read text from Play Button.");

        if play_button_label == "Pause" {
            let source = self.playback_updater.take().unwrap();
            source_remove(source);
            self.playback_updater = None;

            self.play_button.set_label("Play");
            return;
        }

        let (tx, rx) = MainContext::channel(glib::PRIORITY_DEFAULT);

        let audio_location = self.audio_location.as_ref().unwrap().clone();
        let start_pos_ms = (self.playback_widget.progress_bar.value() as usize) * 1000;

        thread::spawn(move || {
            let found_stream = output_stream_from(output_device, start_pos_ms, audio_location);
            if found_stream.is_err() {
                return;
            }

            let (_stream, duration_ms) = found_stream.unwrap();
            let mut current_pos_secs = start_pos_ms / 1000;
            let duration_secs = duration_ms / 1000;

            while current_pos_secs <= duration_secs {
                let send_result = tx.send(current_pos_secs);
                if send_result.is_err() {
                    return;
                }

                current_pos_secs += 1;

                thread::sleep(Duration::from_secs(1));
            }
        });

        let play_button = self.play_button.clone();
        let record_button = self.record_button.clone();
        let stop_button = self.stop_button.clone();

        let mut playback_widgets_clone = self.playback_widget.clone();
        let playback_id = rx.attach(None, move |msg| {
            play_button.set_label("Pause");
            record_button.set_sensitive(false);
            stop_button.set_sensitive(true);

            playback_widgets_clone.set_current(msg * 1000);
            playback_widgets_clone.update();

            if playback_widgets_clone.current_pos == playback_widgets_clone.total {
                play_button.set_label("Play");
                record_button.set_sensitive(true);
                stop_button.set_sensitive(false);

                playback_widgets_clone.set_current(0);
                playback_widgets_clone.update();
                glib::Continue(false);
            }

            glib::Continue(true)
        });

        self.playback_updater = Some(playback_id);
    }

    pub fn stop(&mut self) {
        let source = self.playback_updater.take().unwrap();
        source_remove(source);
        self.playback_updater = None;

        self.playback_widget.set_current(0);
        self.playback_widget.update();

        self.play_button.set_label("Play");
        self.record_button.set_sensitive(true);
        self.stop_button.set_sensitive(false);
    }
}

/// Returns a stream that'll broadcast the input file provided, as well as the expected duration in milliseconds.
fn output_stream_from(
    output_device: Device,
    starting_pos_ms: usize,
    input_file: PathBuf,
) -> Result<(Stream, usize)> {
    let mut file_decoder = WavReader::open(input_file)?;
    let num_samples = file_decoder.duration();
    let sample_rate = file_decoder.spec().sample_rate;
    let channels = file_decoder.spec().channels;
    let samples_to_skip = (starting_pos_ms as u32 / 1000) * (sample_rate as u32);

    if samples_to_skip > num_samples {
        bail!("output_stream_from error: Starting position exceeds file time.");
    }

    file_decoder.seek(samples_to_skip)?;

    let mut output_config: StreamConfig = output_device.default_output_config()?.into();
    output_config.sample_rate = SampleRate(sample_rate);
    output_config.channels = channels;
    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        for (dst, src) in data.iter_mut().zip(file_decoder.samples::<f32>()) {
            *dst = src.unwrap_or(0.0);
        }
    };

    let output_stream =
        output_device.build_output_stream(&output_config, output_data_fn, |error| {
            eprintln!("an error occurred on stream: {:?}", error)
        })?;

    let duration_ms = ((num_samples as f64 / sample_rate as f64).round() as usize) * 1000;

    Ok((output_stream, duration_ms))
}
