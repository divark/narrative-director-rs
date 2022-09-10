// use crate::{input_device_names, media::io::AudioInput, output_device_names, Session};
// use fltk::dialog;
// use std::path::PathBuf;

// /// Returns a File chosen by the user in a Dialog, or None if nothing
// /// was chosen.
// ///
// /// Preconditions: parent_window is a Window reference.
// /// Postconditions: A File wrapped in Some, or None.
// pub fn open() -> Option<PathBuf> {

// }

// /// Returns the paragraph number chosen by a user in a Dialog, or None if
// /// nothing was chosen.
// ///
// /// Preconditions: parent_window is a Window reference, and num_paragraphs is a usize
// ///                representing the total number of paragraphs in ParagraphViewer.
// /// Postconditions: The paragraph number represented as a usize.
// pub fn go_to(parent_window: &Window, num_paragraphs: usize) -> Option<usize> {
//     if num_paragraphs == 0 {
//         return None;
//     }

//     let goto_dialog = Dialog::with_buttons(
//         Some("Select the paragraph number."),
//         Some(parent_window),
//         DialogFlags::MODAL,
//         &[("Ok", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
//     );

//     goto_dialog.set_default_response(ResponseType::Ok);

//     let content_area = goto_dialog.content_area();

//     let goto_spin_button = SpinButton::with_range(1.0, num_paragraphs as f64, 1.0);
//     goto_spin_button.set_activates_default(true);

//     content_area.add(&goto_spin_button);

//     goto_dialog.show_all();

//     let goto_dialog_response = goto_dialog.run();
//     if goto_dialog_response == ResponseType::Ok {
//         goto_dialog.close();
//         return Some((goto_spin_button.value_as_int() - 1) as usize);
//     }

//     goto_dialog.close();
//     None
// }

// /// Shows an About Dialog describing the program.
// ///
// /// Precondition: parent_window is a Window reference.
// /// Postcondition: An About Dialog is shown until it is closed.
// pub fn about(parent_window: &Window) {
//     let logo: Pixbuf =
//         Pixbuf::from_file("resources/images/icon.png").expect("Could not find icon file.");

//     let about_dialog = AboutDialogBuilder::new()
// 		.program_name("Narrative Director")
// 		.comments("Narrative Director is an alternative Audio/Video Recording application tailored for working on medium to large-sized projects. This tool aspires to keep editing to a minimum with the capability of playing, recording and re-recording readings in place at the paragraph level for some text piece, whether it's a script, or a novel.")
// 		.authors(vec!["Tyler Schmidt <tmschmid@protonmail.com>".to_string()])
// 		.license_type(License::Gpl30)
//         .logo(&logo)
// 		.parent(parent_window)
// 		.build();

//     about_dialog.show();
//     about_dialog.run();
//     about_dialog.close();
// }
