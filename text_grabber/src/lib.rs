use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::str::from_utf8;

pub trait TextGrabber {
    fn get_until_delimiter(&mut self) -> Option<String>;

    fn get_next_chunk(&mut self) -> Option<&String>;
    fn get_prev_chunk(&mut self) -> Option<&String>;
    fn get_chunk(&self, chunk_num: usize) -> Option<&String>;
}

pub struct EnglishParagraphRetriever {
    text_file: File,
    delimiters: HashSet<String>,

    current_paragraph_num: usize,
    paragraphs: Vec<String>,
}

impl TextGrabber for EnglishParagraphRetriever {
    fn get_until_delimiter(&mut self) -> Option<String> {
        let mut found_bytes: Vec<u8> = Vec::new();

        let mut found_byte: [u8; 1] = [0; 1];
        while let Ok(n) = self.text_file.read(&mut found_byte) {
            if n == 0 {
                break;
            }

            found_bytes.push(found_byte[0]);
            if let Ok(found_char) = from_utf8(&found_byte) {
                if self.delimiters.contains(found_char) {
                    break;
                }
            }
        }

        if found_bytes.len() == 0 {
            return None;
        }

        Some(String::from_utf8(found_bytes).expect("Invalid UTF-8 given."))
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

    fn get_chunk(&self, chunk_num: usize) -> Option<&String> {
        self.paragraphs.get(chunk_num)
    }
}

impl EnglishParagraphRetriever {
    pub fn new(text_file: File) -> EnglishParagraphRetriever {
        let mut end_of_sentence_symbols = HashSet::new();
        end_of_sentence_symbols.insert(String::from("."));
        end_of_sentence_symbols.insert(String::from("?"));
        end_of_sentence_symbols.insert(String::from("!"));

        EnglishParagraphRetriever {
            text_file,
            delimiters: end_of_sentence_symbols,
            current_paragraph_num: 0,
            paragraphs: Vec::new(),
        }
    }

    fn get_range(&mut self, start: u64, num_bytes: u64) -> Result<String, String> {
        match self.text_file.seek(SeekFrom::Start(start)) {
            Ok(_) => {
                let mut read_nbytes = vec![0; num_bytes as usize];
                match self.text_file.read_exact(read_nbytes.as_mut_slice()) {
                    Ok(_) => Ok(String::from_utf8(read_nbytes).unwrap()),
                    Err(msg) => Err(format!("get_range read_exact error: {}", msg.to_string())),
                }
            }
            Err(msg) => Err(format!("get_range seek error: {}", msg.to_string())),
        }
    }

    pub fn load_paragraphs(&mut self) -> u32 {
        let mut current_paragraph: String = String::new();
        let mut num_paragraphs = 0;

        let mut num_sentences = 0;
        while let Some(sentence) = self.get_until_delimiter() {
            num_sentences += 1;

            if num_sentences == 5 {
                num_sentences = 0;

                self.paragraphs.push(current_paragraph);
                num_paragraphs += 1;
                current_paragraph = String::new();
            }

            current_paragraph.push_str(sentence.as_str());
        }

        if !current_paragraph.is_empty() {
            self.paragraphs.push(current_paragraph);
            num_paragraphs += 1;
        }

        num_paragraphs
    }
}

#[cfg(test)]
mod tests {
    use crate::{EnglishParagraphRetriever, TextGrabber};
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::Write;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn gets_complete_sentence() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        write!(sample_file, "This is a complete sentence.").unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);

        let read_result = paragraph_retriever.get_until_delimiter();
        assert!(read_result.is_some());

        let read_sentence = read_result.unwrap();
        assert_eq!(read_sentence, String::from("This is a complete sentence."));
    }

    #[test]
    fn gets_one_sentence() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        write!(
            sample_file,
            "This is a complete sentence. Here is a follow-up."
        )
        .unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);

        let read_result = paragraph_retriever.get_until_delimiter();
        assert!(read_result.is_some());

        let read_sentence = read_result.unwrap();
        assert_eq!(read_sentence, String::from("This is a complete sentence."));
    }

    #[test]
    fn gets_next_sentence() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        write!(
            sample_file,
            "This is a complete sentence. Here is a follow-up."
        )
        .unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);

        let read_result = paragraph_retriever.get_until_delimiter();
        assert!(read_result.is_some());

        let read_sentence = read_result.unwrap();
        assert_eq!(read_sentence, String::from("This is a complete sentence."));

        let read_result = paragraph_retriever.get_until_delimiter();
        assert!(read_result.is_some());

        let read_sentence = read_result.unwrap();
        assert_eq!(read_sentence, String::from(" Here is a follow-up."));
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

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);

        let read_result = paragraph_retriever.get_until_delimiter();
        assert!(read_result.is_some());

        let read_sentence = read_result.unwrap();
        assert_eq!(
            read_sentence,
            String::from("This is a complete sentence with no ending punctuation")
        );
    }

    #[test]
    fn get_no_sentence_from_empty() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);

        let read_result = paragraph_retriever.get_until_delimiter();
        assert!(read_result.is_none());
    }

    #[test]
    fn gets_no_next_sentence_from_one_sentence() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        write!(sample_file, "This is a complete sentence.").unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);

        let read_result = paragraph_retriever.get_until_delimiter();
        assert!(read_result.is_some());

        let read_sentence = read_result.unwrap();
        assert_eq!(read_sentence, String::from("This is a complete sentence."));

        let read_result = paragraph_retriever.get_until_delimiter();
        assert!(read_result.is_none());
    }

    #[test]
    fn gets_sentence_from_length() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        let sentence = String::from("This is a complete sentence.");
        write!(sample_file, "{}", sentence.as_str()).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);

        let read_result = paragraph_retriever.get_range(0, sentence.len() as u64);
        assert!(read_result.is_ok());

        let read_sentence = read_result.unwrap();
        assert_eq!(read_sentence, sentence);
    }

    #[test]
    fn gets_no_sentence_from_large_length() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        let sentence = String::from("This is a complete sentence.");
        write!(sample_file, "{}", sentence.as_str()).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);

        let read_result = paragraph_retriever.get_range(0, (sentence.len() + 1) as u64);
        assert!(read_result.is_err());
    }

    #[test]
    fn gets_no_sentence_from_empty_file() {
        let sample_file: File = tempfile::tempfile().unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);

        let read_result = paragraph_retriever.get_range(0, 10);
        assert!(read_result.is_err());
    }

    #[test]
    fn gets_word_from_sentence() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        let sentence = String::from("This is a complete sentence.");
        let word = String::from("complete");
        write!(sample_file, "{}", sentence.as_str()).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);

        let read_result = paragraph_retriever.get_range(10, word.len() as u64);
        assert!(read_result.is_ok());

        let read_word = read_result.unwrap();
        assert_eq!(read_word, word);
    }

    #[test]
    fn get_chunk_returns_only_paragraph() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        let paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
        write!(sample_file, "{}", paragraph.as_str()).unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);
        let num_paragraphs_read = paragraph_retriever.load_paragraphs();
        assert_eq!(num_paragraphs_read, 1);

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

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);
        let num_paragraphs_read = paragraph_retriever.load_paragraphs();
        assert_eq!(num_paragraphs_read, 1);

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

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);
        let num_paragraphs_read = paragraph_retriever.load_paragraphs();
        assert_eq!(num_paragraphs_read, 2);

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

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);
        let num_paragraphs_read = paragraph_retriever.load_paragraphs();
        assert_eq!(num_paragraphs_read, 2);

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

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);
        let num_paragraphs_read = paragraph_retriever.load_paragraphs();
        assert_eq!(num_paragraphs_read, 1);

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

        let mut paragraph_retriever = EnglishParagraphRetriever::new(sample_file);
        let num_paragraphs_read = paragraph_retriever.load_paragraphs();
        assert_eq!(num_paragraphs_read, 2);

        assert!(paragraph_retriever.get_next_chunk().is_some());

        let read_result = paragraph_retriever.get_prev_chunk();
        assert!(read_result.is_some());

        let read_paragraph = read_result.unwrap();
        assert_eq!(*read_paragraph, first_paragraph);
    }
}
