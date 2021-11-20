use std::fs::File;
use std::io::Read;

use gtk::prelude::*;


struct Counter {
    progress_label: gtk::Label,

    current_pos: usize,
    total_elements: usize
}

impl Counter {
    pub fn new(progress_label: gtk::Label) -> Self {
        Counter {
            progress_label,
            current_pos: 0,
            total_elements: 0
        }
    }

    pub fn set_current(&mut self, pos: usize) {
        self.current_pos = pos;
    }

    pub fn set_total(&mut self, total: usize) {
        self.total_elements = total;
    }

    pub fn update(&mut self) {
        if self.current_pos > self.total_elements {
            return;
        }

        self.progress_label.set_text(format!("{}/{}", self.current_pos, self.total_elements).as_str());
    }
}

pub struct ViewerWidgets {
    paragraph_view: gtk::TextView,
    progress_counter: gtk::Label,
}

pub struct ParagraphViewer {
    paragraphs: Vec<String>,
    paragraph_num: usize,

    paragraph_view: gtk::TextView,
    progress_counter: Counter,
}

impl ParagraphViewer {
    pub fn new(widgets: ViewerWidgets) -> Self {
        ParagraphViewer {
            paragraphs: Vec::new(),
            paragraph_num: 0,

            paragraph_view: widgets.paragraph_view,
            progress_counter: Counter::new(widgets.progress_counter)
        }
    }

    pub fn load_paragraphs(&mut self, mut text_file: File) {
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

        self.progress_counter.set_current(self.paragraphs.len());
        self.progress_counter.update();
    }

    pub fn show_next_paragraph(&mut self) {
        self.paragraph_num += 1;

        if let Some(paragraph) = self.paragraphs.get(self.paragraph_num) {
            self.paragraph_view.buffer()
                .set_text(paragraph.as_str());

            self.progress_counter.set_current(self.paragraph_num);
            self.progress_counter.update();
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
            self.paragraph_view.buffer()
                .set_text(paragraph.as_str());

            self.progress_counter.set_current(self.paragraph_num);
            self.progress_counter.update();
        } else {
            self.paragraph_num += 1;
        }
    }

    pub fn show_paragraph_at(&mut self, paragraph_num: usize) {
        let old_paragraph_num = self.paragraph_num;

        self.paragraph_num = paragraph_num;
        if let Some(paragraph) = self.paragraphs.get(self.paragraph_num) {
            self.paragraph_view.buffer()
                .set_text(paragraph.as_str());

            self.progress_counter.set_current(self.paragraph_num);
            self.progress_counter.update();
        } else {
            self.paragraph_num = old_paragraph_num;
        }
    }

    pub fn get_num_paragraphs(&self) -> usize {
        self.paragraphs.len()
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::text::{ParagraphRetriever, TextGrabber};
//     use std::fs::File;
//     use std::io::Write;
//     use std::io::{Seek, SeekFrom};

//     #[test]
//     fn gets_complete_sentence() {
//         let mut sample_file: File = tempfile::tempfile().unwrap();
//         write!(sample_file, "This is a complete sentence.").unwrap();
//         sample_file.seek(SeekFrom::Start(0)).unwrap();

//         let mut paragraph_retriever = ParagraphRetriever::new();
//         assert_eq!(paragraph_retriever.load_chunks(sample_file), 1);

//         let read_result = paragraph_retriever.get_chunk(0);
//         assert!(read_result.is_some());

//         let read_sentence = read_result.unwrap();
//         assert_eq!(*read_sentence, String::from("This is a complete sentence."));
//     }

//     #[test]
//     fn gets_incomplete_sentence() {
//         let mut sample_file: File = tempfile::tempfile().unwrap();
//         write!(
//             sample_file,
//             "This is a complete sentence with no ending punctuation"
//         )
//         .unwrap();
//         sample_file.seek(SeekFrom::Start(0)).unwrap();

//         let mut paragraph_retriever = ParagraphRetriever::new();
//         assert_eq!(paragraph_retriever.load_chunks(sample_file), 1);

//         let read_result = paragraph_retriever.get_chunk(0);
//         assert!(read_result.is_some());

//         let read_sentence = read_result.unwrap();
//         assert_eq!(
//             *read_sentence,
//             String::from("This is a complete sentence with no ending punctuation")
//         );
//     }

//     #[test]
//     fn get_no_sentence_from_empty() {
//         let mut sample_file: File = tempfile::tempfile().unwrap();
//         sample_file.seek(SeekFrom::Start(0)).unwrap();

//         let mut paragraph_retriever = ParagraphRetriever::new();
//         assert_eq!(paragraph_retriever.load_chunks(sample_file), 0);

//         let read_result = paragraph_retriever.get_chunk(0);
//         assert!(read_result.is_none());
//     }

//     #[test]
//     fn get_chunk_returns_only_paragraph() {
//         let mut sample_file: File = tempfile::tempfile().unwrap();
//         let paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
//         write!(sample_file, "{}", paragraph.as_str()).unwrap();
//         sample_file.seek(SeekFrom::Start(0)).unwrap();

//         let mut paragraph_retriever = ParagraphRetriever::new();
//         assert_eq!(paragraph_retriever.load_chunks(sample_file), 1);

//         let read_result = paragraph_retriever.get_chunk(0);
//         assert!(read_result.is_some());

//         let read_paragraph = read_result.unwrap();
//         assert_eq!(*read_paragraph, paragraph);
//     }

//     #[test]
//     fn get_next_chunk_returns_nothing_one_paragraph() {
//         let mut sample_file: File = tempfile::tempfile().unwrap();
//         let paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
//         write!(sample_file, "{}", paragraph.as_str()).unwrap();
//         sample_file.seek(SeekFrom::Start(0)).unwrap();

//         let mut paragraph_retriever = ParagraphRetriever::new();
//         assert_eq!(paragraph_retriever.load_chunks(sample_file), 1);

//         let read_result = paragraph_retriever.get_chunk(1);
//         assert!(read_result.is_none());
//     }

//     #[test]
//     fn get_next_chunk_returns_next_paragraph() {
//         let mut sample_file: File = tempfile::tempfile().unwrap();
//         let first_paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
//         let second_paragraph = String::from("This is another paragraph. It still contains four sentences. This is the first. Besides, this is another.");
//         write!(sample_file, "{}", first_paragraph.as_str()).unwrap();
//         write!(sample_file, "{}", second_paragraph.as_str()).unwrap();
//         sample_file.seek(SeekFrom::Start(0)).unwrap();

//         let mut paragraph_retriever = ParagraphRetriever::new();
//         assert_eq!(paragraph_retriever.load_chunks(sample_file), 2);

//         let read_result = paragraph_retriever.get_chunk(1);
//         assert!(read_result.is_some());

//         let read_second_paragraph = read_result.unwrap();
//         assert_eq!(*read_second_paragraph, second_paragraph);
//     }

//     #[test]
//     fn get_next_chunk_returns_incomplete_paragraph() {
//         let mut sample_file: File = tempfile::tempfile().unwrap();
//         let first_paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
//         let second_paragraph = String::from(
//             "This is another paragraph. It now contains three sentences. This is the first.",
//         );
//         write!(sample_file, "{}", first_paragraph.as_str()).unwrap();
//         write!(sample_file, "{}", second_paragraph.as_str()).unwrap();
//         sample_file.seek(SeekFrom::Start(0)).unwrap();

//         let mut paragraph_retriever = ParagraphRetriever::new();
//         assert_eq!(paragraph_retriever.load_chunks(sample_file), 2);

//         let read_result = paragraph_retriever.get_chunk(1);
//         assert!(read_result.is_some());

//         let read_second_paragraph = read_result.unwrap();
//         assert_eq!(*read_second_paragraph, second_paragraph);
//     }

//     #[test]
//     fn get_prev_chunk_returns_paragraph_from_two() {
//         let mut sample_file: File = tempfile::tempfile().unwrap();
//         let first_paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
//         let second_paragraph = String::from("This is another paragraph. It still contains four sentences. This is the first. Besides, this is another.");
//         write!(sample_file, "{}", first_paragraph.as_str()).unwrap();
//         write!(sample_file, "{}", second_paragraph.as_str()).unwrap();
//         sample_file.seek(SeekFrom::Start(0)).unwrap();

//         let mut paragraph_retriever = ParagraphRetriever::new();
//         assert_eq!(paragraph_retriever.load_chunks(sample_file), 2);

//         assert!(paragraph_retriever.get_chunk(1).is_some());

//         let read_result = paragraph_retriever.get_chunk(0);
//         assert!(read_result.is_some());

//         let read_paragraph = read_result.unwrap();
//         assert_eq!(*read_paragraph, first_paragraph);
//     }
// }
