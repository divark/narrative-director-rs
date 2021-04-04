mod text_grabber;

use gtk::prelude::*;
use gtk::{
    Builder, Button, Dialog, DialogFlags, Inhibit, Label, MenuItem, ResponseType, SpinButton,
    TextView, Window,
};
use relm::{connect, Relm, Update, Widget};
use relm_derive::Msg;

use std::fs::File;
use text_grabber::{EnglishParagraphRetriever, TextGrabber};

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
}

/// Represents User Events that could take place
/// in the GUI.
#[derive(Msg)]
enum Msg {
    Next,
    Previous,
    JumpTo,
    LoadFile,
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
    window: Window,
    // Menu Widgets
    open_menu_item: MenuItem,
    goto_menu_item: MenuItem,
    quit_menu_item: MenuItem,
}


struct ChunkViewingUi<'a> {
    progress_label: &'a Label,
    chunk_viewer: &'a TextView,
}

// Populates the UI with the specified chunk.
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
        }
    }

    // This is where all User Events are parsed, influencing how
    // the Model and View changes.
    fn update(&mut self, event: Msg) {
        let progress_label = &self.widgets.chunk_progress_label;
        let text_viewer = &self.widgets.chunk_viewer;

        let paragraph_ui = ChunkViewingUi {
            progress_label,
            chunk_viewer: text_viewer,
        };

        match event {
            Msg::Next => {
                if let Ok(_) = show_chunk(
                    self.model.chunk_number + 1,
                    &self.model.chunk_retriever,
                    paragraph_ui,
                ) {
                    self.model.chunk_number += 1;
                }
            }
            Msg::Previous => {
                if self.model.chunk_number <= 0 {
                    return;
                }

                if let Ok(_) = show_chunk(
                    self.model.chunk_number - 1,
                    &self.model.chunk_retriever,
                    paragraph_ui,
                ) {
                    self.model.chunk_number -= 1;
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

                match goto_dialog.run() {
                    ResponseType::Ok => {
                        let goto_paragraph_num = (goto_spin_button.get_value_as_int() - 1) as u32;
                        if let Ok(_) = show_chunk(
                            goto_paragraph_num,
                            &self.model.chunk_retriever,
                            paragraph_ui,
                        ) {
                            self.model.chunk_number = goto_paragraph_num;
                        }

                        goto_dialog.close();
                    }
                    _ => goto_dialog.close(),
                }
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

                match file_chooser.run() {
                    ResponseType::Ok => {
                        let filename = file_chooser.get_filename().expect("Couldn't get filename");
                        let file = File::open(&filename).expect("Couldn't open file");

                        self.model.chunk_retriever = EnglishParagraphRetriever::new();
                        let num_paragraphs = self.model.chunk_retriever.load_chunks(file);

                        if num_paragraphs == 0 {
                            return;
                        }

                        self.model.chunk_number = 0;
                        self.model.chunk_total = num_paragraphs;
                        show_chunk(0, &self.model.chunk_retriever, paragraph_ui).unwrap();

                        file_chooser.close();
                    }
                    _ => file_chooser.close(),
                }
            }
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
        let glade_src = include_str!("resources/ui/main-window.glade");
        let builder = Builder::from_string(glade_src);

        let window: Window = builder.get_object("window").unwrap();
        window.show_all();

        let chunk_progress_label: Label = builder.get_object("chunk_position_lbl").unwrap();
        let text_viewer: TextView = builder.get_object("chunk_view_txtviewer").unwrap();
        let prev_button: Button = builder.get_object("prev_chunk_btn").unwrap();
        let next_button: Button = builder.get_object("next_chunk_btn").unwrap();

        let open_menu_item: MenuItem = builder.get_object("open_menu").unwrap();
        let goto_menu_item: MenuItem = builder.get_object("goto_menu").unwrap();
        let quit_menu_item: MenuItem = builder.get_object("close_menu").unwrap();

        connect!(relm, prev_button, connect_clicked(_), Msg::Previous);
        connect!(relm, next_button, connect_clicked(_), Msg::Next);
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        connect!(relm, open_menu_item, connect_activate(_), Msg::LoadFile);
        connect!(relm, goto_menu_item, connect_activate(_), Msg::JumpTo);
        connect!(quit_menu_item, connect_activate(_), relm, Msg::Quit);

        Win {
            model,
            widgets: Widgets {
                chunk_progress_label,
                chunk_viewer: text_viewer,
                previous_chunk_button: prev_button,
                next_chunk_button: next_button,
                window,
                open_menu_item,
                goto_menu_item,
                quit_menu_item,
            },
        }
    }
}

/// Spawns the application with a Graphical User Interface.
fn main() {
    Win::run(()).unwrap();
}
