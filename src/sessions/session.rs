use cpal::default_host;
use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};
use std::fs::{write, DirBuilder, File};
use std::io::Read;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct AudioInput {
    input_device_name: String,
    sample_rate: u32,
    channels: u16,
}

impl AudioInput {
    pub fn new() -> AudioInput {
        let host = default_host();
        let input_device = host
            .default_input_device()
            .expect("Could not retrieve a default input device.");

        let input_config = input_device
            .default_input_config()
            .expect("Could not retrieve the properties from the default input device.");

        AudioInput {
            input_device_name: input_device
                .name()
                .unwrap_or_else(|_| "Default".to_string()),
            sample_rate: input_config.sample_rate().0,
            channels: input_config.channels(),
        }
    }

    pub fn set_device_name(&mut self, name: String) {
        self.input_device_name = name;
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    pub fn set_channels(&mut self, channels: u16) {
        self.channels = channels;
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Font {
    size: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Session {
    paragraph_num: usize,
    project_directory: PathBuf,
    font: Font,

    audio_input: AudioInput,
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
            .expect("Could not parse file stem from text file");

        let mut project_directory = PathBuf::new();
        project_directory.push(default_audio_dir);
        project_directory.push(project_name);
        if !project_directory.is_dir() {
            DirBuilder::new()
                .recursive(true)
                .create(project_directory.clone())
                .expect("Could not create directory for recordings.");
        }

        Session {
            paragraph_num: 0,
            project_directory,
            font: Font { size: 12 },

            audio_input: AudioInput::new(),
        }
    }

    fn get_session_path(&self) -> PathBuf {
        let projects_path = get_projects_path();
        let project_name = self
            .project_directory
            .components()
            .last()
            .expect("Could not get project name from path.");

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

        Some(serde_json::from_str(&file_contents).expect("Unable to parse Session from JSON."))
    }

    pub fn set_paragraph_num(&mut self, paragraph_num: usize) {
        self.paragraph_num = paragraph_num;
    }

    pub fn paragraph_num(&self) -> usize {
        self.paragraph_num
    }
}

#[cfg(all(target_os = "linux", test))]
mod linux_tests {
    use super::*;
    use std::fs;

    fn get_text_path() -> PathBuf {
        let mut text_path = PathBuf::new();
        text_path.push("src");
        text_path.push("tests");
        text_path.push("test.txt");

        assert!(text_path.is_file());

        text_path
    }

    fn get_test_session_path() -> PathBuf {
        let mut test_path = PathBuf::new();
        test_path.push(dirs::home_dir().expect("Could not find home directory."));
        test_path.push(".local/share/narrative_director/projects/test/session.json");

        test_path
    }

    #[test]
    fn test_session_path_textfile() {
        let text_path = get_text_path();
        let expected_path = get_test_session_path();
        let actual_path = get_session_path_from_textfile(text_path);

        assert_eq!(actual_path, expected_path.to_path_buf());
    }

    #[test]
    fn test_session_get_path() {
        let text_path = get_text_path();
        let expected_path = get_test_session_path();
        let actual_path = Session::new(text_path).get_session_path();

        assert!(!actual_path.is_file());

        assert_eq!(actual_path, expected_path.to_path_buf());
    }

    #[test]
    fn test_session_save() {
        let text_path = get_text_path();

        let test_session_path = get_test_session_path();
        assert!(!test_session_path.is_file());

        let new_session = Session::new(text_path);
        new_session.save();

        assert!(test_session_path.is_file());

        fs::remove_file(test_session_path).expect("Could not remove session file.");
    }

    #[test]
    fn test_session_load() {
        let text_path = get_text_path();

        let test_session_path = get_test_session_path();
        assert!(!test_session_path.is_file());

        let new_session = Session::new(text_path.clone());
        new_session.save();

        assert!(test_session_path.is_file());

        let loaded_session = Session::load(text_path);
        assert!(loaded_session.is_some());

        if let Some(session) = loaded_session {
            assert_eq!(session, new_session);
        }

        fs::remove_file(test_session_path).expect("Could not remove session file.");
    }
}
// TODO: Write tests before integrating changes into main.
