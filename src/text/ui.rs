use crate::text::{ParagraphRetriever, TextGrabber};
use gtk::prelude::{LabelExt, TextBufferExt, TextViewExt};
use gtk::{Label, TextView};

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
