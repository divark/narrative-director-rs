use crate::text::{ParagraphRetriever, TextGrabber};
use gtk::prelude::{
    DialogExt, DialogExtManual, FileChooserExt, GtkWindowExt, LabelExt, TextBufferExt, TextViewExt,
};
use gtk::{FileFilter, Label, ResponseType, TextView, Window};

use std::path::PathBuf;

pub struct ChunkViewingUi<'a> {
    pub progress_label: &'a Label,
    pub chunk_viewer: &'a TextView,
}

/// Populates the UI with the specified chunk.
pub fn show_chunk(
    chunk_num: u32,
    chunk_getter: &ParagraphRetriever,
    ui: ChunkViewingUi,
) -> Result<(), ()> {
    if let Some(paragraph) = chunk_getter.get_chunk(chunk_num as usize) {
        ui.progress_label
            .set_text(format!("{}/{}", chunk_num + 1, chunk_getter.len()).as_str());

        ui.chunk_viewer
            .buffer()
            .expect("Could not load TextView from Chunk Viewer")
            .set_text(paragraph.as_str());

        return Ok(());
    }

    Err(())
}

/// Returns the file chosen for text parsing.
pub fn get_text_file_from_user(parent_window: &Window) -> Option<PathBuf> {
    // 1: Create the File Chooser dialog that only accepts
    // text files.
    let file_chooser = gtk::FileChooserDialog::new(
        Some("Open File"),
        Some(parent_window),
        gtk::FileChooserAction::Open,
    );

    file_chooser.add_buttons(&[
        ("Open", gtk::ResponseType::Ok),
        ("Cancel", gtk::ResponseType::Cancel),
    ]);

    let text_file_filter = FileFilter::new();
    text_file_filter.set_name(Some("UTF-8 Text Files"));
    text_file_filter.add_pattern("*.txt");

    file_chooser.add_filter(&text_file_filter);

    // 2: Fetch file from user choice, if any.
    let user_response = file_chooser.run();
    if user_response != ResponseType::Ok {
        file_chooser.close();
        return None;
    }

    file_chooser.close();

    let filename = file_chooser.filename().expect("Couldn't get filename");

    Some(filename)
}
