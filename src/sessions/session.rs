use serde::{Deserialize, Serialize};
use std::fs::{write, DirBuilder, File};
use std::io::Read;
use std::path::PathBuf;

use crate::media::io::{AudioInput, AudioOutput};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Session {
    paragraph_num: usize,

    project_file_name: String,
    project_output_directory: PathBuf,

    audio_input: AudioInput,
    audio_output: AudioOutput,

    gathering_choice: String,
    gathering_amount: usize,
    gathering_delimiters: String,
}

fn get_projects_path() -> PathBuf {
    let data_dir = dirs::data_dir().expect("Could not find default data directory.");

    let mut projects_path = PathBuf::new();
    projects_path.push(data_dir);
    projects_path.push("narrative_director");
    projects_path.push("projects");

    projects_path
}

fn get_session_path_from_textfile(text_file_loc: PathBuf) -> PathBuf {
    let projects_path = get_projects_path();
    let project_name = text_file_loc
        .file_stem()
        .expect("Could not parse file stem from text file");

    let mut session_path = PathBuf::new();
    session_path.push(projects_path);
    session_path.push(project_name);
    session_path.push("session.json");

    session_path
}

impl Session {
    pub fn new(text_file_loc: PathBuf) -> Session {
        let default_audio_dir = dirs::audio_dir().expect("Could not find default audio directory.");

        let project_name = text_file_loc
            .file_stem()
            .expect("Could not parse file stem from text file")
            .to_str()
            .expect("Could not convert file name to string")
            .to_string();

        let mut project_directory = PathBuf::new();
        project_directory.push(default_audio_dir);
        project_directory.push(project_name.clone());
        if !project_directory.is_dir() {
            DirBuilder::new()
                .recursive(true)
                .create(project_directory.clone())
                .expect("Could not create directory for recordings.");
        }

        Session {
            paragraph_num: 0,

            project_file_name: project_name,
            project_output_directory: project_directory,

            audio_input: AudioInput::new(),
            audio_output: AudioOutput::new(),

            gathering_choice: String::from("Sentences"),
            gathering_amount: 4,
            gathering_delimiters: String::from(".?!"),
        }
    }

    fn get_session_path(&self) -> PathBuf {
        let projects_path = get_projects_path();
        let project_name = self.project_file_name.clone();

        let mut session_path = PathBuf::new();
        session_path.push(projects_path);
        session_path.push(project_name);
        session_path.push("session.json");

        session_path
    }

    pub fn save(&self) {
        let session_path = self.get_session_path();
        let project_directory = session_path
            .parent()
            .expect("Could not retrieve parent directory from session file.");
        if !project_directory.is_dir() {
            DirBuilder::new()
                .recursive(true)
                .create(project_directory)
                .expect("Could not create directory for recordings.");
        }

        write(
            session_path,
            serde_json::to_string(&self).expect("Could not parse session file."),
        )
        .expect("Could not write session file.");
    }

    pub fn load(text_file_loc: PathBuf) -> Option<Session> {
        let session_location = get_session_path_from_textfile(text_file_loc);
        if !session_location.is_file() {
            return None;
        }

        let mut session_file = File::open(session_location).expect("Could not load session file.");
        let mut file_contents = String::new();
        session_file
            .read_to_string(&mut file_contents)
            .expect("Unable to read contents from session file.");

        match serde_json::from_str(&file_contents) {
            Ok(session) => Some(session),
            Err(_) => None,
        }
    }

    pub fn set_paragraph_num(&mut self, paragraph_num: usize) {
        self.paragraph_num = paragraph_num;
    }

    pub fn paragraph_num(&self) -> usize {
        self.paragraph_num
    }

    pub fn set_project_directory(&mut self, new_directory: PathBuf) {
        self.project_output_directory = new_directory;
    }

    pub fn project_directory(&self) -> PathBuf {
        self.project_output_directory.clone()
    }

    pub fn audio_output(&self) -> &AudioOutput {
        &self.audio_output
    }

    pub fn audio_output_mut(&mut self) -> &mut AudioOutput {
        &mut self.audio_output
    }

    pub fn audio_input(&self) -> &AudioInput {
        &self.audio_input
    }

    pub fn audio_input_mut(&mut self) -> &mut AudioInput {
        &mut self.audio_input
    }

    pub fn gathering_choice(&self) -> String {
        self.gathering_choice.clone()
    }

    pub fn set_gathering_choice(&mut self, gathering_choice: &str) {
        self.gathering_choice = String::from(gathering_choice);
    }

    pub fn gathering_amount(&self) -> usize {
        self.gathering_amount
    }

    pub fn set_gathering_amount(&mut self, amount: usize) {
        self.gathering_amount = amount;
    }

    pub fn gathering_delimiters(&self) -> String {
        self.gathering_delimiters.clone()
    }

    pub fn set_gathering_delimiters(&mut self, delimiters: &str) {
        self.gathering_delimiters = String::from(delimiters);
    }
}
