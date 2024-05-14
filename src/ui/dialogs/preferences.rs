use std::path::PathBuf;

use fltk::{
    app,
    button::{Button, CheckButton},
    dialog,
    enums::{Align, Font, FrameType},
    frame::Frame,
    group::{Flex, FlexType, Group, Tabs},
    input::Input,
    misc::{InputChoice, Spinner},
    prelude::{DisplayExt, GroupExt, WidgetBase, WidgetExt, WindowExt},
    text::{TextBuffer, TextDisplay},
    window::Window,
};

use crate::{
    media::io::{input_device_names, output_device_names, AudioInput},
    sessions::session::Session,
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

    audio_output_name: InputChoice,

    audio_input_name: InputChoice,
    audio_input_sample_rate: InputChoice,
    audio_input_channels: InputChoice,

    save_button: Button,
}

struct GeneralTabWidgets {
    project_directory_text: TextDisplay,
}

fn create_general_tab() -> GeneralTabWidgets {
    let general_tab = Group::new(20, 30, 360, 250, "General\t\t");

    let mut project_widgets_group = Flex::new(20, 40, 360, 50, "Project");
    let project_label_offset = project_widgets_group.label_size();
    project_widgets_group.set_align(Align::TopLeft);
    project_widgets_group.set_pos(
        project_widgets_group.x(),
        project_widgets_group.y() + project_label_offset,
    );
    project_widgets_group.set_label_font(Font::HelveticaBold);
    project_widgets_group.set_frame(FrameType::ThinDownFrame);
    project_widgets_group.set_type(FlexType::Row);
    project_widgets_group.set_spacing(10);
    project_widgets_group.set_margins(80, 5, 10, 5);

    let mut project_directory_text = TextDisplay::default()
        .with_size(0, 40)
        .with_label("Directory:");
    project_directory_text.set_align(Align::Left);
    project_directory_text.set_buffer(TextBuffer::default());

    let mut project_directory_chooser_button =
        Button::default().with_size(60, 30).with_label("Choose");
    project_widgets_group.fixed(&project_directory_chooser_button, 60);

    let project_directory_text_clone = project_directory_text.clone();
    project_directory_chooser_button.set_callback(move |_| {
        let mut folder_chooser =
            dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseDir);
        folder_chooser.show();

        let folder_name = folder_chooser.filename();
        if !folder_name.is_dir() {
            return;
        }

        project_directory_text_clone
            .buffer()
            .expect("General Preferences: Where's the TextBuffer?")
            .set_text(folder_name.to_str().unwrap());
    });

    project_widgets_group.end();

    general_tab.end();

    GeneralTabWidgets {
        project_directory_text,
    }
}

struct TextTabWidgets {
    gatherer_name: InputChoice,
    gatherer_custom_enabler: CheckButton,

    gatherer_amount: Spinner,

    gatherer_delimiters: Input,
}

const TEXT_TAB_LABEL_LENGTH: i32 = 100;
const TEXT_TAB_INPUT_LENGTH: i32 = 155;
const TEXT_TAB_CHECKBUTTON_LENGTH: i32 = 70;
const TEXT_TAB_SPACING: i32 = 10;

fn create_text_tab() -> TextTabWidgets {
    let text_tab = Group::new(20, 30, 360, 250, "Text\t\t");

    let mut extraction_group = Flex::new(20, 40, 360, 130, "Extraction");
    let extraction_group_label_offset = extraction_group.label_size();
    extraction_group.set_align(Align::TopLeft);
    extraction_group.set_pos(
        extraction_group.x(),
        extraction_group.y() + extraction_group_label_offset,
    );
    extraction_group.set_type(FlexType::Column);
    extraction_group.set_label_font(Font::HelveticaBold);
    extraction_group.set_frame(FrameType::ThinDownFrame);
    extraction_group.set_margins(0, TEXT_TAB_SPACING, TEXT_TAB_SPACING, TEXT_TAB_SPACING);
    extraction_group.set_spacing(TEXT_TAB_SPACING);

    let mut gatherer_group = Flex::default().with_type(FlexType::Row);
    extraction_group.fixed(&gatherer_group, 30);
    gatherer_group.set_spacing(TEXT_TAB_SPACING);

    let gatherer_label = Frame::default();
    gatherer_group.fixed(&gatherer_label, TEXT_TAB_LABEL_LENGTH);

    let gatherer_selector = InputChoice::default()
        .with_label("Gatherer:")
        .with_align(Align::Left);
    gatherer_group.fixed(&gatherer_selector, TEXT_TAB_INPUT_LENGTH);

    let gatherer_custom_enabler = CheckButton::default().with_label("Custom");
    gatherer_group.fixed(&gatherer_custom_enabler, TEXT_TAB_CHECKBUTTON_LENGTH);
    gatherer_group.end();

    let mut amount_group = Flex::default().with_type(FlexType::Row);
    extraction_group.fixed(&amount_group, 30);
    amount_group.set_spacing(TEXT_TAB_SPACING);

    let amount_label = Frame::default();
    amount_group.fixed(&amount_label, TEXT_TAB_LABEL_LENGTH);

    let amount_spinner = Spinner::default()
        .with_label("Amount:")
        .with_align(Align::Left);
    amount_group.end();

    let mut ending_with_group = Flex::default().with_type(FlexType::Row);
    extraction_group.fixed(&ending_with_group, 30);
    ending_with_group.set_spacing(TEXT_TAB_SPACING);

    let ending_with_label = Frame::default();
    ending_with_group.fixed(&ending_with_label, TEXT_TAB_LABEL_LENGTH);

    let ending_with_delimiters_input = Input::default()
        .with_label("Ending With:")
        .with_align(Align::Left);
    ending_with_group.end();

    extraction_group.end();
    text_tab.end();

    TextTabWidgets {
        gatherer_name: gatherer_selector,
        gatherer_amount: amount_spinner,
        gatherer_custom_enabler,
        gatherer_delimiters: ending_with_delimiters_input,
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

    let mut output_widget_group = Flex::new(20, 40, 360, 50, "Output");
    output_widget_group.set_type(FlexType::Column);
    let output_label_offset = output_widget_group.label_size();
    output_widget_group.set_align(Align::TopLeft);
    output_widget_group.set_pos(
        output_widget_group.x(),
        output_widget_group.y() + output_label_offset,
    );
    output_widget_group.set_label_font(Font::HelveticaBold);
    output_widget_group.set_frame(FrameType::ThinDownFrame);

    let audio_output_name = InputChoice::default()
        .with_size(0, 30)
        .with_label("Device:");

    output_widget_group.set_margins(TEXT_TAB_LABEL_LENGTH, 10, 10, 0);
    output_widget_group.set_pad(10);
    output_widget_group.fixed(&audio_output_name, 30);
    output_widget_group.end();

    let mut input_widget_group = Flex::new(
        20,
        110 + output_label_offset,
        360,
        170 - output_label_offset,
        "Input",
    );
    input_widget_group.set_type(FlexType::Column);
    input_widget_group.set_align(Align::TopLeft);
    input_widget_group.set_label_font(Font::HelveticaBold);
    input_widget_group.set_frame(FrameType::ThinDownFrame);

    let audio_input_name = InputChoice::default()
        .with_size(0, 30)
        .with_label("Device:");
    let audio_input_sample_rate = InputChoice::default()
        .with_size(0, 30)
        .with_label("Sample Rate:");
    let audio_input_channels = InputChoice::default()
        .with_size(0, 30)
        .with_label("Channels");

    input_widget_group.fixed(&audio_input_name, 30);
    input_widget_group.fixed(&audio_input_sample_rate, 30);
    input_widget_group.fixed(&audio_input_channels, 30);

    input_widget_group.set_margins(TEXT_TAB_LABEL_LENGTH, TEXT_TAB_SPACING, TEXT_TAB_SPACING, 0);
    input_widget_group.set_pad(TEXT_TAB_SPACING);
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

        let preference_topics = Tabs::new(TEXT_TAB_SPACING, TEXT_TAB_SPACING, 380, 280, "");

        let general_tab = create_general_tab();
        let mut audio_tab = create_audio_tab();
        let text_tab = create_text_tab();

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

        let mut sample_rate_input = audio_tab.audio_input_sample_rate.clone();
        let mut channels_input = audio_tab.audio_input_channels.clone();

        // It's important to not bring over invalid sample rates and
        // channels from other chosen input devices, hence a reason
        // to make it repopulate and highlight the default choices for
        // the input device.
        audio_tab.audio_input_name.set_callback(move |device_name| {
            let mut audio_input = AudioInput::new();
            audio_input.set_device_name(device_name.label());

            let audio_input_sample_rates = audio_input.sample_rates();
            repopulate_input_choices(&mut sample_rate_input, &audio_input_sample_rates);
            set_active_in_input_choices(
                &mut sample_rate_input,
                &audio_input_sample_rates,
                &audio_input.sample_rate(),
            );

            let audio_input_channels = audio_input.channels();
            repopulate_input_choices(&mut channels_input, &audio_input_channels);
            set_active_in_input_choices(
                &mut channels_input,
                &audio_input_channels,
                &audio_input.channel(),
            );
        });

        preferences_window.end();

        PreferencesDialog {
            window: preferences_window,

            project_directory_text: general_tab.project_directory_text,

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

        let chosen_audio_output_dir = self.project_directory_text.buffer().unwrap().text();
        let audio_output_dir = PathBuf::from(chosen_audio_output_dir);
        session.set_project_directory(audio_output_dir);

        self.save_audio_preferences(session);
    }
}
