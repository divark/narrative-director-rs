use gtk::prelude::*;
use gtk::{Builder, Button, Inhibit, Label, TextView, Window, MenuItem, FileChooser, FileChooserDialogBuilder, FileChooserAction, FileChooserDialog, ResponseType};
use relm::{connect, Relm, Update, Widget};
use relm_derive::Msg;

use std::fs::File;
use text_grabber::{EnglishParagraphRetriever, TextGrabber};

struct Model {
    chunk_retriever: EnglishParagraphRetriever,
    chunk_number: u32,
    chunk_total: u32,
}

#[derive(Msg)]
enum Msg {
    Next,
    Previous,
    LoadFile,
    Quit,
}

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
    quit_menu_item: MenuItem,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Model {
        let mut chunk_retriever = EnglishParagraphRetriever::new();

        Model {
            chunk_retriever,
            chunk_number: 0,
            chunk_total: 0,
        }
    }

    fn update(&mut self, event: Msg) {
        let progress_label = &self.widgets.chunk_progress_label;
        let text_viewer = &self.widgets.chunk_viewer;

        match event {
            Msg::Next => {
                if let Some(paragraph) = self.model.chunk_retriever.get_next_chunk() {
                    self.model.chunk_number += 1;
                    progress_label.set_text(
                        format!(
                            "{}/{}",
                            &self.model.chunk_number + 1,
                            &self.model.chunk_total
                        )
                        .as_str(),
                    );
                    text_viewer
                        .get_buffer()
                        .expect("Couldn't get text viewer")
                        .set_text(paragraph.as_str());
                }
            }
            Msg::Previous => {
                if let Some(paragraph) = self.model.chunk_retriever.get_prev_chunk() {
                    self.model.chunk_number -= 1;
                    progress_label.set_text(
                        format!(
                            "{}/{}",
                            &self.model.chunk_number + 1,
                            &self.model.chunk_total
                        )
                        .as_str(),
                    );
                    text_viewer
                        .get_buffer()
                        .expect("Couldn't get text viewer")
                        .set_text(paragraph.as_str());
                }
            },
            Msg::LoadFile => {
                let file_chooser_dialog = FileChooserDialogBuilder::new()
                    .action(FileChooserAction::Open)
                    .title("Open File")
                    .build();

                file_chooser_dialog.run();

                let loaded_file_result = File::open(file_chooser_dialog.get_preview_filename().unwrap().as_path());
                if let Ok(loaded_file) = loaded_file_result {
                    self.model.chunk_retriever = EnglishParagraphRetriever::new();
                    let num_paragraphs = self.model.chunk_retriever.load_chunks(loaded_file);

                    if num_paragraphs == 0 {
                        return;
                    }

                    self.model.chunk_total = num_paragraphs;
                    progress_label.set_text(format!("{}/{}", 1, num_paragraphs).as_str());

                    text_viewer.get_buffer().expect("Couldn't get text viewer in load").set_text(self.model.chunk_retriever.get_chunk(0).unwrap());
                }
            },
            Msg::Quit => gtk::main_quit(),
        }
    }
}

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
                quit_menu_item
            },
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
