use std::collections::HashSet;
use std::fs::File;
use std::io::{Read};
use std::str::from_utf8;

pub trait TextGrabber {
    // Returns the number of chunks parsed from some UTF-8 text file.
    fn load_chunks(&mut self, text_file: File) -> u32;

    fn get_next_chunk(&mut self) -> Option<&String>;
    fn get_prev_chunk(&mut self) -> Option<&String>;
    fn get_chunk(&mut self, chunk_num: usize) -> Option<&String>;
}

pub struct EnglishParagraphRetriever {
    delimiters: HashSet<String>,

    current_paragraph_num: usize,
    paragraphs: Vec<String>,
}

impl TextGrabber for EnglishParagraphRetriever {
    fn load_chunks(&mut self, mut text_file: File) -> u32 {
        let mut current_paragraph: Vec<u8> = Vec::new();

        let mut found_byte: [u8; 1] = [0; 1];
        let mut sentence_count: u32 = 0;
        while let Ok(num_bytes_read) = text_file.read(&mut found_byte) {
            if num_bytes_read == 0 {
                break;
            }

            current_paragraph.push(found_byte[0]);
            if let Ok(found_char) = from_utf8(&found_byte) {
                if self.delimiters.contains(found_char) {
                    sentence_count += 1;
                }
            }

            if sentence_count == 4 {
                self.paragraphs.push(String::from_utf8(current_paragraph).expect("Invalid UTF-8 given."));
                current_paragraph = Vec::new();
                sentence_count = 0;
            }
        }

        if current_paragraph.len() > 0 {
            self.paragraphs.push(String::from_utf8(current_paragraph).expect("Invalid UTF-8 given."));
        }

        self.paragraphs.len() as u32
    }

    fn get_next_chunk(&mut self) -> Option<&String> {
        if self.current_paragraph_num + 1 >= self.paragraphs.len() {
            return None;
        }

        self.current_paragraph_num += 1;
        self.paragraphs.get(self.current_paragraph_num)
    }

    fn get_prev_chunk(&mut self) -> Option<&String> {
        if self.current_paragraph_num == 0 {
            return None;
        }

        self.current_paragraph_num -= 1;
        self.paragraphs.get(self.current_paragraph_num)
    }

    fn get_chunk(&mut self, chunk_num: usize) -> Option<&String> {
        if chunk_num < 0 || chunk_num >= self.paragraphs.len() {
            return None;
        }

        self.current_paragraph_num = chunk_num;
        self.paragraphs.get(chunk_num)
    }
}

impl EnglishParagraphRetriever {
    pub fn new() -> EnglishParagraphRetriever {
        let mut end_of_sentence_symbols = HashSet::new();
        end_of_sentence_symbols.insert(String::from("."));
        end_of_sentence_symbols.insert(String::from("?"));
        end_of_sentence_symbols.insert(String::from("!"));

        EnglishParagraphRetriever {
            delimiters: end_of_sentence_symbols,
            current_paragraph_num: 0,
            paragraphs: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{EnglishParagraphRetriever, TextGrabber};
    use std::fs::File;
    use std::io::Write;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn gets_complete_sentence() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        write!(sample_file, "This is a complete sentence.").unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 1);

        let read_result = paragraph_retriever.get_chunk(0);
        assert!(read_result.is_some());

        let read_sentence = read_result.unwrap();
        assert_eq!(*read_sentence, String::from("This is a complete sentence."));
    }

    #[test]
    fn gets_incomplete_sentence() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        write!(
            sample_file,
            "This is a complete sentence with no ending punctuation"
        )
        .unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 1);

        let read_result = paragraph_retriever.get_chunk(0);
        assert!(read_result.is_some());

        let read_sentence = read_result.unwrap();
        assert_eq!(
            *read_sentence,
            String::from("This is a complete sentence with no ending punctuation")
        );
    }

    #[test]
    fn get_no_sentence_from_empty() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 0);

        let read_result = paragraph_retriever.get_chunk(0);
        assert!(read_result.is_none());
    }

    #[test]
    fn get_chunk_returns_only_paragraph() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        let paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
        write!(sample_file, "{}", paragraph.as_str()).unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 1);

        let read_result = paragraph_retriever.get_chunk(0);
        assert!(read_result.is_some());

        let read_paragraph = read_result.unwrap();
        assert_eq!(*read_paragraph, paragraph);
    }

    #[test]
    fn get_next_chunk_returns_nothing_one_paragraph() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        let paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
        write!(sample_file, "{}", paragraph.as_str()).unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 1);

        let read_result = paragraph_retriever.get_next_chunk();
        assert!(read_result.is_none());
    }

    #[test]
    fn get_next_chunk_returns_next_paragraph() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        let first_paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
        let second_paragraph = String::from("This is another paragraph. It still contains four sentences. This is the first. Besides, this is another.");
        write!(sample_file, "{}", first_paragraph.as_str()).unwrap();
        write!(sample_file, "{}", second_paragraph.as_str()).unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 2);

        let read_result = paragraph_retriever.get_next_chunk();
        assert!(read_result.is_some());

        let read_second_paragraph = read_result.unwrap();
        assert_eq!(*read_second_paragraph, second_paragraph);
    }

    #[test]
    fn get_next_chunk_returns_incomplete_paragraph() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        let first_paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
        let second_paragraph = String::from("This is another paragraph. It now contains three sentences. This is the first.");
        write!(sample_file, "{}", first_paragraph.as_str()).unwrap();
        write!(sample_file, "{}", second_paragraph.as_str()).unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 2);

        let read_result = paragraph_retriever.get_next_chunk();
        assert!(read_result.is_some());

        let read_second_paragraph = read_result.unwrap();
        assert_eq!(*read_second_paragraph, second_paragraph);
    }

    #[test]
    fn get_prev_chunk_returns_nothing_one_paragraph() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        let paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
        write!(sample_file, "{}", paragraph.as_str()).unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 1);

        let read_result = paragraph_retriever.get_prev_chunk();
        assert!(read_result.is_none());
    }

    #[test]
    fn get_prev_chunk_returns_paragraph_from_two() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        let first_paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
        let second_paragraph = String::from("This is another paragraph. It still contains four sentences. This is the first. Besides, this is another.");
        write!(sample_file, "{}", first_paragraph.as_str()).unwrap();
        write!(sample_file, "{}", second_paragraph.as_str()).unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 2);

        assert!(paragraph_retriever.get_next_chunk().is_some());

        let read_result = paragraph_retriever.get_prev_chunk();
        assert!(read_result.is_some());

        let read_paragraph = read_result.unwrap();
        assert_eq!(*read_paragraph, first_paragraph);
    }
}
