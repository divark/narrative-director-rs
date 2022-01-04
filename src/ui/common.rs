use crate::{input_device_names, output_device_names, Session};
use gtk::prelude::*;
use std::path::PathBuf;

use gtk::{
    AboutDialogBuilder, ComboBoxText, Dialog, DialogFlags, FileChooser, FileFilter, License,
    ResponseType, SpinButton, Window,
};

use gtk::gdk_pixbuf::Pixbuf;

/// Returns a File chosen by the user in a Dialog, or None if nothing
/// was chosen.
///
/// Preconditions: parent_window is a Window reference.
/// Postconditions: A File wrapped in Some, or None.
pub fn open(parent_window: &Window) -> Option<PathBuf> {
    // 1: Create the File Chooser dialog that only accepts
    // text files.
    let file_chooser = gtk::FileChooserDialog::new(
        Some("Open File"),
        Some(parent_window),
        gtk::FileChooserAction::Open,
    );

    file_chooser.add_buttons(&[
        ("Open", gtk::ResponseType::Ok),
        ("Cancel", gtk::ResponseType::Cancel),
    ]);

    let text_file_filter = FileFilter::new();
    text_file_filter.set_name(Some("UTF-8 Text Files"));
    text_file_filter.add_pattern("*.txt");

    file_chooser.add_filter(&text_file_filter);

    // 2: Fetch file from user choice, if any.
    let user_response = file_chooser.run();
    if user_response != ResponseType::Ok {
        file_chooser.close();
        return None;
    }

    file_chooser.close();

    let filename = file_chooser.filename().expect("Couldn't get filename");

    Some(filename)
}

/// Returns the paragraph number chosen by a user in a Dialog, or None if
/// nothing was chosen.
///
/// Preconditions: parent_window is a Window reference, and num_paragraphs is a usize
///                representing the total number of paragraphs in ParagraphViewer.
/// Postconditions: The paragraph number represented as a usize.
pub fn go_to(parent_window: &Window, num_paragraphs: usize) -> Option<usize> {
    let goto_dialog = Dialog::with_buttons(
        Some("Select the paragraph number."),
        Some(parent_window),
        DialogFlags::MODAL,
        &[("Ok", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
    );

    goto_dialog.set_default_response(ResponseType::Ok);

    let content_area = goto_dialog.content_area();

    let goto_spin_button = SpinButton::with_range(1.0, num_paragraphs as f64, 1.0);
    goto_spin_button.set_activates_default(true);

    content_area.add(&goto_spin_button);

    goto_dialog.show_all();

    let goto_dialog_response = goto_dialog.run();
    if goto_dialog_response == ResponseType::Ok {
        goto_dialog.close();
        return Some((goto_spin_button.value_as_int() - 1) as usize);
    }

    goto_dialog.close();
    None
}

/// Shows an About Dialog describing the program.
///
/// Precondition: parent_window is a Window reference.
/// Postcondition: An About Dialog is shown until it is closed.
pub fn about(parent_window: &Window) {
    let logo: Pixbuf =
        Pixbuf::from_file("resources/images/icon.png").expect("Could not find icon file.");

    let about_dialog = AboutDialogBuilder::new()
		.program_name("Narrative Director")
		.comments("Narrative Director is an alternative Audio/Video Recording application tailored for working on medium to large-sized projects. This tool aspires to keep editing to a minimum with the capability of playing, recording and re-recording readings in place at the paragraph level for some text piece, whether it's a script, or a novel.")
		.authors(vec!["Tyler Schmidt <tmschmid@protonmail.com>".to_string()])
		.artists(vec!["ColorfulSkyWisps https://linktr.ee/ColorfulSkyWisps".to_string()])
		.license_type(License::Gpl30)
        .logo(&logo)
		.parent(parent_window)
		.build();

    about_dialog.show();
    about_dialog.run();
    about_dialog.close();
}

pub struct PreferenceWidgets {
    pub dialog: Dialog,

    pub project_location_chooser: FileChooser,

    pub input_device_name_chooser: ComboBoxText,
    pub input_device_sample_rate_chooser: ComboBoxText,
    pub input_device_channels_chooser: ComboBoxText,

    pub output_device_name_chooser: ComboBoxText,
}

fn populate_combobox<T>(combobox: &ComboBoxText, items: &[T])
where
    T: std::fmt::Display,
{
    combobox.remove_all();
    for item in items.iter() {
        combobox.append_text(&item.to_string());
    }
}

fn set_active_in_combobox<T>(combobox: &ComboBoxText, items: &[T], active_item: &T)
where
    T: PartialEq<T>,
{
    let current_pos = items
        .iter()
        .position(|item| item == active_item)
        .unwrap_or_else(|| 0);
    combobox.set_active(Some(current_pos as u32));
}

pub fn preferences(preference_widgets: &PreferenceWidgets, session: &mut Session) {
    // Set the preference fields to mirror what's set in the current Session.
    preference_widgets
        .project_location_chooser
        .set_current_folder(session.project_directory());

    // Populate Input Names regardless of what's present.
    let input_names = input_device_names();
    populate_combobox(&preference_widgets.input_device_name_chooser, &input_names);
    set_active_in_combobox(
        &preference_widgets.input_device_name_chooser,
        &input_names,
        &session.audio_input().device_name().to_string(),
    );

    // Populate Sample Rates regardless of what's present.
    let sample_rates = session.audio_input().sample_rates();
    populate_combobox(
        &preference_widgets.input_device_sample_rate_chooser,
        &sample_rates,
    );
    set_active_in_combobox(
        &preference_widgets.input_device_sample_rate_chooser,
        &sample_rates,
        &session.audio_input().sample_rate(),
    );

    // Populate Channels regardless of what's present.
    let channels = session.audio_input().channels();
    populate_combobox(&preference_widgets.input_device_channels_chooser, &channels);
    set_active_in_combobox(
        &preference_widgets.input_device_channels_chooser,
        &channels,
        &session.audio_input().channel(),
    );

    // Populate Output Names regardless of what's present.
    let output_names = output_device_names();
    populate_combobox(
        &preference_widgets.output_device_name_chooser,
        &output_names,
    );
    set_active_in_combobox(
        &preference_widgets.output_device_name_chooser,
        &output_names,
        &session.audio_output().device_name().to_string(),
    );

    // Wait for the user's response.
    preference_widgets.dialog.show_all();
    let preference_response = preference_widgets.dialog.run();
    if preference_response != ResponseType::Ok {
        preference_widgets.dialog.hide();
        return;
    }

    // Update the Session based on the user's changes.
    session.set_project_directory(
        preference_widgets
            .project_location_chooser
            .filename()
            .unwrap(),
    );

    let audio_input = session.audio_input_mut();
    audio_input.set_device_name(
        preference_widgets
            .input_device_name_chooser
            .active_text()
            .unwrap()
            .to_string(),
    );

    let sample_rate = preference_widgets
        .input_device_sample_rate_chooser
        .active_text()
        .unwrap()
        .parse::<u32>()
        .unwrap();
    audio_input.set_sample_rate(sample_rate);

    let channels = preference_widgets
        .input_device_channels_chooser
        .active_text()
        .unwrap()
        .parse::<u16>()
        .unwrap();
    audio_input.set_channels(channels);

    let audio_output = session.audio_output_mut();
    audio_output.set_device_name(
        preference_widgets
            .output_device_name_chooser
            .active_text()
            .unwrap()
            .to_string(),
    );

    preference_widgets.dialog.hide();
}
