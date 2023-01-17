use fltk::{
    app,
    button::Button,
    enums::{Align, Font, FrameType},
    group::{Group, Tabs},
    misc::InputChoice,
    prelude::{DisplayExt, GroupExt, WidgetBase, WidgetExt, WindowExt},
    text::{TextBuffer, TextDisplay},
    window::Window,
};

use crate::{
    media::io::{input_device_names, output_device_names},
    sessions::session::Session,
    ui::common::shift_right_by_label,
};

/// Clears, then adds all choices into the given input.
fn repopulate_input_choices<T>(input: &mut InputChoice, choices: &[T])
where
    T: std::fmt::Display,
{
    input.clear();

    for choice in choices {
        input.add(&choice.to_string());
    }
}

/// Selects the given choice in the input from its position in choices.
fn set_active_in_input_choices<T>(input: &mut InputChoice, choices: &[T], choice: &T)
where
    T: PartialEq<T>,
{
    let choice_idx = choices
        .iter()
        .position(|other_choice| choice == other_choice)
        .unwrap_or(0);
    input.set_value_index(choice_idx as i32);
}
pub struct PreferencesDialog {
    window: Window,

    project_directory_text: TextDisplay,
    project_directory_chooser_button: Button,

    audio_output_name: InputChoice,

    audio_input_name: InputChoice,
    audio_input_sample_rate: InputChoice,
    audio_input_channels: InputChoice,

    save_button: Button,
}

struct GeneralTabWidgets {
    project_directory_text: TextDisplay,
    project_directory_chooser_button: Button,
}

fn create_general_tab() -> GeneralTabWidgets {
    let general_tab = Group::new(20, 30, 360, 250, "General\t\t");

    let mut project_widgets_group = Group::new(20, 40, 360, 70, "Project");
    let project_label_offset = project_widgets_group.label_size();
    project_widgets_group.set_align(Align::TopLeft);
    project_widgets_group.set_pos(
        project_widgets_group.x(),
        project_widgets_group.y() + project_label_offset,
    );
    project_widgets_group.set_label_font(Font::HelveticaBold);
    project_widgets_group.set_frame(FrameType::ThinDownFrame);

    let mut project_directory_text = TextDisplay::new(
        30 + 110,
        55 + project_label_offset,
        160,
        40,
        "Output Directory:",
    );
    project_directory_text.set_align(Align::Left);
    project_directory_text.set_buffer(TextBuffer::default());
    let project_directory_chooser_button =
        Button::new(30 + 270 + 10, 60 + project_label_offset, 60, 30, "Choose");

    project_widgets_group.end();

    general_tab.end();

    GeneralTabWidgets {
        project_directory_text,
        project_directory_chooser_button,
    }
}

struct AudioTabWidgets {
    audio_output_name: InputChoice,

    audio_input_name: InputChoice,
    audio_input_sample_rate: InputChoice,
    audio_input_channels: InputChoice,
}

fn create_audio_tab() -> AudioTabWidgets {
    let audio_tab = Group::new(20, 30, 360, 250, "Audio\t\t");

    let mut output_widget_group = Group::new(20, 40, 360, 50, "Output");
    let output_label_offset = output_widget_group.label_size();
    output_widget_group.set_align(Align::TopLeft);
    output_widget_group.set_pos(
        output_widget_group.x(),
        output_widget_group.y() + output_label_offset,
    );
    output_widget_group.set_label_font(Font::HelveticaBold);
    output_widget_group.set_frame(FrameType::ThinDownFrame);
    let audio_output_name =
        InputChoice::new(40 + 90, 50 + output_label_offset, 320 - 80, 30, "Device:");
    output_widget_group.end();

    let mut input_widget_group = Group::new(
        20,
        110 + output_label_offset,
        360,
        170 - output_label_offset,
        "Input",
    );
    input_widget_group.set_align(Align::TopLeft);
    input_widget_group.set_label_font(Font::HelveticaBold);
    input_widget_group.set_frame(FrameType::ThinDownFrame);
    let audio_input_name =
        InputChoice::new(40 + 90, 120 + output_label_offset, 320 - 80, 30, "Device:");
    let audio_input_sample_rate = InputChoice::new(
        40 + 90,
        160 + output_label_offset,
        320 - 80,
        30,
        "Sample Rate:",
    );
    let audio_input_channels = InputChoice::new(
        40 + 90,
        200 + output_label_offset,
        320 - 80,
        30,
        "Channels:",
    );
    input_widget_group.end();

    audio_tab.end();

    AudioTabWidgets {
        audio_output_name,
        audio_input_name,
        audio_input_sample_rate,
        audio_input_channels,
    }
}

// TODO: Turn magic numbers into constants for clarity.
impl PreferencesDialog {
    pub fn new() -> PreferencesDialog {
        let preferences_window = Window::default()
            .with_size(400, 340)
            .with_label("Preferences");

        let preference_topics = Tabs::new(10, 10, 380, 280, "");

        let general_tab = create_general_tab();
        let audio_tab = create_audio_tab();

        preference_topics.end();

        let mut preferences_window_clone = preferences_window.clone();
        let mut cancel_button = Button::new(260, 300, 60, 30, "Cancel");
        cancel_button.set_callback(move |_| {
            preferences_window_clone.hide();
        });

        let mut preferences_window_clone = preferences_window.clone();
        let mut save_button = Button::new(330, 300, 60, 30, "Save");
        save_button.set_callback(move |button| {
            button.deactivate();
            preferences_window_clone.hide();
        });

        preferences_window.end();

        PreferencesDialog {
            window: preferences_window,

            project_directory_text: general_tab.project_directory_text,
            project_directory_chooser_button: general_tab.project_directory_chooser_button,

            audio_output_name: audio_tab.audio_output_name,
            audio_input_name: audio_tab.audio_input_name,
            audio_input_sample_rate: audio_tab.audio_input_sample_rate,
            audio_input_channels: audio_tab.audio_input_channels,

            save_button,
        }
    }

    /// Clears and fills in information about current audio devices
    /// to relevant audio input widgets.
    fn populate_audio_tab_inputs(&mut self, session: &Session) {
        let audio_output_choices = output_device_names();
        repopulate_input_choices(&mut self.audio_output_name, &audio_output_choices);
        set_active_in_input_choices(
            &mut self.audio_output_name,
            &audio_output_choices,
            &session.audio_output().device_name().to_string(),
        );

        let audio_input_choices = input_device_names();
        repopulate_input_choices(&mut self.audio_input_name, &audio_input_choices);
        set_active_in_input_choices(
            &mut self.audio_input_name,
            &audio_input_choices,
            &session.audio_input().device_name().to_string(),
        );

        let audio_input_sample_rates = session.audio_input().sample_rates();
        repopulate_input_choices(&mut self.audio_input_sample_rate, &audio_input_sample_rates);
        set_active_in_input_choices(
            &mut self.audio_input_sample_rate,
            &audio_input_sample_rates,
            &session.audio_input().sample_rate(),
        );

        let audio_input_channels = session.audio_input().channels();
        repopulate_input_choices(&mut self.audio_input_channels, &audio_input_channels);
        set_active_in_input_choices(
            &mut self.audio_input_channels,
            &audio_input_channels,
            &session.audio_input().channel(),
        );
    }

    /// Pulls the currently selected values for all audio input widgets
    /// and updates the current session accordingly.
    fn save_audio_preferences(&self, session: &mut Session) {
        session
            .audio_output_mut()
            .set_device_name(self.audio_output_name.value().unwrap());

        let audio_input = session.audio_input_mut();
        audio_input.set_device_name(self.audio_input_name.value().unwrap());
        audio_input.set_channels(
            self.audio_input_channels
                .value()
                .unwrap()
                .parse::<u16>()
                .expect("Could not get number from channels input."),
        );
        audio_input.set_sample_rate(
            self.audio_input_sample_rate
                .value()
                .unwrap()
                .parse::<u32>()
                .expect("Could not get number from sample rate input."),
        );
    }

    pub fn show(&mut self, session: &mut Session) {
        self.save_button.activate();

        self.project_directory_text
            .buffer()
            .unwrap()
            .set_text(session.project_directory().to_str().unwrap());
        self.populate_audio_tab_inputs(session);

        self.window.show();

        while self.window.shown() {
            app::wait();
        }

        if self.save_button.active() {
            return;
        }

        self.save_audio_preferences(session);
    }
}
