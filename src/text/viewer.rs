use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use fltk::button::Button;
use fltk::prelude::{DisplayExt, WidgetExt};
use fltk::text::TextDisplay;

use crate::ui::app::ViewerWidgets;

struct Counter {
    progress_label: Button,

    current_pos: usize,
    total_elements: usize,
}

impl Counter {
    pub fn new(progress_label: Button) -> Self {
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
            .set_label(format!("{}/{}", self.current_pos, self.total_elements).as_str());
    }

    pub fn at_beginning(&self) -> bool {
        self.current_pos == 0 || self.current_pos == 1
    }

    pub fn at_end(&self) -> bool {
        self.current_pos == self.total_elements
    }
}

pub struct ParagraphViewer {
    paragraphs: Vec<String>,
    paragraph_num: usize,

    paragraph_view: TextDisplay,
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

    pub fn toggle_nav_buttons(&mut self) {
        if self.progress_counter.at_beginning() {
            self.prev_button.deactivate();
        } else {
            self.prev_button.activate();
        }

        if self.progress_counter.at_end() {
            self.next_button.deactivate();
        } else {
            self.next_button.activate();
        }
    }

    pub fn load_paragraphs(&mut self, text_file_path: PathBuf, delimiters: &str, amount: usize) {
        let mut text_file = File::open(text_file_path).expect("Could not load file.");
        let mut whole_text_content = String::new();
        text_file
            .read_to_string(&mut whole_text_content)
            .expect("Could not read text file.");

        let delimiter_tokens = delimiters.chars().collect::<Vec<char>>();
        let split_paragraphs: Vec<&str> = whole_text_content
            .split_inclusive(&*delimiter_tokens)
            .collect();

        self.paragraphs = split_paragraphs
            .chunks(amount)
            .map(|sentences| sentences.concat())
            .collect();

        self.progress_counter.set_current(0);
        self.progress_counter.set_total(self.paragraphs.len());
        self.progress_counter.update();
    }

    /// Changes currently loaded text to be split by the provided
    /// delimiters.
    pub fn reload_text_with(&mut self, delimiters: &str, amount: usize) {
        let existing_text = self.paragraphs.join("");

        let delimiter_tokens = delimiters.chars().collect::<Vec<char>>();
        let new_splitted_text: Vec<&str> =
            existing_text.split_inclusive(&*delimiter_tokens).collect();

        let new_chunked_text: Vec<String> = new_splitted_text
            .chunks(amount)
            .map(|line| line.concat())
            .collect();

        if new_chunked_text == self.paragraphs {
            return;
        }

        self.paragraphs = new_chunked_text;

        self.progress_counter.set_current(0);
        self.progress_counter.set_total(self.paragraphs.len());
        self.progress_counter.update();
        self.show_paragraph_at(0);
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
    use crate::ui::app::MainApplication;

    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    const DELIMITERS: &str = ".?!";
    const GATHERING_AMOUNT: usize = 4;

    const FIRST_PARAGRAPH: &str = "This is the first paragraph. It will eventually contain four sentences. I'm serious! Okay, here is the last sentence.";
    const SECOND_PARAGRAPH: &str = "This is the second paragraph. It will also contain four sentences. This paragraph is similar to the first one. It really is?";

    const MANY_PARAGRAPHS_LEN: usize = 2;

    fn get_paragraph_viewer() -> ParagraphViewer {
        let main_application = MainApplication::new();

        main_application.paragraph_viewer
    }

    fn get_text_from_viewer(parapgraph_view: &TextDisplay) -> String {
        let text_buffer = parapgraph_view
            .buffer()
            .expect("Could not fetch buffer from paragraph view.");

        text_buffer.text()
    }

    fn get_file_one_paragraph() -> NamedTempFile {
        let mut paragraph_file = NamedTempFile::new().expect("Could not create temporary file.");
        paragraph_file
            .reopen()
            .expect("Could not open temporary file for writing.");
        paragraph_file
            .write_all(FIRST_PARAGRAPH.as_bytes())
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
            .write_all(paragraphs.as_bytes())
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
        paragraph_viewer.load_paragraphs(
            get_file_many_paragraphs().path().to_path_buf(),
            DELIMITERS,
            GATHERING_AMOUNT,
        );
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
        paragraph_viewer.load_paragraphs(
            get_file_many_paragraphs().path().to_path_buf(),
            DELIMITERS,
            GATHERING_AMOUNT,
        );
        assert_eq!(MANY_PARAGRAPHS_LEN, paragraph_viewer.num_paragraphs());

        let goto_paragraph_num = 1;
        paragraph_viewer.show_paragraph_at(goto_paragraph_num);

        let actual_paragraph_num = paragraph_viewer.paragraph_num();
        assert_eq!(goto_paragraph_num, actual_paragraph_num);
        assert_eq!(
            &get_text_from_viewer(&paragraph_viewer.paragraph_view),
            SECOND_PARAGRAPH
        );
        assert!(!paragraph_viewer.next_button.active());
        assert!(paragraph_viewer.prev_button.active());
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
        paragraph_viewer.load_paragraphs(
            get_file_one_paragraph().path().to_path_buf(),
            DELIMITERS,
            GATHERING_AMOUNT,
        );
        assert_eq!(1, paragraph_viewer.num_paragraphs());

        paragraph_viewer.show_paragraph_at(0);
        assert!(!paragraph_viewer.next_button.active());
        assert!(!paragraph_viewer.prev_button.active());

        paragraph_viewer.show_next_paragraph();
        let expected_paragraph_num = 0;
        let actual_paragraph_num = paragraph_viewer.paragraph_num;

        assert_eq!(actual_paragraph_num, expected_paragraph_num);
    }

    #[test]
    fn next_paragraph_exists() {
        let mut paragraph_viewer = get_paragraph_viewer();
        paragraph_viewer.load_paragraphs(
            get_file_many_paragraphs().path().to_path_buf(),
            DELIMITERS,
            GATHERING_AMOUNT,
        );
        assert_eq!(MANY_PARAGRAPHS_LEN, paragraph_viewer.num_paragraphs());

        paragraph_viewer.show_paragraph_at(0);
        assert!(paragraph_viewer.next_button.active());
        assert!(!paragraph_viewer.prev_button.active());

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
        paragraph_viewer.load_paragraphs(
            get_file_one_paragraph().path().to_path_buf(),
            DELIMITERS,
            GATHERING_AMOUNT,
        );
        assert_eq!(1, paragraph_viewer.num_paragraphs());

        paragraph_viewer.show_paragraph_at(0);
        assert!(!paragraph_viewer.next_button.active());
        assert!(!paragraph_viewer.prev_button.active());

        paragraph_viewer.show_previous_paragraph();
        let expected_paragraph_num = 0;
        let actual_paragraph_num = paragraph_viewer.paragraph_num;

        assert_eq!(actual_paragraph_num, expected_paragraph_num);
    }

    #[test]
    fn previous_paragraph_exists() {
        let mut paragraph_viewer = get_paragraph_viewer();
        paragraph_viewer.load_paragraphs(
            get_file_many_paragraphs().path().to_path_buf(),
            DELIMITERS,
            GATHERING_AMOUNT,
        );
        assert_eq!(MANY_PARAGRAPHS_LEN, paragraph_viewer.num_paragraphs());

        paragraph_viewer.show_paragraph_at(MANY_PARAGRAPHS_LEN - 1);
        assert!(!paragraph_viewer.next_button.active());
        assert!(paragraph_viewer.prev_button.active());

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
        paragraph_viewer.load_paragraphs(
            get_file_one_paragraph().path().to_path_buf(),
            DELIMITERS,
            GATHERING_AMOUNT,
        );
        assert_eq!(1, paragraph_viewer.num_paragraphs());

        paragraph_viewer.show_paragraph_at(0);

        assert_eq!(
            &get_text_from_viewer(&paragraph_viewer.paragraph_view),
            FIRST_PARAGRAPH
        );
    }
}
