use crate::audio::{AudioIO, FileStatus, InputDeviceSelection, OutputDeviceSelection};
use crate::InputDevicesInfo;
use cpal::traits::DeviceTrait;
use cpal::Device;
use gtk::prelude::{ComboBoxExtManual, ComboBoxTextExt, LabelExt, RangeExt, WidgetExt};
use gtk::{Adjustment, Button, ComboBoxText, Label, Scrollbar};

/// Converts ms to hours:minutes:seconds format
pub fn to_hh_mm_ss_str(ms: u32) -> String {
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Populates the choices available for input devices
pub fn populate_input_options(input_options: &mut ComboBoxText, audio_io: &AudioIO) {
    let input_devices = audio_io.get_input_devices();
    input_options.remove_all();
    for input_device in &input_devices {
        let input_device_name = input_device.name().unwrap();

        input_options.append_text(&input_device_name);
    }

    let default_input_pos = input_devices
        .iter()
        .position(|input_device| {
            audio_io.get_input_device().name().unwrap() == *input_device.name().unwrap()
        })
        .unwrap() as u32;

    input_options.set_active(Some(default_input_pos));
}

/// Populates the choices available for output devices
pub fn populate_output_options(output_options: &mut ComboBoxText, audio_io: &AudioIO) {
    let output_devices = audio_io.get_output_devices();
    output_options.remove_all();
    for output_device in &output_devices {
        let output_device_name = output_device.name().unwrap();

        output_options.append_text(&output_device_name);
    }

    let default_output_pos = output_devices
        .iter()
        .position(|output_device| {
            audio_io.get_output_device().name().unwrap() == *output_device.name().unwrap()
        })
        .unwrap() as u32;

    output_options.set_active(Some(default_output_pos));
}

/// Populates fields for given Input Device
pub fn populate_input_preference_fields(
    input_device: &Device,
    input_devices_info: &InputDevicesInfo,
    input_widgets: &InputPreferenceWidgets,
) {
    // Starting with the channels
    input_widgets.channels_cbox.remove_all();
    let input_device_channels = input_devices_info
        .channels
        .get(&input_device.name().unwrap())
        .unwrap();

    for default_input_channel in input_device_channels {
        input_widgets
            .channels_cbox
            .append_text(&default_input_channel.to_string());
    }

    // Setting the combo box to point to the value of the default channel.
    let current_channel_pos = input_device_channels
        .iter()
        .position(|channel| input_device.default_input_config().unwrap().channels() == *channel)
        .unwrap() as u32;

    input_widgets
        .channels_cbox
        .set_active(Some(current_channel_pos));

    // Then the sample rates
    input_widgets.sample_rate_cbox.remove_all();
    let input_device_sample_rates = input_devices_info
        .sample_rates
        .get(&input_device.name().unwrap())
        .unwrap();

    for default_input_sample_rate in input_device_sample_rates {
        input_widgets
            .sample_rate_cbox
            .append_text(&default_input_sample_rate.to_string());
    }

    // Setting the combo box to point to the value of the default sample rate..
    let current_sample_rate_pos = input_device_sample_rates
        .iter()
        .position(|sample_rate| {
            input_device.default_input_config().unwrap().sample_rate().0 == *sample_rate
        })
        .unwrap() as u32;

    input_widgets
        .sample_rate_cbox
        .set_active(Some(current_sample_rate_pos));
}

pub struct InputPreferenceWidgets<'a> {
    pub input_device_cbox: &'a mut ComboBoxText,
    pub sample_rate_cbox: &'a mut ComboBoxText,
    pub channels_cbox: &'a mut ComboBoxText,
}

/// Returns an InputDeviceSelection from a user's choices in the Audio tab, under the
/// Input section.
pub fn get_input_selection_from(input_widgets: InputPreferenceWidgets) -> InputDeviceSelection {
    let sample_rate_choice = input_widgets
        .sample_rate_cbox
        .active_text()
        .unwrap()
        .to_string();
    let sample_rate = sample_rate_choice.parse::<u32>().unwrap();

    let channel_choice = input_widgets
        .channels_cbox
        .active_text()
        .unwrap()
        .to_string();
    let num_channels = channel_choice.parse::<u16>().unwrap();

    InputDeviceSelection {
        name: input_widgets
            .input_device_cbox
            .active_text()
            .unwrap()
            .to_string(),
        sample_rate,
        num_channels,
    }
}

pub struct OutputPreferenceWidgets<'a> {
    pub output_device_cbox: &'a ComboBoxText,
}

/// Returns an OutputDeviceSelection from a user's choices in the Audio tab, under the
/// output section.
pub fn get_output_selection_from(output_widgets: OutputPreferenceWidgets) -> OutputDeviceSelection {
    OutputDeviceSelection {
        name: output_widgets
            .output_device_cbox
            .active_text()
            .unwrap()
            .to_string(),
    }
}

/// Makes play button active or inactive relative to the file status.
pub fn change_play_button_sensitivity(file_status: FileStatus, play_button: &Button) {
    if file_status == FileStatus::Exists {
        play_button.set_sensitive(true);
    } else {
        play_button.set_sensitive(false);
    }
}

pub struct AudioPlaybackWidgets<'a> {
    pub progress_bar: &'a Scrollbar,
    pub progress_text: &'a Label,
}

pub struct AudioPlaybackProgress {
    pub ms_passed: u32,
    pub ms_total: u32,
}

/// Refreshes UI to take into account new audio file and its properties, such as length,
/// current progress, etc.
pub fn update_playback_widgets(
    playback_widgets: AudioPlaybackWidgets,
    playback_progress: AudioPlaybackProgress,
) {
    let secs_passed = (playback_progress.ms_passed / 1000) as f64;
    let secs_total = (playback_progress.ms_total / 1000) as f64;
    playback_widgets
        .progress_bar
        .set_adjustment(&Adjustment::new(
            secs_passed,
            0.0,
            secs_total,
            1.0,
            1.0,
            1.0,
        ));

    let progress_text = format!(
        "{}/{}",
        to_hh_mm_ss_str(playback_progress.ms_passed),
        to_hh_mm_ss_str(playback_progress.ms_total)
    );
    playback_widgets
        .progress_text
        .set_markup(progress_text.as_str());
}
