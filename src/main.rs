/*
* This is needed to ensure that the application
* does not start with a console in the background
* when the application runs on Windows.
 */
#![windows_subsystem = "windows"]

use std::fs;
use std::fs::{DirBuilder, File};
use std::path::{Path, PathBuf};
use std::{collections::HashMap, io::Read};

use cpal::traits::DeviceTrait;
use cpal::ChannelCount;
use gtk::prelude::*;
use gtk::{
    AboutDialog, Builder, Button, ComboBoxText, Dialog, DialogFlags, EventBox, FileChooser,
    Inhibit, Label, MenuItem, ResponseType, Scrollbar, SpinButton, TextView, Window,
};
use relm::{connect, interval, Relm, Update, Widget};
use relm_derive::Msg;
use serde::{Deserialize, Serialize};

mod audio;
use audio::prelude::*;
use audio::ProcessingStatus;

mod text;
use text::prelude::*;

use crate::Msg::ProgressTick;

#[derive(Serialize, Deserialize, Debug)]
struct ChunksSessionInfo {
    current_paragraph_num: u32,
}

pub struct InputDevicesInfo {
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
pub struct Model {
    chunk_retriever: ParagraphRetriever,
    chunk_number: u32,
    chunk_total: u32,

    audio_processor: AudioIO,
    ms_passed: u32,
    ms_total: u32,
    audio_status: ProcessingStatus,

    input_devices_info: InputDevicesInfo,
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
pub struct Widgets {
    // Main Window Widgets
    chunk_progress_label: Label,
    chunk_progress_eventbox: EventBox,
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

/// Returns if the Project directory has been created.
fn create_project_dir_from(project_path: &Path) -> bool {
    if project_path.is_dir() {
        return false;
    }

    let mut dir_builder = DirBuilder::new();
    dir_builder.recursive(true);

    dir_builder.create(project_path).is_ok()
}

/// Returns the chunk number from a given session file, or zero if none was found.
fn get_session_chunk_number(session_path: PathBuf) -> u32 {
    if !session_path.is_file() {
        return 0;
    }

    let mut session_file = File::open(session_path).expect("Could not load session file.");
    let mut file_contents = String::new();
    session_file
        .read_to_string(&mut file_contents)
        .expect("Unable to read contents from session file.");

    let session_info: ChunksSessionInfo =
        serde_json::from_str(&file_contents).expect("Unable to parse JSON from session file.");

    session_info.current_paragraph_num
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
        let chunk_retriever = ParagraphRetriever::new();
        let project_directory = dirs::audio_dir().unwrap().to_str().unwrap().to_string();
        let audio_processor = AudioIO::new(0, project_directory.clone());

        // Save Sample Rates and Channels for each known Input Device
        let mut input_channels: HashMap<String, Vec<ChannelCount>> = HashMap::new();
        let mut input_sample_rates: HashMap<String, Vec<u32>> = HashMap::new();

        let input_devices = audio_processor.get_input_devices();
        for input_device in input_devices {
            input_channels.insert(
                input_device.name().unwrap(),
                audio_processor.get_input_channels_for(&input_device),
            );
            input_sample_rates.insert(
                input_device.name().unwrap(),
                audio_processor.get_input_sample_rates_for(&input_device),
            );
        }

        let input_devices_info = InputDevicesInfo {
            channels: input_channels,
            sample_rates: input_sample_rates,
        };

        Model {
            chunk_retriever,
            chunk_number: 0,
            chunk_total: 0,

            audio_processor,
            ms_passed: 0,
            ms_total: 0,
            audio_status: ProcessingStatus::Stopped,

            input_devices_info,
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
        let progress_bar = &self.widgets.progress_bar;

        let playback_widgets = AudioPlaybackWidgets {
            progress_bar,
            progress_text: &self.widgets.audio_progress_label,
        };

        let text_widgets = TextNavigationWidgets {
            previous_button: &self.widgets.previous_chunk_button,
            next_button: &self.widgets.next_chunk_button,
            goto_menu: &self.widgets.goto_menu_item,
        };

        let input_widgets = InputPreferenceWidgets {
            input_device_cbox: &mut self.widgets.input_device_cbox,
            sample_rate_cbox: &mut self.widgets.input_sample_rate_cbox,
            channels_cbox: &mut self.widgets.input_channels_cbox,
        };

        let output_widgets = OutputPreferenceWidgets {
            output_device_cbox: &self.widgets.output_device_cbox,
        };

        let paragraph_ui = ChunkViewingUi {
            progress_label: prg_progress_label,
            chunk_viewer: text_viewer,
        };

        match event {
            Msg::ProgressTick => match self.model.audio_status {
                ProcessingStatus::Playing => {
                    self.model.ms_passed += 100;

                    let playback_progress = AudioPlaybackProgress {
                        ms_passed: self.model.ms_passed,
                        ms_total: self.model.ms_total,
                    };

                    update_playback_widgets(playback_widgets, playback_progress);

                    if self.model.ms_passed == self.model.ms_total {
                        self.update(Msg::Stop);
                        return;
                    }

                    progress_bar.set_value(self.model.ms_passed as f64 / 1000.0);
                }
                ProcessingStatus::Recording => {
                    self.model.ms_passed += 100;
                    self.model.ms_total = self.model.ms_passed;

                    let playback_progress = AudioPlaybackProgress {
                        ms_passed: self.model.ms_passed,
                        ms_total: self.model.ms_total,
                    };

                    update_playback_widgets(playback_widgets, playback_progress);
                }
                ProcessingStatus::Stopped => {
                    let playback_progress = AudioPlaybackProgress {
                        ms_passed: self.model.ms_passed,
                        ms_total: self.model.ms_total,
                    };

                    update_playback_widgets(playback_widgets, playback_progress);
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

                    let text_progress = TextNavigationProgress {
                        current_index: self.model.chunk_number,
                        total: self.model.chunk_total,
                    };

                    toggle_text_navigation_widgets(text_widgets, text_progress);

                    if let Ok(file_status) = self.model.audio_processor.next() {
                        change_play_button_sensitivity(file_status, &self.widgets.play_button);

                        self.model.ms_passed = 0;
                        self.model.ms_total = match file_status {
                            FileStatus::Exists => self.model.audio_processor.duration(),
                            FileStatus::New => 0,
                        };

                        let playback_progress = AudioPlaybackProgress {
                            ms_passed: self.model.ms_passed,
                            ms_total: self.model.ms_total,
                        };

                        update_playback_widgets(playback_widgets, playback_progress);
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

                    let text_progress = TextNavigationProgress {
                        current_index: self.model.chunk_number,
                        total: self.model.chunk_total,
                    };

                    toggle_text_navigation_widgets(text_widgets, text_progress);

                    if let Ok(file_status) = self.model.audio_processor.prev() {
                        change_play_button_sensitivity(file_status, &self.widgets.play_button);

                        self.model.ms_passed = 0;
                        self.model.ms_total = match file_status {
                            FileStatus::Exists => self.model.audio_processor.duration(),
                            FileStatus::New => 0,
                        };

                        let playback_progress = AudioPlaybackProgress {
                            ms_passed: self.model.ms_passed,
                            ms_total: self.model.ms_total,
                        };

                        update_playback_widgets(playback_widgets, playback_progress);
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

                let content_area = goto_dialog.content_area();

                let goto_spin_button =
                    SpinButton::with_range(1.0, self.model.chunk_total as f64, 1.0);
                goto_spin_button.set_activates_default(true);

                content_area.add(&goto_spin_button);

                goto_dialog.show_all();

                let goto_dialog_response = goto_dialog.run();
                if goto_dialog_response == ResponseType::Ok {
                    let goto_paragraph_num = (goto_spin_button.value_as_int() - 1) as u32;
                    if show_chunk(
                        goto_paragraph_num,
                        &self.model.chunk_retriever,
                        paragraph_ui,
                    )
                    .is_ok()
                    {
                        self.model.chunk_number = goto_paragraph_num;

                        let text_progress = TextNavigationProgress {
                            current_index: self.model.chunk_number,
                            total: self.model.chunk_total,
                        };

                        toggle_text_navigation_widgets(text_widgets, text_progress);

                        if let Ok(file_status) = self
                            .model
                            .audio_processor
                            .go_to(self.model.chunk_number as usize)
                        {
                            change_play_button_sensitivity(file_status, &self.widgets.play_button);

                            self.model.ms_passed = 0;
                            self.model.ms_total = match file_status {
                                FileStatus::Exists => self.model.audio_processor.duration(),
                                FileStatus::New => 0,
                            };

                            let playback_progress = AudioPlaybackProgress {
                                ms_passed: self.model.ms_passed,
                                ms_total: self.model.ms_total,
                            };

                            update_playback_widgets(playback_widgets, playback_progress);
                        }
                    }
                }

                goto_dialog.close();
            }
            Msg::OpenPreferences => {
                // Show the preferences dialog
                if !self.model.preferences_has_been_shown_once {
                    self.widgets
                        .preferences_dialog
                        .add_buttons(&[("Ok", ResponseType::Ok), ("Cancel", ResponseType::Cancel)]);

                    self.widgets
                        .preferences_dialog
                        .set_default_response(ResponseType::Ok);

                    self.model.preferences_has_been_shown_once = true;
                }

                // Populate UI from current input device
                let current_input_device = self.model.audio_processor.get_input_device();

                populate_input_preference_fields(
                    current_input_device,
                    &self.model.input_devices_info,
                    &input_widgets,
                );

                // Determine if a new Output or Input Device must be created.

                // Capture current positions of input fields if cancelled
                let current_input_device_pos = input_widgets.input_device_cbox.active();
                let current_input_channels_pos = input_widgets.channels_cbox.active();
                let current_input_sample_rate_pos = input_widgets.sample_rate_cbox.active();

                // Now the output fields
                let current_output_device_pos = self.widgets.output_device_cbox.active();

                // Finally, the general fields (Project folder location, etc).
                let current_project_folder = self.widgets.project_file_chooser.current_folder();

                let preference_response = self.widgets.preferences_dialog.run();
                if preference_response == ResponseType::Ok {
                    let project_path = self.widgets.project_file_chooser.filename().unwrap();

                    let new_directory = project_path.join(&self.model.current_filename);
                    create_project_dir_from(&new_directory);

                    self.model.project_directory = project_path.to_str().unwrap().to_string();
                    self.model.audio_processor = AudioIO::new(
                        self.model.chunk_total as usize,
                        new_directory.to_str().unwrap().to_string(),
                    );

                    let input_info = get_input_selection_from(input_widgets);
                    let output_info = get_output_selection_from(output_widgets);

                    self.model.audio_processor.set_input_device(input_info);
                    self.model.audio_processor.set_output_device(output_info);

                    let file_status = self
                        .model
                        .audio_processor
                        .go_to(self.model.chunk_number as usize)
                        .unwrap();

                    change_play_button_sensitivity(file_status, &self.widgets.play_button);

                    self.model.ms_passed = 0;
                    self.model.ms_total = self.model.audio_processor.duration();

                    let playback_progress = AudioPlaybackProgress {
                        ms_passed: self.model.ms_passed,
                        ms_total: self.model.ms_total,
                    };

                    update_playback_widgets(playback_widgets, playback_progress);
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
                // First, we get the user chosen text file.
                let user_chosen_file = get_text_file_from_user(&self.widgets.window);
                if user_chosen_file.is_none() {
                    return;
                }

                let text_file_path = user_chosen_file.unwrap();
                let text_filename = text_file_path.file_stem().unwrap().to_str().unwrap();
                let text_file = File::open(&text_file_path).expect("Couldn't open file");

                self.model.chunk_retriever = ParagraphRetriever::new();
                let num_paragraphs = self.model.chunk_retriever.load_chunks(text_file);
                if num_paragraphs == 0 {
                    return;
                }

                self.model.chunk_total = num_paragraphs;

                // Then redirect where the audio files will be read/written to the
                // current project directory.
                let project_path = Path::new(&self.model.project_directory).join(text_filename);

                create_project_dir_from(&project_path);
                self.model.current_filename = String::from(text_filename);

                // Load the user's last position, since the project path existing must
                // mean that a session file was made.
                let session_file_path = Path::new(&project_path).join(".session.json");
                self.model.chunk_number = get_session_chunk_number(session_file_path);

                // Also, reload the audio processor to utilize the new
                // project path.
                self.model.audio_processor = AudioIO::new(
                    self.model.chunk_total as usize,
                    project_path.to_str().unwrap().to_string(),
                );

                // Keep the input and output selections in preferences if the user has
                // already specified their devices.
                if self.model.preferences_has_been_shown_once {
                    let input_info = get_input_selection_from(input_widgets);
                    let output_info = get_output_selection_from(output_widgets);

                    self.model.audio_processor.set_input_device(input_info);
                    self.model.audio_processor.set_output_device(output_info);
                }

                self.widgets.record_button.set_sensitive(true);

                let file_status = self
                    .model
                    .audio_processor
                    .go_to(self.model.chunk_number as usize)
                    .unwrap();

                change_play_button_sensitivity(file_status, &self.widgets.play_button);

                self.model.ms_passed = 0;
                self.model.ms_total = self.model.audio_processor.duration();

                let playback_progress = AudioPlaybackProgress {
                    ms_passed: self.model.ms_passed,
                    ms_total: self.model.ms_total,
                };

                update_playback_widgets(playback_widgets, playback_progress);

                // Finally, make the right buttons active depending on what chunks are available.
                show_chunk(
                    self.model.chunk_number,
                    &self.model.chunk_retriever,
                    paragraph_ui,
                )
                .unwrap();

                let text_progress = TextNavigationProgress {
                    current_index: self.model.chunk_number,
                    total: self.model.chunk_total,
                };

                toggle_text_navigation_widgets(text_widgets, text_progress);
            }
            Msg::Quit => {
                if !self.model.current_filename.is_empty() {
                    let filepath =
                        Path::new(&self.model.project_directory).join(&self.model.current_filename);
                    debug_assert!(filepath.is_dir());

                    let session_info: ChunksSessionInfo = ChunksSessionInfo {
                        current_paragraph_num: self.model.chunk_number,
                    };

                    fs::write(
                        filepath.join(".session.json"),
                        serde_json::to_string(&session_info).unwrap(),
                    )
                    .expect("Could not write to session file.");
                }

                gtk::main_quit();
            }
            Msg::Play => {
                if self.model.audio_status == ProcessingStatus::Playing {
                    self.model.audio_status = self
                        .model
                        .audio_processor
                        .pause(self.model.ms_passed)
                        .unwrap();
                    self.widgets.play_button.set_label("Play");
                } else {
                    self.model.audio_status = match self.widgets.progress_bar.value() {
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

                self.model.ms_passed = self.widgets.progress_bar.value() as u32 * 1000;
                let playback_progress = AudioPlaybackProgress {
                    ms_passed: self.model.ms_passed,
                    ms_total: self.model.ms_total,
                };

                update_playback_widgets(playback_widgets, playback_progress);
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

                let text_progress = TextNavigationProgress {
                    current_index: self.model.chunk_number,
                    total: self.model.chunk_total,
                };

                toggle_text_navigation_widgets(text_widgets, text_progress);

                self.model.ms_passed = 0;
                let playback_progress = AudioPlaybackProgress {
                    ms_passed: self.model.ms_passed,
                    ms_total: self.model.ms_total,
                };

                update_playback_widgets(playback_widgets, playback_progress);
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

        let window: Window = builder.object("window").unwrap();
        window.show_all();
        let icon_location = PathBuf::new()
            .join("resources")
            .join("images")
            .join("icon.png");

        window
            .set_icon_from_file(icon_location)
            .expect("Could not load icon for application.");

        // Main Window Widgets
        let chunk_progress_label: Label = builder.object("chunk_position_lbl").unwrap();
        let text_viewer: TextView = builder.object("chunk_view_txtviewer").unwrap();
        let chunk_progress_eventbox: EventBox = builder.object("chunk_progress_eventbox").unwrap();

        // Media IO Items
        let prev_button: Button = builder.object("prev_chunk_btn").unwrap();
        let next_button: Button = builder.object("next_chunk_btn").unwrap();
        let stop_button: Button = builder.object("stop_btn").unwrap();
        let record_button: Button = builder.object("record_btn").unwrap();
        let play_button: Button = builder.object("play/pause_btn").unwrap();
        let audio_progress_label: Label = builder.object("audio_progress_lbl").unwrap();
        let progress_bar: Scrollbar = builder.object("progress_bar").unwrap();

        // Main Menu Items
        let open_menu_item: MenuItem = builder.object("open_menu").unwrap();
        let goto_menu_item: MenuItem = builder.object("goto_menu").unwrap();
        let preferences_menu_item: MenuItem = builder.object("preferences_menu").unwrap();
        let about_menu_item: MenuItem = builder.object("about_menu").unwrap();
        let quit_menu_item: MenuItem = builder.object("close_menu").unwrap();

        // Dialogs
        let about_dialog: AboutDialog = builder.object("about_dialog").unwrap();

        // Preferences
        let preferences_dialog: Dialog = builder.object("preferences_dialog").unwrap();
        // General - Preferences
        let project_file_chooser: FileChooser = builder.object("project_file_chooser").unwrap();
        let audio_directory = dirs::audio_dir().unwrap();
        let project_path = Path::new(audio_directory.as_path());
        project_file_chooser.set_current_folder(project_path);
        // Audio - Preferences
        let mut input_device_cbox: ComboBoxText = builder.object("input_device_cbox").unwrap();
        populate_input_options(&mut input_device_cbox, &model.audio_processor);

        let input_sample_rate_cbox: ComboBoxText =
            builder.object("input_sample_rate_cbox").unwrap();
        let input_channels_cbox: ComboBoxText = builder.object("input_channels_cbox").unwrap();

        let mut output_device_cbox: ComboBoxText = builder.object("output_device_cbox").unwrap();
        populate_output_options(&mut output_device_cbox, &model.audio_processor);

        connect!(
            relm,
            chunk_progress_eventbox,
            connect_button_press_event(_, _),
            return (Some(Msg::JumpTo), Inhibit(false))
        );

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
                chunk_progress_eventbox,

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
