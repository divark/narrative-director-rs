mod audio;
mod text_grabber;

use gtk::prelude::*;
use gtk::{
    AboutDialog, Adjustment, Builder, Button, ComboBoxText, Dialog, DialogFlags, FileChooser,
    FileFilter, Inhibit, Label, MenuItem, ResponseType, Scrollbar, SpinButton, TextView, Window,
};
use relm::{connect, interval, Relm, Update, Widget};
use relm_derive::Msg;

use crate::Msg::ProgressTick;
use audio::prelude::*;
use audio::ProcessingStatus;
use cpal::traits::DeviceTrait;
use cpal::{ChannelCount, Device};
use std::collections::HashMap;
use std::fs;
use std::fs::{DirBuilder, File};
use std::path::Path;
use text_grabber::{EnglishParagraphRetriever, TextGrabber};

struct InputDevicesInfo {
    channels: HashMap<String, Vec<ChannelCount>>,
    sample_rates: HashMap<String, Vec<u32>>,
}
/// Holds the variables necessary to navigate chunks
/// in some UTF-8 text file.
///
/// Just like in the software design pattern
/// [Model-view-controller](https://en.wikipedia.org/wiki/Model%E2%80%93view%E2%80%93controller),
/// this model influences the logical flow of the
/// application, depending on its current state.
struct Model {
    chunk_retriever: EnglishParagraphRetriever,
    chunk_number: u32,
    chunk_total: u32,

    audio_processor: AudioIO,
    ms_passed: u32,
    ms_total: u32,
    audio_status: ProcessingStatus,

    input_devices_info: Option<InputDevicesInfo>,
    preferences_has_been_shown_once: bool,
    current_filename: String,
    project_directory: String,
}

/// Represents User Events that could take place
/// in the GUI.
#[derive(Msg)]
enum Msg {
    Next,
    Previous,
    Play,
    Stop,
    Record,
    AudioSkip,

    ProgressTick,

    JumpTo,
    LoadFile,
    OpenPreferences,
    About,
    Quit,
}

/// Keeps track of the Graphical User Interface
/// elements in the application.
///
/// Widgets serve as both a means of viewing, like Labels and Viewers,
/// as well as hooks that spawn User Events, such as Buttons.
#[derive(Clone)]
struct Widgets {
    // Main Window Widgets
    chunk_progress_label: Label,
    chunk_viewer: TextView,

    previous_chunk_button: Button,
    next_chunk_button: Button,
    stop_button: Button,
    record_button: Button,
    play_button: Button,
    audio_progress_label: Label,
    progress_bar: Scrollbar,

    window: Window,
    // Menu Widgets
    open_menu_item: MenuItem,
    goto_menu_item: MenuItem,
    preferences_menu_item: MenuItem,
    about_menu_item: MenuItem,
    quit_menu_item: MenuItem,

    // Dialogs
    about_dialog: AboutDialog,

    // Preferences
    preferences_dialog: Dialog,

    project_file_chooser: FileChooser,

    input_device_cbox: ComboBoxText,
    input_sample_rate_cbox: ComboBoxText,
    input_channels_cbox: ComboBoxText,

    output_device_cbox: ComboBoxText,
}

struct ChunkViewingUi<'a> {
    progress_label: &'a Label,
    chunk_viewer: &'a TextView,
}

/// Populates the UI with the specified chunk.
fn show_chunk(
    chunk_num: u32,
    chunk_getter: &EnglishParagraphRetriever,
    ui: ChunkViewingUi,
) -> Result<(), ()> {
    if let Some(paragraph) = chunk_getter.get_chunk(chunk_num as usize) {
        ui.progress_label
            .set_text(format!("{}/{}", chunk_num + 1, chunk_getter.len()).as_str());

        ui.chunk_viewer
            .get_buffer()
            .expect("Couldn't get text viewer")
            .set_text(paragraph.as_str());

        return Ok(());
    }

    Err(())
}

/// Makes previous button active or inactive depending on the chunk number.
fn change_prev_button_sensitivity(chunk_num: u32, prev_button: &Button) {
    if chunk_num > 0 {
        prev_button.set_sensitive(true);
    } else {
        prev_button.set_sensitive(false);
    }
}

/// Makes next button active or inactive relative to the chunk number and the total.
fn change_next_button_sensitivity(chunk_num: u32, chunk_total: u32, next_button: &Button) {
    if chunk_num == chunk_total - 1 {
        next_button.set_sensitive(false);
    } else {
        next_button.set_sensitive(true);
    }
}

/// Makes play button active or inactive relative to the file status.
fn change_play_button_sensitivity(file_status: FileStatus, play_button: &Button) {
    if file_status == FileStatus::Exists {
        play_button.set_sensitive(true);
    } else {
        play_button.set_sensitive(false);
    }
}

/// Populates Start and End time of current chunk if it exists.
fn reset_progress_bar_info(file_status: FileStatus, model: &mut Model, widgets: &Widgets) {
    change_play_button_sensitivity(file_status, &widgets.play_button);

    model.ms_passed = 0;
    model.ms_total = model.audio_processor.duration();

    let secs_total = (model.ms_total / 1000) as f64;
    widgets
        .progress_bar
        .set_adjustment(&Adjustment::new(0.0, 0.0, secs_total, 1.0, 1.0, 1.0));
}

/// Converts ms to hours:minutes:seconds format
fn to_hh_mm_ss_str(ms: u32) -> String {
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Populates the choices available for input devices
fn populate_input_options(input_options: &mut ComboBoxText, audio_io: &AudioIO) {
    let input_devices = audio_io.get_input_devices();
    input_options.remove_all();
    for input_device in &input_devices {
        let input_device_name = input_device.name().unwrap();

        input_options.append_text(&input_device_name);
    }

    let default_input_pos = input_devices
        .iter()
        .position(|input_device| audio_io.get_input_device().name().unwrap() == *input_device.name().unwrap())
        .unwrap() as u32;

    input_options.set_active(Some(default_input_pos));
}

/// Populates the choices available for output devices
fn populate_output_options(output_options: &mut ComboBoxText, audio_io: &AudioIO) {
    let output_devices = audio_io.get_output_devices();
    output_options.remove_all();
    for output_device in &output_devices {
        let output_device_name = output_device.name().unwrap();

        output_options.append_text(&output_device_name);
    }

    let default_output_pos = output_devices
        .iter()
        .position(|output_device| audio_io.get_output_device().name().unwrap() == *output_device.name().unwrap())
        .unwrap() as u32;

    output_options.set_active(Some(default_output_pos));
}

/// Populates fields for given Input Device
fn populate_input_preference_fields(input_device: &Device, model: &Model, widgets: &mut Widgets) {
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

/// Abstracts the whole application, merging
/// the Model and references to the View and Controller
/// widgets.
struct Win {
    model: Model,
    widgets: Widgets,
}

/// Implements the Event Handler of all User Actions for
/// this application.
///
/// In essence, this is an [Event loop.](https://en.wikipedia.org/wiki/Event_loop)
impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Model {
        let chunk_retriever = EnglishParagraphRetriever::new();
        let project_directory = dirs::home_dir()
            .unwrap()
            .join("ND_Projects")
            .to_str()
            .unwrap()
            .to_string();

        Model {
            chunk_retriever,
            chunk_number: 0,
            chunk_total: 0,

            audio_processor: AudioIO::new(0, project_directory.clone()),
            ms_passed: 0,
            ms_total: 0,
            audio_status: ProcessingStatus::Stopped,

            input_devices_info: None,
            preferences_has_been_shown_once: false,
            current_filename: String::new(),
            project_directory,
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        interval(relm.stream(), 100, || ProgressTick);
    }

    // This is where all User Events are parsed, influencing how
    // the Model and View changes.
    fn update(&mut self, event: Msg) {
        let prg_progress_label = &self.widgets.chunk_progress_label;
        let text_viewer = &self.widgets.chunk_viewer;
        let audio_progress_label = &self.widgets.audio_progress_label;
        let progress_bar = &self.widgets.progress_bar;

        let paragraph_ui = ChunkViewingUi {
            progress_label: prg_progress_label,
            chunk_viewer: text_viewer,
        };

        match event {
            Msg::ProgressTick => match self.model.audio_status {
                ProcessingStatus::Playing => {
                    self.model.ms_passed += 100;
                    let current_time = to_hh_mm_ss_str(self.model.ms_passed);
                    let total_time = to_hh_mm_ss_str(self.model.ms_total);

                    let progress_text = format!("{}/{}", current_time, total_time);
                    audio_progress_label.set_markup(progress_text.as_str());

                    if current_time.eq(&total_time) {
                        self.update(Msg::Stop);
                        return;
                    }

                    progress_bar.set_value(self.model.ms_passed as f64 / 1000.0);
                }
                ProcessingStatus::Recording => {
                    self.model.ms_passed += 100;
                    self.model.ms_total = self.model.ms_passed;
                    let progress_text = format!(
                        "{}/{}",
                        to_hh_mm_ss_str(self.model.ms_passed),
                        to_hh_mm_ss_str(self.model.ms_passed)
                    );
                    audio_progress_label.set_markup(progress_text.as_str());
                }
                ProcessingStatus::Stopped => {
                    let progress_text = format!(
                        "{}/{}",
                        to_hh_mm_ss_str(self.model.ms_passed),
                        to_hh_mm_ss_str(self.model.ms_total)
                    );
                    audio_progress_label.set_markup(progress_text.as_str());
                }
                _ => {}
            },
            Msg::Next => {
                if show_chunk(
                    self.model.chunk_number + 1,
                    &self.model.chunk_retriever,
                    paragraph_ui,
                )
                .is_ok()
                {
                    self.model.chunk_number += 1;

                    self.widgets.previous_chunk_button.set_sensitive(true);
                    change_next_button_sensitivity(
                        self.model.chunk_number,
                        self.model.chunk_total,
                        &self.widgets.next_chunk_button,
                    );

                    if let Ok(file_status) = self.model.audio_processor.next() {
                        change_play_button_sensitivity(file_status, &self.widgets.play_button);

                        let duration_ms = match file_status {
                            FileStatus::Exists => self.model.audio_processor.duration(),
                            FileStatus::New => 0,
                        };

                        self.model.ms_total = duration_ms;
                        let progress_text = format!(
                            "{}/{}",
                            to_hh_mm_ss_str(self.model.ms_passed),
                            to_hh_mm_ss_str(self.model.ms_total)
                        );
                        audio_progress_label.set_markup(progress_text.as_str());

                        self.model.ms_passed = 0;
                        let secs_passed = (self.model.ms_passed / 1000) as f64;
                        let secs_total = (self.model.ms_total / 1000) as f64;
                        progress_bar.set_adjustment(&Adjustment::new(
                            secs_passed,
                            0.0,
                            secs_total,
                            1.0,
                            1.0,
                            1.0,
                        ));
                    }
                }
            }
            Msg::Previous => {
                if self.model.chunk_number == 0 {
                    return;
                }

                if show_chunk(
                    self.model.chunk_number - 1,
                    &self.model.chunk_retriever,
                    paragraph_ui,
                )
                .is_ok()
                {
                    self.model.chunk_number -= 1;

                    self.widgets.next_chunk_button.set_sensitive(true);
                    change_prev_button_sensitivity(
                        self.model.chunk_number,
                        &self.widgets.previous_chunk_button,
                    );

                    if let Ok(file_status) = self.model.audio_processor.prev() {
                        change_play_button_sensitivity(file_status, &self.widgets.play_button);

                        self.model.ms_total = match file_status {
                            FileStatus::Exists => self.model.audio_processor.duration(),
                            FileStatus::New => 0,
                        };

                        self.model.ms_passed = 0;
                        let secs_passed = (self.model.ms_passed / 1000) as f64;
                        let secs_total = (self.model.ms_total / 1000) as f64;
                        progress_bar.set_adjustment(&Adjustment::new(
                            secs_passed,
                            0.0,
                            secs_total,
                            1.0,
                            1.0,
                            1.0,
                        ));
                    }
                }
            }
            Msg::JumpTo => {
                if self.model.chunk_total == 0 {
                    return;
                }

                let goto_dialog = Dialog::with_buttons(
                    Some("Select the paragraph number."),
                    Some(&self.widgets.window),
                    DialogFlags::MODAL,
                    &[("Ok", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
                );
                goto_dialog.set_default_response(ResponseType::Ok);

                let content_area = goto_dialog.get_content_area();

                let goto_spin_button =
                    SpinButton::with_range(1.0, self.model.chunk_total as f64, 1.0);
                content_area.add(&goto_spin_button);

                goto_dialog.show_all();

                let goto_dialog_response = goto_dialog.run();
                if goto_dialog_response == ResponseType::Ok {
                    let goto_paragraph_num = (goto_spin_button.get_value_as_int() - 1) as u32;
                    if show_chunk(
                        goto_paragraph_num,
                        &self.model.chunk_retriever,
                        paragraph_ui,
                    )
                    .is_ok()
                    {
                        self.model.chunk_number = goto_paragraph_num;

                        change_next_button_sensitivity(
                            self.model.chunk_number,
                            self.model.chunk_total,
                            &self.widgets.next_chunk_button,
                        );
                        change_prev_button_sensitivity(
                            self.model.chunk_number,
                            &self.widgets.previous_chunk_button,
                        );

                        if let Ok(file_status) = self
                            .model
                            .audio_processor
                            .go_to(self.model.chunk_number as usize)
                        {
                            change_play_button_sensitivity(file_status, &self.widgets.play_button);

                            self.model.ms_total = match file_status {
                                FileStatus::Exists => self.model.audio_processor.duration(),
                                FileStatus::New => 0,
                            };

                            self.model.ms_passed = 0;
                            let secs_total = (self.model.ms_total / 1000) as f64;
                            progress_bar.set_adjustment(&Adjustment::new(
                                0.0, 0.0, secs_total, 1.0, 1.0, 1.0,
                            ));
                        }
                    }
                }

                goto_dialog.close();
            }
            Msg::OpenPreferences => {
                // Map all Channels and Sample Rates to their respective Input Device if not
                // already done.
                if self.model.input_devices_info.is_none() {
                    // Save Sample Rates and Channels for each known Input Device
                    let mut input_channels: HashMap<String, Vec<ChannelCount>> = HashMap::new();
                    let mut input_sample_rates: HashMap<String, Vec<u32>> = HashMap::new();

                    let input_devices = self.model.audio_processor.get_input_devices();
                    for input_device in input_devices {
                        input_channels.insert(
                            input_device.name().unwrap(),
                            self.model
                                .audio_processor
                                .get_input_channels_for(&input_device),
                        );
                        input_sample_rates.insert(
                            input_device.name().unwrap(),
                            self.model
                                .audio_processor
                                .get_input_sample_rates_for(&input_device),
                        );
                    }

                    self.model.input_devices_info = Some(InputDevicesInfo {
                        channels: input_channels,
                        sample_rates: input_sample_rates,
                    });

                    // Populate UI from current input device (The default device)
                    let default_input_device = self.model.audio_processor.get_input_device();

                    populate_input_preference_fields(
                        default_input_device,
                        &self.model,
                        &mut self.widgets,
                    );
                }

                // Show the preferences dialog
                if !self.model.preferences_has_been_shown_once {
                    self.widgets
                        .preferences_dialog
                        .add_buttons(&[("Ok", ResponseType::Ok), ("Cancel", ResponseType::Cancel)]);
                    self.model.preferences_has_been_shown_once = true;
                }
                self.widgets
                    .preferences_dialog
                    .set_default_response(ResponseType::Ok);

                // Determine if a new Output or Input Device must be created.

                // Capture current positions of input fields if cancelled
                let current_input_device_pos = self.widgets.input_device_cbox.get_active();
                let current_input_channels_pos = self.widgets.input_channels_cbox.get_active();
                let current_input_sample_rate_pos =
                    self.widgets.input_sample_rate_cbox.get_active();

                // Now the output fields
                let current_output_device_pos = self.widgets.output_device_cbox.get_active();

                // Finally, the general fields (Project folder location, etc).
                let current_project_folder = self.widgets.project_file_chooser.get_current_folder();

                let preference_response = self.widgets.preferences_dialog.run();
                if preference_response == ResponseType::Ok {
                    let project_path = self.widgets.project_file_chooser.get_filename().unwrap();

                    let new_directory = project_path.join(&self.model.current_filename);
                    if !new_directory.is_dir() {
                        let mut dir_builder = DirBuilder::new();
                        dir_builder.recursive(true);

                        dir_builder.create(new_directory.clone()).unwrap();
                    }

                    self.model.project_directory = project_path.to_str().unwrap().to_string();
                    self.model.audio_processor = AudioIO::new(
                        self.model.chunk_total as usize,
                        new_directory.to_str().unwrap().to_string(),
                    );

                    let sample_rate_choice = self
                        .widgets
                        .input_sample_rate_cbox
                        .get_active_text()
                        .unwrap()
                        .to_string();
                    let sample_rate = sample_rate_choice.parse::<u32>().unwrap();

                    let channel_choice = self
                        .widgets
                        .input_channels_cbox
                        .get_active_text()
                        .unwrap()
                        .to_string();
                    let num_channels = channel_choice.parse::<u16>().unwrap();

                    let input_info = InputDeviceSelection {
                        name: self
                            .widgets
                            .input_device_cbox
                            .get_active_text()
                            .unwrap()
                            .to_string(),
                        sample_rate,
                        num_channels,
                    };

                    let output_info = OutputDeviceInfo {
                        name: self
                            .widgets
                            .output_device_cbox
                            .get_active_text()
                            .unwrap()
                            .to_string(),
                    };

                    self.model.audio_processor.set_input_device(input_info);
                    self.model.audio_processor.set_output_device(output_info);

                    let file_status = self
                        .model
                        .audio_processor
                        .go_to(self.model.chunk_number as usize)
                        .unwrap();
                    reset_progress_bar_info(file_status, &mut self.model, &self.widgets);
                } else {
                    // Reset all fields back to original positions
                    self.widgets
                        .input_device_cbox
                        .set_active(current_input_device_pos);
                    self.widgets
                        .input_channels_cbox
                        .set_active(current_input_channels_pos);
                    self.widgets
                        .input_sample_rate_cbox
                        .set_active(current_input_sample_rate_pos);

                    self.widgets
                        .output_device_cbox
                        .set_active(current_output_device_pos);

                    self.widgets
                        .project_file_chooser
                        .set_filename(current_project_folder.unwrap());
                }
                self.widgets.preferences_dialog.hide();
            }
            Msg::LoadFile => {
                let file_chooser = gtk::FileChooserDialog::new(
                    Some("Open File"),
                    Some(&self.widgets.window),
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

                let file_chooser_response = file_chooser.run();
                if file_chooser_response == ResponseType::Ok {
                    // Start by making a new paragraph parser with the given file.
                    let filename = file_chooser.get_filename().expect("Couldn't get filename");
                    let file = File::open(&filename).expect("Couldn't open file");

                    self.model.chunk_retriever = EnglishParagraphRetriever::new();
                    let num_paragraphs = self.model.chunk_retriever.load_chunks(file);

                    if num_paragraphs == 0 {
                        return;
                    }

                    self.model.chunk_number = 0;
                    self.model.chunk_total = num_paragraphs;

                    // Then redirect where the audio files will be read/written to the
                    // current project directory.
                    let project_path = Path::new(&self.model.project_directory)
                        .join(filename.file_stem().unwrap());
                    if !project_path.is_dir() {
                        let mut dir_builder = DirBuilder::new();
                        dir_builder.recursive(true);

                        dir_builder.create(project_path.clone()).unwrap();
                    }

                    self.model.current_filename =
                        filename.file_stem().unwrap().to_str().unwrap().to_string();

                    // Also, reload the audio processor to utilize the new
                    // project path.
                    self.model.audio_processor = AudioIO::new(
                        self.model.chunk_total as usize,
                        project_path.to_str().unwrap().to_string(),
                    );

                    // Keep the input and output selections in preferences if the user has
                    // already specified their devices.
                    if self.model.preferences_has_been_shown_once {
                        let sample_rate_choice = self
                            .widgets
                            .input_sample_rate_cbox
                            .get_active_text()
                            .unwrap()
                            .to_string();
                        let sample_rate = sample_rate_choice.parse::<u32>().unwrap();

                        let channel_choice = self
                            .widgets
                            .input_channels_cbox
                            .get_active_text()
                            .unwrap()
                            .to_string();
                        let num_channels = channel_choice.parse::<u16>().unwrap();

                        let input_info = InputDeviceSelection {
                            name: self
                                .widgets
                                .input_device_cbox
                                .get_active_text()
                                .unwrap()
                                .to_string(),
                            sample_rate,
                            num_channels,
                        };

                        let output_info = OutputDeviceInfo {
                            name: self
                                .widgets
                                .output_device_cbox
                                .get_active_text()
                                .unwrap()
                                .to_string(),
                        };

                        self.model.audio_processor.set_input_device(input_info);
                        self.model.audio_processor.set_output_device(output_info);
                    }

                    // Finally, make the right buttons active depending on what chunks are available.
                    show_chunk(0, &self.model.chunk_retriever, paragraph_ui).unwrap();

                    change_next_button_sensitivity(
                        self.model.chunk_number,
                        self.model.chunk_total,
                        &self.widgets.next_chunk_button,
                    );

                    self.widgets.goto_menu_item.set_sensitive(true);
                    self.widgets.record_button.set_sensitive(true);

                    let file_status = self.model.audio_processor.go_to(0).unwrap();
                    reset_progress_bar_info(file_status, &mut self.model, &self.widgets);
                }

                file_chooser.close();
            }
            Msg::Quit => gtk::main_quit(),
            Msg::Play => {
                if self.model.audio_status == ProcessingStatus::Playing {
                    self.model.audio_status = self
                        .model
                        .audio_processor
                        .pause(self.model.ms_passed)
                        .unwrap();
                    self.widgets.play_button.set_label("Play");
                } else {
                    self.model.audio_status = match self.widgets.progress_bar.get_value() {
                        x if x >= 1.0 => {
                            self.model.audio_processor.skip_to(x as u32 * 1000).unwrap()
                        }
                        _ => self.model.audio_processor.play().unwrap(),
                    };

                    self.widgets.play_button.set_label("Pause");
                    if self.model.ms_total == 0 {
                        self.model.ms_total = self.model.audio_processor.duration();
                    }
                }

                self.widgets.record_button.set_sensitive(false);
                self.widgets.stop_button.set_sensitive(true);

                self.widgets.previous_chunk_button.set_sensitive(false);
                self.widgets.next_chunk_button.set_sensitive(false);
            }
            Msg::AudioSkip => {
                if self.model.audio_status == ProcessingStatus::Playing
                    || self.model.audio_status == ProcessingStatus::Recording
                {
                    return;
                }

                self.model.ms_passed = self.widgets.progress_bar.get_value() as u32 * 1000;
                let progress_text = format!(
                    "{}/{}",
                    to_hh_mm_ss_str(self.model.ms_passed),
                    to_hh_mm_ss_str(self.model.ms_total)
                );
                audio_progress_label.set_markup(progress_text.as_str());
            }
            Msg::Stop => {
                if self.model.audio_status == ProcessingStatus::Recording {
                    self.model.audio_status = self.model.audio_processor.stop_recording().unwrap();
                } else {
                    self.model.audio_status = self.model.audio_processor.stop().unwrap();
                }

                self.widgets.record_button.set_sensitive(true);
                self.widgets.play_button.set_sensitive(true);
                self.widgets.play_button.set_label("Play");

                self.widgets.stop_button.set_sensitive(false);

                change_prev_button_sensitivity(
                    self.model.chunk_number,
                    &self.widgets.previous_chunk_button,
                );
                change_next_button_sensitivity(
                    self.model.chunk_number,
                    self.model.chunk_total,
                    &self.widgets.next_chunk_button,
                );

                self.model.ms_passed = 0;

                let secs_total = (self.model.ms_total / 1000) as f64;
                progress_bar.set_adjustment(&Adjustment::new(0.0, 0.0, secs_total, 1.0, 1.0, 1.0));
            }
            Msg::Record => {
                self.model.audio_status = self.model.audio_processor.record().unwrap();

                self.widgets.record_button.set_sensitive(false);
                self.widgets.play_button.set_sensitive(false);
                self.widgets.stop_button.set_sensitive(true);

                self.widgets.previous_chunk_button.set_sensitive(false);
                self.widgets.next_chunk_button.set_sensitive(false);
            }
            Msg::About => {
                self.widgets.about_dialog.run();
                self.widgets.about_dialog.hide();
            }
        }
    }
}

/// Implements the Viewer elements of the application, connecting
/// behaviors to each that invoke Events accordingly.
impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let glade_src = include_str!("ui/main-window.glade");
        let builder = Builder::from_string(glade_src);

        let window: Window = builder.get_object("window").unwrap();
        window.show_all();

        // Main Window Widgets
        let chunk_progress_label: Label = builder.get_object("chunk_position_lbl").unwrap();
        let text_viewer: TextView = builder.get_object("chunk_view_txtviewer").unwrap();

        // Media IO Items
        let prev_button: Button = builder.get_object("prev_chunk_btn").unwrap();
        let next_button: Button = builder.get_object("next_chunk_btn").unwrap();
        let stop_button: Button = builder.get_object("stop_btn").unwrap();
        let record_button: Button = builder.get_object("record_btn").unwrap();
        let play_button: Button = builder.get_object("play/pause_btn").unwrap();
        let audio_progress_label: Label = builder.get_object("audio_progress_lbl").unwrap();
        let progress_bar: Scrollbar = builder.get_object("progress_bar").unwrap();

        // Main Menu Items
        let open_menu_item: MenuItem = builder.get_object("open_menu").unwrap();
        let goto_menu_item: MenuItem = builder.get_object("goto_menu").unwrap();
        let preferences_menu_item: MenuItem = builder.get_object("preferences_menu").unwrap();
        let about_menu_item: MenuItem = builder.get_object("about_menu").unwrap();
        let quit_menu_item: MenuItem = builder.get_object("close_menu").unwrap();

        // Dialogs
        let about_dialog: AboutDialog = builder.get_object("about_dialog").unwrap();

        // Preferences
        let preferences_dialog: Dialog = builder.get_object("preferences_dialog").unwrap();
        // General - Preferences
        let project_file_chooser: FileChooser = builder.get_object("project_file_chooser").unwrap();
        let home_directory = dirs::home_dir().unwrap().join("ND_Projects");
        let project_path = Path::new(home_directory.as_path());
        if !project_path.is_dir() {
            fs::create_dir(project_path).unwrap();
        }
        project_file_chooser.set_current_folder(project_path);
        // Audio - Preferences
        let mut input_device_cbox: ComboBoxText = builder.get_object("input_device_cbox").unwrap();
        populate_input_options(&mut input_device_cbox, &model.audio_processor);

        let input_sample_rate_cbox: ComboBoxText =
            builder.get_object("input_sample_rate_cbox").unwrap();
        let input_channels_cbox: ComboBoxText = builder.get_object("input_channels_cbox").unwrap();

        let mut output_device_cbox: ComboBoxText =
            builder.get_object("output_device_cbox").unwrap();
        populate_output_options(&mut output_device_cbox, &model.audio_processor);

        connect!(relm, prev_button, connect_clicked(_), Msg::Previous);
        connect!(relm, next_button, connect_clicked(_), Msg::Next);
        connect!(relm, stop_button, connect_clicked(_), Msg::Stop);
        connect!(relm, play_button, connect_clicked(_), Msg::Play);
        connect!(relm, record_button, connect_clicked(_), Msg::Record);
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );
        connect!(relm, progress_bar, connect_value_changed(_), Msg::AudioSkip);

        connect!(relm, open_menu_item, connect_activate(_), Msg::LoadFile);
        connect!(relm, goto_menu_item, connect_activate(_), Msg::JumpTo);
        connect!(
            relm,
            preferences_menu_item,
            connect_activate(_),
            Msg::OpenPreferences
        );
        connect!(relm, about_menu_item, connect_activate(_), Msg::About);
        connect!(quit_menu_item, connect_activate(_), relm, Msg::Quit);

        Win {
            model,
            widgets: Widgets {
                chunk_progress_label,
                chunk_viewer: text_viewer,

                previous_chunk_button: prev_button,
                next_chunk_button: next_button,
                stop_button,
                record_button,
                play_button,
                audio_progress_label,
                progress_bar,

                window,

                open_menu_item,
                goto_menu_item,
                preferences_menu_item,
                about_menu_item,
                quit_menu_item,

                about_dialog,

                preferences_dialog,
                project_file_chooser,
                input_device_cbox,
                input_sample_rate_cbox,
                input_channels_cbox,
                output_device_cbox,
            },
        }
    }
}

/// Spawns the application with a Graphical User Interface.
fn main() {
    Win::run(()).unwrap();
}
