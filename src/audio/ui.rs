use crate::audio::{AudioIO, InputDeviceSelection};
use crate::{Model, Widgets};
use cpal::traits::DeviceTrait;
use cpal::Device;
use gtk::prelude::{ComboBoxExtManual, ComboBoxTextExt};
use gtk::ComboBoxText;

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
    model: &Model,
    widgets: &mut Widgets,
) {
    // Starting with the channels
    widgets.input_channels_cbox.remove_all();
    let input_device_channels = model
        .input_devices_info
        .as_ref()
        .unwrap()
        .channels
        .get(&input_device.name().unwrap())
        .unwrap();

    for default_input_channel in input_device_channels {
        widgets
            .input_channels_cbox
            .append_text(&default_input_channel.to_string());
    }

    // Setting the combo box to point to the value of the default channel.
    let current_channel_pos = input_device_channels
        .iter()
        .position(|channel| input_device.default_input_config().unwrap().channels() == *channel)
        .unwrap() as u32;

    widgets
        .input_channels_cbox
        .set_active(Some(current_channel_pos));

    // Then the sample rates
    widgets.input_sample_rate_cbox.remove_all();
    let input_device_sample_rates = model
        .input_devices_info
        .as_ref()
        .unwrap()
        .sample_rates
        .get(&input_device.name().unwrap())
        .unwrap();

    for default_input_sample_rate in input_device_sample_rates {
        widgets
            .input_sample_rate_cbox
            .append_text(&default_input_sample_rate.to_string());
    }

    // Setting the combo box to point to the value of the default sample rate..
    let current_sample_rate_pos = input_device_sample_rates
        .iter()
        .position(|sample_rate| {
            input_device.default_input_config().unwrap().sample_rate().0 == *sample_rate
        })
        .unwrap() as u32;

    widgets
        .input_sample_rate_cbox
        .set_active(Some(current_sample_rate_pos));
}

pub struct InputPreferenceWidgets<'a> {
    pub input_device_cbox: &'a ComboBoxText,
    pub sample_rate_cbox: &'a ComboBoxText,
    pub channels_cbox: &'a ComboBoxText,
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
