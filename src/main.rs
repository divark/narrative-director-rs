use gtk::prelude::*;
use gtk::{Builder, Button, Inhibit, Label, TextView, Window};
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
    Quit,
}

#[derive(Clone)]
struct Widgets {
    chunk_progress_label: Label,
    chunk_viewer: TextView,
    previous_chunk_button: Button,
    next_chunk_button: Button,
    window: Window,
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
        let mut chunk_retriever = EnglishParagraphRetriever::new(
            File::open("/home/divark/Documents/Repositories/narrative-director-rs/src/resources/sample_texts/2600.txt").expect("Unable to open file."),
        );
        let chunk_total = chunk_retriever.load_paragraphs();

        Model {
            chunk_retriever,
            chunk_number: 0,
            chunk_total,
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
            }
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
        // TODO: Hook up Open menu item to load from some file. Change TextGrabber to take in File
        if let Some(first_paragraph) = model.chunk_retriever.get_chunk(0) {
            text_viewer.get_buffer().unwrap().set_text(first_paragraph);

            chunk_progress_label.set_text(format!("{}/{}", 1, model.chunk_total).as_str());
        }

        let prev_button: Button = builder.get_object("prev_chunk_btn").unwrap();
        let next_button: Button = builder.get_object("next_chunk_btn").unwrap();

        connect!(relm, prev_button, connect_clicked(_), Msg::Previous);
        connect!(relm, next_button, connect_clicked(_), Msg::Next);
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        Win {
            model,
            widgets: Widgets {
                chunk_progress_label,
                chunk_viewer: text_viewer,
                previous_chunk_button: prev_button,
                next_chunk_button: next_button,
                window,
            },
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
