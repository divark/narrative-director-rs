/*
* This is needed to ensure that the application
* does not start with a console in the background
* when the application runs on Windows.
 */
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gtk::prelude::*;
use gtk::{Builder, Button, Inhibit, Label, MenuItem, Scrollbar, TextView, Window};
use relm::{connect, Relm, Update, Widget, WidgetTest};
use relm_derive::Msg;
use std::path::PathBuf;

mod media;

mod session;

mod text;
use text::viewer::{ParagraphViewer, ViewerWidgets};

mod ui;
use ui::common::*;

pub struct Model {}

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

    GoTo,
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
    window: Window,

    // Audio-related Widgets
    audio_progress_label: Label,
    progress_bar: Scrollbar,
    stop_button: Button,
    record_button: Button,
    play_button: Button,

    // Menu Widgets
    open_menu_item: MenuItem,
    goto_menu_item: MenuItem,
    preferences_menu_item: MenuItem,
    about_menu_item: MenuItem,
    quit_menu_item: MenuItem,

    // Custom Widgets
    paragraph_viewer: ParagraphViewer,
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
        Model {}
    }

    // fn subscriptions(&mut self, relm: &Relm<Self>) {
    //     interval(relm.stream(), 100, || ProgressTick);
    // }

    // This is where all User Events are parsed, influencing how
    // the Model and View changes.
    fn update(&mut self, event: Msg) {
        match event {
            Msg::Next => self.widgets.paragraph_viewer.show_next_paragraph(),
            Msg::Previous => self.widgets.paragraph_viewer.show_previous_paragraph(),
            Msg::Play => todo!(),
            Msg::Stop => todo!(),
            Msg::Record => todo!(),
            Msg::AudioSkip => todo!(),
            Msg::GoTo => {
                let paragraph_num_choice = go_to(
                    &self.widgets.window,
                    self.widgets.paragraph_viewer.num_paragraphs(),
                );
                if let Some(paragraph_num) = paragraph_num_choice {
                    self.widgets
                        .paragraph_viewer
                        .show_paragraph_at(paragraph_num);
                }
            }
            Msg::LoadFile => {
                let text_file_location = open(&self.widgets.window);
                if let Some(file_location) = text_file_location {
                    self.widgets.paragraph_viewer.load_paragraphs(file_location);
                    self.widgets.paragraph_viewer.show_paragraph_at(0);

                    self.widgets.goto_menu_item.set_sensitive(true);
                }
            }
            Msg::OpenPreferences => todo!(),
            Msg::About => about(&self.widgets.window),
            Msg::Quit => gtk::main_quit(),
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

        // Media IO Items
        let prev_button: Button = builder.object("prev_chunk_btn").unwrap();
        let next_button: Button = builder.object("next_chunk_btn").unwrap();
        let stop_button: Button = builder.object("stop_btn").unwrap();
        let record_button: Button = builder.object("record_btn").unwrap();
        let play_button: Button = builder.object("play/pause_btn").unwrap();
        let audio_progress_label: Label = builder.object("audio_progress_lbl").unwrap();
        let progress_bar: Scrollbar = builder.object("progress_bar").unwrap();

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

        // Main Menu Items
        let open_menu_item: MenuItem = builder.object("open_menu").unwrap();
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

        // Custom Widgets
        let viewer_widgets = ViewerWidgets {
            paragraph_view,
            next_button,
            prev_button,
            progress_counter: text_progress_counter,
        };

        let paragraph_viewer = ParagraphViewer::new(viewer_widgets);
        Win {
            model,
            widgets: Widgets {
                window,

                stop_button,
                record_button,
                play_button,

                audio_progress_label,
                progress_bar,

                open_menu_item,
                goto_menu_item,
                preferences_menu_item,
                about_menu_item,
                quit_menu_item,

                paragraph_viewer,
            },
        }
    }
}

impl WidgetTest for Win {
    type Streams = ();

    fn get_streams(&self) -> Self::Streams {}

    type Widgets = Widgets;

    fn get_widgets(&self) -> Self::Widgets {
        self.widgets.clone()
    }
}

/// Spawns the application with a Graphical User Interface.
fn main() {
    Win::run(()).unwrap();
}
