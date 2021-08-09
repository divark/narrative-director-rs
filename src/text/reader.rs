use std::fs::File;
use std::io::Read;

pub trait TextGrabber {
    // Returns the number of chunks parsed from some UTF-8 text file.
    fn load_chunks(&mut self, text_file: File) -> u32;

    fn get_chunk(&self, chunk_num: usize) -> Option<&String>;
    fn len(&self) -> usize;
}

pub enum LangDelimiters {
    English,
}

impl LangDelimiters {
    fn value(&self) -> &[char] {
        match self {
            LangDelimiters::English => &['.', '?', '!'],
        }
    }
}

pub struct ParagraphRetriever {
    language: LangDelimiters,
    num_sentences: u8,
    paragraphs: Vec<String>,
}

impl TextGrabber for ParagraphRetriever {
    fn load_chunks(&mut self, mut text_file: File) -> u32 {
        let mut whole_text_content = String::new();
        text_file
            .read_to_string(&mut whole_text_content)
            .expect("Could not read text file.");

        let language_delimiters = self.language.value();
        let split_paragraphs: Vec<&str> = whole_text_content
            .split_inclusive(language_delimiters)
            .collect();

        self.paragraphs = split_paragraphs
            .chunks(self.num_sentences as usize)
            .map(|sentences| sentences.concat())
            .collect();
        self.paragraphs.len() as u32
    }

    fn get_chunk(&self, chunk_num: usize) -> Option<&String> {
        if chunk_num >= self.paragraphs.len() {
            return None;
        }

        self.paragraphs.get(chunk_num)
    }

    fn len(&self) -> usize {
        self.paragraphs.len()
    }
}

impl ParagraphRetriever {
    /// Returns a ParagraphRetriever with the following defaults:
    /// - language is set to English,
    /// - A paragraph consists of four sentences.
    pub fn new() -> Self {
        Self {
            language: LangDelimiters::English,
            num_sentences: 4,
            paragraphs: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::text::{ParagraphRetriever, TextGrabber};
    use std::fs::File;
    use std::io::Write;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn gets_complete_sentence() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        write!(sample_file, "This is a complete sentence.").unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = ParagraphRetriever::new();
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

        let mut paragraph_retriever = ParagraphRetriever::new();
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

        let mut paragraph_retriever = ParagraphRetriever::new();
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

        let mut paragraph_retriever = ParagraphRetriever::new();
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

        let mut paragraph_retriever = ParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 1);

        let read_result = paragraph_retriever.get_chunk(1);
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

        let mut paragraph_retriever = ParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 2);

        let read_result = paragraph_retriever.get_chunk(1);
        assert!(read_result.is_some());

        let read_second_paragraph = read_result.unwrap();
        assert_eq!(*read_second_paragraph, second_paragraph);
    }

    #[test]
    fn get_next_chunk_returns_incomplete_paragraph() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        let first_paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
        let second_paragraph = String::from(
            "This is another paragraph. It now contains three sentences. This is the first.",
        );
        write!(sample_file, "{}", first_paragraph.as_str()).unwrap();
        write!(sample_file, "{}", second_paragraph.as_str()).unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = ParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 2);

        let read_result = paragraph_retriever.get_chunk(1);
        assert!(read_result.is_some());

        let read_second_paragraph = read_result.unwrap();
        assert_eq!(*read_second_paragraph, second_paragraph);
    }

    #[test]
    fn get_prev_chunk_returns_paragraph_from_two() {
        let mut sample_file: File = tempfile::tempfile().unwrap();
        let first_paragraph = String::from("This is a complete paragraph. It contains four sentences. This is the first. Also, this is another.");
        let second_paragraph = String::from("This is another paragraph. It still contains four sentences. This is the first. Besides, this is another.");
        write!(sample_file, "{}", first_paragraph.as_str()).unwrap();
        write!(sample_file, "{}", second_paragraph.as_str()).unwrap();
        sample_file.seek(SeekFrom::Start(0)).unwrap();

        let mut paragraph_retriever = ParagraphRetriever::new();
        assert_eq!(paragraph_retriever.load_chunks(sample_file), 2);

        assert!(paragraph_retriever.get_chunk(1).is_some());

        let read_result = paragraph_retriever.get_chunk(0);
        assert!(read_result.is_some());

        let read_paragraph = read_result.unwrap();
        assert_eq!(*read_paragraph, first_paragraph);
    }
}
