mod media_io;
mod text_grabber;

use gtk::prelude::*;
use gtk::{
    AboutDialog, Adjustment, Builder, Button, ComboBoxText, Dialog, DialogFlags, FileChooser,
    FileFilter, Inhibit, Label, MenuItem, ResponseType, Scrollbar, SpinButton, TextView, Window,
};
use relm::{connect, interval, Relm, Update, Widget};
use relm_derive::Msg;

use crate::Msg::ProgressTick;
use cpal::traits::DeviceTrait;
use cpal::ChannelCount;
use media_io::prelude::*;
use media_io::ProcessingStatus;
use std::collections::HashMap;
use std::fs::File;
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
    ChangeInputDevicePreference,
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

/// Converts ms to hours:minutes:seconds format
fn to_hh_mm_ss_str(ms: u32) -> String {
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
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

        Model {
            chunk_retriever,
            chunk_number: 0,
            chunk_total: 0,

            audio_processor: AudioIO::new(0),
            ms_passed: 0,
            ms_total: 0,
            audio_status: ProcessingStatus::Stopped,

            input_devices_info: None,
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
            Msg::ChangeInputDevicePreference => {
                let input_device_info = self.model.input_devices_info.as_ref().unwrap();
                let selected_input_name = self
                    .widgets
                    .input_device_cbox
                    .get_active_text()
                    .unwrap()
                    .to_string();

                let input_channels = input_device_info
                    .channels
                    .get(&selected_input_name)
                    .unwrap();
                self.widgets.input_channels_cbox.remove_all();
                self.widgets.input_channels_cbox.append_text("default");
                for channel in input_channels {
                    self.widgets
                        .input_channels_cbox
                        .append_text(&channel.to_string());
                }
                self.widgets.input_channels_cbox.set_active(Some(0));

                let input_sample_rates = input_device_info
                    .sample_rates
                    .get(&selected_input_name)
                    .unwrap();
                self.widgets.input_sample_rate_cbox.remove_all();
                self.widgets.input_sample_rate_cbox.append_text("default");
                for sample_rate in input_sample_rates {
                    self.widgets
                        .input_sample_rate_cbox
                        .append_text(&sample_rate.to_string());
                }
                self.widgets.input_sample_rate_cbox.set_active(Some(0));
            }
            Msg::OpenPreferences => {
                // 1: Populate the available Input Devices
                let input_devices = self.model.audio_processor.get_input_devices();
                self.widgets.input_device_cbox.remove_all();
                self.widgets.input_device_cbox.append_text("default");
                for input_device in &input_devices {
                    let input_device_name = input_device.name().unwrap();
                    if input_device_name == "default" {
                        continue;
                    }

                    self.widgets
                        .input_device_cbox
                        .append_text(&input_device_name);
                }
                self.widgets.input_device_cbox.set_active(Some(0));

                // 2: Map all Channels and Sample Rates to their respective Input Device.
                let mut input_channels: HashMap<String, Vec<ChannelCount>> = HashMap::new();
                let mut input_sample_rates: HashMap<String, Vec<u32>> = HashMap::new();

                input_devices.iter().for_each(|device| {
                    let supported_configs = device.supported_input_configs().unwrap();

                    let mut channels: Vec<ChannelCount> = Vec::new();
                    let mut sample_rates: Vec<u32> = Vec::new();
                    supported_configs.for_each(|config| {
                        channels.push(config.channels());

                        const SAMPLE_RATES: [u32; 6] = [16000, 32000, 44100, 48000, 88200, 96000];
                        for sample_rate in SAMPLE_RATES.iter() {
                            if *sample_rate >= config.min_sample_rate().0
                                && *sample_rate <= config.max_sample_rate().0
                            {
                                sample_rates.push(*sample_rate);
                            }
                        }
                    });
                    channels.sort();
                    channels.dedup();
                    input_channels.insert(device.name().unwrap().clone(), channels);

                    sample_rates.sort();
                    sample_rates.dedup();
                    input_sample_rates.insert(device.name().unwrap().clone(), sample_rates);
                });

                // 3: Show default's channels and sample rates
                self.widgets.input_channels_cbox.remove_all();
                self.widgets.input_channels_cbox.append_text("default");
                input_channels
                    .get("default")
                    .unwrap()
                    .iter()
                    .for_each(|channel| {
                        self.widgets
                            .input_channels_cbox
                            .append_text(&channel.to_string());
                    });
                self.widgets.input_channels_cbox.set_active(Some(0));

                self.widgets.input_sample_rate_cbox.remove_all();
                self.widgets.input_sample_rate_cbox.append_text("default");
                input_sample_rates
                    .get("default")
                    .unwrap()
                    .iter()
                    .for_each(|sample_rate| {
                        self.widgets
                            .input_sample_rate_cbox
                            .append_text(&sample_rate.to_string());
                    });
                self.widgets.input_sample_rate_cbox.set_active(Some(0));

                // 4: Have mappings available when a user changes the input device.
                self.model.input_devices_info = Some(InputDevicesInfo {
                    channels: input_channels,
                    sample_rates: input_sample_rates,
                });

                // 5: Populate the available Output Devices.
                let output_devices = self.model.audio_processor.get_output_devices();
                self.widgets.output_device_cbox.remove_all();
                self.widgets.output_device_cbox.append_text("default");
                for output_device in &output_devices {
                    let output_device_name = output_device.name().unwrap();
                    if output_device_name == "default" {
                        continue;
                    }

                    self.widgets
                        .output_device_cbox
                        .append_text(&output_device_name);
                }
                self.widgets.output_device_cbox.set_active(Some(0));

                // 6: Show the preferences dialog
                self.widgets.preferences_dialog.show();

                // 7: Determine if a new Output or Input Device must be created.
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
                    let filename = file_chooser.get_filename().expect("Couldn't get filename");
                    let file = File::open(&filename).expect("Couldn't open file");

                    self.model.chunk_retriever = EnglishParagraphRetriever::new();
                    let num_paragraphs = self.model.chunk_retriever.load_chunks(file);

                    if num_paragraphs == 0 {
                        return;
                    }

                    self.model.chunk_number = 0;
                    self.model.chunk_total = num_paragraphs;
                    self.model.audio_processor = AudioIO::new(self.model.chunk_total as usize);
                    show_chunk(0, &self.model.chunk_retriever, paragraph_ui).unwrap();

                    self.widgets.next_chunk_button.set_sensitive(true);
                    self.widgets.goto_menu_item.set_sensitive(true);
                    self.widgets.record_button.set_sensitive(true);

                    if let Ok(file_status) = self
                        .model
                        .audio_processor
                        .go_to(self.model.chunk_number as usize)
                    {
                        change_play_button_sensitivity(file_status, &self.widgets.play_button);

                        self.model.ms_passed = 0;
                        self.model.ms_total = self.model.audio_processor.duration();

                        let secs_total = (self.model.ms_total / 1000) as f64;
                        progress_bar
                            .set_adjustment(&Adjustment::new(0.0, 0.0, secs_total, 1.0, 1.0, 1.0));
                    }
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
        // Audio - Preferences
        let input_device_cbox: ComboBoxText = builder.get_object("input_device_cbox").unwrap();
        let input_sample_rate_cbox: ComboBoxText =
            builder.get_object("input_sample_rate_cbox").unwrap();
        let input_channels_cbox: ComboBoxText = builder.get_object("input_channels_cbox").unwrap();

        let output_device_cbox: ComboBoxText = builder.get_object("output_device_cbox").unwrap();

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
        connect!(
            relm,
            input_device_cbox,
            connect_changed(_),
            Msg::ChangeInputDevicePreference
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
