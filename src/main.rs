// This is needed to ensure that the application
// does not start with a console in the background
// when the application runs on Windows.
#![windows_subsystem = "windows"]

use gtk::prelude::*;
use gtk::{
    Builder, Button, ComboBoxText, Dialog, EventBox, FileChooser, Inhibit, Label, MenuItem,
    RecentFilter, RecentManager, ResponseType, Scrollbar, Statusbar,
    TextView, Window,
};
use gtk::builders::{RecentChooserMenuBuilder};
use relm::{connect, Relm, Update, Widget};
use relm_derive::Msg;
use std::path::PathBuf;
use std::rc::Rc;

mod media;
use media::io::*;

mod sessions;
use sessions::session::Session;

mod text;
use text::viewer::{ParagraphViewer, ViewerWidgets};

mod ui;
use ui::common::*;

pub struct Model {
    current_session: Option<Session>,
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
    AudioSkip(usize),

    GoTo,
    LoadFile,
    LoadRecent(String),
    OpenPreferences,
    About,
    Quit,
}

/// Keeps track of the Graphical User Interface
/// elements in the application.
///
/// Widgets serve as both a means of viewing, like Labels and Viewers,
/// as well as hooks that spawn User Events, such as Buttons.
pub struct Widgets {
    window: Window,

    // Menu Widgets
    goto_menu_item: MenuItem,
    preferences_menu_item: MenuItem,

    // Custom Widgets
    paragraph_viewer: ParagraphViewer,
    media_controller: Media,
    preference_widgets: PreferenceWidgets,
}

/// Abstracts the whole application, merging
/// the Model and references to the View and Controller
/// widgets.
struct Win {
    model: Model,
    widgets: Widgets,
}

fn load_audio_file(model: &Model, widgets: &mut Widgets) {
    let current_session = model
        .current_session
        .as_ref()
        .expect("A session must exist if Next messages can be processed.");

    let audio_file_location = current_session.project_directory().join(format!(
        "part{}.wav",
        widgets.paragraph_viewer.paragraph_num()
    ));

    widgets.media_controller.load(audio_file_location);
}

fn load_text_file(file_location: PathBuf, model: &mut Model, widgets: &mut Widgets) {
    if let Some(session) = &mut model.current_session {
        session.set_paragraph_num(widgets.paragraph_viewer.paragraph_num());
        session.save();
    }

    let session =
        Session::load(file_location.clone()).unwrap_or_else(|| Session::new(file_location.clone()));

    widgets.paragraph_viewer.load_paragraphs(file_location);
    widgets
        .paragraph_viewer
        .show_paragraph_at(session.paragraph_num());

    widgets.goto_menu_item.set_sensitive(true);
    widgets.preferences_menu_item.set_sensitive(true);

    model.current_session = Some(session);
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
        Model {
            current_session: None,
        }
    }

    // This is where all User Events are parsed, influencing how
    // the Model and View changes.
    fn update(&mut self, event: Msg) {
        match event {
            Msg::Next => {
                self.widgets.paragraph_viewer.show_next_paragraph();

                load_audio_file(&self.model, &mut self.widgets);
            }
            Msg::Previous => {
                self.widgets.paragraph_viewer.show_previous_paragraph();

                load_audio_file(&self.model, &mut self.widgets);
            }
            Msg::Play => {
                let output_device = self
                    .model
                    .current_session
                    .as_ref()
                    .expect("Session should exist on playback.")
                    .audio_output()
                    .to_device();

                self.widgets.media_controller.play(output_device);
            }
            Msg::Stop => self.widgets.media_controller.stop(),
            Msg::Record => self.widgets.media_controller.record(
                self.model
                    .current_session
                    .as_ref()
                    .expect("Session should exist on Recording.")
                    .audio_input(),
            ),
            Msg::AudioSkip(pos_secs) => {
                self.widgets.media_controller.pause_at(pos_secs);
            }
            Msg::GoTo => {
                let paragraph_num_choice = go_to(
                    &self.widgets.window,
                    self.widgets.paragraph_viewer.num_paragraphs(),
                );

                if let Some(paragraph_num) = paragraph_num_choice {
                    self.widgets
                        .paragraph_viewer
                        .show_paragraph_at(paragraph_num);

                    load_audio_file(&self.model, &mut self.widgets);
                }
            }
            Msg::LoadRecent(file_uri) => {
                let file_location = PathBuf::from(file_uri);
                load_text_file(file_location, &mut self.model, &mut self.widgets);
                load_audio_file(&self.model, &mut self.widgets);
            }
            Msg::LoadFile => {
                let text_file_location = open(&self.widgets.window);
                if let Some(file_location) = text_file_location {
                    load_text_file(file_location, &mut self.model, &mut self.widgets);
                    load_audio_file(&self.model, &mut self.widgets);
                }
            }
            Msg::OpenPreferences => {
                if let Some(session) = &mut self.model.current_session {
                    preferences(&self.widgets.preference_widgets, session);
                    load_audio_file(&self.model, &mut self.widgets);
                }
            }
            Msg::About => about(&self.widgets.window),
            Msg::Quit => {
                if let Some(session) = &mut self.model.current_session {
                    session.set_paragraph_num(self.widgets.paragraph_viewer.paragraph_num());
                    session.save();
                }

                gtk::main_quit()
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

        // Show icon when the program executes.
        let icon_location = PathBuf::new()
            .join("resources")
            .join("images")
            .join("icon.png");

        window
            .set_icon_from_file(icon_location)
            .expect("Could not load icon for application.");

        // Text Widgets
        let text_progress_counter: Label = builder.object("chunk_position_lbl").unwrap();
        let paragraph_view: TextView = builder.object("chunk_view_txtviewer").unwrap();
        let chunk_progress_eventbox: EventBox = builder.object("chunk_progress_eventbox").unwrap();

        connect!(
            relm,
            chunk_progress_eventbox,
            connect_button_press_event(_, _),
            return (Some(Msg::GoTo), Inhibit(false))
        );

        // Media IO Items
        let prev_button: Button = builder.object("prev_chunk_btn").unwrap();
        let next_button: Button = builder.object("next_chunk_btn").unwrap();
        let stop_button: Button = builder.object("stop_btn").unwrap();
        let record_button: Button = builder.object("record_btn").unwrap();
        let play_button: Button = builder.object("play/pause_btn").unwrap();

        let audio_progress_label: Label = builder.object("audio_progress_lbl").unwrap();
        let progress_bar: Scrollbar = builder.object("progress_bar").unwrap();
        let status_bar: Statusbar = builder.object("playback_status_bar").unwrap();

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

        connect!(
            relm,
            progress_bar,
            connect_button_release_event(bar, _),
            return (Some(Msg::AudioSkip(bar.value() as usize)), Inhibit(false))
        );

        // Main Menu Items
        let open_menu_item: MenuItem = builder.object("open_menu").unwrap();
        let open_recent_menu_item: MenuItem = builder.object("open_recent_menu").unwrap();
        let goto_menu_item: MenuItem = builder.object("goto_menu").unwrap();
        let preferences_menu_item: MenuItem = builder.object("preferences_menu").unwrap();
        let about_menu_item: MenuItem = builder.object("about_menu").unwrap();
        let quit_menu_item: MenuItem = builder.object("close_menu").unwrap();

        connect!(relm, open_menu_item, connect_activate(_), Msg::LoadFile);
        connect!(relm, goto_menu_item, connect_activate(_), Msg::GoTo);
        connect!(
            relm,
            preferences_menu_item,
            connect_activate(_),
            Msg::OpenPreferences
        );
        connect!(relm, about_menu_item, connect_activate(_), Msg::About);
        connect!(quit_menu_item, connect_activate(_), relm, Msg::Quit);
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        let recent_manager: RecentManager =
            RecentManager::default().expect("Could not load file recents manager.");

        let text_file_filter = RecentFilter::new();
        text_file_filter.set_name("UTF-8 Text Files");
        text_file_filter.add_pattern("*.txt");

        let recent_manager_menu = RecentChooserMenuBuilder::new()
            .recent_manager(&recent_manager)
            .filter(&text_file_filter)
            .local_only(true)
            .show_not_found(false)
            .build();

        connect!(
            relm,
            recent_manager_menu,
            connect_item_activated(menubar),
            Msg::LoadRecent(
                menubar
                    .current_item()
                    .expect("Could not retrieve selected menu in Recent Manager")
                    .uri_display()
                    .expect("Could not retrieve URI of recently selected item")
                    .to_string()
            )
        );

        open_recent_menu_item.set_submenu(Some(&recent_manager_menu));

        // Custom Widgets
        let next_button: Rc<Button> = Rc::new(next_button);
        let prev_button: Rc<Button> = Rc::new(prev_button);

        // Paragraph Viewer setup
        let viewer_widgets = ViewerWidgets {
            paragraph_view,
            next_button: next_button.clone(),
            prev_button: prev_button.clone(),
            progress_counter: text_progress_counter,
        };

        let paragraph_viewer = ParagraphViewer::new(viewer_widgets);

        // Media Controller setup
        let media_widgets = MediaWidgets {
            open_menu_item,

            play_button,
            stop_button,
            record_button,

            prev_button,
            next_button,

            progress_bar,
            time_progress_label: audio_progress_label,
            status_bar,
        };

        let media_controller = Media::new(media_widgets);

        // Preference Widgets Setup
        let preferences_dialog: Dialog = builder.object("preferences_dialog").unwrap();
        preferences_dialog
            .add_buttons(&[("Ok", ResponseType::Ok), ("Cancel", ResponseType::Cancel)]);

        preferences_dialog.set_default_response(ResponseType::Ok);
        // General - Preferences
        let project_file_chooser: FileChooser = builder.object("project_file_chooser").unwrap();

        // Audio - Preferences
        let input_device_cbox: ComboBoxText = builder.object("input_device_cbox").unwrap();
        let input_sample_rate_cbox: ComboBoxText =
            builder.object("input_sample_rate_cbox").unwrap();
        let input_channels_cbox: ComboBoxText = builder.object("input_channels_cbox").unwrap();

        let output_device_cbox: ComboBoxText = builder.object("output_device_cbox").unwrap();

        let preference_widgets = PreferenceWidgets {
            dialog: preferences_dialog,

            project_location_chooser: project_file_chooser,

            input_device_name_chooser: input_device_cbox,
            input_device_sample_rate_chooser: input_sample_rate_cbox,
            input_device_channels_chooser: input_channels_cbox,

            output_device_name_chooser: output_device_cbox,
        };

        Win {
            model,
            widgets: Widgets {
                window,

                goto_menu_item,
                preferences_menu_item,

                paragraph_viewer,
                media_controller,
                preference_widgets,
            },
        }
    }
}

/// Spawns the application with a Graphical User Interface.
fn main() {
    Win::run(()).unwrap();
}
