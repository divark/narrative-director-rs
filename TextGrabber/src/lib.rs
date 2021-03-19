use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::str::from_utf8;

trait TextGrabber {
    fn get_until_delimiter(&mut self) -> Option<String>;
    fn get_range(&mut self, start: u64, num_bytes: u64) -> Result<String, String>;
}

struct EnglishParagraphRetriever {
    text_file: File,
    delimiters: HashSet<String>,
}

impl TextGrabber for EnglishParagraphRetriever {
    fn get_until_delimiter(&mut self) -> Option<String> {
        let mut found_line = String::new();

        let mut found_byte: [u8; 1] = [0; 1];
        while let Ok(n) = self.text_file.read(&mut found_byte) {
            if n == 0 {
                break;
            }

            let found_char = from_utf8(&found_byte).ok()?;
            found_line.push_str(found_char);

            if self.delimiters.contains(found_char) {
                break;
            }
        }

        if found_line.len() == 0 {
            return None;
        }

        Some(found_line)
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
        }
    }

    /*
    TODO: Implement and test these
    pub fn get_next_paragraph() -> Option<String> {

    }

    pub fn get_prev_paragraph() -> Option<String> {

    }

    pub fn get_paragraph(index: u32) -> Option<String> {

    }
    */
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
        let mut sample_file: File = tempfile::tempfile().unwrap();

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
}
