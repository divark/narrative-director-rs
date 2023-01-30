use std::path::PathBuf;

use fltk::prelude::WidgetExt;

pub fn shift_right_by_label<T>(choice: &mut T)
where
    T: WidgetExt,
{
    let choice_label_offset = choice.label_size();
    choice.set_pos(choice.x() + choice_label_offset, choice.y());
    choice.set_size(choice.width() - choice_label_offset, choice.height());
}

pub fn get_icon_path() -> PathBuf {
    let mut logo_path = PathBuf::new();
    logo_path = logo_path.join(env!("CARGO_MANIFEST_DIR"));
    logo_path = logo_path.join("resources/images/icon.png");

    logo_path
}

// TODO: Vertical Stacker builder type to reduce magic number usage.
