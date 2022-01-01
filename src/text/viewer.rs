use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Button, Label, TextView};

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
    pub next_button: Rc<Button>,
    pub prev_button: Rc<Button>,

    pub progress_counter: Label,
}

pub struct ParagraphViewer {
    paragraphs: Vec<String>,
    paragraph_num: usize,

    paragraph_view: TextView,
    next_button: Rc<Button>,
    prev_button: Rc<Button>,
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

    pub fn paragraph_num(&self) -> usize {
        self.paragraph_num
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk::Builder;
    use std::io::Write;
    use tempfile::NamedTempFile;

    const FIRST_PARAGRAPH: &str = "This is the first paragraph. It will eventually contain four sentences. I'm serious! Okay, here is the last sentence.";
    const SECOND_PARAGRAPH: &str = "This is the second paragraph. It will also contain four sentences. This paragraph is similar to the first one. It really is?";

    const MANY_PARAGRAPHS_LEN: usize = 2;

    fn get_viewer_widgets() -> ViewerWidgets {
        gtk::init().expect("Unable to initialize gtk.");

        let glade_src = include_str!("../ui/main-window.glade");
        let builder = Builder::from_string(glade_src);

        let paragraph_view: TextView = builder.object("chunk_view_txtviewer").unwrap();

        let prev_button: Rc<Button> = Rc::new(builder.object("prev_chunk_btn").unwrap());
        let next_button: Rc<Button> = Rc::new(builder.object("next_chunk_btn").unwrap());

        let text_progress_counter: Label = builder.object("chunk_position_lbl").unwrap();

        ViewerWidgets {
            paragraph_view,
            next_button,
            prev_button,
            progress_counter: text_progress_counter,
        }
    }

    fn get_paragraph_viewer() -> ParagraphViewer {
        let viewer_widgets = get_viewer_widgets();

        ParagraphViewer::new(viewer_widgets)
    }

    fn get_text_from_viewer(parapgraph_view: &TextView) -> String {
        let text_buffer = parapgraph_view
            .buffer()
            .expect("Could not fetch buffer from paragraph view.");

        let start_iter = text_buffer.start_iter();
        let end_iter = text_buffer.end_iter();

        let text = text_buffer
            .text(&start_iter, &end_iter, false)
            .expect("Could not get text from paragraph view.");
        text.to_string()
    }

    fn get_file_one_paragraph() -> NamedTempFile {
        let mut paragraph_file = NamedTempFile::new().expect("Could not create temporary file.");
        paragraph_file
            .reopen()
            .expect("Could not open temporary file for writing.");
        paragraph_file
            .write(FIRST_PARAGRAPH.as_bytes())
            .expect("Could not write to temporary file.");

        paragraph_file
    }

    fn get_file_many_paragraphs() -> NamedTempFile {
        let mut paragraphs = String::new();
        paragraphs += FIRST_PARAGRAPH;
        paragraphs += SECOND_PARAGRAPH;

        let mut paragraph_file = NamedTempFile::new().expect("Could not create temporary file.");
        paragraph_file
            .reopen()
            .expect("Could not open temporary file for writing.");
        paragraph_file
            .write(paragraphs.as_bytes())
            .expect("Could not write to temporary file.");

        paragraph_file
    }

    #[test]
    fn goto_no_text() {
        let mut paragraph_viewer = get_paragraph_viewer();
        assert_eq!(0, paragraph_viewer.num_paragraphs());

        let goto_paragraph_num = 1;
        paragraph_viewer.show_paragraph_at(goto_paragraph_num);

        let actual_paragraph_num = paragraph_viewer.paragraph_num();
        assert_ne!(goto_paragraph_num, actual_paragraph_num);
        assert_eq!(0, actual_paragraph_num);
    }

    #[test]
    fn goto_exceeds_paragraphs() {
        let mut paragraph_viewer = get_paragraph_viewer();
        paragraph_viewer.load_paragraphs(get_file_many_paragraphs().path().to_path_buf());
        assert_eq!(MANY_PARAGRAPHS_LEN, paragraph_viewer.num_paragraphs());

        let goto_paragraph_num = 3;
        paragraph_viewer.show_paragraph_at(goto_paragraph_num);

        let actual_paragraph_num = paragraph_viewer.paragraph_num();
        assert_ne!(goto_paragraph_num, actual_paragraph_num);
        assert_eq!(0, actual_paragraph_num);
    }

    #[test]
    fn goto_paragraph_exists() {
        let mut paragraph_viewer = get_paragraph_viewer();
        paragraph_viewer.load_paragraphs(get_file_many_paragraphs().path().to_path_buf());
        assert_eq!(MANY_PARAGRAPHS_LEN, paragraph_viewer.num_paragraphs());

        let goto_paragraph_num = 1;
        paragraph_viewer.show_paragraph_at(goto_paragraph_num);

        let actual_paragraph_num = paragraph_viewer.paragraph_num();
        assert_eq!(goto_paragraph_num, actual_paragraph_num);
        assert_eq!(
            &get_text_from_viewer(&paragraph_viewer.paragraph_view),
            SECOND_PARAGRAPH
        );
        assert!(!paragraph_viewer.next_button.is_sensitive());
        assert!(paragraph_viewer.prev_button.is_sensitive());
    }

    #[test]
    fn next_no_text() {
        let mut paragraph_viewer = get_paragraph_viewer();
        assert_eq!(0, paragraph_viewer.num_paragraphs());

        paragraph_viewer.show_next_paragraph();
        let expected_paragraph_num = 0;
        let actual_paragraph_num = paragraph_viewer.paragraph_num;

        assert_eq!(actual_paragraph_num, expected_paragraph_num);
    }

    #[test]
    fn next_exceeds_paragraphs() {
        let mut paragraph_viewer = get_paragraph_viewer();
        paragraph_viewer.load_paragraphs(get_file_one_paragraph().path().to_path_buf());
        assert_eq!(1, paragraph_viewer.num_paragraphs());

        paragraph_viewer.show_paragraph_at(0);
        assert!(!paragraph_viewer.next_button.is_sensitive());
        assert!(!paragraph_viewer.prev_button.is_sensitive());

        paragraph_viewer.show_next_paragraph();
        let expected_paragraph_num = 0;
        let actual_paragraph_num = paragraph_viewer.paragraph_num;

        assert_eq!(actual_paragraph_num, expected_paragraph_num);
    }

    #[test]
    fn next_paragraph_exists() {
        let mut paragraph_viewer = get_paragraph_viewer();
        paragraph_viewer.load_paragraphs(get_file_many_paragraphs().path().to_path_buf());
        assert_eq!(MANY_PARAGRAPHS_LEN, paragraph_viewer.num_paragraphs());

        paragraph_viewer.show_paragraph_at(0);
        assert!(paragraph_viewer.next_button.is_sensitive());
        assert!(!paragraph_viewer.prev_button.is_sensitive());

        paragraph_viewer.show_next_paragraph();
        let expected_paragraph_num = 1;
        let actual_paragraph_num = paragraph_viewer.paragraph_num;

        assert_eq!(actual_paragraph_num, expected_paragraph_num);
        assert_eq!(
            &get_text_from_viewer(&paragraph_viewer.paragraph_view),
            SECOND_PARAGRAPH
        );
    }

    #[test]
    fn previous_no_text() {
        let mut paragraph_viewer = get_paragraph_viewer();
        assert_eq!(0, paragraph_viewer.num_paragraphs());

        paragraph_viewer.show_previous_paragraph();
        let expected_paragraph_num = 0;
        let actual_paragraph_num = paragraph_viewer.paragraph_num;

        assert_eq!(actual_paragraph_num, expected_paragraph_num);
    }

    #[test]
    fn previous_negative_paragraphs() {
        let mut paragraph_viewer = get_paragraph_viewer();
        paragraph_viewer.load_paragraphs(get_file_one_paragraph().path().to_path_buf());
        assert_eq!(1, paragraph_viewer.num_paragraphs());

        paragraph_viewer.show_paragraph_at(0);
        assert!(!paragraph_viewer.next_button.is_sensitive());
        assert!(!paragraph_viewer.prev_button.is_sensitive());

        paragraph_viewer.show_previous_paragraph();
        let expected_paragraph_num = 0;
        let actual_paragraph_num = paragraph_viewer.paragraph_num;

        assert_eq!(actual_paragraph_num, expected_paragraph_num);
    }

    #[test]
    fn previous_paragraph_exists() {
        let mut paragraph_viewer = get_paragraph_viewer();
        paragraph_viewer.load_paragraphs(get_file_many_paragraphs().path().to_path_buf());
        assert_eq!(MANY_PARAGRAPHS_LEN, paragraph_viewer.num_paragraphs());

        paragraph_viewer.show_paragraph_at(MANY_PARAGRAPHS_LEN - 1);
        assert!(!paragraph_viewer.next_button.is_sensitive());
        assert!(paragraph_viewer.prev_button.is_sensitive());

        paragraph_viewer.show_previous_paragraph();
        let expected_paragraph_num = MANY_PARAGRAPHS_LEN - 2;
        let actual_paragraph_num = paragraph_viewer.paragraph_num;

        assert_eq!(actual_paragraph_num, expected_paragraph_num);
        assert_eq!(
            &get_text_from_viewer(&paragraph_viewer.paragraph_view),
            FIRST_PARAGRAPH
        );
    }

    #[test]
    fn shows_paragraph() {
        let mut paragraph_viewer = get_paragraph_viewer();
        paragraph_viewer.load_paragraphs(get_file_one_paragraph().path().to_path_buf());
        assert_eq!(1, paragraph_viewer.num_paragraphs());

        paragraph_viewer.show_paragraph_at(0);

        assert_eq!(
            &get_text_from_viewer(&paragraph_viewer.paragraph_view),
            FIRST_PARAGRAPH
        );
    }
}
