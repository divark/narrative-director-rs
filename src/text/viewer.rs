use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use gtk::prelude::*;
use gtk::{Button, Label, TextView};

#[derive(Clone)]
struct Counter {
    progress_label: Label,

    current_pos: usize,
    total_elements: usize,
}

impl Counter {
    pub fn new(progress_label: Label) -> Self {
        Counter {
            progress_label,
            current_pos: 0,
            total_elements: 0,
        }
    }

    pub fn set_current(&mut self, pos: usize) {
        self.current_pos = pos + 1;
    }

    pub fn set_total(&mut self, total: usize) {
        self.total_elements = total;
    }

    pub fn update(&mut self) {
        if self.current_pos > self.total_elements {
            return;
        }

        self.progress_label
            .set_text(format!("{}/{}", self.current_pos, self.total_elements).as_str());
    }

    pub fn at_beginning(&self) -> bool {
        self.current_pos == 0 || self.current_pos == 1
    }

    pub fn at_end(&self) -> bool {
        self.current_pos == self.total_elements
    }
}

pub struct ViewerWidgets {
    pub paragraph_view: TextView,
    pub next_button: Button,
    pub prev_button: Button,

    pub progress_counter: Label,
}

#[derive(Clone)]
pub struct ParagraphViewer {
    paragraphs: Vec<String>,
    paragraph_num: usize,

    paragraph_view: TextView,
    next_button: Button,
    prev_button: Button,
    progress_counter: Counter,
}

impl ParagraphViewer {
    pub fn new(widgets: ViewerWidgets) -> Self {
        ParagraphViewer {
            paragraphs: Vec::new(),
            paragraph_num: 0,

            paragraph_view: widgets.paragraph_view,
            next_button: widgets.next_button,
            prev_button: widgets.prev_button,
            progress_counter: Counter::new(widgets.progress_counter),
        }
    }

    fn toggle_nav_buttons(&self) {
        if self.progress_counter.at_beginning() {
            self.prev_button.set_sensitive(false);
        } else {
            self.prev_button.set_sensitive(true);
        }

        if self.progress_counter.at_end() {
            self.next_button.set_sensitive(false);
        } else {
            self.next_button.set_sensitive(true);
        }
    }

    pub fn load_paragraphs(&mut self, text_file_path: PathBuf) {
        let mut text_file = File::open(text_file_path).expect("Could not load file.");
        let mut whole_text_content = String::new();
        text_file
            .read_to_string(&mut whole_text_content)
            .expect("Could not read text file.");

        let split_paragraphs: Vec<&str> = whole_text_content
            .split_inclusive(|character| character == '.' || character == '?' || character == '!')
            .collect();

        self.paragraphs = split_paragraphs
            .chunks(4)
            .map(|sentences| sentences.concat())
            .collect();

        self.progress_counter.set_current(0);
        self.progress_counter.set_total(self.paragraphs.len());
        self.progress_counter.update();
    }

    pub fn show_next_paragraph(&mut self) {
        self.paragraph_num += 1;

        if let Some(paragraph) = self.paragraphs.get(self.paragraph_num) {
            self.paragraph_view
                .buffer()
                .expect("Could not retrieve TextView")
                .set_text(paragraph.as_str());

            self.progress_counter.set_current(self.paragraph_num);
            self.progress_counter.update();

            self.toggle_nav_buttons();
        } else {
            self.paragraph_num -= 1;
        }
    }

    pub fn show_previous_paragraph(&mut self) {
        if self.paragraph_num == 0 {
            return;
        }

        self.paragraph_num -= 1;
        if let Some(paragraph) = self.paragraphs.get(self.paragraph_num) {
            self.paragraph_view
                .buffer()
                .expect("Could not retrieve TextView")
                .set_text(paragraph.as_str());

            self.progress_counter.set_current(self.paragraph_num);
            self.progress_counter.update();

            self.toggle_nav_buttons();
        } else {
            self.paragraph_num += 1;
        }
    }

    pub fn show_paragraph_at(&mut self, paragraph_num: usize) {
        let old_paragraph_num = self.paragraph_num;

        self.paragraph_num = paragraph_num;
        if let Some(paragraph) = self.paragraphs.get(self.paragraph_num) {
            self.paragraph_view
                .buffer()
                .expect("Could not retrieve TextView")
                .set_text(paragraph.as_str());

            self.progress_counter.set_current(self.paragraph_num);
            self.progress_counter.update();

            self.toggle_nav_buttons();
        } else {
            self.paragraph_num = old_paragraph_num;
        }
    }

    pub fn num_paragraphs(&self) -> usize {
        self.paragraphs.len()
    }
}
