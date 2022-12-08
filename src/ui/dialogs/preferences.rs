use fltk::{misc::InputChoice, window::Window, prelude::{WidgetExt, WidgetBase, GroupExt}, group::{Tabs, Group}};

use crate::ui::common::{shift_right_by_label, shift_down_by_label};

pub struct PreferencesDialog {
    window: Window,

    audio_output_name: InputChoice,

    audio_input_name: InputChoice,
    audio_input_sample_rate: InputChoice,
    audio_input_channels: InputChoice,
}

impl PreferencesDialog {
    pub fn new() -> PreferencesDialog {
        let preferences_window = Window::default().with_size(400, 300);

        let preference_topics = Tabs::new(10, 10, 380, 280, "");

        let audio_tab = Group::new(10, 20, 380, 260, "Audio\t\t");

        let mut output_tab = Group::new(20, 20, 360, 65, "Output");
        shift_down_by_label(&mut output_tab);
        let mut audio_output_name = InputChoice::new(90, 30, 340 - 90, 25, "Device:");
        shift_right_by_label(&mut audio_output_name);
        output_tab.end();

        let mut input_tab = Group::new(20, 105, 360, 185, "Input");
        shift_down_by_label(&mut input_tab);
        let mut audio_input_name = InputChoice::new(90, 115, 340 - 90, 25, "Device");
        shift_right_by_label(&mut audio_input_name);
        let mut audio_input_sample_rate = InputChoice::new(90, 125, 340 - 90, 25, "Sample Rate");
        shift_right_by_label(&mut audio_input_sample_rate);
        let mut audio_input_channels = InputChoice::new(90, 135, 340 - 90, 25, "Channels");
        shift_right_by_label(&mut audio_input_channels);
        input_tab.end();

        audio_tab.end();

        preference_topics.end();

        PreferencesDialog {
            window: preferences_window,

            audio_output_name,
            audio_input_name,
            audio_input_sample_rate,
            audio_input_channels
        }
    }

    pub fn show(&mut self) {
        self.window.show();
    }
}

// pub struct PreferenceWidgets {
//     pub dialog: Dialog,

//     pub project_location_chooser: FileChooser,

//     pub input_device_name_chooser: ComboBoxText,
//     pub input_device_sample_rate_chooser: ComboBoxText,
//     pub input_device_channels_chooser: ComboBoxText,

//     pub output_device_name_chooser: ComboBoxText,
// }

// fn populate_combobox<T>(combobox: &ComboBoxText, items: &[T])
// where
//     T: std::fmt::Display,
// {
//     combobox.remove_all();
//     for item in items.iter() {
//         combobox.append_text(&item.to_string());
//     }
// }

// fn set_active_in_combobox<T>(combobox: &ComboBoxText, items: &[T], active_item: &T)
// where
//     T: PartialEq<T>,
// {
//     let current_pos = items
//         .iter()
//         .position(|item| item == active_item)
//         .unwrap_or(0);
//     combobox.set_active(Some(current_pos as u32));
// }

// /// Shows the Preferences dialog, modifying the Session if the user commits any changes.
// ///
// /// Preconditions: preference_widgets contains widgets that map to the Session's variables, and
// ///                session is loaded from the current project.
// /// Postconditions: session is modified if a user saves the changes.
// pub fn preferences(preference_widgets: &PreferenceWidgets, session: &mut Session) {
//     // Set the preference fields to mirror what's set in the current Session.
//     preference_widgets
//         .project_location_chooser
//         .set_current_folder(session.project_directory());

//     // Populate Input Names regardless of what's present.
//     let input_names = input_device_names();
//     populate_combobox(&preference_widgets.input_device_name_chooser, &input_names);
//     set_active_in_combobox(
//         &preference_widgets.input_device_name_chooser,
//         &input_names,
//         &session.audio_input().device_name().to_string(),
//     );

//     // Populate Sample Rates regardless of what's present.
//     let sample_rates = session.audio_input().sample_rates();
//     populate_combobox(
//         &preference_widgets.input_device_sample_rate_chooser,
//         &sample_rates,
//     );
//     set_active_in_combobox(
//         &preference_widgets.input_device_sample_rate_chooser,
//         &sample_rates,
//         &session.audio_input().sample_rate(),
//     );

//     // Populate Channels regardless of what's present.
//     let channels = session.audio_input().channels();
//     populate_combobox(&preference_widgets.input_device_channels_chooser, &channels);
//     set_active_in_combobox(
//         &preference_widgets.input_device_channels_chooser,
//         &channels,
//         &session.audio_input().channel(),
//     );

//     // Populate Output Names regardless of what's present.
//     let output_names = output_device_names();
//     populate_combobox(
//         &preference_widgets.output_device_name_chooser,
//         &output_names,
//     );
//     set_active_in_combobox(
//         &preference_widgets.output_device_name_chooser,
//         &output_names,
//         &session.audio_output().device_name().to_string(),
//     );

//     // Change the Sample Rate and Channels when a new Input Device has been chosen.
//     let input_sample_rate_chooser = preference_widgets.input_device_sample_rate_chooser.clone();
//     let input_channel_chooser = preference_widgets.input_device_channels_chooser.clone();
//     let device_changer_id = preference_widgets
//         .input_device_name_chooser
//         .connect_changed(move |combobox| {
//             let device_name = combobox
//                 .active_text()
//                 .expect("Could not get device name for input.")
//                 .to_string();

//             let mut audio_input = AudioInput::new();
//             audio_input.set_device_name(device_name);

//             // Populate Sample Rates regardless of what's present.
//             let sample_rates = audio_input.sample_rates();
//             populate_combobox(&input_sample_rate_chooser, &sample_rates);
//             set_active_in_combobox(
//                 &input_sample_rate_chooser,
//                 &sample_rates,
//                 &audio_input.sample_rate(),
//             );

//             // Populate Channels regardless of what's present.
//             let channels = audio_input.channels();
//             populate_combobox(&input_channel_chooser, &channels);
//             set_active_in_combobox(&input_channel_chooser, &channels, &audio_input.channel());
//         });

//     // Wait for the user's response.
//     preference_widgets.dialog.show_all();
//     let preference_response = preference_widgets.dialog.run();
//     if preference_response != ResponseType::Ok {
//         preference_widgets
//             .input_device_name_chooser
//             .disconnect(device_changer_id);
//         preference_widgets.dialog.hide();
//         return;
//     }

//     // Update the Session based on the user's changes.
//     session.set_project_directory(
//         preference_widgets
//             .project_location_chooser
//             .filename()
//             .unwrap(),
//     );

//     let audio_input = session.audio_input_mut();
//     audio_input.set_device_name(
//         preference_widgets
//             .input_device_name_chooser
//             .active_text()
//             .unwrap()
//             .to_string(),
//     );

//     let sample_rate = preference_widgets
//         .input_device_sample_rate_chooser
//         .active_text()
//         .unwrap()
//         .parse::<u32>()
//         .unwrap();
//     audio_input.set_sample_rate(sample_rate);

//     let channels = preference_widgets
//         .input_device_channels_chooser
//         .active_text()
//         .unwrap()
//         .parse::<u16>()
//         .unwrap();
//     audio_input.set_channels(channels);

//     let audio_output = session.audio_output_mut();
//     audio_output.set_device_name(
//         preference_widgets
//             .output_device_name_chooser
//             .active_text()
//             .unwrap()
//             .to_string(),
//     );

//     preference_widgets
//         .input_device_name_chooser
//         .disconnect(device_changer_id);
//     preference_widgets.dialog.hide();
// }